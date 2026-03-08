//! Bayesian cost-quality optimizer using Thompson sampling.

use dashmap::DashMap;
use std::sync::Arc;
use tracing::debug;

#[derive(Debug, Clone, Default)]
pub struct ModelRequirements {
    pub max_cost_per_1k_tokens: Option<f64>,
    pub max_latency_ms: Option<u64>,
    pub min_quality_score: Option<f32>,
    pub context_length_needed: Option<u32>,
    pub needs_vision: bool,
    pub needs_function_calling: bool,
    pub needs_json_mode: bool,
    pub preferred_provider: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ModelSelection {
    pub provider: String,
    pub model: String,
    pub expected_cost_per_1k: f64,
    pub expected_latency_ms: u64,
    pub expected_quality: f32,
    pub confidence: f32,
}

#[derive(Debug, Default, Clone)]
struct ModelStats {
    calls: u64,
    total_latency_ms: u64,
    total_cost_usd: f64,
    quality_sum: f64,
    failures: u64,
    alpha: f64,
    beta: f64,
}

impl ModelStats {
    fn avg_latency_ms(&self) -> u64 {
        if self.calls == 0 { 1000 } else { self.total_latency_ms / self.calls }
    }
    fn avg_cost_per_1k(&self) -> f64 {
        if self.calls == 0 { 0.01 } else { self.total_cost_usd / self.calls as f64 * 1000.0 }
    }
    fn avg_quality(&self) -> f32 {
        if self.calls == 0 { 0.8 } else { (self.quality_sum / self.calls as f64) as f32 }
    }
    fn success_rate(&self) -> f64 {
        let total = self.calls + self.failures;
        if total == 0 { 0.9 } else { self.calls as f64 / total as f64 }
    }
    fn thompson_sample(&self) -> f64 {
        let a = self.alpha.max(1.0);
        let b = self.beta.max(1.0);
        let mean = a / (a + b);
        let variance = (a * b) / ((a + b).powi(2) * (a + b + 1.0));
        mean + variance.sqrt() * pseudo_normal()
    }
}

pub struct LlmOptimizer {
    models: DashMap<String, ModelStats>,
}

impl LlmOptimizer {
    pub fn new() -> Self {
        let opt = Self { models: DashMap::new() };
        opt.seed_defaults();
        opt
    }

    fn seed_defaults(&self) {
        let defaults: &[(&str, u64, f64, f32)] = &[
            ("openai/gpt-4o",           1200, 0.005,  0.95),
            ("openai/gpt-4o-mini",       600, 0.0003, 0.85),
            ("openai/gpt-3.5-turbo",     400, 0.001,  0.75),
            ("anthropic/claude-3-5-sonnet", 1500, 0.003, 0.96),
            ("anthropic/claude-3-haiku",  500, 0.00025, 0.82),
            ("google/gemini-1.5-pro",   1300, 0.0035, 0.93),
            ("google/gemini-1.5-flash",  550, 0.0002, 0.83),
            ("groq/llama-3.3-70b-versatile", 200, 0.0006, 0.88),
            ("groq/llama-3.1-8b-instant",   80,  0.00005, 0.75),
            ("mistral/mistral-large",   1100, 0.002,  0.90),
            ("deepseek/deepseek-chat",   900, 0.00014, 0.87),
            ("ollama/llama3.2",          300, 0.0, 0.78),
        ];
        for (model, latency, cost, quality) in defaults {
            let mut s = ModelStats { calls: 10, ..Default::default() };
            s.total_latency_ms = latency * 10;
            s.total_cost_usd = cost * 10.0 / 1000.0;
            s.quality_sum = *quality as f64 * 10.0;
            s.alpha = 9.0;
            s.beta = 1.0;
            self.models.insert(model.to_string(), s);
        }
    }

    pub fn select(&self, reqs: &ModelRequirements) -> ModelSelection {
        let mut best: Option<(String, f64)> = None;

        for entry in self.models.iter() {
            let model_id = entry.key().clone();
            let stats = entry.value();

            if let Some(max_cost) = reqs.max_cost_per_1k_tokens {
                if stats.avg_cost_per_1k() > max_cost { continue; }
            }
            if let Some(max_lat) = reqs.max_latency_ms {
                if stats.avg_latency_ms() > max_lat { continue; }
            }
            if let Some(min_q) = reqs.min_quality_score {
                if stats.avg_quality() < min_q { continue; }
            }
            if let Some(pref) = &reqs.preferred_provider {
                if !model_id.starts_with(pref.as_str()) { continue; }
            }

            let score = stats.thompson_sample();
            if best.as_ref().map_or(true, |(_, s)| score > *s) {
                best = Some((model_id, score));
            }
        }

        let model_id = best
            .map(|(id, _)| id)
            .unwrap_or_else(|| "openai/gpt-4o-mini".to_string());

        let stats = self.models.get(&model_id).map(|s| s.clone()).unwrap_or_default();
        let (provider, model) = model_id.split_once('/').unwrap_or(("openai", &model_id));

        ModelSelection {
            provider: provider.to_string(),
            model: model.to_string(),
            expected_cost_per_1k: stats.avg_cost_per_1k(),
            expected_latency_ms: stats.avg_latency_ms(),
            expected_quality: stats.avg_quality(),
            confidence: stats.success_rate() as f32,
        }
    }

    pub fn record_success(&self, model_id: &str, latency_ms: u64, cost_usd: f64, quality: f32) {
        let mut s = self.models.entry(model_id.to_string()).or_default();
        s.calls += 1;
        s.total_latency_ms += latency_ms;
        s.total_cost_usd += cost_usd;
        s.quality_sum += quality as f64;
        s.alpha += 1.0;
    }

    pub fn record_failure(&self, model_id: &str) {
        let mut s = self.models.entry(model_id.to_string()).or_default();
        s.failures += 1;
        s.beta += 1.0;
    }
}

impl Default for LlmOptimizer {
    fn default() -> Self { Self::new() }
}

fn pseudo_normal() -> f64 {
    let u = pseudo_uniform();
    let v = pseudo_uniform();
    (-2.0 * u.ln()).sqrt() * (2.0 * std::f64::consts::PI * v).cos()
}

fn pseudo_uniform() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let x = ns.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (x >> 11) as f64 / (1u64 << 53) as f64
}
