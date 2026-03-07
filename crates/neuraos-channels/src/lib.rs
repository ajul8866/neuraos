//! # neuraos-channels
//! Channel adapters for NeuraOS -- connects agents to Slack, Discord, Telegram, email, and more.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod adapter;
pub mod registry;
pub mod router;

use async_trait::async_trait;
use neuraos_types::{NeuraError, NeuraResult};

/// A channel adapter that can send and receive messages.
#[async_trait]
pub trait ChannelAdapter: Send + Sync + 'static {
    /// Unique identifier for this adapter type.
    fn id(&self) -> &str;
    /// Human-readable name.
    fn name(&self) -> &str;
    /// Send a text message to the specified destination.
    async fn send(&self, destination: &str, content: &str) -> NeuraResult<()>;
}

pub mod adapter {
    //! Built-in adapter stubs.
    use super::*;

    /// Slack channel adapter.
    pub struct SlackAdapter;

    #[async_trait]
    impl ChannelAdapter for SlackAdapter {
        fn id(&self) -> &str { "slack" }
        fn name(&self) -> &str { "Slack" }
        async fn send(&self, _dest: &str, _content: &str) -> NeuraResult<()> {
            Err(NeuraError::Internal("SlackAdapter::send not yet wired".into()))
        }
    }
}

pub mod registry {
    //! Channel adapter registry.
    use std::collections::HashMap;
    use super::ChannelAdapter;

    /// Registry that maps adapter IDs to their implementations.
    #[derive(Default)]
    pub struct AdapterRegistry {
        adapters: HashMap<String, Box<dyn ChannelAdapter>>,
    }

    impl AdapterRegistry {
        /// Register a new adapter.
        pub fn register(&mut self, adapter: Box<dyn ChannelAdapter>) {
            self.adapters.insert(adapter.id().to_string(), adapter);
        }
        /// Get an adapter by ID.
        pub fn get(&self, id: &str) -> Option<&dyn ChannelAdapter> {
            self.adapters.get(id).map(|a| a.as_ref())
        }
    }
}

pub mod router {
    //! Message routing logic.
}
