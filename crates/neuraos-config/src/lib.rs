//! Configuration loading, validation, and hot-reload for NeuraOS.
//!
//! Configuration is layered: defaults → file (TOML/YAML) → environment variables.
//! Environment variables use prefix `NEURA_` and double-underscore separators,
//! e.g. `NEURA_KERNEL__MAX_AGENTS=32`.

#![forbid(unsafe_code)]
#![deny(clippy::all)]

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tracing::{debug, error, info, warn};

pub use neuraos_types::MemoryConfig;

// ─── Re-export ──────────────────────────────────────────────────────────────

/// Top-level NeuraOS configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuraConfig {
    pub kernel: KernelConfig,
    pub memory: MemoryConfig,
    pub llm: LlmConfig,
    pub security: SecurityConfig,
    pub api: ApiConfig,
    pub telemetry: TelemetryConfig,
    pub tools: ToolsConfig,
    pub database: DatabaseConfig,
}

impl Default for NeuraConfig {
    fn default() -> Self {
        Self {
            kernel: KernelConfig::default(),
            memory: MemoryConfig::default(),
            llm: LlmConfig::default(),
            security: SecurityConfig::default(),
            api: ApiConfig::default(),
            telemetry: TelemetryConfig::default(),
            tools: ToolsConfig::default(),
            database: DatabaseConfig::default(),
        }
    }
}

// ─── Kernel ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelConfig {
    /// Maximum number of concurrently running agents.
    pub max_agents: usize,
    /// Maximum task queue depth.
    pub task_queue_depth: usize,
    /// Worker threads for the task scheduler.
    pub scheduler_workers: usize,
    /// Event bus broadcast channel capacity.
    pub event_bus_capacity: usize,
    /// How long to wait before timing out a stalled task.
    pub task_timeout_secs: u64,
    /// Maximum ReAct loop iterations per task.
    pub max_react_iterations: u32,
    /// Enable MCTS planner (slower but better quality).
    pub enable_mcts_planner: bool,
    /// MCTS simulation budget.
    pub mcts_simulations: u32,
}

impl Default for KernelConfig {
    fn default() -> Self {
        Self {
            max_agents: 64,
            task_queue_depth: 10_000,
            scheduler_workers: num_cpus::get(),
            event_bus_capacity: 10_000,
            task_timeout_secs: 300,
            max_react_iterations: 50,
            enable_mcts_planner: true,
            mcts_simulations: 100,
        }
    }
}

// ─── LLM ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Default provider to use.
    pub default_provider: String,
    /// Default model for general tasks.
    pub default_model: String,
    /// Fallback provider chain.
    pub fallback_providers: Vec<String>,
    /// Routing strategy.
    pub routing_strategy: RoutingStrategy,
    pub openai: Option<OpenAiConfig>,
    pub anthropic: Option<AnthropicConfig>,
    pub google: Option<GoogleConfig>,
    pub groq: Option<GroqConfig>,
    pub mistral: Option<MistralConfig>,
    pub together: Option<TogetherConfig>,
    pub ollama: Option<OllamaConfig>,
    pub deepseek: Option<DeepSeekConfig>,
    /// Enable semantic response caching.
    pub cache_enabled: bool,
    /// Cache TTL in seconds.
    pub cache_ttl_secs: u64,
    /// Cosine similarity threshold for cache hits.
    pub cache_similarity_threshold: f32,
    /// Maximum tokens per request.
    pub max_tokens: u32,
    /// Default temperature.
    pub temperature: f32,
    /// Circuit breaker failure threshold before opening.
    pub circuit_breaker_threshold: u32,
    /// Circuit breaker recovery timeout in seconds.
    pub circuit_breaker_timeout_secs: u64,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            default_provider: "openai".into(),
            default_model: "gpt-4o-mini".into(),
            fallback_providers: vec!["anthropic".into(), "groq".into()],
            routing_strategy: RoutingStrategy::CostOptimized,
            openai: None,
            anthropic: None,
            google: None,
            groq: None,
            mistral: None,
            together: None,
            ollama: None,
            deepseek: None,
            cache_enabled: true,
            cache_ttl_secs: 3600,
            cache_similarity_threshold: 0.97,
            max_tokens: 4096,
            temperature: 0.7,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout_secs: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RoutingStrategy {
    #[default]
    CostOptimized,
    LatencyOptimized,
    QualityOptimized,
    RoundRobin,
    Failover,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    pub api_key: String,
    pub base_url: String,
    pub org_id: Option<String>,
    pub project_id: Option<String>,
}

