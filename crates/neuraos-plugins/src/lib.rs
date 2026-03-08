//! # neuraos-plugins
//! Dynamic plugin system for NeuraOS -- load native (.so/.dll) or WASM plugins at runtime.

#![warn(missing_docs)]

pub mod loader;
pub mod manifest;
pub mod plugin;
pub mod registry;
pub mod sandbox;

use async_trait::async_trait;
use neuraos_types::NeuraResult;
use serde_json::Value;

/// A plugin extends NeuraOS with dynamically-loaded capabilities.
#[async_trait]
pub trait Plugin: Send + Sync + 'static {
    /// Unique identifier.
    fn id(&self) -> &str;
    /// Human-readable name.
    fn name(&self) -> &str;
    /// Version string.
    fn version(&self) -> &str;
    /// Invoke a named method on the plugin.
    async fn call(&self, method: &str, args: Value) -> NeuraResult<Value>;
}

pub mod plugin {
    //! Core plugin trait re-export.
    pub use super::Plugin;
}

pub mod manifest {
    //! Plugin manifest definition.
    use serde::{Deserialize, Serialize};

    /// Metadata descriptor for a plugin.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PluginManifest {
        /// Unique plugin ID.
        pub id: String,
        /// Human-readable name.
        pub name: String,
        /// Semantic version string.
        pub version: String,
        /// Whether this is a native or WASM plugin.
        pub plugin_type: PluginType,
        /// Path to the plugin binary or WASM module.
        pub entry: String,
    }

    /// Plugin runtime type.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum PluginType {
        /// Native shared library (.so / .dll / .dylib).
        Native,
        /// WebAssembly module.
        Wasm,
    }
}

pub mod registry {
    //! Plugin registry.
    use std::collections::HashMap;
    use super::Plugin;

    /// Registry that maps plugin IDs to their implementations.
    #[derive(Default)]
    pub struct PluginRegistry {
        plugins: HashMap<String, Box<dyn Plugin>>,
    }

    impl PluginRegistry {
        /// Register a new plugin.
        pub fn register(&mut self, plugin: Box<dyn Plugin>) {
            self.plugins.insert(plugin.id().to_string(), plugin);
        }
        /// Get a plugin by ID.
        pub fn get(&self, id: &str) -> Option<&dyn Plugin> {
            self.plugins.get(id).map(|p| p.as_ref())
        }
        /// List all registered plugins as (id, name) pairs.
        pub fn list(&self) -> Vec<(&str, &str)> {
            self.plugins.values().map(|p| (p.id(), p.name())).collect()
        }
    }
}

pub mod loader {
    //! Plugin loading from filesystem.
}

pub mod sandbox {
    //! WASM sandbox for safe plugin execution.
}
