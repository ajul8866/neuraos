// neuraos-api/src/handlers/tasks.rs
use axum::{extract::{Path, State}, Json};
use crate::{AppState, ApiError, ApiResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub name: String,
    pub payload: Value,
}

pub async fn list_tasks(State(_state): State<AppState>) -> ApiResult<Json<Value>> {
    Ok(Json(json!({ "tasks": [], "total": 0 })))
}

pub async fn create_task(
    State(_state): State<AppState>,
    Json(body): Json<CreateTaskRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "name": body.name,
        "status": "queued",
    })))
}

pub async fn get_task(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    Err(ApiError::NotFound(format!("Task {} not found", id)))
}
