// neuraos-types/src/event.rs
// System event types used across the NeuraOS event bus

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    AgentStarted,
    AgentStopped,
    AgentError,
    TaskCreated,
    TaskCompleted,
    TaskFailed,
    MessageReceived,
    MemoryStored,
    MemoryRetrieved,
    ToolCalled,
    ToolCompleted,
    SystemStartup,
    SystemShutdown,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub kind: EventKind,
    pub source: String,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Event {
    pub fn new(kind: EventKind, source: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            kind,
            source: source.into(),
            payload,
            timestamp: Utc::now(),
            correlation_id: None,
            metadata: HashMap::new(),
        }
    }
}
