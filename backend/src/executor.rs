use std::{convert::Infallible, io::Write, process::Stdio, time::Duration};

use axum::response::sse::Event;
use chrono::Utc;
use futures::Stream;
use serde::Serialize;
use serde_json::{Value, json};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, BufReader},
    process::Command,
    sync::mpsc,
    task::JoinHandle,
    time::{Instant, timeout},
};
use tokio_stream::{StreamExt, wrappers::ReceiverStream};

use crate::{
    db::Database,
    domain::{
        ExecutionPlan, ExecutionResult, ExecutionSource, ExecutionStatus, NewCallLog,
        OUTPUT_LIMIT_BYTES, Tool, build_execution_plan,
    },
    error::AppError,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
enum OutputStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Serialize)]
struct StreamPayload {
    stream: Option<OutputStream>,
    data: String,
    exit_code: Option<i32>,
    duration_ms: Option<i64>,
    status: Option<ExecutionStatus>,
}

pub async fn execute_and_log(
    db: &Database,
    tool: Tool,
    params: Value,
    source: ExecutionSource,
    working_dir_override: Option<String>,
) -> Result<ExecutionResult, AppError> {
    let settings = db.get_settings().await?;
    let plan = build_execution_plan(&tool, &params, working_dir_override.as_deref(), &settings)?;
    let started_at = Utc::now();
    let started = Instant::now();
    let result = execute_plan(&tool, &plan, None)
        .await
        .unwrap_or_else(|error| execution_error_result(error, started.elapsed()));
    let finished_at = Utc::now();

    db.insert_call_log(NewCallLog {
        tool_id: Some(tool.id),
        tool_name: tool.name,
        source,
        params_json: plan.params_json,
        stdout: result.stdout.clone(),
        stderr: result.stderr.clone(),
        exit_code: result.exit_code,
        started_at,
        finished_at,
        duration_ms: result.duration_ms,
        status: result.status.clone(),
    })
    .await?;

    Ok(result)
}

pub async fn stream_and_log(
    db: Database,
    tool: Tool,
    params: Value,
    source: ExecutionSource,
    working_dir_override: Option<String>,
) -> Result<impl Stream<Item = Result<Event, Infallible>>, AppError> {
    let settings = db.get_settings().await?;
    let plan = build_execution_plan(&tool, &params, working_dir_override.as_deref(), &settings)?;
    let (sender, receiver) = mpsc::channel::<StreamPayload>(128);

    tokio::spawn(async move {
        let started_at = Utc::now();
        let started = Instant::now();
        let result = execute_plan(&tool, &plan, Some(sender.clone())).await;
        let finished_at = Utc::now();

        let (result, emit_captured_output) = match result {
            Ok(result) => (result, false),
            Err(error) => (execution_error_result(error, started.elapsed()), true),
        };

        if emit_captured_output {
            emit_captured_streams(&sender, &result).await;
        }

        let _ = sender
            .send(StreamPayload {
                stream: None,
                data: String::new(),
                exit_code: result.exit_code,
                duration_ms: Some(result.duration_ms),
                status: Some(result.status.clone()),
            })
            .await;

        let _ = db
            .insert_call_log(NewCallLog {
                tool_id: Some(tool.id),
                tool_name: tool.name,
                source,
                params_json: plan.params_json,
                stdout: result.stdout,
                stderr: result.stderr,
                exit_code: result.exit_code,
                started_at,
                finished_at,
                duration_ms: result.duration_ms,
                status: result.status,
            })
            .await;
    });

    Ok(ReceiverStream::new(receiver).map(|payload| {
        let event_name = if payload.status.is_some() {
            "done"
        } else {
            match payload.stream {
                Some(OutputStream::Stdout) => "stdout",
                Some(OutputStream::Stderr) => "stderr",
                None => "message",
            }
        };

        Ok(sse_event(event_name, &payload))
    }))
}

