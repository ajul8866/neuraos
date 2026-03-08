//! neuraos-types — shared domain types for the NeuraOS platform.

pub mod error;

pub use error::{NeuraError, Result};

/// Convenience alias — `NeuraResult<T>` instead of `Result<T, NeuraError>`.
pub type NeuraResult<T> = std::result::Result<T, NeuraError>;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ── Agent ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

impl AgentId {
    pub fn new() -> Self { Self(Uuid::new_v4().to_string()) }
    pub fn from(s: impl Into<String>) -> Self { Self(s.into()) }
}
impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
}
impl Default for AgentId { fn default() -> Self { Self::new() } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability { pub name: String, pub description: String, pub version: String }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus { Idle, Running, Paused, Stopped, Error(String) }

// ── Task ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub String);
impl TaskId {
    pub fn new() -> Self { Self(Uuid::new_v4().to_string()) }
    pub fn from(s: impl Into<String>) -> Self { Self(s.into()) }
}
impl Default for TaskId { fn default() -> Self { Self::new() } }
impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority { Low = 0, Normal = 1, High = 2, Critical = 3 }
impl Default for Priority { fn default() -> Self { Priority::Normal } }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus { Queued, Pending, Running, Completed, Failed(String), Cancelled }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep {
    pub id: String, pub name: String, pub description: String,
    pub tool: Option<String>, pub args: HashMap<String, serde_json::Value>,
    pub depends_on: Vec<String>, pub status: TaskStatus,
}
impl TaskStep {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(), name: name.into(), description: String::new(),
            tool: None, args: HashMap::new(), depends_on: Vec::new(), status: TaskStatus::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String, pub goal: String, pub description: String,
    pub steps: Vec<TaskStep>, pub priority: Priority, pub status: TaskStatus,
    pub dependencies: Vec<String>, pub budget_tokens: Option<u32>,
    pub budget_cost_usd: Option<f64>, pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
impl Task {
    pub fn new(goal: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(), goal: goal.into(), description: description.into(),
            steps: Vec::new(), priority: Priority::Normal, status: TaskStatus::Pending,
            dependencies: Vec::new(), budget_tokens: None, budget_cost_usd: None,
            metadata: HashMap::new(), created_at: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String, pub status: TaskStatus, pub output: Option<serde_json::Value>,
    pub error: Option<String>, pub tokens_used: u32, pub cost_usd: f64, pub duration_ms: u64,
}

// ── Tool ─────────────────────────────────────────────────────────────────────

/// Tool capability — what a tool can do. Used as an enum so agents can list their capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCapability {
    HttpGet,
    HttpPost,
    WebSearch,
    WebScrape,
    BashExec,
    PythonExec,
    ReadFile,
    WriteFile,
    SqlQuery,
    GitDiff,
    SendEmail,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall { pub id: String, pub name: String, pub arguments: serde_json::Value }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String, pub output: serde_json::Value, pub error: Option<String>,
}

// ── Event ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String, pub kind: EventKind, pub source: String,
    pub payload: serde_json::Value, pub timestamp: chrono::DateTime<chrono::Utc>,
}
impl Event {
    pub fn new(kind: EventKind, source: impl Into<String>, payload: serde_json::Value) -> Self {
        Self { id: Uuid::new_v4().to_string(), kind, source: source.into(), payload, timestamp: chrono::Utc::now() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    AgentStarted, AgentStopped, AgentError,
    TaskQueued, TaskStarted, TaskCompleted, TaskFailed, TaskCancelled,
    ToolExecuted, MemoryStored, MemoryRetrieved,
    LlmRequest, LlmResponse,
    BudgetWarning, BudgetExceeded,
    CircuitOpen, CircuitClosed,
    Custom(String),
}
impl std::fmt::Display for EventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { Self::Custom(s) => write!(f, "custom:{s}"), other => write!(f, "{:?}", other) }
    }
}

// ── Policy / RBAC ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyEffect { Allow, Deny, RequireApproval }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule { pub resource: String, pub action: String, pub condition: Option<String> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String, pub name: String, pub description: String,
    pub rules: Vec<PolicyRule>, pub effect: PolicyEffect, pub priority: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDecision {
    Allow,
    Deny { reason: String },
    RequireApproval { approver: String },
}

// ── Message / LLM ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role { System, User, Assistant, Tool }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role, pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")] pub name: Option<String>,
}
impl Message {
    pub fn system(content: impl Into<String>) -> Self { Self { role: Role::System, content: content.into(), name: None } }
    pub fn user(content: impl Into<String>) -> Self { Self { role: Role::User, content: content.into(), name: None } }
    pub fn assistant(content: impl Into<String>) -> Self { Self { role: Role::Assistant, content: content.into(), name: None } }
}

// ── Memory ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoryId(pub String);
impl MemoryId { pub fn new() -> Self { Self(Uuid::new_v4().to_string()) } }
impl Default for MemoryId { fn default() -> Self { Self::new() } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: MemoryId, pub agent_id: AgentId, pub content: String,
    pub tags: Vec<String>, pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ── Config ───────────────────────────────────────────────────────────────────

pub type ConfigMap = HashMap<String, serde_json::Value>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn agent_id_display() { let id = AgentId::from("test-agent"); assert_eq!(id.to_string(), "test-agent"); }
    #[test]
    fn priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Normal);
        assert!(Priority::Normal > Priority::Low);
    }
    #[test]
    fn message_constructors() { let m = Message::user("hello"); assert_eq!(m.role, Role::User); }
    #[test]
    fn task_step_new() { let step = TaskStep::new("analyse"); assert_eq!(step.status, TaskStatus::Pending); }
    #[test]
    fn event_kind_display() {
        assert_eq!(EventKind::TaskQueued.to_string(), "TaskQueued");
        assert_eq!(EventKind::Custom("foo".into()).to_string(), "custom:foo");
    }
}
