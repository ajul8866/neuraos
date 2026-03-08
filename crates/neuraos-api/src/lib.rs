// neuraos-api/src/lib.rs
// NeuraOS HTTP & WebSocket API Server

pub mod router;
pub mod handlers;
pub mod middleware;
pub mod ws;
pub mod state;
pub mod error;

pub use router::create_router;
pub use state::AppState;
pub use error::{ApiError, ApiResult};

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

pub type ApiResult<T> = Result<T, ApiError>;
