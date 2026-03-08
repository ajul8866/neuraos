//! neuraos-telemetry — tracing, metrics, and observability for NeuraOS.

use std::time::Duration;
use tracing::{info, warn, error, Level};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// Initialise the global tracing subscriber.
/// Call once at process start. Safe to call multiple times (no-op after first).
pub fn init_tracing(service_name: &str, json: bool) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    if json {
        let subscriber = tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().json());
        let _ = tracing::subscriber::set_global_default(subscriber);
    } else {
        let subscriber = tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().pretty());
        let _ = tracing::subscriber::set_global_default(subscriber);
    }

    info!(service = service_name, "tracing initialised");
}

/// Simple span timer helper — logs duration on drop.
pub struct SpanTimer {
    name: &'static str,
    start: std::time::Instant,
}

impl SpanTimer {
    pub fn new(name: &'static str) -> Self {
        Self { name, start: std::time::Instant::now() }
    }
}

impl Drop for SpanTimer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        info!(name = self.name, elapsed_ms = elapsed.as_millis(), "span completed");
    }
}

/// Structured event emitter for agent lifecycle events.
#[derive(Debug, Clone)]
pub struct AgentEvent {
    pub agent_id: String,
    pub event:    String,
    pub duration: Option<Duration>,
}

impl AgentEvent {
    pub fn new(agent_id: impl Into<String>, event: impl Into<String>) -> Self {
        Self { agent_id: agent_id.into(), event: event.into(), duration: None }
    }

    pub fn with_duration(mut self, d: Duration) -> Self {
        self.duration = Some(d);
        self
    }

    pub fn emit(&self) {
        match self.event.as_str() {
            e if e.contains("error") || e.contains("fail") => {
                error!(agent_id = %self.agent_id, event = %self.event,
                       duration_ms = self.duration.map(|d| d.as_millis()),
                       "agent error event");
            }
            e if e.contains("warn") => {
                warn!(agent_id = %self.agent_id, event = %self.event,
                      duration_ms = self.duration.map(|d| d.as_millis()),
                      "agent warning event");
            }
            _ => {
                info!(agent_id = %self.agent_id, event = %self.event,
                      duration_ms = self.duration.map(|d| d.as_millis()),
                      "agent event");
            }
        }
    }
}

/// Counter metric (in-memory, for lightweight use without OTel).
#[derive(Debug, Default)]
pub struct Counter {
    name:  String,
    value: std::sync::atomic::AtomicU64,
}

impl Counter {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), value: std::sync::atomic::AtomicU64::new(0) }
    }

    pub fn increment(&self) {
        self.value.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn add(&self, n: u64) {
        self.value.fetch_add(n, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn get(&self) -> u64 {
        self.value.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn report(&self) {
        info!(counter = %self.name, value = self.get(), "metric");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_increments() {
        let c = Counter::new("test_counter");
        c.increment();
        c.increment();
        c.add(3);
        assert_eq!(c.get(), 5);
    }

    #[test]
    fn agent_event_builds() {
        let e = AgentEvent::new("agent-1", "task_complete")
            .with_duration(Duration::from_millis(42));
        assert_eq!(e.agent_id, "agent-1");
        assert_eq!(e.duration.unwrap().as_millis(), 42);
    }
}
