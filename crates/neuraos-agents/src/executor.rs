//! Agent executor — drives a single agent run-loop.

use crate::builder::AgentConfig;
use neuraos_types::{NeuraError, NeuraResult};

/// Executes an agent according to its [`AgentConfig`].
pub struct AgentExecutor {
    config: AgentConfig,
}

impl AgentExecutor {
    /// Create a new executor for the given agent configuration.
    pub fn new(config: AgentConfig) -> Self {
        Self { config }
    }

    /// Run the agent. Returns the final response string.
    pub async fn run(&self, input: &str) -> NeuraResult<String> {
        if input.is_empty() {
            return Err(NeuraError::InvalidInput("input must not be empty".into()));
        }
        // Placeholder: real implementation hooks into the LLM + tool loop.
        Ok(format!(
            "[{}] processed: {}",
            self.config.name, input
        ))
    }
}
