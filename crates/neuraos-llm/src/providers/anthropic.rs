//! Anthropic Claude provider.

use super::{check_status, json_str, json_u64, LlmProvider, ModelInfo, ProviderError};
use crate::router::{CompletionRequest, CompletionResponse};
use crate::streaming::StreamChunk;
use async_trait::async_trait;
use futures::Stream;
use neuraos_types::{MessageContent, Role, TokenUsage};
use std::pin::Pin;
use uuid::Uuid;

const ANTHROPIC_API: &str = "https://api.anthropic.com/v1";
const ANTHROPIC_VERSION: &str = "2023-06-01";

pub struct AnthropicProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
    models: Vec<ModelInfo>,
}

impl AnthropicProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: ANTHROPIC_API.into(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("HTTP client"),
            models: Self::static_models(),
        }
    }

    pub fn from_env() -> Option<Self> {
        std::env::var("ANTHROPIC_API_KEY").ok().map(Self::new)
    }

    fn static_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo { id: "claude-3-5-sonnet-20241022".into(), display_name: "Claude 3.5 Sonnet".into(),
                context_length: 200000, cost_per_1k_input: 0.003, cost_per_1k_output: 0.015,
                supports_vision: true, supports_function_calling: true, supports_json_mode: true },
            ModelInfo { id: "claude-3-5-haiku-20241022".into(), display_name: "Claude 3.5 Haiku".into(),
                context_length: 200000, cost_per_1k_input: 0.001, cost_per_1k_output: 0.005,
                supports_vision: true, supports_function_calling: true, supports_json_mode: true },
            ModelInfo { id: "claude-3-opus-20240229".into(), display_name: "Claude 3 Opus".into(),
                context_length: 200000, cost_per_1k_input: 0.015, cost_per_1k_output: 0.075,
                supports_vision: true, supports_function_calling: true, supports_json_mode: true },
            ModelInfo { id: "claude-3-haiku-20240307".into(), display_name: "Claude 3 Haiku".into(),
                context_length: 200000, cost_per_1k_input: 0.00025, cost_per_1k_output: 0.00125,
                supports_vision: true, supports_function_calling: true, supports_json_mode: true },
        ]
    }

    fn build_messages_body(&self, req: &CompletionRequest) -> (Option<String>, Vec<serde_json::Value>) {
        let mut system_prompt: Option<String> = None;
        let mut messages: Vec<serde_json::Value> = Vec::new();

        for msg in &req.messages {
            let content = match &msg.content {
                MessageContent::Text { text } => text.clone(),
                _ => "[complex content]".into(),
            };
            match msg.role {
                Role::System => {
                    system_prompt = Some(content);
                }
                Role::User => {
                    messages.push(serde_json::json!({ "role": "user", "content": content }));
                }
                Role::Assistant => {
                    messages.push(serde_json::json!({ "role": "assistant", "content": content }));
                }
                Role::Tool => {
                    messages.push(serde_json::json!({ "role": "user", "content": format!("[Tool Result] {}", content) }));
                }
            }
        }
        (system_prompt, messages)
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str { "anthropic" }
    fn models(&self) -> &[ModelInfo] { &self.models }
    fn is_available(&self) -> bool { !self.api_key.is_empty() }

    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        if self.api_key.is_empty() {
            return Err(ProviderError::NotConfigured);
        }
        let model = req.model.as_deref().unwrap_or("claude-3-5-sonnet-20241022");
        let (system, messages) = self.build_messages_body(req);

        let mut body = serde_json::json!({
            "model": model,
            "messages": messages,
            "max_tokens": req.max_tokens.unwrap_or(4096),
            "temperature": req.temperature.unwrap_or(0.7),
        });

        if let Some(sys) = system {
            body["system"] = serde_json::Value::String(sys);
        }

        if let Some(tools) = &req.tools {
            body["tools"] = tools.clone();
        }

        let resp = self.client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let resp = check_status(resp).await?;
        let json: serde_json::Value = resp.json().await?;

        let content = json.get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|block| {
                if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                    block.get("text").and_then(|t| t.as_str()).map(|s| s.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let finish = json_str(&json, &["stop_reason"]).unwrap_or_else(|| "end_turn".into());
        let input_tokens = json_u64(&json, &["usage", "input_tokens"]).unwrap_or(0) as u32;
        let output_tokens = json_u64(&json, &["usage", "output_tokens"]).unwrap_or(0) as u32;
        let cost = (input_tokens as f64 / 1000.0) * 0.003 + (output_tokens as f64 / 1000.0) * 0.015;

        Ok(CompletionResponse {
            id: json_str(&json, &["id"]).unwrap_or_else(|| Uuid::new_v4().to_string()),
            model: model.to_string(),
            provider: "anthropic".into(),
            content,
            tool_calls: None,
            finish_reason: finish,
            usage: TokenUsage::new(input_tokens, output_tokens, cost),
            latency_ms: 0,
        })
    }

    async fn stream(&self, req: &CompletionRequest)
        -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, ProviderError>> + Send>>, ProviderError>
    {
        Err(ProviderError::Stream("streaming not yet implemented for anthropic".into()))
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, ProviderError> {
        // Anthropic does not expose a public embeddings API — use voyage-3 via openai-compat
        Err(ProviderError::Stream("Anthropic does not support embeddings directly".into()))
    }
}
