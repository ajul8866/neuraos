//! Google Gemini provider.

use super::{check_status, LlmProvider, ModelInfo, ProviderError};
use crate::router::{CompletionRequest, CompletionResponse};
use crate::streaming::StreamChunk;
use async_trait::async_trait;
use futures::Stream;
use neuraos_types::{MessageContent, Role, TokenUsage};
use std::pin::Pin;
use uuid::Uuid;

const GEMINI_BASE: &str = "https://generativelanguage.googleapis.com/v1beta";
const EMBED_MODEL: &str = "text-embedding-004";

pub struct GoogleProvider {
    api_key: String,
    client: reqwest::Client,
    models: Vec<ModelInfo>,
}

impl GoogleProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("HTTP client"),
            models: Self::static_models(),
        }
    }

    pub fn from_env() -> Option<Self> {
        std::env::var("GOOGLE_API_KEY").ok().map(Self::new)
    }

    fn static_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo { id: "gemini-1.5-pro".into(), display_name: "Gemini 1.5 Pro".into(),
                context_length: 1_000_000, cost_per_1k_input: 0.00125, cost_per_1k_output: 0.005,
                supports_vision: true, supports_function_calling: true, supports_json_mode: true },
            ModelInfo { id: "gemini-1.5-flash".into(), display_name: "Gemini 1.5 Flash".into(),
                context_length: 1_000_000, cost_per_1k_input: 0.000075, cost_per_1k_output: 0.0003,
                supports_vision: true, supports_function_calling: true, supports_json_mode: true },
            ModelInfo { id: "gemini-2.0-flash-exp".into(), display_name: "Gemini 2.0 Flash Exp".into(),
                context_length: 1_000_000, cost_per_1k_input: 0.0, cost_per_1k_output: 0.0,
                supports_vision: true, supports_function_calling: true, supports_json_mode: true },
        ]
    }

    fn messages_to_gemini(req: &CompletionRequest) -> (Option<String>, Vec<serde_json::Value>) {
        let mut system_instruction: Option<String> = None;
        let mut contents: Vec<serde_json::Value> = Vec::new();

        for msg in &req.messages {
            let text = match &msg.content {
                MessageContent::Text { text } => text.clone(),
                _ => "[complex content]".into(),
            };
            match msg.role {
                Role::System => {
                    system_instruction = Some(text);
                }
                Role::User => {
                    contents.push(serde_json::json!({
                        "role": "user",
                        "parts": [{"text": text}]
                    }));
                }
                Role::Assistant => {
                    contents.push(serde_json::json!({
                        "role": "model",
                        "parts": [{"text": text}]
                    }));
                }
                Role::Tool => {
                    contents.push(serde_json::json!({
                        "role": "user",
                        "parts": [{"text": format!("[Tool] {}", text)}]
                    }));
                }
            }
        }
        (system_instruction, contents)
    }
}

#[async_trait]
impl LlmProvider for GoogleProvider {
    fn name(&self) -> &str { "google" }
    fn models(&self) -> &[ModelInfo] { &self.models }
    fn is_available(&self) -> bool { !self.api_key.is_empty() }

    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        if self.api_key.is_empty() {
            return Err(ProviderError::NotConfigured);
        }
        let model = req.model.as_deref().unwrap_or("gemini-1.5-flash");
        let (system_instruction, contents) = Self::messages_to_gemini(req);

        let mut body = serde_json::json!({
            "contents": contents,
            "generationConfig": {
                "temperature": req.temperature.unwrap_or(0.7),
                "maxOutputTokens": req.max_tokens.unwrap_or(4096),
            }
        });

        if let Some(sys) = system_instruction {
            body["systemInstruction"] = serde_json::json!({
                "parts": [{"text": sys}]
            });
        }

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            GEMINI_BASE, model, self.api_key
        );

        let resp = self.client.post(&url).json(&body).send().await?;
        let resp = check_status(resp).await?;
        let json: serde_json::Value = resp.json().await?;

        let content = json
            .get("candidates").and_then(|c| c.as_array()).and_then(|a| a.first())
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts")).and_then(|p| p.as_array()).and_then(|a| a.first())
            .and_then(|p| p.get("text")).and_then(|t| t.as_str())
            .unwrap_or("").to_string();

        let finish = json
            .get("candidates").and_then(|c| c.as_array()).and_then(|a| a.first())
            .and_then(|c| c.get("finishReason")).and_then(|r| r.as_str())
            .unwrap_or("STOP").to_string();

        let input_tokens = json
            .get("usageMetadata").and_then(|u| u.get("promptTokenCount")).and_then(|t| t.as_u64())
            .unwrap_or(0) as u32;
        let output_tokens = json
            .get("usageMetadata").and_then(|u| u.get("candidatesTokenCount")).and_then(|t| t.as_u64())
            .unwrap_or(0) as u32;

        Ok(CompletionResponse {
            id: Uuid::new_v4().to_string(),
            model: model.to_string(),
            provider: "google".into(),
            content,
            tool_calls: None,
            finish_reason: finish.to_lowercase(),
            usage: TokenUsage::new(input_tokens, output_tokens, 0.0),
            latency_ms: 0,
        })
    }

    async fn stream(&self, req: &CompletionRequest)
        -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, ProviderError>> + Send>>, ProviderError>
    {
        Err(ProviderError::Stream("streaming not yet implemented for google".into()))
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, ProviderError> {
        if self.api_key.is_empty() {
            return Err(ProviderError::NotConfigured);
        }
        let mut all_embeddings = Vec::new();
        for text in texts {
            let url = format!(
                "{}/models/{}:embedContent?key={}",
                GEMINI_BASE, EMBED_MODEL, self.api_key
            );
            let body = serde_json::json!({
                "model": format!("models/{}", EMBED_MODEL),
                "content": { "parts": [{"text": text}] }
            });
            let resp = self.client.post(&url).json(&body).send().await?;
            let resp = check_status(resp).await?;
            let json: serde_json::Value = resp.json().await?;
            let emb = json.get("embedding").and_then(|e| e.get("values")).and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_f64().map(|f| f as f32)).collect())
                .unwrap_or_default();
            all_embeddings.push(emb);
        }
        Ok(all_embeddings)
    }
}
