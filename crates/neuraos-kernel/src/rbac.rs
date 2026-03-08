//! Role-Based Access Control engine with deny-override policy evaluation.

use neuraos_types::{Policy, PolicyDecision, PolicyEffect, PolicyRule};
use dashmap::DashMap;
use std::collections::HashSet;
use tracing::{debug, warn};

/// Built-in roles with predefined permission sets.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BuiltinRole {
    System,      // unrestricted
    Admin,       // all operations, can manage users
    Operator,    // run agents, manage tasks
    Developer,   // read + execute tools, no admin
    ReadOnly,    // read-only across all resources
    Custom(String),
}

impl std::fmt::Display for BuiltinRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "system"),
            Self::Admin => write!(f, "admin"),
            Self::Operator => write!(f, "operator"),
            Self::Developer => write!(f, "developer"),
            Self::ReadOnly => write!(f, "readonly"),
            Self::Custom(n) => write!(f, "custom:{n}"),
        }
    }
}

/// Subject → roles mapping.
#[derive(Default)]
struct SubjectRoles {
    roles: HashSet<String>,
}

/// RBAC engine — evaluates access decisions for (subject, action, resource) triples.
pub struct RbacEngine {
    /// subject_id → set of role names
    subject_roles: DashMap<String, SubjectRoles>,
    /// role_name → policies
    role_policies: DashMap<String, Vec<Policy>>,
    /// Global policies (apply to all subjects).
    global_policies: Vec<Policy>,
}

impl RbacEngine {
    pub fn new() -> Self {
        let engine = Self {
            subject_roles: DashMap::new(),
            role_policies: DashMap::new(),
            global_policies: Vec::new(),
        };
        engine.seed_defaults();
        engine
    }

    fn seed_defaults(&self) {
        use neuraos_types::{PolicyEffect, PolicyRule};

        // System role: allow everything
        self.add_policy_to_role(
            "system",
            Policy {
                id: "system-allow-all".into(),
                name: "System Allow All".into(),
                description: "System role has unrestricted access".into(),
                rules: vec![PolicyRule {
                    resource: "*".into(),
                    action: "*".into(),
                    condition: None,
                }],
                effect: PolicyEffect::Allow,
                priority: 1000,
            },
        );

        // Admin role: allow everything except system internals
        self.add_policy_to_role(
            "admin",
            Policy {
                id: "admin-allow-all".into(),
                name: "Admin Allow All".into(),
                description: "Admins can do everything".into(),
                rules: vec![PolicyRule {
                    resource: "*".into(),
                    action: "*".into(),
                    condition: None,
                }],
                effect: PolicyEffect::Allow,
                priority: 900,
            },
        );

        // Operator: can run agents and tasks
        self.add_policy_to_role(
            "operator",
            Policy {
                id: "operator-run".into(),
                name: "Operator Run".into(),
                description: "Operators can run agents and tasks".into(),
                rules: vec![
                    PolicyRule { resource: "agent:*".into(), action: "*".into(), condition: None },
                    PolicyRule { resource: "task:*".into(), action: "*".into(), condition: None },
                    PolicyRule { resource: "tool:*".into(), action: "execute".into(), condition: None },
                    PolicyRule { resource: "memory:*".into(), action: "read".into(), condition: None },
                ],
                effect: PolicyEffect::Allow,
                priority: 700,
            },
        );

        // Developer: read + safe tools
        self.add_policy_to_role(
            "developer",
            Policy {
                id: "developer-safe".into(),
                name: "Developer Safe".into(),
                description: "Developers can use safe tools".into(),
                rules: vec![
                    PolicyRule { resource: "agent:*".into(), action: "read".into(), condition: None },
                    PolicyRule { resource: "task:*".into(), action: "*".into(), condition: None },
                    PolicyRule { resource: "tool:http:*".into(), action: "execute".into(), condition: None },
                    PolicyRule { resource: "tool:filesystem:*".into(), action: "execute".into(), condition: None },
                    PolicyRule { resource: "tool:web_search".into(), action: "execute".into(), condition: None },
                    PolicyRule { resource: "memory:*".into(), action: "read".into(), condition: None },
                ],
                effect: PolicyEffect::Allow,
                priority: 500,
            },
        );

        // ReadOnly: only read actions
        self.add_policy_to_role(
            "readonly",
            Policy {
                id: "readonly-read".into(),
                name: "Read Only".into(),
                description: "Read-only access".into(),
                rules: vec![PolicyRule {
                    resource: "*".into(),
                    action: "read".into(),
                    condition: None,
                }],
                effect: PolicyEffect::Allow,
                priority: 100,
            },
        );
    }

