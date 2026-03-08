//! Ollama provider — local LLM inference via Ollama REST API.

use super::{check_status, json_str, json_u64, messages_to_openai, LlmProvider, ModelInfo, ProviderError};
use crate::router::{CompletionRequest, CompletionResponse};
use crate::streaming::StreamChunk;
use async_trait::async_trait;
use futures::Stream;
use neuraos_types::TokenUsage;
use std::pin::Pin;
use tracing::debug;
use uuid::Uuid;

pub struct OllamaProvider {
    base_url: String,
    client: reqwest::Client,
    models: Vec<ModelInfo>,
}

impl OllamaProvider {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .build()
                .expect("HTTP client"),
            models: Self::static_models(),
        }
    }

    /// Creates an Ollama provider from environment variables.
    /// Uses OLLAMA_BASE_URL or defaults to http://localhost:11434
    pub fn from_env() -> Self {
        let base_url = std::env::var("OLLAMA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434".into());
        Self::new(base_url)
    }

    fn static_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "llama3.2".into(),
                display_name: "Llama 3.2 (local)".into(),
                context_length: 131072,
                cost_per_1k_input: 0.0,
                cost_per_1k_output: 0.0,
                supports_vision: false,
                supports_function_calling: false,
                supports_json_mode: true,
            },
            ModelInfo {
                id: "mistral".into(),
                display_name: "Mistral 7B (local)".into(),
                context_length: 32768,
                cost_per_1k_input: 0.0,
                cost_per_1k_output: 0.0,
                supports_vision: false,
                supports_function_calling: false,
                supports_json_mode: true,
            },
            ModelInfo {
                id: "gemma2".into(),
                display_name: "Gemma 2 (local)".into(),
                context_length: 8192,
                cost_per_1k_input: 0.0,
                cost_per_1k_output: 0.0,
                supports_vision: false,
                supports_function_calling: false,
                supports_json_mode: false,
            },
            ModelInfo {
                id: "qwen2.5-coder".into(),
                display_name: "Qwen 2.5 Coder (local)".into(),
                context_length: 32768,
                cost_per_1k_input: 0.0,
                cost_per_1k_output: 0.0,
                supports_vision: false,
                supports_function_calling: false,
                supports_json_mode: true,
            },
        ]
    }

    fn default_model(&self) -> &str {
        self.models.first().map(|m| m.id.as_str()).unwrap_or("llama3.2")
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str { "ollama" }
    fn models(&self) -> &[ModelInfo] { &self.models }
    fn is_available(&self) -> bool { true } // always try; server may be local

    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        let model = req.model.as_deref().unwrap_or_else(|| self.default_model());
        debug!(provider = "ollama", model, "sending completion request");

        // Use OpenAI-compatible /v1/chat/completions endpoint (Ollama >= 0.1.24)
        let body = serde_json::json!({
            "model": model,
            "messages": messages_to_openai(&req.messages),
            "temperature": req.temperature.unwrap_or(0.7),
            "stream": false,
            "options": {
                "num_predict": req.max_tokens.unwrap_or(4096),
            }
        });

        let resp = self.client
            .post(format!("{}/v1/chat/completions", self.base_url))
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
            provider: "ollama".into(),
            content,
            tool_calls: None,
            finish_reason: finish,
            usage: TokenUsage::new(prompt_tokens, completion_tokens, 0.0),
            latency_ms: 0,
        })
    }

    async fn stream(&self, req: &CompletionRequest)
        -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, ProviderError>> + Send>>, ProviderError>
    {
        Err(ProviderError::Stream("streaming not yet implemented for ollama".into()))
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, ProviderError> {
        let mut result = Vec::with_capacity(texts.len());
        for text in texts {
            let body = serde_json::json!({
                "model": "nomic-embed-text",
                "prompt": text,
            });
            let resp = self.client
                .post(format!("{}/api/embeddings", self.base_url))
                .json(&body)
                .send()
                .await?;
            let resp = check_status(resp).await?;
            let json: serde_json::Value = resp.json().await?;
            let emb = json.get("embedding").and_then(|e| e.as_array())
                .ok_or_else(|| ProviderError::Stream("no embedding in ollama response".into()))?;
            result.push(emb.iter().filter_map(|v| v.as_f64().map(|f| f as f32)).collect());
        }
        Ok(result)
    }
}
