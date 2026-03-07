//! Pre-built agent manifests and runtime registry for NeuraOS.

pub mod catalog;
pub mod registry;

#[cfg(test)]
mod tests;

// Re-export public API at crate root for convenience
pub use catalog::{catalog, get_agent};
pub use catalog::{
    researcher, coder, devops, data_analyst, writer,
    secretary, security_analyst, product_manager, financial_analyst, teacher,
};
pub use registry::AgentRegistry;
