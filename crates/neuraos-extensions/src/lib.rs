//! neuraos-extensions -- Extension/plugin system for NeuraOS

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub type ExtensionId = String;
pub const EXTENSION_API_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionState { Discovered, Loaded, Active, Disabled, Error(String) }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtensionCapabilities {
    pub agent_actions: bool,
    pub llm_hooks: bool,
    pub memory_backends: bool,
    pub channel_adapters: bool,
    pub http_endpoints: bool,
    pub kernel_hooks: bool,
}

#[async_trait]
pub trait Extension: Send + Sync {
    fn id(&self) -> &ExtensionId;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn capabilities(&self) -> ExtensionCapabilities;
    async fn on_load(&self, ctx: &ExtensionContext) -> anyhow::Result<()>;
    async fn on_enable(&self, ctx: &ExtensionContext) -> anyhow::Result<()>;
    async fn on_disable(&self, ctx: &ExtensionContext) -> anyhow::Result<()>;
    async fn on_unload(&self, ctx: &ExtensionContext) -> anyhow::Result<()>;
    async fn health_check(&self) -> bool { true }
}

#[derive(Clone)]
pub struct ExtensionContext {
    pub extension_id: ExtensionId,
    pub storage: Arc<tokio::sync::RwLock<HashMap<String, serde_json::Value>>>,
    pub config: HashMap<String, serde_json::Value>,
}

impl ExtensionContext {
    pub fn new(extension_id: impl Into<String>) -> Self {
        Self { extension_id: extension_id.into(),
               storage: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
               config: HashMap::new() }
    }
    pub async fn store(&self, key: impl Into<String>, value: serde_json::Value) {
        self.storage.write().await.insert(key.into(), value);
    }
    pub async fn load(&self, key: &str) -> Option<serde_json::Value> {
        self.storage.read().await.get(key).cloned()
    }
}

pub mod manifest {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ExtensionManifest {
        pub id: ExtensionId,
        pub name: String,
        pub version: String,
        pub description: Option<String>,
        pub author: Option<String>,
        pub license: Option<String>,
        pub api_version: u32,
        pub capabilities: ExtensionCapabilities,
        pub entry_point: String,
        #[serde(default)]
        pub dependencies: Vec<ExtensionDependency>,
        #[serde(default)]
        pub config_schema: serde_json::Value,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ExtensionDependency { pub id: ExtensionId, pub version_req: String }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ExtensionMetadata {
        pub manifest: ExtensionManifest,
        pub state: ExtensionState,
        pub loaded_at: Option<chrono::DateTime<chrono::Utc>>,
        pub error: Option<String>,
    }
}

pub mod registry {
    use super::*;
    use tokio::sync::RwLock;
    pub struct ExtensionRegistry {
        extensions: RwLock<HashMap<ExtensionId, Arc<dyn Extension>>>,
        metadata: RwLock<HashMap<ExtensionId, manifest::ExtensionMetadata>>,
    }
    impl ExtensionRegistry {
        pub fn new() -> Self {
            Self { extensions: RwLock::new(HashMap::new()), metadata: RwLock::new(HashMap::new()) }
        }
        pub async fn register(&self, ext: Arc<dyn Extension>, meta: manifest::ExtensionMetadata) {
            let id = ext.id().clone();
            self.extensions.write().await.insert(id.clone(), ext);
            self.metadata.write().await.insert(id, meta);
        }
        pub async fn get(&self, id: &ExtensionId) -> Option<Arc<dyn Extension>> {
            self.extensions.read().await.get(id).cloned()
        }
        pub async fn list_ids(&self) -> Vec<ExtensionId> {
            self.extensions.read().await.keys().cloned().collect()
        }
        pub async fn unregister(&self, id: &ExtensionId) -> bool {
            self.extensions.write().await.remove(id).is_some()
        }
    }
}

pub mod hooks {
    use super::*;
    pub type HookPoint = String;

    #[derive(Debug, Clone)]
    pub struct HookContext {
        pub hook_point: HookPoint,
        pub data: serde_json::Value,
        pub metadata: HashMap<String, String>,
    }

    #[derive(Debug, Clone)]
    pub enum HookResult { Continue(serde_json::Value), Halt(serde_json::Value), Error(String) }

    #[async_trait]
    pub trait Hook: Send + Sync {
        fn hook_point(&self) -> &HookPoint;
        fn priority(&self) -> i32 { 0 }
        async fn execute(&self, ctx: HookContext) -> HookResult;
    }

    pub struct HookRegistry { hooks: tokio::sync::RwLock<HashMap<HookPoint, Vec<Arc<dyn Hook>>>> }
    impl HookRegistry {
        pub fn new() -> Self { Self { hooks: tokio::sync::RwLock::new(HashMap::new()) } }
        pub async fn register(&self, hook: Arc<dyn Hook>) {
            let point = hook.hook_point().clone();
            let mut map = self.hooks.write().await;
            let list = map.entry(point).or_default();
            list.push(hook);
            list.sort_by_key(|h| -h.priority());
        }
        pub async fn fire(&self, point: &HookPoint, mut ctx: HookContext) -> serde_json::Value {
            let map = self.hooks.read().await;
            if let Some(handlers) = map.get(point) {
                for h in handlers {
                    match h.execute(ctx.clone()).await {
                        HookResult::Continue(d) => ctx.data = d,
                        HookResult::Halt(d) => return d,
                        HookResult::Error(e) => tracing::error!("Hook error at {}: {}", point, e),
                    }
                }
            }
            ctx.data
        }
    }
}

pub mod loader {
    pub struct ExtensionLoader { search_paths: Vec<std::path::PathBuf> }
    impl ExtensionLoader {
        pub fn new() -> Self { Self { search_paths: vec![] } }
        pub fn add_search_path(&mut self, path: impl Into<std::path::PathBuf>) {
            self.search_paths.push(path.into());
        }
        pub fn search_paths(&self) -> &[std::path::PathBuf] { &self.search_paths }
    }
}

pub mod sandbox {
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct SandboxConfig {
        pub max_memory_bytes: u64,
        pub max_cpu_ms: u64,
        pub allow_network: bool,
        pub allow_filesystem: bool,
    }
    impl Default for SandboxConfig {
        fn default() -> Self {
            Self { max_memory_bytes: 64 * 1024 * 1024, max_cpu_ms: 5000,
                   allow_network: false, allow_filesystem: false }
        }
    }
}
