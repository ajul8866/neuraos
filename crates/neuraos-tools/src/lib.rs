// neuraos-tools/src/lib.rs
// NeuraOS Tool Registry & Execution Framework

pub mod registry;
pub mod executor;
pub mod builtin;
pub mod schema;

pub use registry::{ToolRegistry, ToolEntry};
pub use executor::{ToolExecutor, ToolCall, ToolResult};
pub use schema::{ToolSchema, ToolParameter};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),
    #[error("Tool execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),
    #[error("Tool timeout")]
    Timeout,
    #[error("Permission denied for tool: {0}")]
    PermissionDenied(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub type ToolResult2<T> = Result<T, ToolError>;
