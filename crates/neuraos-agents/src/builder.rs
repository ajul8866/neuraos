use anyhow::Result;
use std::collections::HashMap;

/// Builder for constructing agent configurations
#[derive(Debug, Default)]
pub struct AgentBuilder {
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    model: Option<String>,
    system_prompt: Option<String>,
    tools: Vec<String>,
    memory_enabled: bool,
    max_iterations: u32,
    metadata: HashMap<String, serde_json::Value>,
}

impl AgentBuilder {
    pub fn new() -> Self {
        Self {
            max_iterations: 10,
            ..Default::default()
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    pub fn tool(mut self, tool: impl Into<String>) -> Self {
        self.tools.push(tool.into());
        self
    }

    pub fn tools(mut self, tools: Vec<String>) -> Self {
        self.tools.extend(tools);
        self
    }

    pub fn memory(mut self, enabled: bool) -> Self {
        self.memory_enabled = enabled;
        self
    }

    pub fn max_iterations(mut self, n: u32) -> Self {
        self.max_iterations = n;
        self
    }

    pub fn metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    pub fn build(self) -> Result<AgentConfig> {
        Ok(AgentConfig {
            id: self.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            name: self.name.unwrap_or_else(|| "unnamed-agent".to_string()),
            description: self.description,
            model: self.model.unwrap_or_else(|| "gpt-4o".to_string()),
            system_prompt: self.system_prompt,
            tools: self.tools,
            memory_enabled: self.memory_enabled,
            max_iterations: self.max_iterations,
            metadata: self.metadata,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub model: String,
    pub system_prompt: Option<String>,
    pub tools: Vec<String>,
    pub memory_enabled: bool,
    pub max_iterations: u32,
    pub metadata: HashMap<String, serde_json::Value>,
}
