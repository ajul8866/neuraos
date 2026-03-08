// neuraos-shield/src/sandbox.rs
// Sandboxing and resource isolation for agent execution

use crate::{ShieldError, ShieldResult};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub max_memory_mb: u64,
    pub max_cpu_pct: f32,
    pub max_execution_secs: u64,
    pub allowed_syscalls: Vec<String>,
    pub blocked_syscalls: Vec<String>,
    pub network_access: NetworkPolicy,
    pub filesystem_access: FilesystemPolicy,
    pub allowed_env_vars: Vec<String>,
    pub max_file_descriptors: u32,
    pub max_processes: u32,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 512,
            max_cpu_pct: 50.0,
            max_execution_secs: 60,
            allowed_syscalls: vec![],
            blocked_syscalls: vec![
                "ptrace".to_string(),
                "kexec_load".to_string(),
                "mount".to_string(),
                "umount2".to_string(),
            ],
            network_access: NetworkPolicy::Restricted(vec![]),
            filesystem_access: FilesystemPolicy::ReadOnly(vec!["/tmp".to_string()]),
            allowed_env_vars: vec!["PATH".to_string(), "HOME".to_string()],
            max_file_descriptors: 256,
            max_processes: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkPolicy {
    None,
    Restricted(Vec<String>), // allowed domains/IPs
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilesystemPolicy {
    None,
    ReadOnly(Vec<String>),   // allowed read paths
    ReadWrite(Vec<String>),  // allowed read-write paths
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SandboxState {
    Created,
    Active,
    Suspended,
    Terminated,
    Violated,
}

pub struct Sandbox {
    pub id: String,
    pub config: SandboxConfig,
    state: Arc<RwLock<SandboxState>>,
    violations: Arc<RwLock<Vec<SandboxViolation>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxViolation {
    pub kind: String,
    pub detail: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Sandbox {
    pub fn new(config: SandboxConfig) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            config,
            state: Arc::new(RwLock::new(SandboxState::Created)),
            violations: Arc::new(RwLock::new(vec![])),
        }
    }

    pub async fn activate(&self) -> ShieldResult<()> {
        let mut state = self.state.write().await;
        if *state != SandboxState::Created {
            return Err(ShieldError::SandboxViolation);
        }
        *state = SandboxState::Active;
        info!("Sandbox {} activated", self.id);
        Ok(())
    }

    pub async fn check_network_access(&self, target: &str) -> ShieldResult<()> {
        match &self.config.network_access {
            NetworkPolicy::None => Err(ShieldError::Denied(format!("Network access denied in sandbox {}", self.id))),
            NetworkPolicy::Full => Ok(()),
            NetworkPolicy::Restricted(allowed) => {
                if allowed.iter().any(|a| target.contains(a.as_str())) {
                    Ok(())
                } else {
                    self.record_violation("network", &format!("Blocked access to {}", target)).await;
                    Err(ShieldError::Denied(format!("Network target not allowed: {}", target)))
                }
            }
        }
    }

    pub async fn check_filesystem_access(&self, path: &str, write: bool) -> ShieldResult<()> {
        match &self.config.filesystem_access {
            FilesystemPolicy::None => Err(ShieldError::Denied("Filesystem access denied".to_string())),
            FilesystemPolicy::Full => Ok(()),
            FilesystemPolicy::ReadOnly(paths) => {
                if write {
                    return Err(ShieldError::Denied("Write access denied in read-only sandbox".to_string()));
                }
                if paths.iter().any(|p| path.starts_with(p.as_str())) {
                    Ok(())
                } else {
                    Err(ShieldError::Denied(format!("Path not allowed: {}", path)))
                }
            }
            FilesystemPolicy::ReadWrite(paths) => {
                if paths.iter().any(|p| path.starts_with(p.as_str())) {
                    Ok(())
                } else {
                    Err(ShieldError::Denied(format!("Path not allowed: {}", path)))
                }
            }
        }
    }

    async fn record_violation(&self, kind: &str, detail: &str) {
        let violation = SandboxViolation {
            kind: kind.to_string(),
            detail: detail.to_string(),
            timestamp: chrono::Utc::now(),
        };
        warn!("Sandbox {} violation: {} - {}", self.id, kind, detail);
        self.violations.write().await.push(violation);
        *self.state.write().await = SandboxState::Violated;
    }

    pub async fn violations(&self) -> Vec<SandboxViolation> {
        self.violations.read().await.clone()
    }

    pub async fn terminate(&self) {
        *self.state.write().await = SandboxState::Terminated;
        info!("Sandbox {} terminated", self.id);
    }
}
