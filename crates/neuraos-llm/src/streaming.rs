//! Streaming response types for LLM completions.

use serde::{Deserialize, Serialize};

/// A single chunk in a streaming completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub id: String,
    pub delta: String,
    pub finish_reason: Option<FinishReason>,
    pub model: String,
    pub tokens: Option<u32>,
}

impl StreamChunk {
    pub fn text(id: impl Into<String>, delta: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            delta: delta.into(),
            finish_reason: None,
            model: model.into(),
            tokens: None,
        }
    }

    pub fn done(id: impl Into<String>, model: impl Into<String>, tokens: u32) -> Self {
        Self {
            id: id.into(),
            delta: String::new(),
            finish_reason: Some(FinishReason::Stop),
            model: model.into(),
            tokens: Some(tokens),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    Error,
}
