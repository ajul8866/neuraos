// neuraos-api/src/error.rs
// Axum-compatible error type with JSON responses

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Service unavailable")]
    ServiceUnavailable,
    #[error("Conflict: {0}")]
    Conflict(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            ApiError::ServiceUnavailable => (StatusCode::SERVICE_UNAVAILABLE, "Service unavailable".to_string()),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
        };
        let body = Json(json!({ "error": message, "status": status.as_u16() }));
        (status, body).into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
