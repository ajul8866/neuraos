// neuraos-types/src/memory.rs
// Memory domain types

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub type MemoryId = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryKind {
    Episodic,
    Semantic,
    Procedural,
    Working,
    LongTerm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: MemoryId,
    pub kind: MemoryKind,
    pub agent_id: Option<String>,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub importance: f32,
    pub access_count: u32,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_accessed: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl MemoryEntry {
    pub fn new(kind: MemoryKind, content: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            kind,
            agent_id: None,
            content: content.into(),
            embedding: None,
            importance: 0.5,
            access_count: 0,
            tags: vec![],
            created_at: Utc::now(),
            last_accessed: None,
            expires_at: None,
            metadata: HashMap::new(),
        }
    }
}