async fn execute_plan(
    tool: &Tool,
    plan: &ExecutionPlan,
    stream_sender: Option<mpsc::Sender<StreamPayload>>,
) -> Result<ExecutionResult, AppError> {
    let ToolCommand {
        mut command,
        summary,
        _script_file,
    } = command_for_tool(tool)?;
    command.envs(&plan.env);
    if let Some(working_dir) = &plan.working_dir {
        command.current_dir(working_dir);
    }
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    command.kill_on_drop(true);
    hide_child_window(&mut command);

    let mut child = command.spawn().map_err(|error| {
        AppError::Execution(format!("failed to start command `{summary}`: {error}"))
    })?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::Execution("stdout pipe was unavailable".to_string()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::Execution("stderr pipe was unavailable".to_string()))?;

    let stdout_task = read_pipe(stdout, OutputStream::Stdout, stream_sender.clone());
    let stderr_task = read_pipe(stderr, OutputStream::Stderr, stream_sender);
    let started = Instant::now();

    let wait_result = timeout(
        Duration::from_millis(tool.timeout_ms.max(1) as u64),
        child.wait(),
    )
    .await;

    let (exit_code, status) = match wait_result {
        Ok(Ok(status)) => {
            let code = status.code();
            let status = if status.success() {
                ExecutionStatus::Success
            } else {
                ExecutionStatus::Error
            };
            (code, status)
        }
        Ok(Err(error)) => return Err(AppError::Io(error)),
        Err(_) => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            (None, ExecutionStatus::Timeout)
        }
    };

    let stdout_capture = wait_for_capture(stdout_task).await;
    let stderr_capture = wait_for_capture(stderr_task).await;

    Ok(ExecutionResult {
        stdout: stdout_capture.output,
        stderr: stderr_capture.output,
        exit_code,
        duration_ms: started.elapsed().as_millis() as i64,
        status,
        stdout_truncated: stdout_capture.truncated,
        stderr_truncated: stderr_capture.truncated,
    })
}

struct ToolCommand {
    command: Command,
    summary: String,
    _script_file: Option<tempfile::TempPath>,
}

fn command_for_tool(tool: &Tool) -> Result<ToolCommand, AppError> {
    if let Some(script_body) = tool
        .script_body
        .as_deref()
        .filter(|body| !body.trim().is_empty())
    {
        if cfg!(windows) {
            return command_for_windows_script_body(script_body);
        }

        let (shell, flag) = script_shell();
        let mut command = Command::new(&shell);
        command.arg(flag).arg(script_body);
        return Ok(ToolCommand {
            command,
            summary: format!("{shell} {flag} {}", command_preview(script_body)),
            _script_file: None,
        });
    }

    if let Some(script_path) = tool
        .script_path
        .as_deref()
        .filter(|path| !path.trim().is_empty())
    {
        return Ok(ToolCommand {
            command: Command::new(script_path),
            summary: script_path.to_string(),
            _script_file: None,
        });
    }

    Err(AppError::Validation(
        "tool must include script_body or script_path".to_string(),
    ))
}

fn command_for_windows_script_body(script_body: &str) -> Result<ToolCommand, AppError> {
    const POWERSHELL_UTF8_PRELUDE: &[u8] = br#"$utf8NoBom = [System.Text.UTF8Encoding]::new($false)
[Console]::InputEncoding = $utf8NoBom
[Console]::OutputEncoding = $utf8NoBom
$OutputEncoding = $utf8NoBom
"#;

    let mut file = tempfile::Builder::new()
        .prefix("conjure-")
        .suffix(".ps1")
        .tempfile()?;
    file.write_all(b"\xEF\xBB\xBF")?;
    file.write_all(POWERSHELL_UTF8_PRELUDE)?;
    file.write_all(script_body.as_bytes())?;
    file.flush()?;

    let script_file = file.into_temp_path();
    let script_path = script_file.to_path_buf();
    let mut command = Command::new("powershell.exe");
    command
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-File")
        .arg(&script_path);

    Ok(ToolCommand {
        command,
        summary: format!(
            "powershell.exe -NoProfile -ExecutionPolicy Bypass -File {}",
            script_path.display()
        ),
        _script_file: Some(script_file),
    })
}

