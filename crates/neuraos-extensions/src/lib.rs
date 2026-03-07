//! # neuraos-extensions
//! Third-party extension and integration system for NeuraOS.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod extension;
pub mod loader;
pub mod manifest;
pub mod registry;

use async_trait::async_trait;
use neuraos_types::NeuraResult;

/// An extension adds new capabilities to NeuraOS.
#[async_trait]
pub trait Extension: Send + Sync + 'static {
    /// Unique identifier.
    fn id(&self) -> &str;
    /// Human-readable name.
    fn name(&self) -> &str;
    /// Version string.
    fn version(&self) -> &str;
    /// Load and initialize the extension.
    async fn load(&self) -> NeuraResult<()>;
    /// Unload and clean up the extension.
    async fn unload(&self) -> NeuraResult<()>;
}

pub mod extension {
    //! Core extension trait re-export.
    pub use super::Extension;
}

pub mod manifest {
    //! Extension manifest definition.
    use serde::{Deserialize, Serialize};

    /// Metadata descriptor for an extension.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ExtensionManifest {
        /// Unique extension ID.
        pub id: String,
        /// Human-readable name.
        pub name: String,
        /// Semantic version string.
        pub version: String,
        /// Description of the extension.
        pub description: String,
        /// Entry point (e.g. shared library path or WASM module).
        pub entry_point: String,
    }
}

pub mod registry {
    //! Extension registry.
    use std::collections::HashMap;
    use super::Extension;

    /// Registry that maps extension IDs to their implementations.
    #[derive(Default)]
    pub struct ExtensionRegistry {
        extensions: HashMap<String, Box<dyn Extension>>,
    }

    impl ExtensionRegistry {
        /// Register a new extension.
        pub fn register(&mut self, ext: Box<dyn Extension>) {
            self.extensions.insert(ext.id().to_string(), ext);
        }
        /// Get an extension by ID.
        pub fn get(&self, id: &str) -> Option<&dyn Extension> {
            self.extensions.get(id).map(|e| e.as_ref())
        }
    }
}

pub mod loader {
    //! Extension loading from disk or remote registry.
}
