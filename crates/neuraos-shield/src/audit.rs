// neuraos-shield/src/audit.rs
// Immutable audit log for security events

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditSeverity {
    Info,
    Warning,
    Critical,
    Alert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub severity: AuditSeverity,
    pub category: String,
    pub principal_id: Option<String>,
    pub resource: Option<String>,
    pub action: String,
    pub outcome: AuditOutcome,
    pub detail: String,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
    pub source_ip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditOutcome {
    Success,
    Failure,
    Attempt,
    Blocked,
}

impl AuditEvent {
    pub fn new(
        severity: AuditSeverity,
        category: impl Into<String>,
        action: impl Into<String>,
        outcome: AuditOutcome,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            severity,
            category: category.into(),
            principal_id: None,
            resource: None,
            action: action.into(),
            outcome,
            detail: detail.into(),
            metadata: std::collections::HashMap::new(),
            source_ip: None,
        }
    }

    pub fn with_principal(mut self, principal_id: impl Into<String>) -> Self {
        self.principal_id = Some(principal_id.into());
        self
    }

    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    pub fn with_source_ip(mut self, ip: impl Into<String>) -> Self {
        self.source_ip = Some(ip.into());
        self
    }
}

pub struct AuditLogger {
    events: Arc<RwLock<Vec<AuditEvent>>>,
    max_events: usize,
}

impl AuditLogger {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::with_capacity(1024))),
            max_events,
        }
    }

    pub async fn log(&self, event: AuditEvent) {
        info!(
            "[AUDIT] {:?} | {} | {} | {:?} | {}",
            event.severity, event.category, event.action, event.outcome, event.detail
        );
        let mut events = self.events.write().await;
        if events.len() >= self.max_events {
            events.remove(0); // rolling window
        }
        events.push(event);
    }

    pub async fn query(
        &self,
        category: Option<&str>,
        severity: Option<AuditSeverity>,
        limit: usize,
    ) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .rev()
            .filter(|e| {
                category.map_or(true, |c| e.category == c)
                    && severity.as_ref().map_or(true, |s| &e.severity == s)
            })
            .take(limit)
            .cloned()
            .collect()
    }

    pub async fn count(&self) -> usize {
        self.events.read().await.len()
    }

    pub async fn clear(&self) {
        self.events.write().await.clear();
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(100_000)
    }
}
