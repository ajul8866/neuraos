//! neuraos-channels -- Multi-channel communication layer for NeuraOS

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub type ChannelId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelCapabilities {
    pub can_send_text: bool,
    pub can_send_files: bool,
    pub can_send_images: bool,
    pub can_receive: bool,
    pub supports_threads: bool,
    pub supports_reactions: bool,
    pub max_message_length: Option<usize>,
}

impl Default for ChannelCapabilities {
    fn default() -> Self {
        Self { can_send_text: true, can_send_files: false, can_send_images: false,
               can_receive: false, supports_threads: false, supports_reactions: false,
               max_message_length: None }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelKind {
    Slack, Discord, Telegram, Email, Webhook, Terminal, Custom(String),
}

impl std::fmt::Display for ChannelKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelKind::Slack => write!(f, "slack"),
            ChannelKind::Discord => write!(f, "discord"),
            ChannelKind::Telegram => write!(f, "telegram"),
            ChannelKind::Email => write!(f, "email"),
            ChannelKind::Webhook => write!(f, "webhook"),
            ChannelKind::Terminal => write!(f, "terminal"),
            ChannelKind::Custom(s) => write!(f, "custom:{}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeliveryResult {
    pub channel_id: ChannelId,
    pub message_id: Option<String>,
    pub success: bool,
    pub error: Option<String>,
}

#[async_trait]
pub trait Channel: Send + Sync {
    fn id(&self) -> &ChannelId;
    fn name(&self) -> &str;
    fn kind(&self) -> ChannelKind;
    fn capabilities(&self) -> ChannelCapabilities;
    async fn send(&self, message: &message::Message) -> anyhow::Result<DeliveryResult>;
    async fn health_check(&self) -> bool;
}

pub mod message {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Message {
        pub id: String,
        pub kind: MessageKind,
        pub content: String,
        pub subject: Option<String>,
        pub attachments: Vec<Attachment>,
        pub metadata: HashMap<String, serde_json::Value>,
        pub recipient: Option<String>,
        pub thread_id: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum MessageKind { Text, Markdown, Html, Json, Alert }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Attachment { pub filename: String, pub content_type: String, pub data: Vec<u8> }

    pub struct MessageBuilder {
        id: String, kind: MessageKind, content: String, subject: Option<String>,
        attachments: Vec<Attachment>, metadata: HashMap<String, serde_json::Value>,
        recipient: Option<String>, thread_id: Option<String>,
    }

    impl MessageBuilder {
        pub fn new(content: impl Into<String>) -> Self {
            Self { id: uuid::Uuid::new_v4().to_string(), kind: MessageKind::Text,
                   content: content.into(), subject: None, attachments: vec![],
                   metadata: HashMap::new(), recipient: None, thread_id: None }
        }
        pub fn kind(mut self, kind: MessageKind) -> Self { self.kind = kind; self }
        pub fn subject(mut self, s: impl Into<String>) -> Self { self.subject = Some(s.into()); self }
        pub fn recipient(mut self, r: impl Into<String>) -> Self { self.recipient = Some(r.into()); self }
        pub fn build(self) -> Message {
            Message { id: self.id, kind: self.kind, content: self.content,
                      subject: self.subject, attachments: self.attachments,
                      metadata: self.metadata, recipient: self.recipient, thread_id: self.thread_id }
        }
    }
}

pub mod registry {
    use super::*;
    use tokio::sync::RwLock;
    pub struct ChannelRegistry { channels: RwLock<HashMap<ChannelId, Arc<dyn Channel>>> }
    impl ChannelRegistry {
        pub fn new() -> Self { Self { channels: RwLock::new(HashMap::new()) } }
        pub async fn register(&self, ch: Arc<dyn Channel>) {
            self.channels.write().await.insert(ch.id().clone(), ch);
        }
        pub async fn get(&self, id: &ChannelId) -> Option<Arc<dyn Channel>> {
            self.channels.read().await.get(id).cloned()
        }
        pub async fn list(&self) -> Vec<ChannelId> {
            self.channels.read().await.keys().cloned().collect()
        }
    }
}

pub mod router {
    use super::*;
    use crate::registry::ChannelRegistry;
    #[derive(Debug, Clone)]
    pub struct RouteRule { pub name: String, pub channel_id: ChannelId, pub priority: i32 }
    pub struct Router { registry: Arc<ChannelRegistry>, rules: Vec<RouteRule> }
    impl Router {
        pub fn new(registry: Arc<ChannelRegistry>) -> Self { Self { registry, rules: vec![] } }
        pub fn add_rule(&mut self, rule: RouteRule) {
            self.rules.push(rule);
            self.rules.sort_by_key(|r| -r.priority);
        }
        pub async fn send_to(&self, channel_id: &ChannelId, message: &message::Message)
            -> anyhow::Result<DeliveryResult>
        {
            match self.registry.get(channel_id).await {
                Some(ch) => ch.send(message).await,
                None => anyhow::bail!("Channel '{}' not found", channel_id),
            }
        }
    }
}

pub mod adapters {
    use super::*;
    pub struct TerminalChannel { id: ChannelId }
    impl TerminalChannel { pub fn new() -> Self { Self { id: "terminal".to_string() } } }
    #[async_trait]
    impl Channel for TerminalChannel {
        fn id(&self) -> &ChannelId { &self.id }
        fn name(&self) -> &str { "Terminal" }
        fn kind(&self) -> ChannelKind { ChannelKind::Terminal }
        fn capabilities(&self) -> ChannelCapabilities {
            ChannelCapabilities { can_send_text: true, can_receive: true, ..Default::default() }
        }
        async fn send(&self, msg: &message::Message) -> anyhow::Result<DeliveryResult> {
            println!("[NeuraOS] {}", msg.content);
            Ok(DeliveryResult { channel_id: self.id.clone(), message_id: None, success: true, error: None })
        }
        async fn health_check(&self) -> bool { true }
    }
}
