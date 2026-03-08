//! # neuraos-skills
//! Composable, reusable agent capability modules.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod executor;
pub mod loader;
pub mod registry;
pub mod skill;

use async_trait::async_trait;
use neuraos_types::NeuraResult;
use serde_json::Value;

/// A Skill is a named, versioned capability that an agent can invoke.
#[async_trait]
pub trait Skill: Send + Sync + 'static {
    /// Unique identifier.
    fn id(&self) -> &str;
    /// Human-readable name.
    fn name(&self) -> &str;
    /// Version string.
    fn version(&self) -> &str;
    /// Description of what this skill does.
    fn description(&self) -> &str;
    /// Execute the skill with the given parameters.
    async fn execute(&self, params: Value) -> NeuraResult<Value>;
}

pub mod skill {
    //! Core skill trait re-export.
    pub use super::Skill;
}

pub mod registry {
    //! Skill registry.
    use std::collections::HashMap;
    use super::Skill;

    /// Registry that maps skill IDs to their implementations.
    #[derive(Default)]
    pub struct SkillRegistry {
        skills: HashMap<String, Box<dyn Skill>>,
    }

    impl SkillRegistry {
        /// Register a new skill.
        pub fn register(&mut self, skill: Box<dyn Skill>) {
            self.skills.insert(skill.id().to_string(), skill);
        }
        /// Get a skill by ID.
        pub fn get(&self, id: &str) -> Option<&dyn Skill> {
            self.skills.get(id).map(|s| s.as_ref())
        }
        /// List all registered skill IDs.
        pub fn list_ids(&self) -> Vec<&str> {
            self.skills.keys().map(|s| s.as_str()).collect()
        }
    }
}

pub mod executor {
    //! Skill execution engine.
}

pub mod loader {
    //! Skill loading from filesystem or registry.
}