impl Default for OpenAiConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
            base_url: "https://api.openai.com/v1".into(),
            org_id: None,
            project_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    pub api_key: String,
    pub base_url: String,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
            base_url: "https://api.anthropic.com".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleConfig {
    pub api_key: String,
    pub project_id: Option<String>,
}

impl Default for GoogleConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("GOOGLE_API_KEY").unwrap_or_default(),
            project_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroqConfig {
    pub api_key: String,
    pub base_url: String,
}

impl Default for GroqConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("GROQ_API_KEY").unwrap_or_default(),
            base_url: "https://api.groq.com/openai/v1".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MistralConfig {
    pub api_key: String,
    pub base_url: String,
}

impl Default for MistralConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("MISTRAL_API_KEY").unwrap_or_default(),
            base_url: "https://api.mistral.ai/v1".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TogetherConfig {
    pub api_key: String,
    pub base_url: String,
}

impl Default for TogetherConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("TOGETHER_API_KEY").unwrap_or_default(),
            base_url: "https://api.together.xyz/v1".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub base_url: String,
    pub default_model: String,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".into(),
            default_model: "llama3.2".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekConfig {
    pub api_key: String,
    pub base_url: String,
}

impl Default for DeepSeekConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("DEEPSEEK_API_KEY").unwrap_or_default(),
            base_url: "https://api.deepseek.com/v1".into(),
        }
    }
}

// ─── Security ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// JWT signing secret for API auth.
    pub jwt_secret: String,
    /// JWT token TTL in seconds.
    pub jwt_ttl_secs: u64,
    /// Enable prompt injection detection.
    pub injection_detection: bool,
    /// Score threshold [0.0, 1.0] above which requests are blocked.
    pub injection_threshold: f32,
    /// Enable DLP scanning on all inputs/outputs.
    pub dlp_enabled: bool,
    /// Redact PII in audit logs.
    pub audit_redact_pii: bool,
    /// Enable rate limiting on the API.
    pub rate_limit_enabled: bool,
    /// Requests per minute per IP.
    pub rate_limit_rpm: u32,
    /// Requests per minute per API key.
    pub rate_limit_rpm_per_key: u32,
    /// Ed25519 key for signing audit entries (hex-encoded seed).
    pub audit_signing_key: Option<String>,
    /// Allowed CORS origins.
    pub cors_origins: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            jwt_secret: std::env::var("NEURA_JWT_SECRET")
                .unwrap_or_else(|_| "change-me-in-production".into()),
            jwt_ttl_secs: 86400,
            injection_detection: true,
            injection_threshold: 0.7,
            dlp_enabled: true,
            audit_redact_pii: true,
            rate_limit_enabled: true,
            rate_limit_rpm: 120,
            rate_limit_rpm_per_key: 600,
            audit_signing_key: None,
            cors_origins: vec!["*".into()],
        }
    }
}

// ─── API ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    /// Enable TLS (requires cert_path / key_path).
    pub tls_enabled: bool,
    pub cert_path: Option<PathBuf>,
    pub key_path: Option<PathBuf>,
    /// Maximum request body size in bytes.
    pub max_body_bytes: usize,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// Enable Swagger/OpenAPI UI at /docs.
    pub enable_docs: bool,
    /// Enable Prometheus metrics at /metrics.
    pub enable_metrics: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".into(),
            port: 8080,
            tls_enabled: false,
            cert_path: None,
            key_path: None,
            max_body_bytes: 10 * 1024 * 1024, // 10 MB
            timeout_secs: 30,
            enable_docs: true,
            enable_metrics: true,
        }
    }
}

