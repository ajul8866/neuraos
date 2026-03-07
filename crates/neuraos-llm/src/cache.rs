//! Semantic response cache with embedding-based similarity deduplication.

use crate::router::{CompletionRequest, CompletionResponse};
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// One cached entry.
struct CacheEntry {
    response: CompletionResponse,
    embedding: Vec<f32>,
    inserted_at: Instant,
    ttl: Duration,
    hits: u32,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        self.inserted_at.elapsed() > self.ttl
    }
}

/// Semantic response cache.
pub struct SemanticCache {
    entries: DashMap<String, CacheEntry>,
    similarity_threshold: f32,
    ttl: Duration,
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
    saved_tokens: Arc<AtomicU64>,
    saved_cost_micros: Arc<AtomicU64>,
}

impl SemanticCache {
    pub fn new(similarity_threshold: f32, ttl_secs: u64) -> Self {
        Self {
            entries: DashMap::new(),
            similarity_threshold,
            ttl: Duration::from_secs(ttl_secs),
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            saved_tokens: Arc::new(AtomicU64::new(0)),
            saved_cost_micros: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Try to find a cached response for the given prompt embedding.
    pub fn get(&self, embedding: &[f32]) -> Option<CompletionResponse> {
        let mut best_score = 0.0f32;
        let mut best_response: Option<CompletionResponse> = None;

        for entry in self.entries.iter() {
            if entry.is_expired() {
                continue;
            }
            let score = cosine_sim(embedding, &entry.embedding);
            if score > self.similarity_threshold && score > best_score {
                best_score = score;
                best_response = Some(entry.response.clone());
            }
        }

        if let Some(resp) = best_response {
            self.hits.fetch_add(1, Ordering::Relaxed);
            self.saved_tokens.fetch_add(resp.usage.total as u64, Ordering::Relaxed);
            debug!("Cache hit (score={:.3})", best_score);
            Some(resp)
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    /// Insert a response with its prompt embedding.
    pub fn put(&self, key: String, embedding: Vec<f32>, response: CompletionResponse) {
        self.entries.insert(
            key,
            CacheEntry {
                response,
                embedding,
                inserted_at: Instant::now(),
                ttl: self.ttl,
                hits: 0,
            },
        );
        // Evict expired entries opportunistically
        self.entries.retain(|_, v| !v.is_expired());
    }

    pub fn hit_rate(&self) -> f64 {
        let h = self.hits.load(Ordering::Relaxed);
        let m = self.misses.load(Ordering::Relaxed);
        if h + m == 0 {
            0.0
        } else {
            h as f64 / (h + m) as f64
        }
    }

    pub fn saved_tokens(&self) -> u64 {
        self.saved_tokens.load(Ordering::Relaxed)
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    pub fn clear(&self) {
        self.entries.clear();
    }
}

impl Default for SemanticCache {
    fn default() -> Self {
        Self::new(0.97, 3600)
    }
}

fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len().min(b.len());
    let dot: f32 = a[..len].iter().zip(b[..len].iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a[..len].iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b[..len].iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 { 0.0 } else { dot / (na * nb) }
}
