// neuraos-types/src/tool.rs
// Tool domain types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type ToolId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub id: ToolId,
    pub name: String,
    pub description: String,
    pub version: String,
    pub category: ToolCategory,
    pub parameters: serde_json::Value, // JSON Schema object
    pub returns: serde_json::Value,
    pub enabled: bool,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    Web,
    Filesystem,
    Database,
    Api,
    Compute,
    Communication,
    Memory,
    Custom,
}
