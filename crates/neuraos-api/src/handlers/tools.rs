// neuraos-api/src/handlers/tools.rs
use axum::{extract::{Path, State}, Json};
use crate::{AppState, ApiError, ApiResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Serialize, Deserialize)]
pub struct RunToolRequest {
    pub arguments: Value,
}

pub async fn list_tools(State(_state): State<AppState>) -> ApiResult<Json<Value>> {
    Ok(Json(json!({ "tools": [], "total": 0 })))
}

pub async fn run_tool(
    State(_state): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<RunToolRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(json!({
        "tool": name,
        "result": null,
        "success": true,
    })))
}
