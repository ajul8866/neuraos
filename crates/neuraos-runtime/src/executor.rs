// neuraos-runtime/src/executor.rs
// Core async runtime executor

use crate::{RuntimeError, RuntimeResult, RuntimeTask, TaskId, TaskState};
use dashmap::DashMap;
use flume::{Receiver, Sender};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Debug)]
pub struct RuntimeConfig {
    pub worker_threads: usize,
    pub max_concurrent_tasks: usize,
    pub task_timeout_secs: u64,
    pub queue_capacity: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus(),
            max_concurrent_tasks: 256,
            task_timeout_secs: 300,
            queue_capacity: 4096,
        }
    }
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

pub struct RuntimeExecutor {
    config: RuntimeConfig,
    tasks: Arc<DashMap<TaskId, RuntimeTask>>,
    task_tx: Sender<RuntimeTask>,
    task_rx: Receiver<RuntimeTask>,
    running: Arc<RwLock<bool>>,
    handles: Arc<DashMap<TaskId, JoinHandle<()>>>,
}

impl RuntimeExecutor {
    pub fn new(config: RuntimeConfig) -> Self {
        let (task_tx, task_rx) = flume::bounded(config.queue_capacity);
        Self {
            config,
            tasks: Arc::new(DashMap::new()),
            task_tx,
            task_rx,
            running: Arc::new(RwLock::new(false)),
            handles: Arc::new(DashMap::new()),
        }
    }

    pub async fn start(&self) -> RuntimeResult<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;
        info!("RuntimeExecutor started with {} worker threads", self.config.worker_threads);
        Ok(())
    }

    pub async fn submit(&self, task: RuntimeTask) -> RuntimeResult<TaskId> {
        let id = task.id.clone();
        self.tasks.insert(id.clone(), task.clone());
        self.task_tx.send_async(task).await
            .map_err(|e| RuntimeError::SpawnError(e.to_string()))?;
        debug!("Task {} submitted to executor", id);
        Ok(id)
    }

    pub async fn cancel(&self, task_id: &TaskId) -> RuntimeResult<()> {
        if let Some(handle) = self.handles.remove(task_id) {
            handle.1.abort();
            info!("Task {} cancelled", task_id);
        }
        if let Some(mut task) = self.tasks.get_mut(task_id) {
            task.state = TaskState::Cancelled;
        }
        Ok(())
    }

    pub async fn get_task(&self, task_id: &TaskId) -> Option<RuntimeTask> {
        self.tasks.get(task_id).map(|t| t.clone())
    }

    pub fn active_task_count(&self) -> usize {
        self.handles.len()
    }

    pub async fn shutdown(&self) -> RuntimeResult<()> {
        let mut running = self.running.write().await;
        *running = false;
        warn!("RuntimeExecutor shutting down, {} tasks active", self.handles.len());
        for entry in self.handles.iter() {
            entry.value().abort();
        }
        self.handles.clear();
        info!("RuntimeExecutor shutdown complete");
        Ok(())
    }
}
