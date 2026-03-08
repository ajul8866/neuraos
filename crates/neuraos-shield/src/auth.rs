// neuraos-shield/src/auth.rs
// Authentication and identity management

use crate::{ShieldError, ShieldResult};
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principal {
    pub id: String,
    pub name: String,
    pub kind: PrincipalKind,
    pub roles: Vec<String>,
    pub attributes: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrincipalKind {
    User,
    Agent,
    Service,
    System,
    ApiKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token_id: String,
    pub principal_id: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub scopes: Vec<String>,
    pub revoked: bool,
}

impl AuthToken {
    pub fn new(principal_id: impl Into<String>, ttl_secs: i64, scopes: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            token_id: Uuid::new_v4().to_string(),
            principal_id: principal_id.into(),
            issued_at: now,
            expires_at: now + Duration::seconds(ttl_secs),
            scopes,
            revoked: false,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.revoked && Utc::now() < self.expires_at
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == scope || s == "*")
    }
}

pub struct AuthManager {
    principals: Arc<DashMap<String, Principal>>,
    tokens: Arc<DashMap<String, AuthToken>>,
    api_keys: Arc<DashMap<String, String>>, // key -> principal_id
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            principals: Arc::new(DashMap::new()),
            tokens: Arc::new(DashMap::new()),
            api_keys: Arc::new(DashMap::new()),
        }
    }

    pub fn register_principal(&self, principal: Principal) {
        info!("Registering principal: {} ({})", principal.name, principal.id);
        self.principals.insert(principal.id.clone(), principal);
    }

    pub fn register_api_key(&self, api_key: impl Into<String>, principal_id: impl Into<String>) {
        self.api_keys.insert(api_key.into(), principal_id.into());
    }

    pub fn issue_token(&self, principal_id: &str, ttl_secs: i64, scopes: Vec<String>) -> ShieldResult<AuthToken> {
        if !self.principals.contains_key(principal_id) {
            return Err(ShieldError::AuthFailed(format!("Unknown principal: {}", principal_id)));
        }
        let token = AuthToken::new(principal_id, ttl_secs, scopes);
        debug!("Issued token {} for principal {}", token.token_id, principal_id);
        self.tokens.insert(token.token_id.clone(), token.clone());
        Ok(token)
    }

    pub fn validate_token(&self, token_id: &str) -> ShieldResult<Principal> {
        let token = self.tokens.get(token_id)
            .ok_or_else(|| ShieldError::InvalidToken(token_id.to_string()))?;

        if !token.is_valid() {
            return Err(ShieldError::TokenExpired);
        }

        let principal = self.principals.get(&token.principal_id)
            .ok_or_else(|| ShieldError::AuthFailed("Principal not found".to_string()))?;

        Ok(principal.clone())
    }

    pub fn validate_api_key(&self, api_key: &str) -> ShieldResult<Principal> {
        let principal_id = self.api_keys.get(api_key)
            .ok_or_else(|| ShieldError::InvalidToken("Invalid API key".to_string()))?;

        let principal = self.principals.get(principal_id.value())
            .ok_or_else(|| ShieldError::AuthFailed("Principal not found".to_string()))?;

        Ok(principal.clone())
    }

    pub fn revoke_token(&self, token_id: &str) {
        if let Some(mut token) = self.tokens.get_mut(token_id) {
            token.revoked = true;
            warn!("Token {} revoked", token_id);
        }
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}
