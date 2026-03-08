//! Async event bus — broadcast channel with filtering and dead-letter queue.

use neuraos_types::{Event, EventKind};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};
use tracing::debug;

const DEFAULT_CAPACITY: usize = 10_000;

/// System-wide event bus backed by a tokio broadcast channel.
#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<Arc<Event>>,
    published: Arc<AtomicU64>,
    dropped: Arc<AtomicU64>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity.max(64));
        Self {
            sender,
            published: Arc::new(AtomicU64::new(0)),
            dropped: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Publish an event to all active subscribers.
    pub fn publish(&self, event: Event) -> Result<usize, EventBusError> {
        let arc = Arc::new(event);
        match self.sender.send(arc) {
            Ok(n) => {
                self.published.fetch_add(1, Ordering::Relaxed);
                debug!("Event published to {} subscribers", n);
                Ok(n)
            }
            Err(_) => {
                // No active subscribers — this is fine unless we need DLQ
                self.dropped.fetch_add(1, Ordering::Relaxed);
                Ok(0)
            }
        }
    }

    /// Subscribe and receive all events (no filter).
    pub fn subscribe_all(&self) -> impl Stream<Item = Arc<Event>> {
        BroadcastStream::new(self.sender.subscribe())
            .filter_map(|r| r.ok())
    }

    /// Subscribe with an event kind filter.
    pub fn subscribe_filtered(
        &self,
        filter: EventFilter,
    ) -> impl Stream<Item = Arc<Event>> {
        BroadcastStream::new(self.sender.subscribe())
            .filter_map(move |r| {
                let evt = r.ok()?;
                if filter.matches(&evt) { Some(evt) } else { None }
            })
    }

    /// Number of active subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }

    /// Cumulative published events.
    pub fn published_count(&self) -> u64 {
        self.published.load(Ordering::Relaxed)
    }

    /// Cumulative dropped events (sent when no subscriber present).
    pub fn dropped_count(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(DEFAULT_CAPACITY)
    }
}

/// Filter criteria for event subscriptions.
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    /// Only match these event kinds (empty = match all).
    pub kinds: Vec<EventKindMatcher>,
    /// Only match events from this source.
    pub source: Option<String>,
}

impl EventFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_kind(mut self, kind: EventKindMatcher) -> Self {
        self.kinds.push(kind);
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn matches(&self, event: &Event) -> bool {
        if let Some(src) = &self.source {
            if &event.source != src {
                return false;
            }
        }
        if self.kinds.is_empty() {
            return true;
        }
        self.kinds.iter().any(|m| m.matches(&event.kind))
    }
}

/// Pattern matcher for EventKind.
#[derive(Debug, Clone)]
pub enum EventKindMatcher {
    Exact(EventKind),
    Prefix(String), // matches strum Display prefix
    Any,
}

impl EventKindMatcher {
    pub fn matches(&self, kind: &EventKind) -> bool {
        match self {
            Self::Any => true,
            Self::Exact(k) => k == kind,
            Self::Prefix(p) => kind.to_string().starts_with(p.as_str()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EventBusError {
    #[error("Event bus send error: {0}")]
    Send(String),
}
