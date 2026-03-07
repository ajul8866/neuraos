//! Circuit breaker pattern for LLM providers and external tools.

use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{info, warn};

/// Circuit breaker states.
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Normal operation — calls pass through.
    Closed,
    /// Too many failures — calls are rejected immediately.
    Open { opened_at: Instant },
    /// Testing recovery — one probe call allowed.
    HalfOpen,
}

impl CircuitState {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::Open { .. } => "open",
            Self::HalfOpen => "half_open",
        }
    }
}

/// Configuration for a single circuit breaker.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening.
    pub failure_threshold: u32,
    /// How long to stay open before allowing a probe (seconds).
    pub reset_timeout_secs: u64,
    /// Success count in HalfOpen needed to close.
    pub half_open_successes: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            reset_timeout_secs: 60,
            half_open_successes: 2,
        }
    }
}

/// State for one named circuit.
struct Circuit {
    state: CircuitState,
    consecutive_failures: u32,
    half_open_successes: u32,
    config: CircuitBreakerConfig,
}

impl Circuit {
    fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: CircuitState::Closed,
            consecutive_failures: 0,
            half_open_successes: 0,
            config,
        }
    }

    fn is_callable(&mut self) -> bool {
        match &self.state {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true,
            CircuitState::Open { opened_at } => {
                let elapsed = opened_at.elapsed();
                if elapsed >= Duration::from_secs(self.config.reset_timeout_secs) {
                    info!("Circuit transitioning to HalfOpen after {}s", elapsed.as_secs());
                    self.state = CircuitState::HalfOpen;
                    self.half_open_successes = 0;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn record_success(&mut self) {
        match self.state {
            CircuitState::HalfOpen => {
                self.half_open_successes += 1;
                if self.half_open_successes >= self.config.half_open_successes {
                    info!("Circuit closing after {} successes in HalfOpen", self.half_open_successes);
                    self.state = CircuitState::Closed;
                    self.consecutive_failures = 0;
                }
            }
            CircuitState::Closed => {
                self.consecutive_failures = 0;
            }
            _ => {}
        }
    }

    fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        if self.consecutive_failures >= self.config.failure_threshold {
            warn!(
                "Circuit opening after {} consecutive failures",
                self.consecutive_failures
            );
            self.state = CircuitState::Open { opened_at: Instant::now() };
        }
        // In HalfOpen, a single failure re-opens
        if matches!(self.state, CircuitState::HalfOpen) {
            self.state = CircuitState::Open { opened_at: Instant::now() };
        }
    }
}

/// Registry of named circuit breakers.
pub struct CircuitBreakerRegistry {
    circuits: DashMap<String, Mutex<Circuit>>,
    default_config: CircuitBreakerConfig,
}

impl CircuitBreakerRegistry {
    pub fn new(default_config: CircuitBreakerConfig) -> Self {
        Self {
            circuits: DashMap::new(),
            default_config,
        }
    }

    /// Execute a closure through the named circuit breaker.
    pub async fn call<F, T, E>(
        &self,
        name: &str,
        f: F,
    ) -> Result<T, CircuitBreakerError<E>>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Debug,
    {
        // Get or create circuit
        if !self.circuits.contains_key(name) {
            self.circuits.insert(
                name.to_string(),
                Mutex::new(Circuit::new(self.default_config.clone())),
            );
        }

        let entry = self.circuits.get(name).unwrap();
        let mut circuit = entry.lock().await;

        if !circuit.is_callable() {
            return Err(CircuitBreakerError::Open {
                name: name.to_string(),
            });
        }

        drop(circuit);

        // Execute
        let result = f.await;

        let entry = self.circuits.get(name).unwrap();
        let mut circuit = entry.lock().await;

        match &result {
            Ok(_) => circuit.record_success(),
            Err(_) => circuit.record_failure(),
        }

        result.map_err(CircuitBreakerError::Inner)
    }

    /// Get the state of a named circuit.
    pub async fn state(&self, name: &str) -> Option<String> {
        if let Some(entry) = self.circuits.get(name) {
            let circuit = entry.lock().await;
            Some(circuit.state.name().to_string())
        } else {
            None
        }
    }

    /// Force-reset a circuit to Closed.
    pub async fn reset(&self, name: &str) {
        if let Some(entry) = self.circuits.get(name) {
            let mut circuit = entry.lock().await;
            circuit.state = CircuitState::Closed;
            circuit.consecutive_failures = 0;
            info!("Circuit '{}' manually reset to Closed", name);
        }
    }
}

impl Default for CircuitBreakerRegistry {
    fn default() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError<E: std::fmt::Debug> {
    #[error("Circuit '{name}' is OPEN — calls rejected")]
    Open { name: String },
    #[error("Inner error: {0:?}")]
    Inner(E),
}
