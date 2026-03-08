// neuraos-tools/src/schema.rs
// JSON Schema definitions for tool parameters

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub param_type: ParamType,
    pub required: bool,
    pub default: Option<serde_json::Value>,
    pub enum_values: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Object,
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
    pub returns: String,
    pub examples: Vec<ToolExample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExample {
    pub description: String,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
}

impl ToolSchema {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: vec![],
            returns: "object".to_string(),
            examples: vec![],
        }
    }

    pub fn with_param(mut self, param: ToolParameter) -> Self {
        self.parameters.push(param);
        self
    }

    pub fn required_params(&self) -> Vec<&ToolParameter> {
        self.parameters.iter().filter(|p| p.required).collect()
    }

    pub fn to_json_schema(&self) -> serde_json::Value {
        let props: HashMap<String, serde_json::Value> = self.parameters.iter().map(|p| {
            let mut schema = serde_json::json!({
                "type": format!("{:?}", p.param_type).to_lowercase(),
                "description": p.description,
            });
            if let Some(ref ev) = p.enum_values {
                schema["enum"] = serde_json::Value::Array(ev.clone());
            }
            (p.name.clone(), schema)
        }).collect();

        let required: Vec<String> = self.parameters.iter()
            .filter(|p| p.required)
            .map(|p| p.name.clone())
            .collect();

        serde_json::json!({
            "type": "object",
            "properties": props,
            "required": required,
        })
    }
}
