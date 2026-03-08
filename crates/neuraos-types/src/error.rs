//! neuraos-types — unified error type for the NeuraOS platform.

use thiserror::Error;

/// Platform-wide result alias.
pub type Result<T, E = NeuraError> = std::result::Result<T, E>;

/// Top-level error enum covering all NeuraOS subsystems.
#[derive(Debug, Error)]
pub enum NeuraError {
    // ── Agent ────────────────────────────────────────────────────────────
    #[error("agent not found: {0}")]
    AgentNotFound(String),

    #[error("agent already exists: {0}")]
    AgentAlreadyExists(String),

    #[error("agent error: {0}")]
    AgentError(String),

    // ── Task ─────────────────────────────────────────────────────────────
    #[error("task not found: {0}")]
    TaskNotFound(String),

    #[error("task failed: {0}")]
    TaskFailed(String),

    #[error("task cancelled")]
    TaskCancelled,

    // ── LLM / Provider ───────────────────────────────────────────────────
    #[error("LLM provider error: {0}")]
    LlmProvider(String),

    #[error("LLM rate limit exceeded")]
    LlmRateLimit,

    #[error("LLM context window exceeded (tokens: {0})")]
    LlmContextOverflow(usize),

    #[error("LLM response parse error: {0}")]
    LlmParse(String),

    // ── Memory ───────────────────────────────────────────────────────────
    #[error("memory error: {0}")]
    Memory(String),

    #[error("memory entry not found: {0}")]
    MemoryNotFound(String),

    // ── Tool ─────────────────────────────────────────────────────────────
    #[error("tool not found: {0}")]
    ToolNotFound(String),

    #[error("tool execution error: {0}")]
    ToolExecution(String),

    #[error("tool argument error: {0}")]
    ToolArgument(String),

    // ── Config ───────────────────────────────────────────────────────────
    #[error("config error: {0}")]
    Config(String),

    #[error("config key not found: {0}")]
    ConfigNotFound(String),

    // ── IO / Network ─────────────────────────────────────────────────────
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {status} — {body}")]
    Http { status: u16, body: String },

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    // ── Database ─────────────────────────────────────────────────────────
    #[error("database error: {0}")]
    Database(String),

    // ── Auth ─────────────────────────────────────────────────────────────
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    // ── Generic ──────────────────────────────────────────────────────────
    #[error("internal error: {0}")]
    Internal(String),

    #[error("not implemented: {0}")]
    NotImplemented(String),

    #[error("timeout after {0}ms")]
    Timeout(u64),
}

impl NeuraError {
    /// Returns true if the error is transient and the operation can be retried.
    pub fn is_retryable(&self) -> bool {
        matches!(self, NeuraError::LlmRateLimit | NeuraError::Timeout(_) | NeuraError::Http { status, .. } if *status >= 500)
    }

    /// HTTP-like status code for this error (useful for API responses).
    pub fn status_code(&self) -> u16 {
        match self {
            NeuraError::AgentNotFound(_)
            | NeuraError::TaskNotFound(_)
            | NeuraError::MemoryNotFound(_)
            | NeuraError::ToolNotFound(_)
            | NeuraError::ConfigNotFound(_) => 404,

            NeuraError::AgentAlreadyExists(_) => 409,

            NeuraError::Unauthorized(_) => 401,
            NeuraError::Forbidden(_)    => 403,

            NeuraError::LlmRateLimit    => 429,
            NeuraError::Timeout(_)      => 408,

            NeuraError::ToolArgument(_)
            | NeuraError::Config(_)
            | NeuraError::LlmParse(_) => 400,

            NeuraError::NotImplemented(_) => 501,

            _ => 500,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_codes() {
        assert_eq!(NeuraError::AgentNotFound("x".into()).status_code(), 404);
        assert_eq!(NeuraError::Unauthorized("x".into()).status_code(), 401);
        assert_eq!(NeuraError::LlmRateLimit.status_code(), 429);
        assert_eq!(NeuraError::Internal("oops".into()).status_code(), 500);
    }

    #[test]
    fn retryable() {
        assert!(NeuraError::LlmRateLimit.is_retryable());
        assert!(NeuraError::Timeout(1000).is_retryable());
        assert!(!NeuraError::AgentNotFound("x".into()).is_retryable());
    }
}
