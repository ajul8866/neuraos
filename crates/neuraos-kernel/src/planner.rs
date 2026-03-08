//! MCTS-based task planner with greedy fallback.

use neuraos_types::{Task, TaskStep, ToolCapability};
use std::time::{Duration, Instant};
use tracing::{info, warn};
use uuid::Uuid;

/// Context provided to the planner for goal decomposition.
#[derive(Debug, Clone)]
pub struct PlanContext {
    pub available_tools: Vec<ToolCapability>,
    pub constraints: Vec<String>,
    pub budget_tokens: Option<u32>,
    pub budget_cost_usd: Option<f64>,
    pub max_steps: usize,
    pub agent_id: String,
}

impl Default for PlanContext {
    fn default() -> Self {
        Self {
            available_tools: Vec::new(),
            constraints: Vec::new(),
            budget_tokens: None,
            budget_cost_usd: None,
            max_steps: 20,
            agent_id: String::new(),
        }
    }
}

/// An ordered plan ready for execution.
#[derive(Debug, Clone)]
pub struct Plan {
    pub id: String,
    pub goal: String,
    pub steps: Vec<TaskStep>,
    pub estimated_tokens: u32,
    pub estimated_cost_usd: f64,
    pub confidence: f32,
    pub method: PlanMethod,
}

impl Plan {
    pub fn into_task(self, description: impl Into<String>) -> Task {
        let mut task = Task::new(&self.goal, description);
        task.steps = self.steps;
        task.budget_tokens = Some(self.estimated_tokens);
        task.budget_cost_usd = Some(self.estimated_cost_usd);
        task
    }
}

#[derive(Debug, Clone)]
pub enum PlanMethod {
    Mcts { simulations: u32 },
    Greedy,
}

/// MCTS node for tree search.
#[derive(Debug, Clone)]
struct MctsNode {
    step: Option<TaskStep>,
    visits: u32,
    total_reward: f64,
    children: Vec<usize>,
    parent: Option<usize>,
    depth: usize,
}

impl MctsNode {
    fn uct_score(&self, parent_visits: u32, exploration: f64) -> f64 {
        if self.visits == 0 {
            return f64::INFINITY;
        }
        let exploitation = self.total_reward / self.visits as f64;
        let exploration_term =
            exploration * ((parent_visits as f64).ln() / self.visits as f64).sqrt();
        exploitation + exploration_term
    }
}

/// Planner configuration.
#[derive(Debug, Clone)]
pub struct PlannerConfig {
    pub simulations: u32,
    pub timeout_ms: u64,
    pub exploration_constant: f64,
    pub max_depth: usize,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            simulations: 100,
            timeout_ms: 5000,
            exploration_constant: 1.414,
            max_depth: 15,
        }
    }
}

/// Main planner struct.
pub struct Planner {
    config: PlannerConfig,
}

impl Planner {
    pub fn new(config: PlannerConfig) -> Self {
        Self { config }
    }

    /// Decompose a goal into an ordered Plan.
    /// Uses MCTS if time budget allows, otherwise falls back to greedy.
    pub fn plan(&self, goal: &str, context: &PlanContext) -> Result<Plan, PlannerError> {
        info!("Planning goal: {}", goal);
        let deadline = Instant::now() + Duration::from_millis(self.config.timeout_ms);

        // Try MCTS
        match self.mcts_plan(goal, context, deadline) {
            Ok(plan) => Ok(plan),
            Err(e) => {
                warn!("MCTS planning failed ({e}), falling back to greedy");
                self.greedy_plan(goal, context)
            }
        }
    }

