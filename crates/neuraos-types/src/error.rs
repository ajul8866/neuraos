//! neuraos-types — unified error type for the NeuraOS platform.

use thiserror::Error;

/// Platform-wide result alias.
pub type Result<T, E = NeuraError> = std::result::Result<T, E>;

/// Top-level error enum covering all NeuraOS subsystems.
#[derive(Debug, Error)]
pub enum NeuraError {
    // ── Agent ──────────────────────────────────────────────────────────────
    #[error("agent not found: {0}")]
    AgentNotFound(String),

    #[error("agent already exists: {0}")]
    AgentAlreadyExists(String),

    #[error("agent error: {0}")]
    AgentError(String),

    // ── Task ──────────────────────────────────────────────────────────────
    #[error("task not found: {0}")]
    TaskNotFound(String),

    #[error("task failed: {0}")]
    TaskFailed(String),

    #[error("task cancelled")]
    TaskCancelled,

    // ── LLM / Provider ────────────────────────────────────────────────────
    #[error("LLM provider error: {0}")]
    LlmProvider(String),

    #[error("LLM rate limit exceeded")]
    LlmRateLimit,

    #[error("LLM context window exceeded (tokens: {0})")]
    LlmContextOverflow(usize),

    #[error("LLM response parse error: {0}")]
    LlmParse(String),

    // ── Memory ──────────────────────────────────────────────────────────────
    #[error("memory error: {0}")]
    Memory(String),

    #[error("memory entry not found: {0}")]
    MemoryNotFound(String),

    // ── Tool ─────────────────────────────────────────────────────────────────
    #[error("tool not found: {0}")]
    ToolNotFound(String),

    #[error("tool execution error: {0}")]
    ToolExecution(String),

    #[error("tool argument error: {0}")]
    ToolArgument(String),

    // ── Config ──────────────────────────────────────────────────────────
    #[error("config error: {0}")]
    Config(String),

    #[error("config key not found: {0}")]
    ConfigNotFound(String),

    // ── IO / Network ───────────────────────────────────────────────────
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {status} — {body}")]
    Http { status: u16, body: String },

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("deserialization error: {0}")]
    Deserialization(String),

    // ── Storage / DB ────────────────────────────────────────────────────
    #[error("storage error: {0}")]
    Storage(String),

    #[error("record not found: {0}")]
    RecordNotFound(String),

    // ── Auth / Security ────────────────────────────────────────────────
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    // ── Internal ────────────────────────────────────────────────────────
    #[error("internal error: {0}")]
    Internal(String),

    #[error("not implemented: {0}")]
    NotImplemented(String),

    #[error("timeout: {0}")]
    Timeout(String),

    #[error("resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),
}