    /// Assign a role to a subject (user or agent).
    pub fn assign_role(&self, subject_id: &str, role: &str) {
        self.subject_roles
            .entry(subject_id.to_string())
            .or_default()
            .roles
            .insert(role.to_string());
    }

    /// Remove a role from a subject.
    pub fn revoke_role(&self, subject_id: &str, role: &str) {
        if let Some(mut entry) = self.subject_roles.get_mut(subject_id) {
            entry.roles.remove(role);
        }
    }

    /// Add a policy to a role.
    pub fn add_policy_to_role(&self, role: &str, policy: Policy) {
        self.role_policies
            .entry(role.to_string())
            .or_default()
            .push(policy);
    }

    /// Check if subject can perform action on resource.
    /// Deny-override: any explicit Deny wins over Allow.
    pub fn check(&self, subject: &str, action: &str, resource: &str) -> PolicyDecision {
        debug!("RBAC check: subject={} action={} resource={}", subject, action, resource);

        let roles: Vec<String> = self
            .subject_roles
            .get(subject)
            .map(|s| s.roles.iter().cloned().collect())
            .unwrap_or_default();

        // Collect all applicable policies
        let mut applicable: Vec<&Policy> = self.global_policies.iter().collect();
        for role in &roles {
            if let Some(policies) = self.role_policies.get(role) {
                let iter: Vec<&Policy> = policies.iter().collect();
                applicable.extend(iter);
            }
        }

        // Sort by priority descending
        applicable.sort_by(|a, b| b.priority.cmp(&a.priority));

        let mut allow = false;

        for policy in applicable {
            let rule_matches = policy.rules.iter().any(|rule| {
                glob_match(&rule.resource, resource) && glob_match(&rule.action, action)
            });

            if rule_matches {
                match &policy.effect {
                    PolicyEffect::Deny => {
                        warn!("RBAC DENY: {} {} {} (policy: {})", subject, action, resource, policy.id);
                        return PolicyDecision::Deny {
                            reason: format!("Policy '{}' denies this action", policy.name),
                        };
                    }
                    PolicyEffect::Allow => {
                        allow = true;
                    }
                    PolicyEffect::RequireApproval => {
                        return PolicyDecision::RequireApproval {
                            approver: "admin".into(),
                        };
                    }
                    _ => {}
                }
            }
        }

        if allow {
            PolicyDecision::Allow
        } else {
            PolicyDecision::Deny {
                reason: format!("No policy allows {subject} to {action} on {resource}"),
            }
        }
    }

    /// Get all roles for a subject.
    pub fn roles(&self, subject: &str) -> Vec<String> {
        self.subject_roles
            .get(subject)
            .map(|s| s.roles.iter().cloned().collect())
            .unwrap_or_default()
    }
}

impl Default for RbacEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple glob matching: `*` matches any sequence, `?` matches one char.
fn glob_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    glob_match_inner(&pattern.chars().collect::<Vec<_>>(), &text.chars().collect::<Vec<_>>(), 0, 0)
}

fn glob_match_inner(pat: &[char], txt: &[char], pi: usize, ti: usize) -> bool {
    if pi == pat.len() {
        return ti == txt.len();
    }
    if pat[pi] == '*' {
        // Try matching zero or more characters
        for skip in ti..=txt.len() {
            if glob_match_inner(pat, txt, pi + 1, skip) {
                return true;
            }
        }
        return false;
    }
    if ti == txt.len() {
        return false;
    }
    if pat[pi] == '?' || pat[pi] == txt[ti] {
        return glob_match_inner(pat, txt, pi + 1, ti + 1);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match() {
        assert!(glob_match("*", "anything"));
        assert!(glob_match("tool:*", "tool:bash"));
        assert!(glob_match("tool:bash:*", "tool:bash:execute"));
        assert!(!glob_match("tool:bash", "tool:other"));
        assert!(glob_match("*:read", "resource:read"));
    }

    #[test]
    fn test_rbac_admin_allows_all() {
        let engine = RbacEngine::new();
        engine.assign_role("alice", "admin");
        let d = engine.check("alice", "execute", "tool:bash");
        assert_eq!(d, PolicyDecision::Allow);
    }

    #[test]
    fn test_rbac_unknown_subject_denied() {
        let engine = RbacEngine::new();
        let d = engine.check("unknown", "execute", "tool:bash");
        assert!(matches!(d, PolicyDecision::Deny { .. }));
    }
}
