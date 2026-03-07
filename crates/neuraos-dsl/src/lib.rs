//! # neuraos-dsl
//! A YAML/TOML-based DSL for defining NeuraOS agent workflows declaratively.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod ast;
pub mod compiler;
pub mod parser;
pub mod validator;

pub use ast::Workflow;
pub use parser::parse_workflow;

pub mod ast {
    //! Abstract syntax tree for DSL workflows.
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    /// A complete workflow definition.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Workflow {
        /// Workflow name.
        pub name: String,
        /// Semantic version string.
        pub version: String,
        /// Optional human-readable description.
        pub description: Option<String>,
        /// Ordered list of workflow steps.
        pub steps: Vec<WorkflowStep>,
        /// Arbitrary workflow-level metadata.
        pub metadata: HashMap<String, serde_json::Value>,
    }

    /// A single step in a workflow.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WorkflowStep {
        /// Unique step ID within this workflow.
        pub id: String,
        /// Human-readable step name.
        pub name: String,
        /// Action to invoke (e.g. "tool.bash", "agent.spawn").
        pub action: String,
        /// Parameters passed to the action.
        pub params: HashMap<String, serde_json::Value>,
        /// IDs of steps that must complete before this one.
        pub depends_on: Vec<String>,
        /// Optional retry policy.
        pub retry: Option<RetryPolicy>,
    }

    /// Retry policy for a workflow step.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RetryPolicy {
        /// Maximum number of attempts (including the first).
        pub max_attempts: u32,
        /// Delay between attempts in milliseconds.
        pub delay_ms: u64,
    }
}

pub mod parser {
    //! YAML parser for workflow definitions.
    use super::ast::Workflow;
    use anyhow::Result;

    /// Parse a YAML string into a Workflow.
    pub fn parse_workflow(yaml: &str) -> Result<Workflow> {
        let wf: Workflow = serde_yaml::from_str(yaml)?;
        Ok(wf)
    }
}

pub mod compiler {
    //! Compiles DSL workflows to executable runtime plans.
}

pub mod validator {
    //! Validates workflow definitions for correctness.
    use super::ast::Workflow;
    use anyhow::Result;

    /// Validate a workflow definition.
    pub fn validate(workflow: &Workflow) -> Result<()> {
        if workflow.name.is_empty() {
            anyhow::bail!("Workflow name cannot be empty");
        }
        if workflow.steps.is_empty() {
            anyhow::bail!("Workflow must have at least one step");
        }
        Ok(())
    }
}
