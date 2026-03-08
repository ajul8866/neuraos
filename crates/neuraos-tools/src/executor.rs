// neuraos-tools/src/executor.rs
// Tool call executor with validation and tracing

use crate::{ToolError, ToolResult2, ToolRegistry};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub tool_name: String,
    pub arguments: Value,
    pub caller_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl ToolCall {
    pub fn new(tool_name: impl Into<String>, arguments: Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            tool_name: tool_name.into(),
            arguments,
            caller_id: None,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: String,
    pub tool_name: String,
    pub success: bool,
    pub output: Value,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

pub struct ToolExecutor {
    registry: Arc<ToolRegistry>,
    timeout_secs: u64,
}

impl ToolExecutor {
    pub fn new(registry: Arc<ToolRegistry>, timeout_secs: u64) -> Self {
        Self { registry, timeout_secs }
    }

    pub async fn execute(&self, call: ToolCall) -> ToolResult {
        let start = std::time::Instant::now();
        let entry = match self.registry.get(&call.tool_name) {
            Some(e) => e,
            None => {
                return ToolResult {
                    call_id: call.id,
                    tool_name: call.tool_name,
                    success: false,
                    output: Value::Null,
                    error: Some(format!("Tool not found")),
                    duration_ms: 0,
                    timestamp: Utc::now(),
                };
            }
        };

        debug!("Executing tool: {} (call_id={})", call.tool_name, call.id);
        let result = timeout(
            Duration::from_secs(self.timeout_secs),
            entry.tool.execute(call.arguments.clone()),
        )
        .await;

        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(output)) => {
                info!("Tool {} succeeded in {}ms", call.tool_name, duration_ms);
                ToolResult {
                    call_id: call.id,
                    tool_name: call.tool_name,
                    success: true,
                    output,
                    error: None,
                    duration_ms,
                    timestamp: Utc::now(),
                }
            }
            Ok(Err(e)) => {
                error!("Tool {} failed: {}", call.tool_name, e);
                ToolResult {
                    call_id: call.id,
                    tool_name: call.tool_name,
                    success: false,
                    output: Value::Null,
                    error: Some(e.to_string()),
                    duration_ms,
                    timestamp: Utc::now(),
                }
            }
            Err(_) => ToolResult {
                call_id: call.id,
                tool_name: call.tool_name,
                success: false,
                output: Value::Null,
                error: Some("Tool execution timed out".to_string()),
                duration_ms,
                timestamp: Utc::now(),
            },
        }
    }
}