#[cfg(windows)]
fn hide_child_window(command: &mut Command) {
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn hide_child_window(_command: &mut Command) {}

fn script_shell() -> (String, &'static str) {
    ("sh".to_string(), "-c")
}

fn command_preview(command: &str) -> String {
    const MAX_PREVIEW_CHARS: usize = 160;

    let single_line = command.split_whitespace().collect::<Vec<_>>().join(" ");
    if single_line.chars().count() <= MAX_PREVIEW_CHARS {
        return single_line;
    }

    let preview = single_line
        .chars()
        .take(MAX_PREVIEW_CHARS)
        .collect::<String>();
    format!("{preview}...")
}

async fn emit_captured_streams(sender: &mpsc::Sender<StreamPayload>, result: &ExecutionResult) {
    for line in result.stdout.lines() {
        let _ = sender
            .send(StreamPayload {
                stream: Some(OutputStream::Stdout),
                data: line.to_string(),
                exit_code: None,
                duration_ms: None,
                status: None,
            })
            .await;
    }

    for line in result.stderr.lines() {
        let _ = sender
            .send(StreamPayload {
                stream: Some(OutputStream::Stderr),
                data: line.to_string(),
                exit_code: None,
                duration_ms: None,
                status: None,
            })
            .await;
    }
}

fn read_pipe<R>(
    reader: R,
    stream: OutputStream,
    sender: Option<mpsc::Sender<StreamPayload>>,
) -> JoinHandle<LimitedCapture>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut capture = LimitedCapture::new(OUTPUT_LIMIT_BYTES);
        let mut lines = BufReader::new(reader).lines();

        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    capture.push_line(&line);
                    if let Some(sender) = &sender {
                        let _ = sender
                            .send(StreamPayload {
                                stream: Some(stream.clone()),
                                data: line,
                                exit_code: None,
                                duration_ms: None,
                                status: None,
                            })
                            .await;
                    }
                }
                Ok(None) => break,
                Err(error) => {
                    capture.push_line(&format!("output read error: {error}"));
                    break;
                }
            }
        }

        capture
    })
}

async fn wait_for_capture(task: JoinHandle<LimitedCapture>) -> LimitedCapture {
    task.await
        .unwrap_or_else(|error| LimitedCapture::from_error(&error.to_string()))
}

fn execution_error_result(error: AppError, duration: Duration) -> ExecutionResult {
    ExecutionResult {
        stdout: String::new(),
        stderr: error.to_string(),
        exit_code: None,
        duration_ms: duration.as_millis() as i64,
        status: ExecutionStatus::Error,
        stdout_truncated: false,
        stderr_truncated: false,
    }
}

fn sse_event<T: Serialize>(event_name: &str, payload: &T) -> Event {
    Event::default()
        .event(event_name)
        .data(serde_json::to_string(payload).unwrap_or_else(|_| "{}".to_string()))
}

struct LimitedCapture {
    output: String,
    limit: usize,
    truncated: bool,
}

impl LimitedCapture {
    fn new(limit: usize) -> Self {
        Self {
            output: String::new(),
            limit,
            truncated: false,
        }
    }

    fn from_error(error: &str) -> Self {
        let mut capture = Self::new(OUTPUT_LIMIT_BYTES);
        capture.push_line(error);
        capture
    }

    fn push_line(&mut self, line: &str) {
        if self.output.len() >= self.limit {
            self.truncated = true;
            return;
        }

        let remaining = self.limit - self.output.len();
        let line_with_newline = format!("{line}\n");

        if line_with_newline.len() > remaining {
            self.output.push_str(&line_with_newline[..remaining]);
            self.truncated = true;
        } else {
            self.output.push_str(&line_with_newline);
        }
    }
}

