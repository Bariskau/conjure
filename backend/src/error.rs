use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("execution error: {0}")]
    Execution(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidRequest(_) | Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Database(_)
            | Self::Execution(_)
            | Self::Serialization(_)
            | Self::Io(_)
            | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = Json(ErrorBody {
            error: self.to_string(),
        });

        (status, body).into_response()
    }
}
