//! neuraos-wings — plugin/extension wings system for NeuraOS.
//!
//! Wings are dynamically-loaded capabilities that extend agent behaviour
//! without modifying the core kernel. Each wing is a self-contained unit
//! that can be enabled/disabled at runtime.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

pub use neuraos_types::{AgentId, NeuraError, Result as NeuraResult};

// ── Wing trait ────────────────────────────────────────────────────────────────

/// A Wing is a pluggable capability that can be attached to an agent.
#[async_trait]
pub trait Wing: Send + Sync + std::fmt::Debug {
    /// Unique name identifying this wing type.
    fn name(&self) -> &str;

    /// Human-readable description.
    fn description(&self) -> &str;

    /// Version string (semver).
    fn version(&self) -> &str;

    /// Called once when the wing is attached to an agent.
    async fn on_attach(&mut self, agent_id: &AgentId) -> NeuraResult<()>;

    /// Called once when the wing is detached from an agent.
    async fn on_detach(&mut self, agent_id: &AgentId) -> NeuraResult<()>;

    /// Execute the wing's action with the given input.
    async fn execute(&self, input: WingInput) -> NeuraResult<WingOutput>;

    /// Whether this wing is currently healthy/available.
    async fn health_check(&self) -> bool { true }
}

// ── Wing I/O ──────────────────────────────────────────────────────────────────

/// Input passed to a wing for execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WingInput {
    pub action:  String,
    pub payload: serde_json::Value,
    pub context: HashMap<String, serde_json::Value>,
}

impl WingInput {
    pub fn new(action: impl Into<String>, payload: serde_json::Value) -> Self {
        Self { action: action.into(), payload, context: HashMap::new() }
    }
}

/// Output returned from a wing execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WingOutput {
    pub success: bool,
    pub data:    serde_json::Value,
    pub error:   Option<String>,
}

impl WingOutput {
    pub fn ok(data: serde_json::Value) -> Self {
        Self { success: true, data, error: None }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self { success: false, data: serde_json::Value::Null, error: Some(msg.into()) }
    }
}

// ── Wing metadata ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WingMetadata {
    pub name:        String,
    pub description: String,
    pub version:     String,
    pub enabled:     bool,
    pub agent_id:    Option<AgentId>,
}

// ── Wing registry ─────────────────────────────────────────────────────────────

/// Registry that manages all available wings and their lifecycle.
#[derive(Debug)]
pub struct WingRegistry {
    wings: RwLock<HashMap<String, Box<dyn Wing>>>,
}

impl WingRegistry {
    pub fn new() -> Self {
        Self { wings: RwLock::new(HashMap::new()) }
    }

    /// Register a new wing.
    pub async fn register(&self, wing: Box<dyn Wing>) -> NeuraResult<()> {
        let name = wing.name().to_string();
        let mut wings = self.wings.write().await;
        if wings.contains_key(&name) {
            return Err(NeuraError::AgentAlreadyExists(format!("wing '{name}' already registered")));
        }
        info!(wing = %name, "wing registered");
        wings.insert(name, wing);
        Ok(())
    }

    /// Attach a wing to an agent (calls on_attach).
    pub async fn attach(&self, wing_name: &str, agent_id: &AgentId) -> NeuraResult<()> {
        let mut wings = self.wings.write().await;
        let wing = wings.get_mut(wing_name)
            .ok_or_else(|| NeuraError::ToolNotFound(wing_name.to_string()))?;
        wing.on_attach(agent_id).await?;
        info!(wing = %wing_name, agent = %agent_id, "wing attached");
        Ok(())
    }

    /// Detach a wing from an agent (calls on_detach).
    pub async fn detach(&self, wing_name: &str, agent_id: &AgentId) -> NeuraResult<()> {
        let mut wings = self.wings.write().await;
        let wing = wings.get_mut(wing_name)
            .ok_or_else(|| NeuraError::ToolNotFound(wing_name.to_string()))?;
        wing.on_detach(agent_id).await?;
        info!(wing = %wing_name, agent = %agent_id, "wing detached");
        Ok(())
    }

    /// Execute a wing action.
    pub async fn execute(&self, wing_name: &str, input: WingInput) -> NeuraResult<WingOutput> {
        let wings = self.wings.read().await;
        let wing = wings.get(wing_name)
            .ok_or_else(|| NeuraError::ToolNotFound(wing_name.to_string()))?;
        wing.execute(input).await
    }

    /// List metadata for all registered wings.
    pub async fn list(&self) -> Vec<WingMetadata> {
        let wings = self.wings.read().await;
        wings.values().map(|w| WingMetadata {
            name:        w.name().to_string(),
            description: w.description().to_string(),
            version:     w.version().to_string(),
            enabled:     true,
            agent_id:    None,
        }).collect()
    }

    /// Health-check all wings; return names of unhealthy ones.
    pub async fn health_check_all(&self) -> Vec<String> {
        let wings = self.wings.read().await;
        let mut unhealthy = Vec::new();
        for (name, wing) in wings.iter() {
            if !wing.health_check().await {
                warn!(wing = %name, "wing health check failed");
                unhealthy.push(name.clone());
            }
        }
        unhealthy
    }
}

impl Default for WingRegistry {
    fn default() -> Self { Self::new() }
}

// ── Built-in no-op wing (for testing) ────────────────────────────────────────

#[derive(Debug)]
pub struct NoopWing {
    name: String,
}

impl NoopWing {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[async_trait]
impl Wing for NoopWing {
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> &str { "No-op wing for testing" }
    fn version(&self) -> &str { "0.1.0" }

    async fn on_attach(&mut self, agent_id: &AgentId) -> NeuraResult<()> {
        info!(wing = %self.name, agent = %agent_id, "noop wing attached");
        Ok(())
    }

    async fn on_detach(&mut self, agent_id: &AgentId) -> NeuraResult<()> {
        info!(wing = %self.name, agent = %agent_id, "noop wing detached");
        Ok(())
    }

    async fn execute(&self, input: WingInput) -> NeuraResult<WingOutput> {
        Ok(WingOutput::ok(serde_json::json!({
            "wing": self.name,
            "action": input.action,
            "result": "noop"
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn registry_register_and_list() {
        let reg = WingRegistry::new();
        reg.register(Box::new(NoopWing::new("test-wing"))).await.unwrap();
        let list = reg.list().await;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "test-wing");
    }

    #[tokio::test]
    async fn noop_wing_execute() {
        let reg = WingRegistry::new();
        reg.register(Box::new(NoopWing::new("noop"))).await.unwrap();
        let input = WingInput::new("ping", serde_json::json!({}));
        let out = reg.execute("noop", input).await.unwrap();
        assert!(out.success);
    }

    #[tokio::test]
    async fn duplicate_registration_fails() {
        let reg = WingRegistry::new();
        reg.register(Box::new(NoopWing::new("dup"))).await.unwrap();
        let result = reg.register(Box::new(NoopWing::new("dup"))).await;
        assert!(result.is_err());
    }
}
