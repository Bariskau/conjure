use std::{convert::Infallible, env, path::PathBuf, time::Duration};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        IntoResponse,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{get, patch},
};
use futures::Stream;
use serde_json::{Value, json};
use tokio_stream::{StreamExt, wrappers::BroadcastStream};
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};
use uuid::Uuid;

use crate::{
    AppState,
    domain::{
        CategoryRequest, EnabledRequest, ExecutionSource, LogFilter, NewParameter, NewTool,
        RunToolRequest, ToolsExport, UpdateParameter, UpdateSettings, UpdateTool, input_schema,
    },
    error::AppError,
    executor::{empty_params, execute_and_log, stream_and_log},
};

pub async fn serve(state: AppState, port_override: Option<u16>) -> anyhow::Result<()> {
    let port = http_port(port_override);
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;

    tracing::info!("HTTP server listening on http://127.0.0.1:{port}");
    axum::serve(listener, router(state)).await?;

    Ok(())
}

pub fn router(state: AppState) -> Router {
    let frontend_dir = frontend_dist_dir();
    let frontend_index = frontend_dir.join("index.html");

    if frontend_index.is_file() {
        tracing::info!("Serving UI from {}", frontend_dir.display());
    } else {
        tracing::warn!(
            "Frontend build was not found at {}; API routes are still available",
            frontend_dir.display()
        );
    }

    Router::new()
        .nest("/api", api_router(state))
        .fallback_service(
            ServeDir::new(frontend_dir).not_found_service(ServeFile::new(frontend_index)),
        )
        .layer(CorsLayer::permissive())
}

fn api_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/tools", get(list_tools).post(create_tool))
        .route("/tools/export", get(export_tools))
        .route("/tools/import", get(method_not_allowed).post(import_tools))
        .route(
            "/tools/{tool_id}",
            get(get_tool).put(update_tool).delete(delete_tool),
        )
        .route("/tools/{tool_id}/enabled", patch(set_tool_enabled))
        .route(
            "/tools/{tool_id}/parameters",
            get(list_parameters).post(create_parameter),
        )
        .route(
            "/tools/{tool_id}/parameters/{parameter_id}",
            get(get_parameter)
                .put(update_parameter)
                .delete(delete_parameter),
        )
        .route("/tools/{tool_id}/schema", get(get_tool_schema))
        .route(
            "/tools/{tool_id}/run",
            get(method_not_allowed).post(run_tool),
        )
        .route("/tools/{tool_id}/run/stream", get(stream_tool))
        .route("/logs", get(list_logs))
        .route("/categories", get(list_categories).post(create_category))
        .route(
            "/categories/{category_name}",
            patch(rename_category).delete(delete_category),
        )
        .route("/settings", get(get_settings).put(update_settings))
        .route("/mcp/notifications", get(registry_notifications))
        .fallback(api_not_found)
        .with_state(state)
}

fn http_port(port_override: Option<u16>) -> u16 {
    port_override
        .or_else(|| env::var("CONJURE_HTTP_PORT").ok()?.parse::<u16>().ok())
        .unwrap_or(5174)
}

fn frontend_dist_dir() -> PathBuf {
    if let Some(path) = env_path("CONJURE_FRONTEND_DIST") {
        return path;
    }

    frontend_dist_candidates()
        .into_iter()
        .find(|path| path.join("index.html").is_file())
        .unwrap_or_else(|| PathBuf::from("frontend/dist"))
}

fn frontend_dist_candidates() -> Vec<PathBuf> {
    let mut candidates = vec![PathBuf::from("frontend/dist")];

    if let Some(executable_dir) = executable_dir() {
        candidates.push(executable_dir.join("frontend"));
        candidates.push(executable_dir.join("frontend/dist"));
    }

    if let Some(data_dir) = platform_data_dir() {
        candidates.push(data_dir);
    }

    candidates
}

fn executable_dir() -> Option<PathBuf> {
    env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
}

