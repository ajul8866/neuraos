//! DeepSeek provider — DeepSeek AI API (OpenAI-compatible).

use super::{check_status, json_str, json_u64, messages_to_openai, LlmProvider, ModelInfo, ProviderError};
use crate::router::{CompletionRequest, CompletionResponse};
use crate::streaming::StreamChunk;
use async_trait::async_trait;
use futures::Stream;
use neuraos_types::TokenUsage;
use std::pin::Pin;
use tracing::debug;
use uuid::Uuid;

pub struct DeepSeekProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
    models: Vec<ModelInfo>,
}

impl DeepSeekProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.deepseek.com/v1".into(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("HTTP client"),
            models: Self::static_models(),
        }
    }

    pub fn from_env() -> Option<Self> {
        std::env::var("DEEPSEEK_API_KEY").ok().map(Self::new)
    }

    fn static_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "deepseek-chat".into(),
                display_name: "DeepSeek Chat (V3)".into(),
                context_length: 65536,
                cost_per_1k_input: 0.00014,
                cost_per_1k_output: 0.00028,
                supports_vision: false,
                supports_function_calling: true,
                supports_json_mode: true,
            },
            ModelInfo {
                id: "deepseek-reasoner".into(),
                display_name: "DeepSeek Reasoner (R1)".into(),
                context_length: 65536,
                cost_per_1k_input: 0.00055,
                cost_per_1k_output: 0.00219,
                supports_vision: false,
                supports_function_calling: false,
                supports_json_mode: false,
            },
            ModelInfo {
                id: "deepseek-coder".into(),
                display_name: "DeepSeek Coder".into(),
                context_length: 65536,
                cost_per_1k_input: 0.00014,
                cost_per_1k_output: 0.00028,
                supports_vision: false,
                supports_function_calling: true,
                supports_json_mode: true,
            },
        ]
    }

    fn default_model(&self) -> &str {
        self.models.first().map(|m| m.id.as_str()).unwrap_or("deepseek-chat")
    }
}

#[async_trait]
impl LlmProvider for DeepSeekProvider {
    fn name(&self) -> &str { "deepseek" }
    fn models(&self) -> &[ModelInfo] { &self.models }
    fn is_available(&self) -> bool { !self.api_key.is_empty() }

    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        if self.api_key.is_empty() {
            return Err(ProviderError::NotConfigured);
        }

        let model = req.model.as_deref().unwrap_or_else(|| self.default_model());
        debug!(provider = "deepseek", model, "sending completion request");

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
            provider: "deepseek".into(),
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
        Err(ProviderError::Stream("streaming not yet implemented for deepseek".into()))
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, ProviderError> {
        if self.api_key.is_empty() {
            return Err(ProviderError::NotConfigured);
        }
        let body = serde_json::json!({
            "model": "deepseek-embedding",
            "input": texts,
        });
        let resp = self.client
            .post(format!("{}/embeddings", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;
        let resp = check_status(resp).await?;
        let json: serde_json::Value = resp.json().await?;
        let data = json.get("data").and_then(|d| d.as_array())
            .ok_or_else(|| ProviderError::Stream("no data in embed response".into()))?;
        data.iter().map(|item| {
            let emb = item.get("embedding").and_then(|e| e.as_array())
                .ok_or_else(|| ProviderError::Stream("no embedding".into()))?;
            Ok(emb.iter().filter_map(|v| v.as_f64().map(|f| f as f32)).collect())
        }).collect()
    }
}
