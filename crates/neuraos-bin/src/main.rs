//! neuraos-bin — main entry point for the NeuraOS server binary.

use anyhow::Result;
use clap::Parser;
use tracing::info;

/// NeuraOS — Autonomous Agent Operating System
#[derive(Parser, Debug)]
#[command(name = "neuraos", version, about, long_about = None)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "config/default.toml")]
    config: String,

    /// Bind address for the HTTP API
    #[arg(short, long, default_value = "0.0.0.0:8080")]
    bind: String,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Output logs as JSON
    #[arg(long, default_value_t = false)]
    json_logs: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialise telemetry
    neuraos_telemetry::init_tracing("neuraos", cli.json_logs);

    info!(
        config  = %cli.config,
        bind    = %cli.bind,
        version = env!("CARGO_PKG_VERSION"),
        "NeuraOS starting"
    );

    // Load config
    let config = load_config(&cli.config).await?;
    info!("configuration loaded");

    // Start kernel
    let kernel = neuraos_kernel::Kernel::new(config).await?;
    info!("kernel initialised");

    // Start HTTP API
    neuraos_wire::serve(kernel, &cli.bind).await?;

    Ok(())
}

async fn load_config(path: &str) -> Result<neuraos_kernel::KernelConfig> {
    use std::path::Path;
    let p = Path::new(path);
    if p.exists() {
        let raw = tokio::fs::read_to_string(p).await?;
        let cfg: neuraos_kernel::KernelConfig = toml::from_str(&raw)?;
        Ok(cfg)
    } else {
        tracing::warn!(path = %path, "config file not found, using defaults");
        Ok(neuraos_kernel::KernelConfig::default())
    }
}