fn env_path(name: &str) -> Option<PathBuf> {
    match env::var_os(name) {
        Some(value) if !value.is_empty() => Some(PathBuf::from(value)),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
fn platform_data_dir() -> Option<PathBuf> {
    home_dir().map(|home| {
        home.join("Library")
            .join("Application Support")
            .join("Conjure")
            .join("frontend")
    })
}

#[cfg(target_os = "windows")]
fn platform_data_dir() -> Option<PathBuf> {
    env_path("APPDATA").map(|data_dir| data_dir.join("Conjure").join("frontend"))
}

#[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
fn platform_data_dir() -> Option<PathBuf> {
    env_path("XDG_DATA_HOME")
        .or_else(|| home_dir().map(|home| home.join(".local").join("share")))
        .map(|data_dir| data_dir.join("conjure").join("frontend"))
}

#[cfg(any(
    target_os = "macos",
    all(not(target_os = "macos"), not(target_os = "windows"))
))]
fn home_dir() -> Option<PathBuf> {
    env_path("HOME")
}

async fn health() -> Json<Value> {
    Json(json!({ "ok": true }))
}

async fn api_not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}

async fn method_not_allowed() -> StatusCode {
    StatusCode::METHOD_NOT_ALLOWED
}

async fn list_tools(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    Ok(Json(json!({ "tools": state.db.list_tools().await? })))
}

async fn create_tool(
    State(state): State<AppState>,
    Json(payload): Json<NewTool>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let tool = state.db.create_tool(payload).await?;
    refresh_registry(&state).await?;

    Ok((StatusCode::CREATED, Json(json!({ "tool": tool }))))
}

async fn export_tools(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    Ok(Json(json!({ "export": state.db.export_tools().await? })))
}

async fn import_tools(
    State(state): State<AppState>,
    Json(payload): Json<ToolsExport>,
) -> Result<Json<Value>, AppError> {
    let result = state.db.import_tools(payload).await?;
    refresh_registry(&state).await?;

    Ok(Json(json!({ "result": result })))
}

async fn get_tool(
    State(state): State<AppState>,
    Path(tool_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let tool = state
        .db
        .get_tool(tool_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("tool `{tool_id}`")))?;

    Ok(Json(json!({ "tool": tool })))
}

async fn update_tool(
    State(state): State<AppState>,
    Path(tool_id): Path<Uuid>,
    Json(payload): Json<UpdateTool>,
) -> Result<Json<Value>, AppError> {
    let tool = state.db.update_tool(tool_id, payload).await?;
    refresh_registry(&state).await?;

    Ok(Json(json!({ "tool": tool })))
}