pub fn empty_params() -> Value {
    json!({})
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::domain::{ParameterType, ToolParameter, ValidationRules, default_settings};

    fn echo_env_script() -> &'static str {
        if cfg!(windows) {
            "[Console]::Write($env:name)"
        } else {
            "printf '%s' \"$name\""
        }
    }

    fn sleep_script() -> &'static str {
        if cfg!(windows) {
            "Start-Sleep -Seconds 1"
        } else {
            "sleep 1"
        }
    }

    fn multiline_script() -> &'static str {
        if cfg!(windows) {
            "Write-Output 'first'\r\nWrite-Output 'second'"
        } else {
            "printf '%s\n' first\nprintf '%s\n' second"
        }
    }

    fn large_output_script() -> &'static str {
        if cfg!(windows) {
            "[Console]::Out.Write(('x' * 1100000))"
        } else {
            "yes x | head -n 700000"
        }
    }

    fn test_tool(script_body: &str, timeout_ms: i64) -> Tool {
        let tool_id = Uuid::new_v4();

        Tool {
            id: tool_id,
            name: "echo_env".to_string(),
            description: "Echoes env".to_string(),
            category: None,
            script_body: Some(script_body.to_string()),
            script_path: None,
            working_dir: None,
            working_dir_expose: false,
            working_dir_required: false,
            timeout_ms,
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            parameters: vec![ToolParameter {
                id: Uuid::new_v4(),
                tool_id,
                name: "name".to_string(),
                parameter_type: ParameterType::String,
                description: None,
                required: true,
                default_value: None,
                validation: ValidationRules::default(),
                position: 0,
            }],
            last_run_at: None,
        }
    }

    #[test]
    fn script_body_uses_platform_default_shell() {
        let tool = test_tool("Write-Output 'ok'", 1_000);
        let command = command_for_tool(&tool).expect("command");

        if cfg!(windows) {
            assert!(
                command
                    .summary
                    .starts_with("powershell.exe -NoProfile -ExecutionPolicy Bypass -File ")
            );
        } else {
            assert!(command.summary.starts_with("sh -c "));
        }
    }

    #[tokio::test]
    async fn parameter_values_are_passed_as_literal_env_vars() {
        let tool = test_tool(echo_env_script(), 1_000);
        let params = json!({ "name": "; echo injected" });
        let plan =
            build_execution_plan(&tool, &params, None, &default_settings()).expect("valid plan");

        let result = execute_plan(&tool, &plan, None).await.expect("execution");

        assert_eq!(result.stdout.trim_end(), "; echo injected");
        assert_eq!(result.status, ExecutionStatus::Success);
    }

    #[tokio::test]
    async fn script_body_runs_multiple_lines() {
        let tool = test_tool(multiline_script(), 1_000);
        let plan = build_execution_plan(&tool, &json!({ "name": "x" }), None, &default_settings())
            .expect("valid plan");

        let result = execute_plan(&tool, &plan, None).await.expect("execution");

        assert_eq!(result.status, ExecutionStatus::Success);
        assert_eq!(
            result.stdout.lines().collect::<Vec<_>>(),
            ["first", "second"]
        );
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn windows_script_body_preserves_utf8_output() {
        let tool = test_tool("[Console]::Write('çğıöşü')", 1_000);
        let plan = build_execution_plan(&tool, &json!({ "name": "x" }), None, &default_settings())
            .expect("valid plan");

        let result = execute_plan(&tool, &plan, None).await.expect("execution");

        assert_eq!(result.stdout.trim_end(), "çğıöşü");
        assert_eq!(result.status, ExecutionStatus::Success);
    }

    #[tokio::test]
    async fn process_is_marked_timeout_when_limit_is_exceeded() {
        let tool = test_tool(sleep_script(), 25);
        let plan = build_execution_plan(&tool, &json!({ "name": "x" }), None, &default_settings())
            .expect("valid plan");

        let result = execute_plan(&tool, &plan, None).await.expect("execution");

        assert_eq!(result.status, ExecutionStatus::Timeout);
    }

    #[tokio::test]
    async fn stdout_is_truncated_when_limit_is_exceeded() {
        let tool = test_tool(large_output_script(), 5_000);
        let plan = build_execution_plan(&tool, &json!({ "name": "x" }), None, &default_settings())
            .expect("valid plan");

        let result = execute_plan(&tool, &plan, None).await.expect("execution");

        assert!(
            result.stdout_truncated,
            "status={:?} stdout_len={} stderr_len={} stderr={:?}",
            result.status,
            result.stdout.len(),
            result.stderr.len(),
            result.stderr
        );
        assert!(result.stdout.len() <= OUTPUT_LIMIT_BYTES);
    }
}
