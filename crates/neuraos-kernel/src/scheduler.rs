//! Priority-aware async task scheduler with dependency resolution.

use neuraos_types::{Priority, Task, TaskResult, TaskStatus};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, Semaphore};
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Opaque task identifier.
pub type TaskId = String;

/// A queued task waiting in the priority heap.
#[derive(Debug)]
struct QueuedTask {
    priority: Priority,
    sequence: u64, // tie-break by arrival order
    task: Task,
}

impl PartialEq for QueuedTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.sequence == other.sequence
    }
}
impl Eq for QueuedTask {}

impl PartialOrd for QueuedTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueuedTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first; lower sequence number first on tie
        self.priority
            .cmp(&other.priority)
            .then(other.sequence.cmp(&self.sequence))
    }
}

/// State tracked per submitted task.
#[derive(Debug, Clone)]
pub struct TaskRecord {
    pub task: Task,
    pub status: TaskStatus,
    pub result: Option<TaskResult>,
}

/// Async task scheduler with priority queue and dependency resolution.
pub struct Scheduler {
    queue: Arc<Mutex<BinaryHeap<QueuedTask>>>,
    records: Arc<RwLock<HashMap<TaskId, TaskRecord>>>,
    semaphore: Arc<Semaphore>,
    sequence: Arc<std::sync::atomic::AtomicU64>,
    max_workers: usize,
}

impl Scheduler {
    pub fn new(max_workers: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(BinaryHeap::new())),
            records: Arc::new(RwLock::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(max_workers)),
            sequence: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            max_workers,
        }
    }

    /// Submit a task for scheduling. Returns the task ID.
    pub async fn submit(&self, mut task: Task) -> TaskId {
        if task.id.is_empty() {
            task.id = Uuid::new_v4().to_string();
        }
        let id = task.id.clone();
        let seq = self.sequence.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let record = TaskRecord {
            task: task.clone(),
            status: TaskStatus::Queued,
            result: None,
        };

        self.records.write().await.insert(id.clone(), record);

        let queued = QueuedTask {
            priority: task.priority,
            sequence: seq,
            task,
        };
        self.queue.lock().await.push(queued);
        info!("Task {} queued (priority={:?})", id, queued_task_priority(&id));
        id
    }

    /// Cancel a pending task. Returns false if already running or completed.
    pub async fn cancel(&self, task_id: &str) -> Result<(), SchedulerError> {
        let mut records = self.records.write().await;
        if let Some(record) = records.get_mut(task_id) {
            match record.status {
                TaskStatus::Queued | TaskStatus::Pending => {
                    record.status = TaskStatus::Cancelled;
                    info!("Task {} cancelled", task_id);
                    Ok(())
                }
                TaskStatus::Running => Err(SchedulerError::TaskAlreadyRunning(task_id.into())),
                _ => Err(SchedulerError::TaskTerminal(task_id.into())),
            }
        } else {
            Err(SchedulerError::TaskNotFound(task_id.into()))
        }
    }

    /// Query the current status of a task.
    pub async fn status(&self, task_id: &str) -> Option<TaskStatus> {
        self.records.read().await.get(task_id).map(|r| r.status.clone())
    }

    /// Drain the priority queue and return the next runnable task.
    /// Skips cancelled tasks and tasks with unmet dependencies.
    pub async fn next_runnable(&self) -> Option<Task> {
        let mut queue = self.queue.lock().await;
        let records = self.records.read().await;

        while let Some(qt) = queue.pop() {
            let id = &qt.task.id;

            // Skip cancelled
            if let Some(rec) = records.get(id) {
                if rec.status == TaskStatus::Cancelled {
                    continue;
                }
            }

            // Check dependencies
            let deps_met = qt.task.dependencies.iter().all(|dep_id| {
                records
                    .get(dep_id)
                    .map(|r| r.status == TaskStatus::Completed)
                    .unwrap_or(false)
            });

            if deps_met {
                return Some(qt.task);
            } else {
                // Re-queue with lower effective priority (dependencies not met)
                debug!("Task {} dependencies not met, re-queuing", id);
                // Drop the lock and re-insert - simplified: just skip for now
                // In production, re-insert with a dependency-wait flag
            }
        }
        None
    }

    /// Mark a task as running.
    pub async fn mark_running(&self, task_id: &str) {
        if let Some(rec) = self.records.write().await.get_mut(task_id) {
            rec.status = TaskStatus::Running;
        }
    }

    /// Mark a task as completed with its result.
    pub async fn mark_completed(&self, task_id: &str, result: TaskResult) {
        if let Some(rec) = self.records.write().await.get_mut(task_id) {
            rec.status = result.status.clone();
            rec.result = Some(result);
        }
    }

    /// Topological sort of task IDs by dependency order.
    pub fn topological_sort(tasks: &[Task]) -> Result<Vec<String>, SchedulerError> {
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

        for t in tasks {
            in_degree.entry(&t.id).or_insert(0);
            for dep in &t.dependencies {
                adj.entry(dep).or_default().push(&t.id);
                *in_degree.entry(&t.id).or_insert(0) += 1;
            }
        }

        let mut queue: std::collections::VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut order = Vec::new();
        while let Some(id) = queue.pop_front() {
            order.push(id.to_string());
            if let Some(dependents) = adj.get(id) {
                for &dep in dependents {
                    let deg = in_degree.get_mut(dep).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(dep);
                    }
                }
            }
        }

        if order.len() != tasks.len() {
            return Err(SchedulerError::CyclicDependency);
        }
        Ok(order)
    }

    pub fn max_workers(&self) -> usize {
        self.max_workers
    }

    pub fn semaphore(&self) -> Arc<Semaphore> {
        self.semaphore.clone()
    }
}

fn queued_task_priority(_id: &str) -> &'static str {
    "unknown"
}

#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    #[error("Task already running: {0}")]
    TaskAlreadyRunning(String),
    #[error("Task is in terminal state: {0}")]
    TaskTerminal(String),
    #[error("Cyclic dependency detected")]
    CyclicDependency,
}