async fn delete_tool(
    State(state): State<AppState>,
    Path(tool_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.db.delete_tool(tool_id).await?;
    refresh_registry(&state).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn set_tool_enabled(
    State(state): State<AppState>,
    Path(tool_id): Path<Uuid>,
    Json(payload): Json<EnabledRequest>,
) -> Result<Json<Value>, AppError> {
    let tool = state.db.set_tool_enabled(tool_id, payload.enabled).await?;
    refresh_registry(&state).await?;

    Ok(Json(json!({ "tool": tool })))
}

async fn list_parameters(
    State(state): State<AppState>,
    Path(tool_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(json!({
        "parameters": state.db.list_parameters(tool_id).await?
    })))
}

async fn get_parameter(
    State(state): State<AppState>,
    Path((_tool_id, parameter_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Value>, AppError> {
    let parameter = state
        .db
        .get_parameter(parameter_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("parameter `{parameter_id}`")))?;

    Ok(Json(json!({ "parameter": parameter })))
}

async fn create_parameter(
    State(state): State<AppState>,
    Path(tool_id): Path<Uuid>,
    Json(payload): Json<NewParameter>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let parameter = state.db.create_parameter(tool_id, payload).await?;
    refresh_registry(&state).await?;

    Ok((StatusCode::CREATED, Json(json!({ "parameter": parameter }))))
}

async fn update_parameter(
    State(state): State<AppState>,
    Path((tool_id, parameter_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateParameter>,
) -> Result<Json<Value>, AppError> {
    let parameter = state
        .db
        .update_parameter(tool_id, parameter_id, payload)
        .await?;
    refresh_registry(&state).await?;

    Ok(Json(json!({ "parameter": parameter })))
}

async fn delete_parameter(
    State(state): State<AppState>,
    Path((tool_id, parameter_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    state.db.delete_parameter(tool_id, parameter_id).await?;
    refresh_registry(&state).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn get_tool_schema(
    State(state): State<AppState>,
    Path(tool_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let tool = state
        .db
        .get_tool(tool_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("tool `{tool_id}`")))?;

    Ok(Json(json!({ "inputSchema": input_schema(&tool) })))
}

async fn run_tool(
    State(state): State<AppState>,
    Path(tool_id): Path<Uuid>,
    Json(payload): Json<RunToolRequest>,
) -> Result<Json<Value>, AppError> {
    let tool = require_enabled_tool(&state, tool_id).await?;
    let params = if payload.params.is_null() {
        empty_params()
    } else {
        payload.params
    };
    let result = execute_and_log(
        &state.db,
        tool,
        params,
        ExecutionSource::ManualTest,
        payload.working_dir,
    )
    .await?;

    Ok(Json(json!({ "result": result })))
}

async fn stream_tool(
    State(state): State<AppState>,
    Path(tool_id): Path<Uuid>,
    Query(query): Query<StreamRunQuery>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let tool = require_enabled_tool(&state, tool_id).await?;
    let params = query.params_json()?;
    let stream = stream_and_log(
        state.db.clone(),
        tool,
        params,
        ExecutionSource::ManualTest,
        query.working_dir,
    )
    .await?;

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(10))))
}

async fn list_logs(
    State(state): State<AppState>,
    Query(filter): Query<LogFilter>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(
        json!({ "logs": state.db.list_call_logs(filter).await? }),
    ))
}

async fn list_categories(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    Ok(Json(
        json!({ "categories": state.db.list_categories().await? }),
    ))
}

async fn create_category(
    State(state): State<AppState>,
    Json(payload): Json<CategoryRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let category = state.db.create_category(payload.name).await?;
    let categories = state.db.list_categories().await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({ "category": category, "categories": categories })),
    ))
}

async fn rename_category(
    State(state): State<AppState>,
    Path(category_name): Path<String>,
    Json(payload): Json<CategoryRequest>,
) -> Result<Json<Value>, AppError> {
    let category = state
        .db
        .rename_category(category_name, payload.name)
        .await?;
    let categories = state.db.list_categories().await?;

    Ok(Json(
        json!({ "category": category, "categories": categories }),
    ))
}

async fn delete_category(
    State(state): State<AppState>,
    Path(category_name): Path<String>,
) -> Result<Json<Value>, AppError> {
    state.db.delete_category(category_name).await?;
    let categories = state.db.list_categories().await?;

    Ok(Json(json!({ "categories": categories })))
}

async fn get_settings(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    Ok(Json(json!({ "settings": state.db.get_settings().await? })))
}

async fn update_settings(
    State(state): State<AppState>,
    Json(payload): Json<UpdateSettings>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(
        json!({ "settings": state.db.update_settings(payload).await? }),
    ))
}

async fn registry_notifications(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let receiver = state.registry.subscribe();
    let stream = BroadcastStream::new(receiver).filter_map(|notification| match notification {
        Ok(notification) => Some(Ok(Event::default()
            .event("tools_list_changed")
            .data(serde_json::to_string(&notification).unwrap_or_default()))),
        Err(_) => None,
    });

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(10)))
}

async fn refresh_registry(state: &AppState) -> Result<(), AppError> {
    state.registry.sync_from_db(&state.db).await?;
    state.registry.notify_tools_changed();

    Ok(())
}

async fn require_enabled_tool(
    state: &AppState,
    tool_id: Uuid,
) -> Result<crate::domain::Tool, AppError> {
    let tool = state
        .db
        .get_tool(tool_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("tool `{tool_id}`")))?;

    if tool.enabled {
        Ok(tool)
    } else {
        Err(AppError::Validation(format!(
            "tool `{}` is disabled",
            tool.name
        )))
    }
}

#[derive(serde::Deserialize)]
struct StreamRunQuery {
    params: Option<String>,
    working_dir: Option<String>,
}

impl StreamRunQuery {
    fn params_json(&self) -> Result<Value, AppError> {
        match &self.params {
            Some(params) => Ok(serde_json::from_str(params)?),
            None => Ok(empty_params()),
        }
    }
}

impl IntoResponse for StreamRunQuery {
    fn into_response(self) -> axum::response::Response {
        StatusCode::OK.into_response()
    }
}
