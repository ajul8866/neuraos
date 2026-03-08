// neuraos-api/src/state.rs
// Shared application state for Axum handlers

use std::sync::Arc;
use neuraos_tools::ToolRegistry;

#[derive(Clone)]
pub struct AppState {
    pub tool_registry: Arc<ToolRegistry>,
    pub config: Arc<neuraos_types::config::NeuraOsConfig>,
}

impl AppState {
    pub fn new(
        tool_registry: Arc<ToolRegistry>,
        config: Arc<neuraos_types::config::NeuraOsConfig>,
    ) -> Self {
        Self { tool_registry, config }
    }
}
