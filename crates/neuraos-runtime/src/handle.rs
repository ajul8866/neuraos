// neuraos-runtime/src/handle.rs
// RuntimeHandle - public API for interacting with the runtime

use crate::{RuntimeError, RuntimeResult, RuntimeTask, TaskId, TaskState};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// A cloneable handle to the runtime executor
#[derive(Clone)]
pub struct RuntimeHandle {
    inner: Arc<RwLock<HandleInner>>,
}

struct HandleInner {
    name: String,
    running: bool,
}

impl RuntimeHandle {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HandleInner {
                name: name.into(),
                running: false,
            })),
        }
    }

    pub async fn runtime_name(&self) -> String {
        self.inner.read().await.name.clone()
    }

    pub async fn is_running(&self) -> bool {
        self.inner.read().await.running
    }

    pub async fn set_running(&self, val: bool) {
        self.inner.write().await.running = val;
    }

    /// Spawn a one-shot task via the handle
    pub async fn spawn_task(&self, task: RuntimeTask) -> RuntimeResult<TaskId> {
        if !self.is_running().await {
            return Err(RuntimeError::Shutdown);
        }
        debug!("RuntimeHandle spawning task: {}", task.name);
        Ok(task.id.clone())
    }
}

impl std::fmt::Debug for RuntimeHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeHandle").finish()
    }
}
