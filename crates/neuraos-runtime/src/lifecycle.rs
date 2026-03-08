// neuraos-runtime/src/lifecycle.rs
// Lifecycle management for runtime components

use crate::{RuntimeError, RuntimeResult};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleState {
    Uninitialized,
    Initializing,
    Running,
    Pausing,
    Paused,
    Resuming,
    ShuttingDown,
    Stopped,
    Failed(String),
}

impl std::fmt::Display for LifecycleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uninitialized  => write!(f, "Uninitialized"),
            Self::Initializing   => write!(f, "Initializing"),
            Self::Running        => write!(f, "Running"),
            Self::Pausing        => write!(f, "Pausing"),
            Self::Paused         => write!(f, "Paused"),
            Self::Resuming       => write!(f, "Resuming"),
            Self::ShuttingDown   => write!(f, "ShuttingDown"),
            Self::Stopped        => write!(f, "Stopped"),
            Self::Failed(e)      => write!(f, "Failed({})", e),
        }
    }
}

#[async_trait]
pub trait ManagedComponent: Send + Sync {
    fn component_name(&self) -> &str;
    async fn initialize(&self) -> RuntimeResult<()>;
    async fn start(&self) -> RuntimeResult<()>;
    async fn stop(&self) -> RuntimeResult<()>;
    async fn health_check(&self) -> RuntimeResult<bool>;
}

pub struct LifecycleManager {
    state: Arc<RwLock<LifecycleState>>,
    components: Arc<RwLock<Vec<Box<dyn ManagedComponent>>>>,
}

impl LifecycleManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(LifecycleState::Uninitialized)),
            components: Arc::new(RwLock::new(vec![])),
        }
    }

    pub async fn register<C: ManagedComponent + 'static>(&self, component: C) {
        self.components.write().await.push(Box::new(component));
    }

    pub async fn state(&self) -> LifecycleState {
        self.state.read().await.clone()
    }

    pub async fn start_all(&self) -> RuntimeResult<()> {
        {
            let mut state = self.state.write().await;
            *state = LifecycleState::Initializing;
        }

        let components = self.components.read().await;
        for component in components.iter() {
            info!("Initializing component: {}", component.component_name());
            component.initialize().await?;
            component.start().await?;
            info!("Component started: {}", component.component_name());
        }

        let mut state = self.state.write().await;
        *state = LifecycleState::Running;
        info!("All components started, lifecycle state: Running");
        Ok(())
    }

    pub async fn stop_all(&self) -> RuntimeResult<()> {
        {
            let mut state = self.state.write().await;
            *state = LifecycleState::ShuttingDown;
        }

        let components = self.components.read().await;
        for component in components.iter().rev() {
            warn!("Stopping component: {}", component.component_name());
            component.stop().await?;
        }

        let mut state = self.state.write().await;
        *state = LifecycleState::Stopped;
        info!("All components stopped");
        Ok(())
    }

    pub async fn health_check_all(&self) -> Vec<(String, bool)> {
        let components = self.components.read().await;
        let mut results = vec![];
        for component in components.iter() {
            let healthy = component.health_check().await.unwrap_or(false);
            results.push((component.component_name().to_string(), healthy));
        }
        results
    }
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}
