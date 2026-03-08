pub mod env;
pub mod loader;
pub mod schema;

pub use loader::ConfigLoader;
pub use schema::{AppConfig, KernelConfig, ApiConfig, LlmConfig, DatabaseConfig};

use anyhow::Result;
use once_cell::sync::OnceCell;

static GLOBAL_CONFIG: OnceCell<AppConfig> = OnceCell::new();

/// Initialize global configuration from file + environment
pub fn init(config_path: Option<&str>) -> Result<()> {
    let cfg = ConfigLoader::load(config_path)?;
    GLOBAL_CONFIG.set(cfg).map_err(|_| anyhow::anyhow!("Config already initialized"))?;
    Ok(())
}

/// Access the global config (panics if not initialized)
pub fn get() -> &'static AppConfig {
    GLOBAL_CONFIG.get().expect("Config not initialized — call neuraos_config::init() first")
}
