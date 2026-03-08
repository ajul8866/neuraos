// neuraos-tools/src/builtin.rs
// Built-in tool implementations

use crate::{ToolError, ToolResult2, schema::{ToolSchema, ToolParameter, ParamType}};
use async_trait::async_trait;
use serde_json::Value;

/// HTTP fetch tool
pub struct HttpFetchTool;

#[async_trait]
impl crate::registry::Tool for HttpFetchTool {
    fn name(&self) -> &str { "http_fetch" }
    fn description(&self) -> &str { "Fetch content from a URL via HTTP GET" }
    fn schema(&self) -> ToolSchema {
        ToolSchema::new("http_fetch", "Fetch content from a URL")
            .with_param(ToolParameter {
                name: "url".to_string(),
                description: "The URL to fetch".to_string(),
                param_type: ParamType::String,
                required: true,
                default: None,
                enum_values: None,
            })
    }
    async fn execute(&self, args: Value) -> ToolResult2<Value> {
        let url = args["url"].as_str()
            .ok_or_else(|| ToolError::InvalidArguments("url is required".to_string()))?;
        // In production, use reqwest
        Ok(serde_json::json!({ "url": url, "status": "ok", "content": "" }))
    }
}

/// JSON parse tool
pub struct JsonParseTool;

#[async_trait]
impl crate::registry::Tool for JsonParseTool {
    fn name(&self) -> &str { "json_parse" }
    fn description(&self) -> &str { "Parse a JSON string into a structured value" }
    fn schema(&self) -> ToolSchema {
        ToolSchema::new("json_parse", "Parse JSON string")
            .with_param(ToolParameter {
                name: "input".to_string(),
                description: "JSON string to parse".to_string(),
                param_type: ParamType::String,
                required: true,
                default: None,
                enum_values: None,
            })
    }
    async fn execute(&self, args: Value) -> ToolResult2<Value> {
        let input = args["input"].as_str()
            .ok_or_else(|| ToolError::InvalidArguments("input is required".to_string()))?;
        serde_json::from_str(input)
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))
    }
}

/// Echo tool (for testing)
pub struct EchoTool;

#[async_trait]
impl crate::registry::Tool for EchoTool {
    fn name(&self) -> &str { "echo" }
    fn description(&self) -> &str { "Echo back the input unchanged" }
    fn schema(&self) -> ToolSchema {
        ToolSchema::new("echo", "Echo input")
            .with_param(ToolParameter {
                name: "message".to_string(),
                description: "Message to echo".to_string(),
                param_type: ParamType::String,
                required: true,
                default: None,
                enum_values: None,
            })
    }
    async fn execute(&self, args: Value) -> ToolResult2<Value> {
        Ok(serde_json::json!({ "echo": args["message"] }))
    }
}

pub fn register_builtins(registry: &crate::ToolRegistry) {
    registry.register(HttpFetchTool);
    registry.register(JsonParseTool);
    registry.register(EchoTool);
}
