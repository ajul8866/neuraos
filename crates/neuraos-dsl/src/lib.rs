//! neuraos-dsl -- Domain-Specific Language for NeuraOS agent workflows

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const DSL_VERSION: &str = "1.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DslDocument {
    pub version: String,
    pub workflows: HashMap<String, ast::Workflow>,
    #[serde(default)]
    pub globals: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub imports: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum DslError {
    #[error("Parse error: {0}")] ParseError(String),
    #[error("Serialize error: {0}")] SerializeError(String),
    #[error("Validation error: {0}")] ValidationError(String),
    #[error("Compilation error: {0}")] CompilationError(String),
    #[error("Runtime error: {0}")] RuntimeError(String),
    #[error("Undefined reference: {0}")] UndefinedReference(String),
    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch { expected: String, got: String },
}

impl DslDocument {
    pub fn from_yaml(input: &str) -> Result<Self, DslError> {
        serde_yaml::from_str(input).map_err(|e| DslError::ParseError(e.to_string()))
    }
    pub fn from_toml(input: &str) -> Result<Self, DslError> {
        toml::from_str(input).map_err(|e| DslError::ParseError(e.to_string()))
    }
    pub fn to_yaml(&self) -> Result<String, DslError> {
        serde_yaml::to_string(self).map_err(|e| DslError::SerializeError(e.to_string()))
    }
    pub fn validate(&self) -> Result<(), Vec<validator::ValidationError>> {
        validator::Validator::new().validate_document(self)
    }
}

pub mod ast {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Workflow {
        pub name: String,
        pub description: Option<String>,
        pub trigger: Option<Trigger>,
        pub steps: Vec<Step>,
        #[serde(default)]
        pub on_error: ErrorHandler,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Step {
        pub id: String,
        pub name: Option<String>,
        pub action: String,
        #[serde(default)]
        pub inputs: HashMap<String, Expr>,
        #[serde(default)]
        pub outputs: Vec<String>,
        pub depends_on: Option<Vec<String>>,
        pub condition: Option<Expr>,
        pub timeout_secs: Option<u64>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum Expr {
        Literal(serde_json::Value),
        Ref(String),
        Template(String),
        Call { func: String, args: Vec<Expr> },
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Pipeline { pub steps: Vec<Step> }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum Trigger {
        Manual,
        Schedule { cron: String },
        Event { event_type: String },
        Webhook { path: String },
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum ErrorHandler { #[default] Fail, Continue, Retry, Custom(String) }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RetryPolicy {
        pub max_attempts: u32,
        pub backoff_secs: u64,
        pub backoff_multiplier: f64,
    }
    impl Default for RetryPolicy {
        fn default() -> Self { Self { max_attempts: 3, backoff_secs: 1, backoff_multiplier: 2.0 } }
    }
}

pub mod validator {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct ValidationError { pub path: String, pub message: String }
    impl std::fmt::Display for ValidationError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}: {}", self.path, self.message)
        }
    }

    pub struct Validator;
    impl Validator {
        pub fn new() -> Self { Self }
        pub fn validate_document(&self, doc: &DslDocument) -> Result<(), Vec<ValidationError>> {
            let mut errors = Vec::new();
            for (name, wf) in &doc.workflows {
                if wf.steps.is_empty() {
                    errors.push(ValidationError {
                        path: format!("workflows.{}.steps", name),
                        message: "Workflow must have at least one step".to_string(),
                    });
                }
            }
            if errors.is_empty() { Ok(()) } else { Err(errors) }
        }
    }
}

pub mod compiler {
    use super::*;
    pub struct Compiler;
    impl Compiler {
        pub fn new() -> Self { Self }
        pub fn compile(&self, doc: &DslDocument) -> Result<CompiledDocument, DslError> {
            doc.validate().map_err(|e| DslError::ValidationError(e[0].to_string()))?;
            Ok(CompiledDocument { workflows: doc.workflows.keys().cloned().collect() })
        }
    }
    #[derive(Debug)]
    pub struct CompiledDocument { pub workflows: Vec<String> }
}

pub mod runtime {
    use super::*;
    pub struct ExecutionContext {
        pub variables: HashMap<String, serde_json::Value>,
        pub workflow_name: String,
    }
    impl ExecutionContext {
        pub fn new(workflow_name: impl Into<String>) -> Self {
            Self { variables: HashMap::new(), workflow_name: workflow_name.into() }
        }
        pub fn set(&mut self, key: impl Into<String>, value: serde_json::Value) {
            self.variables.insert(key.into(), value);
        }
        pub fn get(&self, key: &str) -> Option<&serde_json::Value> { self.variables.get(key) }
    }
    pub struct Runtime;
    impl Runtime { pub fn new() -> Self { Self } }
}
