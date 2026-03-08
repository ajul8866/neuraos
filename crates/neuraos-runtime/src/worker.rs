// neuraos-runtime/src/worker.rs
// Worker pool for concurrent task execution

use crate::{RuntimeError, RuntimeResult, RuntimeTask, TaskId, TaskState, TaskResult};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::{Semaphore, Notify};
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

pub struct WorkerPool {
    concurrency: usize,
    semaphore: Arc<Semaphore>,
    active_count: Arc<AtomicUsize>,
    shutdown_notify: Arc<Notify>,
}

impl WorkerPool {
    pub fn new(concurrency: usize) -> Self {
        Self {
            concurrency,
            semaphore: Arc::new(Semaphore::new(concurrency)),
            active_count: Arc::new(AtomicUsize::new(0)),
            shutdown_notify: Arc::new(Notify::new()),
        }
    }

    pub async fn execute<F, Fut>(&self, task_id: TaskId, f: F) -> RuntimeResult<TaskResult>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = RuntimeResult<TaskResult>> + Send + 'static,
    {
        let permit = self.semaphore.clone().acquire_owned().await
            .map_err(|_| RuntimeError::Shutdown)?;

        self.active_count.fetch_add(1, Ordering::SeqCst);
        let active = self.active_count.clone();

        let result = tokio::spawn(async move {
            let _permit = permit;
            let res = f().await;
            active.fetch_sub(1, Ordering::SeqCst);
            res
        })
        .await
        .map_err(|e| RuntimeError::SpawnError(e.to_string()))?;

        result
    }

    pub fn active_count(&self) -> usize {
        self.active_count.load(Ordering::SeqCst)
    }

    pub fn available_slots(&self) -> usize {
        self.semaphore.available_permits()
    }

    pub fn is_at_capacity(&self) -> bool {
        self.semaphore.available_permits() == 0
    }

    pub async fn drain(&self) {
        // Wait until all active workers finish
        while self.active_count.load(Ordering::SeqCst) > 0 {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        info!("WorkerPool drained successfully");
    }

    pub fn notify_shutdown(&self) {
        self.shutdown_notify.notify_waiters();
    }
}

/// Worker statistics snapshot
#[derive(Debug, Clone)]
pub struct WorkerStats {
    pub concurrency: usize,
    pub active: usize,
    pub available: usize,
    pub utilization_pct: f32,
}

impl WorkerPool {
    pub fn stats(&self) -> WorkerStats {
        let active = self.active_count.load(Ordering::SeqCst);
        WorkerStats {
            concurrency: self.concurrency,
            active,
            available: self.semaphore.available_permits(),
            utilization_pct: (active as f32 / self.concurrency as f32) * 100.0,
        }
    }
}
