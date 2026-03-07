//! LLM Router — multi-provider routing with fallback, caching, and circuit breakers.

use crate::cache::SemanticCache;
use crate::optimizer::{LlmOptimizer, ModelRequirements, ModelSelection};
use crate::providers::LlmProvider;
use crate::streaming::StreamChunk;
use neuraos_types::{Message, Role, TokenUsage};
use std::collections::HashMap;
use std::sync::Arc;
use futures::Stream;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// ─── Request / Response ──────────────────────────────────────────────────────

/// Input to an LLM completion call.
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub stop: Vec<String>,
    pub tools: Option<serde_json::Value>,
    pub response_format: Option<ResponseFormat>,
    pub stream: bool,
    pub metadata: HashMap<String, serde_json::Value>,
    pub requirements: ModelRequirements,
}

impl Default for CompletionRequest {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            model: None,
            provider: None,
            temperature: Some(0.7),
            max_tokens: Some(4096),
            top_p: Some(1.0),
            stop: Vec::new(),
            tools: None,
            response_format: None,
            stream: false,
            metadata: HashMap::new(),
            requirements: ModelRequirements::default(),
        }
    }
}

impl CompletionRequest {
    pub fn simple(system: &str, user: &str) -> Self {
        Self {
            messages: vec![
                Message::system(system),
                Message::user(user),
            ],
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResponseFormat {
    Text,
    JsonObject,
    JsonSchema { schema: serde_json::Value },
}

/// Response from an LLM completion call.
#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub id: String,
    pub model: String,
    pub provider: String,
    pub content: String,
    pub tool_calls: Option<serde_json::Value>,
    pub finish_reason: String,
    pub usage: TokenUsage,
    pub latency_ms: u64,
}

impl CompletionResponse {
    pub fn text_content(&self) -> &str {
        &self.content
    }
}

// ─── Routing strategy ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub enum RoutingStrategy {
    #[default]
    CostOptimized,
    LatencyOptimized,
    QualityOptimized,
    RoundRobin,
    Failover,
}

// ─── LlmRouter ────────────────────────────────────────────────────────────────

/// Main LLM routing entrypoint.
pub struct LlmRouter {
    providers: Vec<Arc<dyn LlmProvider + Send + Sync>>,
    strategy: RoutingStrategy,
    cache: Arc<SemanticCache>,
    optimizer: Arc<LlmOptimizer>,
    round_robin_idx: Arc<std::sync::atomic::AtomicUsize>,
}

impl LlmRouter {
    pub fn new(strategy: RoutingStrategy, cache: Arc<SemanticCache>, optimizer: Arc<LlmOptimizer>) -> Self {
        Self {
            providers: Vec::new(),
            strategy,
            cache,
            optimizer,
            round_robin_idx: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    pub fn add_provider(&mut self, provider: Arc<dyn LlmProvider + Send + Sync>) {
        info!("Registered LLM provider: {}", provider.name());
        self.providers.push(provider);
    }

    /// Complete a request, using cache and routing strategy.
    pub async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, RouterError> {
        if self.providers.is_empty() {
            return Err(RouterError::NoProvidersConfigured);
        }

        // Select provider(s) to try
        let ordered = self.select_providers(&req);

        let mut last_err: Option<RouterError> = None;
        for provider in &ordered {
            debug!("Trying provider: {}", provider.name());
            let start = std::time::Instant::now();
            match provider.complete(&req).await {
                Ok(mut resp) => {
                    let latency = start.elapsed().as_millis() as u64;
                    resp.latency_ms = latency;
                    self.optimizer.record_success(
                        &format!("{}/{}", provider.name(), resp.model),
                        latency,
                        resp.usage.cost_usd,
                        0.9,
                    );
                    return Ok(resp);
                }
                Err(e) => {
                    warn!("Provider {} failed: {}", provider.name(), e);
                    self.optimizer.record_failure(&provider.name().to_string());
                    last_err = Some(RouterError::ProviderError {
                        provider: provider.name().to_string(),
                        message: e.to_string(),
                    });
                }
            }
        }

        Err(last_err.unwrap_or(RouterError::NoProvidersConfigured))
    }

    /// Select providers in priority order based on the routing strategy.
    fn select_providers<'a>(&'a self, req: &CompletionRequest) -> Vec<&'a Arc<dyn LlmProvider + Send + Sync>> {
        let available: Vec<_> = self.providers.iter().filter(|p| p.is_available()).collect();
        if available.is_empty() {
            return self.providers.iter().collect();
        }

        // If a specific provider is requested, put it first
        if let Some(pref_name) = &req.provider {
            let mut ordered: Vec<_> = available.clone();
            ordered.sort_by_key(|p| if p.name() == pref_name { 0usize } else { 1 });
            return ordered;
        }

        match &self.strategy {
            RoutingStrategy::RoundRobin => {
                let idx = self.round_robin_idx.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                    % available.len();
                let mut v: Vec<_> = available.into_iter().collect();
                v.rotate_left(idx);
                v
            }
            RoutingStrategy::Failover => available,
            RoutingStrategy::CostOptimized | RoutingStrategy::LatencyOptimized | RoutingStrategy::QualityOptimized => {
                let sel = self.optimizer.select(&req.requirements);
                let mut ordered: Vec<_> = available.clone();
                ordered.sort_by_key(|p| if p.name() == sel.provider { 0usize } else { 1 });
                ordered
            }
        }
    }

    /// Embed texts using the first available provider.
    pub async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, RouterError> {
        for provider in &self.providers {
            if provider.is_available() {
                return provider.embed(texts).await
                    .map_err(|e| RouterError::ProviderError {
                        provider: provider.name().to_string(),
                        message: e.to_string(),
                    });
            }
        }
        Err(RouterError::NoProvidersConfigured)
    }

    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }

    pub fn available_models(&self) -> Vec<String> {
        self.providers.iter().flat_map(|p| p.models().iter().map(|m| m.id.clone())).collect()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("No LLM providers configured")]
    NoProvidersConfigured,
    #[error("Provider '{provider}' error: {message}")]
    ProviderError { provider: String, message: String },
    #[error("All {count} providers failed")]
    AllProvidersFailed { count: usize },
    #[error("Rate limited by '{provider}', retry after {retry_after_secs}s")]
    RateLimited { provider: String, retry_after_secs: u64 },
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
}
