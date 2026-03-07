//! Runtime registry of active agent manifests.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::catalog::{catalog, AgentManifest};

/// Runtime registry of active agent manifests.
///
/// Provides thread-safe insertion, lookup, and removal of [`AgentManifest`]
/// entries keyed by agent name slug. Pre-populated from [`catalog()`] on
/// construction via [`AgentRegistry::with_catalog()`].
#[derive(Debug, Default)]
pub struct AgentRegistry {
    inner: RwLock<HashMap<String, Arc<AgentManifest>>>,
}

impl AgentRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a registry pre-populated with all catalog agents.
    pub fn with_catalog() -> Self {
        let registry = Self::new();
        for manifest in catalog() {
            registry.register(manifest);
        }
        registry
    }

    /// Register an agent manifest. Overwrites any existing entry with the same name.
    pub fn register(&self, manifest: AgentManifest) {
        let mut map = self.inner.write().expect("AgentRegistry lock poisoned");
        map.insert(manifest.name.clone(), Arc::new(manifest));
    }

    /// Retrieve a manifest by name slug.
    pub fn get(&self, name: &str) -> Option<Arc<AgentManifest>> {
        let map = self.inner.read().expect("AgentRegistry lock poisoned");
        map.get(name).cloned()
    }

    /// Remove an agent by name slug. Returns the removed manifest if it existed.
    pub fn remove(&self, name: &str) -> Option<Arc<AgentManifest>> {
        let mut map = self.inner.write().expect("AgentRegistry lock poisoned");
        map.remove(name)
    }

    /// Returns a snapshot of all registered manifests sorted by name.
    pub fn list(&self) -> Vec<Arc<AgentManifest>> {
        let map = self.inner.read().expect("AgentRegistry lock poisoned");
        let mut agents: Vec<_> = map.values().cloned().collect();
        agents.sort_by(|a, b| a.name.cmp(&b.name));
        agents
    }

    /// Number of registered agents.
    pub fn len(&self) -> usize {
        let map = self.inner.read().expect("AgentRegistry lock poisoned");
        map.len()
    }

    /// Returns `true` if no agents are registered.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
