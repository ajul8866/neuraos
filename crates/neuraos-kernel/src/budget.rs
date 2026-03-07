//! Token and cost budget governor — soft warnings + hard blocks.

use dashmap::DashMap;
use std::sync::Arc;
use tracing::{info, warn};

/// Decision returned by budget check.
#[derive(Debug, Clone, PartialEq)]
pub enum BudgetDecision {
    Allow,
    Warn { message: String },
    Block { reason: String },
}

impl BudgetDecision {
    pub fn is_allowed(&self) -> bool {
        !matches!(self, BudgetDecision::Block { .. })
    }
}

/// Per-agent budget limits.
#[derive(Debug, Clone)]
pub struct AgentBudget {
    pub agent_id: String,
    /// Hard token limit (None = unlimited).
    pub max_tokens: Option<u32>,
    /// Hard USD cost limit (None = unlimited).
    pub max_cost_usd: Option<f64>,
    /// Warn when tokens exceed this fraction of the limit.
    pub warn_threshold: f32,
}

/// Usage counters per agent.
#[derive(Debug, Default, Clone)]
pub struct AgentUsage {
    pub tokens_used: u32,
    pub cost_usd: f64,
    pub calls: u32,
}

/// Budget governor — thread-safe, single instance per kernel.
#[derive(Default)]
pub struct BudgetGovernor {
    budgets: DashMap<String, AgentBudget>,
    usage: DashMap<String, AgentUsage>,
}

impl BudgetGovernor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register or update a budget for an agent.
    pub fn set_budget(&self, budget: AgentBudget) {
        self.budgets.insert(budget.agent_id.clone(), budget);
    }

    /// Check if an agent can spend the given tokens/cost.
    pub fn check(&self, agent_id: &str, tokens: u32, cost: f64) -> BudgetDecision {
        let usage = self.usage.get(agent_id).map(|u| u.clone()).unwrap_or_default();
        let new_tokens = usage.tokens_used + tokens;
        let new_cost = usage.cost_usd + cost;

        if let Some(budget) = self.budgets.get(agent_id) {
            // Hard token limit
            if let Some(max_tok) = budget.max_tokens {
                if new_tokens > max_tok {
                    return BudgetDecision::Block {
                        reason: format!(
                            "Token limit exceeded: {} > {} for agent {}",
                            new_tokens, max_tok, agent_id
                        ),
                    };
                }
                let warn_tok = (max_tok as f32 * budget.warn_threshold) as u32;
                if new_tokens > warn_tok {
                    warn!("Agent {} approaching token limit ({}/{})", agent_id, new_tokens, max_tok);
                    return BudgetDecision::Warn {
                        message: format!("Approaching token limit: {}/{}", new_tokens, max_tok),
                    };
                }
            }

            // Hard cost limit
            if let Some(max_cost) = budget.max_cost_usd {
                if new_cost > max_cost {
                    return BudgetDecision::Block {
                        reason: format!(
                            "Cost limit exceeded: ${:.4} > ${:.4} for agent {}",
                            new_cost, max_cost, agent_id
                        ),
                    };
                }
                let warn_cost = max_cost * budget.warn_threshold as f64;
                if new_cost > warn_cost {
                    return BudgetDecision::Warn {
                        message: format!(
                            "Approaching cost limit: ${:.4}/${:.4}",
                            new_cost, max_cost
                        ),
                    };
                }
            }
        }

        BudgetDecision::Allow
    }

    /// Record actual token/cost consumption.
    pub fn consume(&self, agent_id: &str, tokens: u32, cost: f64) {
        let mut usage = self.usage.entry(agent_id.to_string()).or_default();
        usage.tokens_used += tokens;
        usage.cost_usd += cost;
        usage.calls += 1;
    }

    /// Reset usage counters for an agent (e.g. start of new billing period).
    pub fn reset(&self, agent_id: &str) {
        self.usage.remove(agent_id);
        info!("Budget reset for agent {}", agent_id);
    }

    /// Get current usage for an agent.
    pub fn usage(&self, agent_id: &str) -> AgentUsage {
        self.usage.get(agent_id).map(|u| u.clone()).unwrap_or_default()
    }

    /// Total tokens used across all agents.
    pub fn total_tokens_used(&self) -> u64 {
        self.usage.iter().map(|u| u.tokens_used as u64).sum()
    }

    /// Total cost across all agents.
    pub fn total_cost_usd(&self) -> f64 {
        self.usage.iter().map(|u| u.cost_usd).sum()
    }
}
