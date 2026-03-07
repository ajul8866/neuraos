//! Ollama local provider — runs models on-device.

use super::{check_status, LlmProvider, ModelInfo, ProviderError};
use crate::router::{CompletionRequest, CompletionResponse};
use crate::streaming::StreamChunk;
use async_trait::async_trait;
use futures::Stream;
use neuraos_types::TokenUsage;
use std::pin::Pin;
use uuid::Uuid;

pub struct OllamaProvider {
    base_url: String,
    client: reqwest::Client,
    models: Vec<ModelInfo>,
}

impl OllamaProvider {
    pub fn new(base_url: impl Into<String>) -> Self {
        let url = base_url.into();
        Self {
            base_url: url,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .build()
                .expect("HTTP client"),
            models: Self::default_models(),
        }
    }

    pub fn default() -> Self {
        Self::new("http://localhost:11434")
    }

    fn default_models() -> Vec<ModelInfo> {
        // Ollama's available models depend on what is pulled locally
        vec![
            ModelInfo { id: "llama3.2".into(), display_name: "LLaMA 3.2".into(),
                context_length: 128000, cost_per_1k_input: 0.0, cost_per_1k_output: 0.0,
                supports_vision: false, supports_function_calling: true, supports_json_mode: true },
            ModelInfo { id: "mistral".into(), display_name: "Mistral 7B".into(),
                context_length: 32768, cost_per_1k_input: 0.0, cost_per_1k_output: 0.0,
                supports_vision: false, supports_function_calling: false, supports_json_mode: false },
            ModelInfo { id: "codellama".into(), display_name: "CodeLlama".into(),
                context_length: 100000, cost_per_1k_input: 0.0, cost_per_1k_output: 0.0,
                supports_vision: false, supports_function_calling: false, supports_json_mode: false },
        ]
    }

    fn is_reachable(&self) -> bool {
        // Quick synchronous check — in practice use an async health endpoint
        true
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str { "ollama" }
    fn models(&self) -> &[ModelInfo] { &self.models }
    fn is_available(&self) -> bool { true } // Always available (local)

    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        let model = req.model.as_deref().unwrap_or("llama3.2");

        // Use /api/chat endpoint (OpenAI-compatible in newer Ollama versions)
        let messages: Vec<serde_json::Value> = req.messages.iter().map(|m| {
            use neuraos_types::{MessageContent, Role};
            let role = match m.role {
                Role::System => "system",
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::Tool => "user",
            };
            let content = match &m.content {
                MessageContent::Text { text } => text.clone(),
                _ => "[complex content]".into(),
            };
            serde_json::json!({ "role": role, "content": content })
        }).collect();

        let body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": false,
            "options": {
                "temperature": req.temperature.unwrap_or(0.7),
                "num_predict": req.max_tokens.unwrap_or(4096),
            }
        });

        let resp = self.client
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::Http { status: 503, body: e.to_string() })?;

        let resp = check_status(resp).await?;
        let json: serde_json::Value = resp.json().await?;

        let content = json.get("message")
            .and_then(|m| m.get("content")).and_then(|c| c.as_str())
            .unwrap_or("").to_string();

        let prompt_tokens = json.get("prompt_eval_count").and_then(|t| t.as_u64()).unwrap_or(0) as u32;
        let completion_tokens = json.get("eval_count").and_then(|t| t.as_u64()).unwrap_or(0) as u32;

        Ok(CompletionResponse {
            id: Uuid::new_v4().to_string(),
            model: model.to_string(),
            provider: "ollama".into(),
            content,
            tool_calls: None,
            finish_reason: "stop".into(),
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
        let mut results = Vec::new();
        for text in texts {
            let body = serde_json::json!({
                "model": "nomic-embed-text",
                "prompt": text,
            });
            let resp = self.client
                .post(format!("{}/api/embeddings", self.base_url))
                .json(&body)
                .send()
                .await
                .map_err(|e| ProviderError::Http { status: 503, body: e.to_string() })?;
            let resp = check_status(resp).await?;
            let json: serde_json::Value = resp.json().await?;
            let emb = json.get("embedding").and_then(|e| e.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_f64().map(|f| f as f32)).collect())
                .unwrap_or_default();
            results.push(emb);
        }
        Ok(results)
    }
}
