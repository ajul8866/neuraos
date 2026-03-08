//! neuraos-types — shared domain types for the NeuraOS platform.

pub mod error;

pub use error::{NeuraError, Result};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ── Agent ────────────────────────────────────────────────────────────────────

/// Unique identifier for an agent instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

impl AgentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
    pub fn from(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for AgentId {
    fn default() -> Self { Self::new() }
}

/// Agent capability descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    pub name:        String,
    pub description: String,
    pub version:     String,
}

/// Runtime status of an agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Idle,
    Running,
    Paused,
    Stopped,
    Error(String),
}

// ── Task ─────────────────────────────────────────────────────────────────────

/// Unique identifier for a task.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub String);

impl TaskId {
    pub fn new() -> Self { Self(Uuid::new_v4().to_string()) }
    pub fn from(s: impl Into<String>) -> Self { Self(s.into()) }
}

impl Default for TaskId {
    fn default() -> Self { Self::new() }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Priority level for scheduling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low    = 0,
    Normal = 1,
    High   = 2,
    Critical = 3,
}

impl Default for Priority {
    fn default() -> Self { Priority::Normal }
}

/// Lifecycle status of a task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    Cancelled,
}

// ── Message / LLM ────────────────────────────────────────────────────────────

/// Chat message roles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// A single chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role:    Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name:    Option<String>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: Role::System, content: content.into(), name: None }
    }
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: Role::User, content: content.into(), name: None }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: Role::Assistant, content: content.into(), name: None }
    }
}

// ── Memory ───────────────────────────────────────────────────────────────────

/// Unique identifier for a memory entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoryId(pub String);

impl MemoryId {
    pub fn new() -> Self { Self(Uuid::new_v4().to_string()) }
}

impl Default for MemoryId {
    fn default() -> Self { Self::new() }
}

/// Memory entry stored in the platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id:         MemoryId,
    pub agent_id:   AgentId,
    pub content:    String,
    pub tags:       Vec<String>,
    pub metadata:   HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ── Tool ─────────────────────────────────────────────────────────────────────

/// Tool call request from an LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id:        String,
    pub name:      String,
    pub arguments: serde_json::Value,
}

/// Result of executing a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub output:       serde_json::Value,
    pub error:        Option<String>,
}

// ── Config ───────────────────────────────────────────────────────────────────

/// Generic key-value config map.
pub type ConfigMap = HashMap<String, serde_json::Value>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_id_display() {
        let id = AgentId::from("test-agent");
        assert_eq!(id.to_string(), "test-agent");
    }

    #[test]
    fn priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Normal);
        assert!(Priority::Normal > Priority::Low);
    }

    #[test]
    fn message_constructors() {
        let m = Message::user("hello");
        assert_eq!(m.role, Role::User);
        assert_eq!(m.content, "hello");
    }
}
