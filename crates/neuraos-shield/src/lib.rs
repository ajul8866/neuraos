// neuraos-shield/src/lib.rs
// NeuraOS Security, Sandboxing & Policy Enforcement

pub mod auth;
pub mod policy;
pub mod sandbox;
pub mod audit;
pub mod crypto;

pub use auth::{AuthManager, AuthToken, Principal};
pub use policy::{PolicyEngine, Policy, PolicyDecision};
pub use sandbox::{Sandbox, SandboxConfig};
pub use audit::{AuditLogger, AuditEvent};
pub use crypto::CryptoUtils;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShieldError {
    #[error("Authentication failed: {0}")]
    AuthFailed(String),
    #[error("Authorization denied: {0}")]
    Denied(String),
    #[error("Policy violation: {0}")]
    PolicyViolation(String),
    #[error("Sandbox escape attempt detected")]
    SandboxViolation,
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Crypto error: {0}")]
    CryptoError(String),
    #[error("Rate limit exceeded")]
    RateLimited,
    #[error("IP blocked: {0}")]
    IpBlocked(String),
}

pub type ShieldResult<T> = Result<T, ShieldError>;