    fn mcts_plan(
        &self,
        goal: &str,
        context: &PlanContext,
        deadline: Instant,
    ) -> Result<Plan, PlannerError> {
        let mut nodes: Vec<MctsNode> = vec![MctsNode {
            step: None,
            visits: 0,
            total_reward: 0.0,
            children: Vec::new(),
            parent: None,
            depth: 0,
        }];

        let mut simulations_run = 0u32;

        for _ in 0..self.config.simulations {
            if Instant::now() >= deadline {
                break;
            }

            // Selection
            let mut node_idx = 0usize;
            loop {
                let node = &nodes[node_idx];
                if node.children.is_empty() {
                    break;
                }
                let parent_visits = node.visits;
                let best = node
                    .children
                    .iter()
                    .copied()
                    .max_by(|&a, &b| {
                        nodes[a]
                            .uct_score(parent_visits, self.config.exploration_constant)
                            .partial_cmp(&nodes[b].uct_score(
                                parent_visits,
                                self.config.exploration_constant,
                            ))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .unwrap_or(node_idx);
                if best == node_idx {
                    break;
                }
                node_idx = best;
            }

            // Expansion
            let depth = nodes[node_idx].depth;
            if depth < self.config.max_depth && depth < context.max_steps {
                let new_step = self.generate_step(goal, context, depth);
                let child_idx = nodes.len();
                nodes.push(MctsNode {
                    step: Some(new_step),
                    visits: 0,
                    total_reward: 0.0,
                    children: Vec::new(),
                    parent: Some(node_idx),
                    depth: depth + 1,
                });
                nodes[node_idx].children.push(child_idx);
                node_idx = child_idx;
            }

            // Simulation (rollout)
            let reward = self.rollout_reward(goal, context, depth);

            // Backpropagation
            let mut idx = node_idx;
            loop {
                nodes[idx].visits += 1;
                nodes[idx].total_reward += reward;
                if let Some(parent) = nodes[idx].parent {
                    idx = parent;
                } else {
                    break;
                }
            }

            simulations_run += 1;
        }

        // Extract best path from root
        let steps = self.extract_best_path(&nodes, context);
        if steps.is_empty() {
            return Err(PlannerError::NoPlanFound);
        }

        let confidence = if nodes[0].visits > 0 {
            (nodes[0].total_reward / nodes[0].visits as f64).min(1.0) as f32
        } else {
            0.5
        };

        Ok(Plan {
            id: Uuid::new_v4().to_string(),
            goal: goal.to_string(),
            steps,
            estimated_tokens: context.budget_tokens.unwrap_or(2000),
            estimated_cost_usd: context.budget_cost_usd.unwrap_or(0.01),
            confidence,
            method: PlanMethod::Mcts { simulations: simulations_run },
        })
    }

    fn greedy_plan(&self, goal: &str, context: &PlanContext) -> Result<Plan, PlannerError> {
        let steps: Vec<TaskStep> = (0..context.max_steps.min(5))
            .map(|i| self.generate_step(goal, context, i))
            .collect();

        Ok(Plan {
            id: Uuid::new_v4().to_string(),
            goal: goal.to_string(),
            steps,
            estimated_tokens: context.budget_tokens.unwrap_or(2000),
            estimated_cost_usd: context.budget_cost_usd.unwrap_or(0.01),
            confidence: 0.6,
            method: PlanMethod::Greedy,
        })
    }

    fn generate_step(&self, goal: &str, _context: &PlanContext, depth: usize) -> TaskStep {
        let name = match depth {
            0 => format!("Analyse goal: {}", &goal[..goal.len().min(60)]),
            1 => "Gather required information".to_string(),
            2 => "Execute primary action".to_string(),
            3 => "Verify results".to_string(),
            _ => format!("Refinement step {}", depth - 3),
        };
        TaskStep::new(name)
    }

    fn rollout_reward(&self, _goal: &str, _context: &PlanContext, depth: usize) -> f64 {
        // Heuristic: shallower plans score better to avoid padding
        1.0 / (1.0 + depth as f64 * 0.1)
    }

    fn extract_best_path(&self, nodes: &[MctsNode], context: &PlanContext) -> Vec<TaskStep> {
        let mut idx = 0usize;
        let mut steps = Vec::new();

        for _ in 0..context.max_steps {
            if nodes[idx].children.is_empty() {
                break;
            }
            let best_child = nodes[idx]
                .children
                .iter()
                .copied()
                .max_by(|&a, &b| {
                    let va = nodes[a].visits;
                    let vb = nodes[b].visits;
                    va.cmp(&vb)
                });

            if let Some(child) = best_child {
                if let Some(step) = &nodes[child].step {
                    steps.push(step.clone());
                }
                idx = child;
            } else {
                break;
            }
        }
        steps
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PlannerError {
    #[error("No plan found within simulation budget")]
    NoPlanFound,
    #[error("Planning timeout")]
    Timeout,
    #[error("Goal too complex: {0}")]
    TooComplex(String),
}
