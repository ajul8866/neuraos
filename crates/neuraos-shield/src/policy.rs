// neuraos-shield/src/policy.rs
// Policy engine for access control and enforcement

use crate::{ShieldError, ShieldResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolicyEffect {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCondition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    Contains,
    StartsWith,
    EndsWith,
    GreaterThan,
    LessThan,
    In,
    NotIn,
    Regex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub name: String,
    pub effect: PolicyEffect,
    pub principals: Vec<String>,   // principal ids or "*"
    pub resources: Vec<String>,    // resource patterns
    pub actions: Vec<String>,      // action names or "*"
    pub conditions: Vec<PolicyCondition>,
    pub priority: i32,
    pub enabled: bool,
}

impl Policy {
    pub fn allow(name: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            effect: PolicyEffect::Allow,
            principals: vec![],
            resources: vec![],
            actions: vec![],
            conditions: vec![],
            priority: 0,
            enabled: true,
        }
    }

    pub fn deny(name: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            effect: PolicyEffect::Deny,
            principals: vec![],
            resources: vec![],
            actions: vec![],
            conditions: vec![],
            priority: 100,
            enabled: true,
        }
    }

    pub fn for_principal(mut self, principal: impl Into<String>) -> Self {
        self.principals.push(principal.into());
        self
    }

    pub fn on_resource(mut self, resource: impl Into<String>) -> Self {
        self.resources.push(resource.into());
        self
    }

    pub fn allow_action(mut self, action: impl Into<String>) -> Self {
        self.actions.push(action.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRequest {
    pub principal_id: String,
    pub resource: String,
    pub action: String,
    pub context: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub allowed: bool,
    pub reason: String,
    pub matched_policy: Option<String>,
}

impl PolicyDecision {
    pub fn allow(reason: impl Into<String>, policy: Option<String>) -> Self {
        Self { allowed: true, reason: reason.into(), matched_policy: policy }
    }

    pub fn deny(reason: impl Into<String>, policy: Option<String>) -> Self {
        Self { allowed: false, reason: reason.into(), matched_policy: policy }
    }
}

pub struct PolicyEngine {
    policies: Arc<DashMap<String, Policy>>,
    default_effect: PolicyEffect,
}

impl PolicyEngine {
    pub fn new(default_effect: PolicyEffect) -> Self {
        Self {
            policies: Arc::new(DashMap::new()),
            default_effect,
        }
    }

    pub fn add_policy(&self, policy: Policy) {
        info!("Adding policy: {} (effect={:?})", policy.name, policy.effect);
        self.policies.insert(policy.id.clone(), policy);
    }

    pub fn remove_policy(&self, policy_id: &str) {
        self.policies.remove(policy_id);
    }

    pub fn evaluate(&self, request: &PolicyRequest) -> PolicyDecision {
        let mut sorted: Vec<_> = self.policies.iter()
            .filter(|p| p.enabled)
            .map(|p| p.clone())
            .collect();
        sorted.sort_by_key(|p| std::cmp::Reverse(p.priority));

        for policy in &sorted {
            if self.matches_policy(policy, request) {
                debug!("Request matched policy: {}", policy.name);
                return match policy.effect {
                    PolicyEffect::Allow => PolicyDecision::allow(
                        format!("Allowed by policy: {}", policy.name),
                        Some(policy.id.clone()),
                    ),
                    PolicyEffect::Deny => PolicyDecision::deny(
                        format!("Denied by policy: {}", policy.name),
                        Some(policy.id.clone()),
                    ),
                };
            }
        }

        match self.default_effect {
            PolicyEffect::Allow => PolicyDecision::allow("Default allow", None),
            PolicyEffect::Deny => PolicyDecision::deny("Default deny - no matching policy", None),
        }
    }

    fn matches_policy(&self, policy: &Policy, request: &PolicyRequest) -> bool {
        let principal_match = policy.principals.is_empty()
            || policy.principals.iter().any(|p| p == "*" || p == &request.principal_id);

        let resource_match = policy.resources.is_empty()
            || policy.resources.iter().any(|r| r == "*" || r == &request.resource || request.resource.starts_with(r));

        let action_match = policy.actions.is_empty()
            || policy.actions.iter().any(|a| a == "*" || a == &request.action);

        principal_match && resource_match && action_match
    }
}

use std::cmp::Reverse;
