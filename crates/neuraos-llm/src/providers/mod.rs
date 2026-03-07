//! LLM provider trait and all provider implementations.

use crate::router::{CompletionRequest, CompletionResponse};
use crate::streaming::StreamChunk;
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

pub mod anthropic;
pub mod deepseek;
pub mod google;
pub mod groq;
pub mod mistral;
pub mod ollama;
pub mod openai;
pub mod together;

pub use anthropic::AnthropicProvider;
pub use deepseek::DeepSeekProvider;
pub use google::GoogleProvider;
pub use groq::GroqProvider;
pub use mistral::MistralProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenAiProvider;
pub use together::TogetherProvider;

/// Information about a single LLM model.
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub context_length: u32,
    pub cost_per_1k_input: f64,
    pub cost_per_1k_output: f64,
    pub supports_vision: bool,
    pub supports_function_calling: bool,
    pub supports_json_mode: bool,
}

/// Provider trait — all LLM backends implement this.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    fn models(&self) -> &[ModelInfo];
    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, ProviderError>;
    async fn stream(&self, req: &CompletionRequest)
        -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, ProviderError>> + Send>>, ProviderError>;
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, ProviderError>;
    fn is_available(&self) -> bool;
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("HTTP error {status}: {body}")]
    Http { status: u16, body: String },
    #[error("Rate limited, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
    #[error("Authentication failed")]
    AuthFailed,
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Context length exceeded")]
    ContextLengthExceeded,
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Stream error: {0}")]
    Stream(String),
    #[error("Not configured (missing API key)")]
    NotConfigured,
}

// ─── Shared HTTP helpers ───────────────────────────────────────────────────────

/// Map reqwest status to ProviderError.
pub(crate) async fn check_status(resp: reqwest::Response) -> Result<reqwest::Response, ProviderError> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    if status.as_u16() == 429 {
        return Err(ProviderError::RateLimited { retry_after_secs: 60 });
    }
    if status.as_u16() == 401 || status.as_u16() == 403 {
        return Err(ProviderError::AuthFailed);
    }
    let body = resp.text().await.unwrap_or_default();
    Err(ProviderError::Http { status: status.as_u16(), body })
}

/// Extract text from nested JSON path.
pub(crate) fn json_str(v: &serde_json::Value, path: &[&str]) -> Option<String> {
    let mut cur = v;
    for &key in path {
        cur = cur.get(key)?;
    }
    cur.as_str().map(|s| s.to_string())
}

pub(crate) fn json_u64(v: &serde_json::Value, path: &[&str]) -> Option<u64> {
    let mut cur = v;
    for &key in path {
        cur = cur.get(key)?;
    }
    cur.as_u64()
}

/// Convert NeuraOS Message list to OpenAI-compatible JSON.
pub(crate) fn messages_to_openai(messages: &[neuraos_types::Message]) -> serde_json::Value {
    use neuraos_types::MessageContent;
    let arr: Vec<serde_json::Value> = messages.iter().map(|m| {
        let role = match m.role {
            neuraos_types::Role::System => "system",
            neuraos_types::Role::User => "user",
            neuraos_types::Role::Assistant => "assistant",
            neuraos_types::Role::Tool => "tool",
        };
        let content = match &m.content {
            MessageContent::Text { text } => serde_json::Value::String(text.clone()),
            _ => serde_json::Value::String("[complex content]".into()),
        };
        serde_json::json!({ "role": role, "content": content })
    }).collect();
    serde_json::Value::Array(arr)
}
