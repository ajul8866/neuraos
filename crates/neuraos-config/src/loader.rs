use anyhow::Result;
use config::{Config, Environment, File};

use crate::schema::AppConfig;

pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from file + environment variables.
    /// Priority (highest to lowest): env vars > config file > defaults
    pub fn load(path: Option<&str>) -> Result<AppConfig> {
        dotenvy::dotenv().ok();

        let mut builder = Config::builder()
            // Defaults
            .set_default("kernel.max_agents", 10)?
            .set_default("kernel.max_tasks_per_agent", 100)?
            .set_default("kernel.event_bus_capacity", 1024)?
            .set_default("kernel.scheduler_tick_ms", 100)?
            .set_default("api.host", "0.0.0.0")?
            .set_default("api.port", 8080)?
            .set_default("api.request_timeout_secs", 30)?
            .set_default("database.max_connections", 10)?
            .set_default("database.min_connections", 1)?
            .set_default("database.connect_timeout_secs", 30)?;

        // Optional config file
        if let Some(p) = path {
            builder = builder.add_source(File::with_name(p).required(false));
        } else {
            // Try default locations
            builder = builder
                .add_source(File::with_name("config/default").required(false))
                .add_source(File::with_name("config/local").required(false));
        }

        // Environment variables with NEURAOS_ prefix
        builder = builder.add_source(
            Environment::with_prefix("NEURAOS")
                .separator("__")
                .try_parsing(true),
        );

        let cfg = builder.build()?;
        let app_config: AppConfig = cfg.try_deserialize()?;
        Ok(app_config)
    }
}