impl ApiConfig {
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

// ─── Telemetry ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub log_level: String,
    /// Structured JSON logs (false = pretty-print).
    pub log_json: bool,
    /// OpenTelemetry OTLP endpoint.
    pub otlp_endpoint: Option<String>,
    pub service_name: String,
    pub service_version: String,
    /// Jaeger agent host for trace reporting.
    pub jaeger_host: Option<String>,
    pub jaeger_port: u16,
    /// Metrics reporting interval in seconds.
    pub metrics_interval_secs: u64,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            log_level: "info".into(),
            log_json: false,
            otlp_endpoint: None,
            service_name: "neuraos".into(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            jaeger_host: None,
            jaeger_port: 6831,
            metrics_interval_secs: 15,
        }
    }
}

// ─── Tools ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    /// Default tool execution timeout in seconds.
    pub default_timeout_secs: u32,
    /// Maximum output size captured per tool call (bytes).
    pub max_output_bytes: usize,
    /// Directory where temporary sandbox files are created.
    pub sandbox_dir: PathBuf,
    /// Brave Search API key for web_search tool.
    pub brave_api_key: Option<String>,
    /// Enable SSRF protection on HTTP tool.
    pub ssrf_protection: bool,
    /// Allow shell commands (bash tool).
    pub allow_shell: bool,
    /// Shell command denylist patterns.
    pub shell_denylist: Vec<String>,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            default_timeout_secs: 30,
            max_output_bytes: 1024 * 1024, // 1 MB
            sandbox_dir: PathBuf::from("/tmp/neuraos-sandbox"),
            brave_api_key: std::env::var("BRAVE_API_KEY").ok(),
            ssrf_protection: true,
            allow_shell: true,
            shell_denylist: vec![
                "rm -rf /".into(),
                ":(){ :|:& };:".into(),
                "dd if=/dev/zero".into(),
                "mkfs".into(),
            ],
        }
    }
}

// ─── Database ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// SQLite path for local storage.
    pub sqlite_path: PathBuf,
    /// Optional PostgreSQL DSN for production.
    pub postgres_dsn: Option<String>,
    /// Redis URL for caching and rate limiting.
    pub redis_url: Option<String>,
    /// Connection pool size.
    pub pool_size: u32,
    /// Connection timeout in seconds.
    pub connect_timeout_secs: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            sqlite_path: PathBuf::from("data/neuraos.db"),
            postgres_dsn: std::env::var("DATABASE_URL").ok(),
            redis_url: std::env::var("REDIS_URL").ok(),
            pool_size: 10,
            connect_timeout_secs: 10,
        }
    }
}

// ─── ConfigLoader ────────────────────────────────────────────────────────────

/// Loads and validates NeuraOS configuration from multiple sources.
pub struct ConfigLoader {
    config_path: Option<PathBuf>,
}

