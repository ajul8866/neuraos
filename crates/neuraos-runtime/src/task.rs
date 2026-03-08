// neuraos-runtime/src/task.rs
// Runtime task definitions and state machine

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub type TaskId = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    Pending,
    Queued,
    Running,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
    TimedOut,
}

impl TaskState {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TaskState::Completed | TaskState::Failed(_) | TaskState::Cancelled | TaskState::TimedOut
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPayload {
    AgentAction {
        agent_id: String,
        action: String,
        input: serde_json::Value,
    },
    LlmCall {
        provider: String,
        model: String,
        prompt: String,
        parameters: HashMap<String, serde_json::Value>,
    },
    ToolExecution {
        tool_name: String,
        arguments: HashMap<String, serde_json::Value>,
    },
    MemoryOperation {
        operation: String,
        data: serde_json::Value,
    },
    Custom {
        kind: String,
        data: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeTask {
    pub id: TaskId,
    pub name: String,
    pub state: TaskState,
    pub payload: TaskPayload,
    pub priority: u8,
    pub parent_id: Option<TaskId>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub timeout_secs: Option<u64>,
    pub retries: u32,
    pub max_retries: u32,
    pub result: Option<TaskResult>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl RuntimeTask {
    pub fn new(name: impl Into<String>, payload: TaskPayload) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            state: TaskState::Pending,
            payload,
            priority: 5,
            parent_id: None,
            tags: vec![],
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            timeout_secs: Some(300),
            retries: 0,
            max_retries: 3,
            result: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    pub fn with_parent(mut self, parent_id: TaskId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn mark_running(&mut self) {
        self.state = TaskState::Running;
        self.started_at = Some(Utc::now());
    }

    pub fn mark_completed(&mut self, result: TaskResult) {
        self.state = TaskState::Completed;
        self.completed_at = Some(Utc::now());
        self.result = Some(result);
    }

    pub fn mark_failed(&mut self, error: impl Into<String>) {
        let err = error.into();
        self.state = TaskState::Failed(err.clone());
        self.completed_at = Some(Utc::now());
    }

    pub fn can_retry(&self) -> bool {
        self.retries < self.max_retries
    }
}
