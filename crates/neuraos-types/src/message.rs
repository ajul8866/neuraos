// neuraos-types/src/message.rs
// Chat message and content types

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
    Function,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageContent {
    Text { text: String },
    Image { url: String, alt: Option<String> },
    ToolCall { tool_name: String, arguments: serde_json::Value, call_id: String },
    ToolResult { call_id: String, result: serde_json::Value, is_error: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: Vec<MessageContent>,
    pub name: Option<String>,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Message {
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            role: MessageRole::User,
            content: vec![MessageContent::Text { text: text.into() }],
            name: None,
            agent_id: None,
            session_id: None,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            role: MessageRole::Assistant,
            content: vec![MessageContent::Text { text: text.into() }],
            name: None,
            agent_id: None,
            session_id: None,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn system(text: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            role: MessageRole::System,
            content: vec![MessageContent::Text { text: text.into() }],
            name: None,
            agent_id: None,
            session_id: None,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn text_content(&self) -> Option<&str> {
        self.content.iter().find_map(|c| {
            if let MessageContent::Text { text } = c { Some(text.as_str()) } else { None }
        })
    }
}