impl ConfigLoader {
    pub fn new() -> Self {
        Self { config_path: None }
    }

    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config_path = Some(path.into());
        self
    }

    /// Load configuration layering: defaults → file → environment.
    pub fn load(&self) -> Result<NeuraConfig, ConfigError> {
        use config::{Config, Environment, File, FileFormat};

        let mut builder = Config::builder();

        // Layer 1: compiled defaults (serialised default struct → JSON)
        let defaults = serde_json::to_string(&NeuraConfig::default())
            .map_err(|e| ConfigError::Serialization(e.to_string()))?;
        builder = builder.add_source(config::File::from_str(&defaults, FileFormat::Json));

        // Layer 2: config file
        if let Some(path) = &self.config_path {
            if path.exists() {
                info!("Loading config from {}", path.display());
                builder = builder.add_source(File::from(path.as_path()).required(false));
            } else {
                warn!("Config file not found: {}", path.display());
            }
        } else {
            // Try well-known locations
            for candidate in &["config/default.toml", "neuraos.toml", "/etc/neuraos/config.toml"] {
                let p = PathBuf::from(candidate);
                if p.exists() {
                    info!("Loading config from {}", p.display());
                    builder = builder.add_source(File::from(p.as_path()).required(false));
                    break;
                }
            }
        }

        // Layer 3: environment variables (NEURA_ prefix, __ separator)
        builder = builder.add_source(
            Environment::with_prefix("NEURA")
                .separator("__")
                .try_parsing(true),
        );

        let cfg = builder
            .build()
            .map_err(|e| ConfigError::Load(e.to_string()))?;

        let neura: NeuraConfig = cfg
            .try_deserialize()
            .map_err(|e| ConfigError::Deserialize(e.to_string()))?;

        neura.validate()?;
        Ok(neura)
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl NeuraConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.api.port == 0 {
            return Err(ConfigError::Validation("api.port must not be 0".into()));
        }
        if self.kernel.max_agents == 0 {
            return Err(ConfigError::Validation(
                "kernel.max_agents must be > 0".into(),
            ));
        }
        if self.security.jwt_secret == "change-me-in-production" {
            warn!("Using default JWT secret — set NEURA_SECURITY__JWT_SECRET in production!");
        }
        if self.llm.temperature < 0.0 || self.llm.temperature > 2.0 {
            return Err(ConfigError::Validation(
                "llm.temperature must be in [0.0, 2.0]".into(),
            ));
        }
        Ok(())
    }
}

// ─── ConfigWatcher ────────────────────────────────────────────────────────────

/// Watches a config file and broadcasts new config on change.
pub struct ConfigWatcher {
    path: PathBuf,
    sender: watch::Sender<Arc<NeuraConfig>>,
    pub receiver: watch::Receiver<Arc<NeuraConfig>>,
}

impl ConfigWatcher {
    pub fn new(path: impl Into<PathBuf>, initial: NeuraConfig) -> Self {
        let (sender, receiver) = watch::channel(Arc::new(initial));
        Self {
            path: path.into(),
            sender,
            receiver,
        }
    }

    /// Start background task that reloads config when the file changes.
    pub fn start(self) -> tokio::task::JoinHandle<()> {
        let path = self.path.clone();
        let sender = self.sender;

        tokio::spawn(async move {
            use notify::{Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher};
            use tokio::sync::mpsc;

            let (tx, mut rx) = mpsc::channel(16);

            let mut watcher: RecommendedWatcher =
                match Watcher::new(
                    move |res: Result<Event, _>| {
                        if let Ok(event) = res {
                            let _ = tx.blocking_send(event);
                        }
                    },
                    NotifyConfig::default(),
                ) {
                    Ok(w) => w,
                    Err(e) => {
                        error!("Failed to create file watcher: {e}");
                        return;
                    }
                };

            if let Err(e) = watcher.watch(&path, RecursiveMode::NonRecursive) {
                error!("Failed to watch config file: {e}");
                return;
            }

            info!("Watching config file for changes: {}", path.display());

            while let Some(_event) = rx.recv().await {
                debug!("Config file changed, reloading…");
                tokio::time::sleep(Duration::from_millis(200)).await; // debounce

                let loader = ConfigLoader::new().with_path(&path);
                match loader.load() {
                    Ok(new_cfg) => {
                        info!("Config reloaded successfully");
                        let _ = sender.send(Arc::new(new_cfg));
                    }
                    Err(e) => {
                        error!("Config reload failed: {e}");
                    }
                }
            }
        })
    }
}

// ─── Error ───────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Config load error: {0}")]
    Load(String),
    #[error("Config deserialize error: {0}")]
    Deserialize(String),
    #[error("Config validation error: {0}")]
    Validation(String),
    #[error("Config serialization error: {0}")]
    Serialization(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
