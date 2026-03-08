// neuraos-runtime/src/lib.rs
// NeuraOS Async Runtime & Task Execution Engine

pub mod executor;
pub mod task;
pub mod worker;
pub mod handle;
pub mod lifecycle;

pub use executor::RuntimeExecutor;
pub use task::{RuntimeTask, TaskId, TaskState};
pub use handle::RuntimeHandle;
pub use lifecycle::LifecycleManager;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Task execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    #[error("Runtime shutdown")]
    Shutdown,
    #[error("Spawn error: {0}")]
    SpawnError(String),
    #[error("Timeout exceeded")]
    Timeout,
    #[error("Worker pool exhausted")]
    WorkerExhausted,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;
