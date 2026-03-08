//! Groq provider — ultra-fast inference via Groq Cloud API.

use super::{check_status, json_str, json_u64, messages_to_openai, LlmProvider, ModelInfo, ProviderError};
use crate::router::{CompletionRequest, CompletionResponse};
use crate::streaming::StreamChunk;
use async_trait::async_trait;
use futures::Stream;
use neuraos_types::TokenUsage;
use std::pin::Pin;
use tracing::{debug, warn};
use uuid::Uuid;

pub struct GroqProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
    models: Vec<ModelInfo>,
}

impl GroqProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.groq.com/openai/v1".into(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .expect("HTTP client"),
            models: Self::static_models(),
        }
    }

    pub fn from_env() -> Option<Self> {
        std::env::var("GROQ_API_KEY").ok().map(Self::new)
    }

    fn static_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "llama-3.3-70b-versatile".into(),
                display_name: "Llama 3.3 70B Versatile".into(),
                context_length: 128000,
                cost_per_1k_input: 0.00059,
                cost_per_1k_output: 0.00079,
                supports_vision: false,
                supports_function_calling: true,
                supports_json_mode: true,
            },
            ModelInfo {
                id: "llama-3.1-8b-instant".into(),
                display_name: "Llama 3.1 8B Instant".into(),
                context_length: 128000,
                cost_per_1k_input: 0.00005,
                cost_per_1k_output: 0.00008,
                supports_vision: false,
                supports_function_calling: true,
                supports_json_mode: true,
            },
            ModelInfo {
                id: "mixtral-8x7b-32768".into(),
                display_name: "Mixtral 8x7B".into(),
                context_length: 32768,
                cost_per_1k_input: 0.00024,
                cost_per_1k_output: 0.00024,
                supports_vision: false,
                supports_function_calling: true,
                supports_json_mode: true,
            },
            ModelInfo {
                id: "gemma2-9b-it".into(),
                display_name: "Gemma 2 9B IT".into(),
                context_length: 8192,
                cost_per_1k_input: 0.0002,
                cost_per_1k_output: 0.0002,
                supports_vision: false,
                supports_function_calling: false,
                supports_json_mode: true,
            },
        ]
    }

    fn default_model(&self) -> &str {
        self.models.first().map(|m| m.id.as_str()).unwrap_or("llama-3.3-70b-versatile")
    }
}

#[async_trait]
impl LlmProvider for GroqProvider {
    fn name(&self) -> &str { "groq" }
    fn models(&self) -> &[ModelInfo] { &self.models }
    fn is_available(&self) -> bool { !self.api_key.is_empty() }

    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        if self.api_key.is_empty() {
            return Err(ProviderError::NotConfigured);
        }

        let model = req.model.as_deref().unwrap_or_else(|| self.default_model());
        debug!(provider = "groq", model, "sending completion request");

        let body = serde_json::json!({
            "model": model,
            "messages": messages_to_openai(&req.messages),
            "temperature": req.temperature.unwrap_or(0.7),
            "max_tokens": req.max_tokens.unwrap_or(4096),
            "stream": false,
        });

        let resp = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let resp = check_status(resp).await?;
        let json: serde_json::Value = resp.json().await?;

        let content = json_str(&json, &["choices", "0", "message", "content"])
            .unwrap_or_default();
        let finish = json_str(&json, &["choices", "0", "finish_reason"])
            .unwrap_or_else(|| "stop".into());
        let prompt_tokens = json_u64(&json, &["usage", "prompt_tokens"]).unwrap_or(0) as u32;
        let completion_tokens = json_u64(&json, &["usage", "completion_tokens"]).unwrap_or(0) as u32;

        Ok(CompletionResponse {
            id: json_str(&json, &["id"]).unwrap_or_else(|| Uuid::new_v4().to_string()),
            model: model.to_string(),
            provider: "groq".into(),
            content,
            tool_calls: json.get("choices").and_then(|c| c.get(0))
                .and_then(|c| c.get("message")).and_then(|m| m.get("tool_calls")).cloned(),
            finish_reason: finish,
            usage: TokenUsage::new(prompt_tokens, completion_tokens, 0.0),
            latency_ms: 0,
        })
    }

    async fn stream(&self, req: &CompletionRequest)
        -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, ProviderError>> + Send>>, ProviderError>
    {
        Err(ProviderError::Stream("streaming not yet implemented for groq".into()))
    }

    async fn embed(&self, _texts: &[String]) -> Result<Vec<Vec<f32>>, ProviderError> {
        warn!(provider = "groq", "embed not supported by Groq");
        Err(ProviderError::Stream("Groq does not support embeddings".into()))
    }
}
