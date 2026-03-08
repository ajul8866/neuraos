// neuraos-api/src/handlers/memory.rs
use axum::{extract::State, Json};
use crate::{AppState, ApiResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<usize>,
}

pub async fn list_memory(State(_state): State<AppState>) -> ApiResult<Json<Value>> {
    Ok(Json(json!({ "memories": [], "total": 0 })))
}

pub async fn search_memory(
    State(_state): State<AppState>,
    Json(body): Json<SearchRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(json!({
        "query": body.query,
        "results": [],
        "total": 0,
    })))
}
