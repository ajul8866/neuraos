// neuraos-api/src/handlers/agents.rs
use axum::{extract::{Path, State}, Json};
use crate::{AppState, ApiError, ApiResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub description: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub session_id: Option<String>,
}

pub async fn list_agents(State(_state): State<AppState>) -> ApiResult<Json<Value>> {
    Ok(Json(json!({ "agents": [], "total": 0 })))
}

pub async fn create_agent(
    State(_state): State<AppState>,
    Json(body): Json<CreateAgentRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "name": body.name,
        "status": "created",
    })))
}

pub async fn get_agent(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    Err(ApiError::NotFound(format!("Agent {} not found", id)))
}

pub async fn delete_agent(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    Ok(Json(json!({ "deleted": id })))
}

pub async fn chat(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<ChatRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(json!({
        "agent_id": id,
        "response": format!("Echo: {}", body.message),
        "session_id": body.session_id,
    })))
}
