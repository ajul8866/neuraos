//! NeuraOS LLM Router — multi-provider routing, caching, streaming, optimization.

#![forbid(unsafe_code)]
#![deny(clippy::all)]

pub mod cache;
pub mod optimizer;
pub mod providers;
pub mod router;

pub use router::LlmRouter;
pub use cache::LlmCache;
pub use optimizer::CostOptimizer;
