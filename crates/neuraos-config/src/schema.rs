use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub kernel: KernelConfig,
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelConfig {
    /// Maximum number of concurrent agents
    pub max_agents: usize,
    /// Maximum tasks queued per agent
    pub max_tasks_per_agent: usize,
    /// Event bus channel capacity
    pub event_bus_capacity: usize,
    /// Scheduler polling interval in milliseconds
    pub scheduler_tick_ms: u64,
}

impl Default for KernelConfig {
    fn default() -> Self {
        Self {
            max_agents: 10,
            max_tasks_per_agent: 100,
            event_bus_capacity: 1024,
            scheduler_tick_ms: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub request_timeout_secs: u64,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub cors_origins: Vec<String>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            request_timeout_secs: 30,
            api_key: None,
            cors_origins: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmConfig {
    #[serde(default)]
    pub default_provider: String,
    #[serde(default)]
    pub openai: Option<OpenAiConfig>,
    #[serde(default)]
    pub anthropic: Option<AnthropicConfig>,
    #[serde(default)]
    pub ollama: Option<OllamaConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    pub api_key: String,
    #[serde(default = "default_openai_model")]
    pub default_model: String,
    #[serde(default = "default_openai_base_url")]
    pub base_url: String,
}

fn default_openai_model() -> String { "gpt-4o".to_string() }
fn default_openai_base_url() -> String { "https://api.openai.com/v1".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    pub api_key: String,
    #[serde(default = "default_anthropic_model")]
    pub default_model: String,
}

fn default_anthropic_model() -> String { "claude-3-5-sonnet-20241022".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    #[serde(default = "default_ollama_base_url")]
    pub base_url: String,
    #[serde(default = "default_ollama_model")]
    pub default_model: String,
}

fn default_ollama_base_url() -> String { "http://localhost:11434".to_string() }
fn default_ollama_model() -> String { "llama3".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default)]
    pub url: Option<String>,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: None,
            max_connections: 10,
            min_connections: 1,
            connect_timeout_secs: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub json: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            json: false,
        }
    }
}

fn default_log_level() -> String { "info".to_string() }
