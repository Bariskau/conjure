use std::sync::Arc;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::mpsc,
};

use crate::{
    AppState,
    domain::{ExecutionSource, input_schema},
    error::AppError,
    executor::{empty_params, execute_and_log},
};

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcMessage {
    jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

pub async fn run_stdio(state: AppState) -> anyhow::Result<()> {
    let state = Arc::new(state);
    let (writer, mut receiver) = mpsc::channel::<JsonRpcMessage>(128);
    let mut notifications = state.registry.subscribe();
    let notification_writer = writer.clone();

    let notification_task = tokio::spawn(async move {
        while notifications.recv().await.is_ok() {
            let _ = notification_writer
                .send(JsonRpcMessage {
                    jsonrpc: "2.0",
                    id: None,
                    result: None,
                    error: None,
                    method: Some("notifications/tools/list_changed"),
                    params: Some(json!({})),
                })
                .await;
        }
    });

    let writer_task = tokio::spawn(async move {
        let mut stdout = tokio::io::stdout();
        while let Some(message) = receiver.recv().await {
            if let Ok(line) = serde_json::to_string(&message) {
                let _ = stdout.write_all(line.as_bytes()).await;
                let _ = stdout.write_all(b"\n").await;
                let _ = stdout.flush().await;
            }
        }
    });

    let stdin = tokio::io::stdin();
    let mut lines = BufReader::new(stdin).lines();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        let request = serde_json::from_str::<JsonRpcRequest>(&line)
            .with_context(|| format!("invalid JSON-RPC line: {line}"))?;
        if let Some(response) = handle_request(state.clone(), request).await {
            writer.send(response).await?;
        }
    }

    notification_task.abort();
    drop(writer);
    let _ = writer_task.await;

    Ok(())
}

async fn handle_request(state: Arc<AppState>, request: JsonRpcRequest) -> Option<JsonRpcMessage> {
    let id = response_id(request.id)?;
    match handle_method(state, &request.method, request.params).await {
        Ok(result) => Some(JsonRpcMessage {
            jsonrpc: "2.0",
            id: Some(id),
            result: Some(result),
            error: None,
            method: None,
            params: None,
        }),
        Err(error) => Some(JsonRpcMessage {
            jsonrpc: "2.0",
            id: Some(id),
            result: None,
            error: Some(JsonRpcError {
                code: -32000,
                message: error.to_string(),
            }),
            method: None,
            params: None,
        }),
    }
}

fn response_id(id: Option<Value>) -> Option<Value> {
    match id {
        Some(value @ (Value::String(_) | Value::Number(_))) => Some(value),
        _ => None,
    }
}

async fn handle_method(
    state: Arc<AppState>,
    method: &str,
    params: Value,
) -> Result<Value, AppError> {
    match method {
        "initialize" => Ok(json!({
            "protocolVersion": "2024-11-05",
            "serverInfo": {
                "name": "conjure-mcp-tool-platform",
                "version": env!("CARGO_PKG_VERSION")
            },
            "capabilities": {
                "tools": { "listChanged": true }
            }
        })),
        "notifications/initialized" => Ok(json!({})),
        "tools/list" => list_tools(state).await,
        "tools/call" => call_tool(state, params).await,
        other => Err(AppError::InvalidRequest(format!(
            "unsupported MCP method `{other}`"
        ))),
    }
}

async fn list_tools(state: Arc<AppState>) -> Result<Value, AppError> {
    let tools = state.db.list_enabled_tools().await?;
    let tools = tools
        .iter()
        .map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": input_schema(tool)
            })
        })
        .collect::<Vec<_>>();

    Ok(json!({ "tools": tools }))
}

async fn call_tool(state: Arc<AppState>, params: Value) -> Result<Value, AppError> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::Validation("tools/call requires params.name".to_string()))?;
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(empty_params);
    let tool = state
        .db
        .get_tool_by_name(name)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("tool `{name}`")))?;

    if !tool.enabled {
        return Err(AppError::Validation(format!("tool `{name}` is disabled")));
    }

    let result = execute_and_log(&state.db, tool, arguments, ExecutionSource::Mcp, None).await?;
    let text = format_mcp_result_text(&result);

    Ok(json!({
        "content": [{ "type": "text", "text": text }],
        "isError": result.status.as_str() != "success"
    }))
}

fn format_mcp_result_text(result: &crate::domain::ExecutionResult) -> String {
    let mut text = String::new();
    if !result.stdout.is_empty() {
        text.push_str(&result.stdout);
    }
    if !result.stderr.is_empty() {
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str("stderr:\n");
        text.push_str(&result.stderr);
    }
    if text.is_empty() {
        text.push_str("(no output)");
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_id_accepts_only_json_rpc_request_ids() {
        assert_eq!(response_id(Some(json!("abc"))), Some(json!("abc")));
        assert_eq!(response_id(Some(json!(7))), Some(json!(7)));
        assert_eq!(response_id(None), None);
        assert_eq!(response_id(Some(Value::Null)), None);
        assert_eq!(response_id(Some(json!({ "bad": true }))), None);
    }

    #[test]
    fn notification_message_has_no_response_fields() {
        let message = JsonRpcMessage {
            jsonrpc: "2.0",
            id: None,
            result: None,
            error: None,
            method: Some("notifications/tools/list_changed"),
            params: Some(json!({})),
        };

        let serialized = serde_json::to_value(message).expect("serialize message");

        assert_eq!(serialized["jsonrpc"], "2.0");
        assert_eq!(serialized["method"], "notifications/tools/list_changed");
        assert!(serialized.get("id").is_none());
        assert!(serialized.get("result").is_none());
        assert!(serialized.get("error").is_none());
    }
}
