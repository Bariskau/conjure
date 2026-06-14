use std::{convert::Infallible, process::Stdio, time::Duration};

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
    let result = execute_plan(&tool, &plan, None).await?;
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
        let result = execute_plan(&tool, &plan, Some(sender.clone())).await;
        let finished_at = Utc::now();

        let result = match result {
            Ok(result) => result,
            Err(error) => execution_error_result(error),
        };

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
    let mut command = command_for_tool(tool)?;
    command.envs(&plan.env);
    if let Some(working_dir) = &plan.working_dir {
        command.current_dir(working_dir);
    }
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    command.kill_on_drop(true);

    let mut child = command.spawn()?;
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

fn command_for_tool(tool: &Tool) -> Result<Command, AppError> {
    if let Some(script_body) = tool
        .script_body
        .as_deref()
        .filter(|body| !body.trim().is_empty())
    {
        let mut command = Command::new("sh");
        command.arg("-c").arg(script_body);
        return Ok(command);
    }

    if let Some(script_path) = tool
        .script_path
        .as_deref()
        .filter(|path| !path.trim().is_empty())
    {
        return Ok(Command::new(script_path));
    }

    Err(AppError::Validation(
        "tool must include script_body or script_path".to_string(),
    ))
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

fn execution_error_result(error: AppError) -> ExecutionResult {
    ExecutionResult {
        stdout: String::new(),
        stderr: error.to_string(),
        exit_code: None,
        duration_ms: 0,
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

    #[tokio::test]
    async fn parameter_values_are_passed_as_literal_env_vars() {
        let tool = test_tool("printf '%s' \"$name\"", 1_000);
        let params = json!({ "name": "; echo injected" });
        let plan =
            build_execution_plan(&tool, &params, None, &default_settings()).expect("valid plan");

        let result = execute_plan(&tool, &plan, None).await.expect("execution");

        assert_eq!(result.stdout, "; echo injected\n");
        assert_eq!(result.status, ExecutionStatus::Success);
    }

    #[tokio::test]
    async fn process_is_marked_timeout_when_limit_is_exceeded() {
        let tool = test_tool("sleep 1", 25);
        let plan = build_execution_plan(&tool, &json!({ "name": "x" }), None, &default_settings())
            .expect("valid plan");

        let result = execute_plan(&tool, &plan, None).await.expect("execution");

        assert_eq!(result.status, ExecutionStatus::Timeout);
    }

    #[tokio::test]
    async fn stdout_is_truncated_when_limit_is_exceeded() {
        let tool = test_tool("yes x | head -n 700000", 5_000);
        let plan = build_execution_plan(&tool, &json!({ "name": "x" }), None, &default_settings())
            .expect("valid plan");

        let result = execute_plan(&tool, &plan, None).await.expect("execution");

        assert!(result.stdout_truncated);
        assert!(result.stdout.len() <= OUTPUT_LIMIT_BYTES);
    }
}
