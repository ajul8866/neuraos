// neuraos-types/src/config.rs
// Top-level NeuraOS configuration structure

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuraConfig {
    pub app_name: String,
    pub environment: Environment,
    pub log_level: String,
    pub database: DatabaseConfig,
    pub api: ApiConfig,
    pub llm: LlmConfig,
    pub memory: MemoryConfig,
    pub extras: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://localhost/neuraos".to_string(),
            max_connections: 20,
            min_connections: 2,
            connect_timeout_secs: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub request_timeout_secs: u64,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: 4,
            request_timeout_secs: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub default_provider: String,
    pub default_model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub request_timeout_secs: u64,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            default_provider: "openai".to_string(),
            default_model: "gpt-4o".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            request_timeout_secs: 120,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub vector_dimensions: usize,
    pub max_entries_per_agent: usize,
    pub ttl_days: u32,
    pub consolidation_interval_secs: u64,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            vector_dimensions: 1536,
            max_entries_per_agent: 10_000,
            ttl_days: 90,
            consolidation_interval_secs: 3600,
        }
    }
}

impl Default for NeuraConfig {
    fn default() -> Self {
        Self {
            app_name: "neuraos".to_string(),
            environment: Environment::Development,
            log_level: "info".to_string(),
            database: DatabaseConfig::default(),
            api: ApiConfig::default(),
            llm: LlmConfig::default(),
            memory: MemoryConfig::default(),
            extras: HashMap::new(),
        }
    }
}
