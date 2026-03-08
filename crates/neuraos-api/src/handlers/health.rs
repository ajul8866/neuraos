// neuraos-api/src/handlers/health.rs
use axum::{extract::State, Json};
use crate::AppState;
use serde_json::{json, Value};
use chrono::Utc;

pub async fn health_check(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "status": "ok",
        "app": state.app_name,
        "timestamp": Utc::now().to_rfc3339(),
    }))
}

pub async fn version(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "app": state.app_name,
        "version": state.version,
    }))
}
