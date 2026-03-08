// neuraos-types/src/id.rs
// Typed ID wrapper to prevent mix-ups between different entity IDs

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Generic typed ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NeuraId(String);

impl NeuraId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for NeuraId {
    fn default() -> Self { Self::new() }
}

impl fmt::Display for NeuraId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for NeuraId {
    fn from(s: String) -> Self { Self(s) }
}

impl From<&str> for NeuraId {
    fn from(s: &str) -> Self { Self(s.to_string()) }
}
