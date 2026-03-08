// neuraos-types/src/agent.rs
// Agent-related shared types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

pub type AgentId = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentKind {
    Assistant,
    Planner,
    Executor,
    Researcher,
    Critic,
    Orchestrator,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Idle,
    Running,
    Waiting,
    Paused,
    Error,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: AgentId,
    pub name: String,
    pub kind: AgentKind,
    pub model: String,
    pub system_prompt: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
    pub tools: Vec<String>,
    pub memory_enabled: bool,
    pub max_iterations: u32,
    pub timeout_secs: u64,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "agent".to_string(),
            kind: AgentKind::Assistant,
            model: "gpt-4o".to_string(),
            system_prompt: None,
            temperature: 0.7,
            max_tokens: 4096,
            tools: vec![],
            memory_enabled: true,
            max_iterations: 10,
            timeout_secs: 120,
            tags: vec![],
            metadata: HashMap::new(),
            created_at: Utc::now(),
        }
    }
}
