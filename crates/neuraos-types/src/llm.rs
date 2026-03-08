// neuraos-types/src/llm.rs
//! LLM provider types for NeuraOS.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported LLM providers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    OpenAi,
    Anthropic,
    Google,
    Groq,
    Mistral,
    Together,
    DeepSeek,
    Ollama,
    /// Any other provider identified by name.
    Other(String),
}

impl std::fmt::Display for LlmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenAi => write!(f, "openai"),
            Self::Anthropic => write!(f, "anthropic"),
            Self::Google => write!(f, "google"),
            Self::Groq => write!(f, "groq"),
            Self::Mistral => write!(f, "mistral"),
            Self::Together => write!(f, "together"),
            Self::DeepSeek => write!(f, "deepseek"),
            Self::Ollama => write!(f, "ollama"),
            Self::Other(s) => write!(f, "{s}"),
        }
    }
}

/// A single message in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role: "system", "user", "assistant", or "tool".
    pub role: String,
    /// Text content of the message.
    pub content: String,
    /// Optional name (used for tool results).
    pub name: Option<String>,
    /// Tool call ID when role is "tool".
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    /// Create a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".into(), content: content.into(), name: None, tool_call_id: None }
    }
    /// Create a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".into(), content: content.into(), name: None, tool_call_id: None }
    }
    /// Create an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: "assistant".into(), content: content.into(), name: None, tool_call_id: None }
    }
}

/// Parameters for an LLM completion request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    /// Target model identifier (e.g. "gpt-4o", "claude-3-5-sonnet-20241022").
    pub model: String,
    /// Conversation messages.
    pub messages: Vec<ChatMessage>,
    /// Sampling temperature (0.0 – 2.0).
    pub temperature: Option<f32>,
    /// Maximum tokens to generate.
    pub max_tokens: Option<u32>,
    /// Stop sequences.
    pub stop: Vec<String>,
    /// Whether to stream the response.
    pub stream: bool,
    /// Available tools/functions the model may call.
    pub tools: Vec<serde_json::Value>,
    /// Tool choice override.
    pub tool_choice: Option<serde_json::Value>,
    /// Extra provider-specific parameters.
    pub extra: HashMap<String, serde_json::Value>,
}

impl LlmRequest {
    /// Create a minimal request with a single user message.
    pub fn simple(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            messages: vec![ChatMessage::user(prompt)],
            temperature: Some(0.7),
            max_tokens: None,
            stop: vec![],
            stream: false,
            tools: vec![],
            tool_choice: None,
            extra: HashMap::new(),
        }
    }
}

/// Token usage reported by an LLM provider.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cached_tokens: u32,
}

impl TokenUsage {
    pub fn total(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
}

/// Reason why generation stopped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Model finished naturally.
    Stop,
    /// Hit max_tokens limit.
    Length,
    /// Model requested a tool call.
    ToolCalls,
    /// Content was filtered.
    ContentFilter,
    /// Unknown / provider-specific.
    Other(String),
}

/// A tool call requested by the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// A single completion choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: FinishReason,
    pub tool_calls: Vec<LlmToolCall>,
}

/// Full response from an LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    /// Provider-assigned response ID.
    pub id: String,
    /// Model that generated this response.
    pub model: String,
    /// Which provider handled the request.
    pub provider: LlmProvider,
    /// Generated choices (usually 1).
    pub choices: Vec<LlmChoice>,
    /// Token usage statistics.
    pub usage: TokenUsage,
    /// Estimated cost in USD.
    pub cost_usd: f64,
    /// Latency in milliseconds.
    pub latency_ms: u64,
}

impl LlmResponse {
    /// Return the text of the first choice, if any.
    pub fn text(&self) -> Option<&str> {
        self.choices.first().map(|c| c.message.content.as_str())
    }

    /// Return tool calls from the first choice.
    pub fn tool_calls(&self) -> &[LlmToolCall] {
        self.choices
            .first()
            .map(|c| c.tool_calls.as_slice())
            .unwrap_or_default()
    }
}

/// A streaming chunk from an LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmStreamChunk {
    pub id: String,
    pub model: String,
    pub delta: String,
    pub tool_call_delta: Option<LlmToolCall>,
    pub finish_reason: Option<FinishReason>,
    pub usage: Option<TokenUsage>,
}

/// Model capability metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub provider: LlmProvider,
    pub context_window: u32,
    pub max_output_tokens: u32,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_streaming: bool,
    /// Cost per million input tokens in USD.
    pub input_cost_per_million: f64,
    /// Cost per million output tokens in USD.
    pub output_cost_per_million: f64,
}
