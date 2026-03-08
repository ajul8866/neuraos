// neuraos-tools/src/registry.rs
// Tool registry — register, discover, and resolve tools

use crate::{ToolError, ToolResult2, ToolSchema};
use async_trait::async_trait;
use dashmap::DashMap;
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, info};

#[async_trait]
pub trait Tool: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> ToolSchema;
    async fn execute(&self, args: Value) -> ToolResult2<Value>;
}

#[derive(Clone)]
pub struct ToolEntry {
    pub name: String,
    pub description: String,
    pub schema: ToolSchema,
    pub tool: Arc<dyn Tool>,
}

pub struct ToolRegistry {
    tools: Arc<DashMap<String, ToolEntry>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Arc::new(DashMap::new()) }
    }

    pub fn register<T: Tool>(&self, tool: T) {
        let entry = ToolEntry {
            name: tool.name().to_string(),
            description: tool.description().to_string(),
            schema: tool.schema(),
            tool: Arc::new(tool),
        };
        info!("Registering tool: {}", entry.name);
        self.tools.insert(entry.name.clone(), entry);
    }

    pub fn get(&self, name: &str) -> Option<ToolEntry> {
        self.tools.get(name).map(|e| e.clone())
    }

    pub fn list(&self) -> Vec<ToolEntry> {
        self.tools.iter().map(|e| e.clone()).collect()
    }

    pub fn count(&self) -> usize {
        self.tools.len()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self { Self::new() }
}
