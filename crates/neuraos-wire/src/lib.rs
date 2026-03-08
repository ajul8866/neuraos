//! neuraos-wire — HTTP API layer for the NeuraOS platform.
//!
//! Provides Axum-based REST routes that expose the Kernel to external clients.

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

// ── Re-exports ────────────────────────────────────────────────────────────────

pub use neuraos_types::{AgentId, TaskId, Result as NeuraResult};

// ── App state ────────────────────────────────────────────────────────────────

pub type AppState = Arc<neuraos_kernel::Kernel>;

// ── Entry point ──────────────────────────────────────────────────────────────

/// Start the HTTP API server and block until shutdown.
pub async fn serve(kernel: neuraos_kernel::Kernel, bind: &str) -> Result<()> {
    let state: AppState = Arc::new(kernel);
    let app = router(state);
    let listener = TcpListener::bind(bind).await?;
    info!(addr = %listener.local_addr()?, "HTTP API listening");
    axum::serve(listener, app).await?;
    Ok(())
}

/// Build the Axum router (useful for testing without binding).
pub fn router(state: AppState) -> Router {
    Router::new()
        // Health
        .route("/health",           get(health_handler))
        .route("/version",          get(version_handler))
        // Agents
        .route("/agents",           get(list_agents).post(create_agent))
        .route("/agents/:id",       get(get_agent).delete(delete_agent))
        .route("/agents/:id/run",   post(run_agent))
        // Tasks
        .route("/tasks",            get(list_tasks).post(create_task))
        .route("/tasks/:id",        get(get_task).delete(cancel_task))
        // Memory
        .route("/memory",           get(list_memory).post(store_memory))
        .route("/memory/:id",       get(get_memory).delete(delete_memory))
        // LLM
        .route("/llm/chat",         post(chat_completion))
        .with_state(state)
}

// ── Health ────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct HealthResponse {
    status:  &'static str,
    version: &'static str,
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok", version: env!("CARGO_PKG_VERSION") })
}

async fn version_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "name": "neuraos"
    }))
}

// ── Agents ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CreateAgentRequest {
    name:        String,
    description: Option<String>,
    model:       Option<String>,
}

#[derive(Serialize)]
struct AgentResponse {
    id:     String,
    name:   String,
    status: String,
}

async fn list_agents(State(_state): State<AppState>) -> impl IntoResponse {
    // TODO: delegate to kernel.list_agents()
    Json(serde_json::json!({ "agents": [] }))
}

async fn create_agent(
    State(_state): State<AppState>,
    Json(req): Json<CreateAgentRequest>,
) -> impl IntoResponse {
    let id = AgentId::new();
    info!(name = %req.name, id = %id, "creating agent");
    (
        StatusCode::CREATED,
        Json(AgentResponse { id: id.to_string(), name: req.name, status: "idle".into() }),
    )
}

async fn get_agent(
    Path(id): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    Json(serde_json::json!({ "id": id, "status": "idle" }))
}

async fn delete_agent(
    Path(id): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    info!(id = %id, "deleting agent");
    StatusCode::NO_CONTENT
}

async fn run_agent(
    Path(id): Path<String>,
    State(_state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!(id = %id, "running agent");
    Json(serde_json::json!({ "task_id": TaskId::new().to_string(), "agent_id": id }))
}

// ── Tasks ─────────────────────────────────────────────────────────────────────

async fn list_tasks(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({ "tasks": [] }))
}

async fn create_task(
    State(_state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let id = TaskId::new();
    (StatusCode::CREATED, Json(serde_json::json!({ "task_id": id.to_string() })))
}

async fn get_task(
    Path(id): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    Json(serde_json::json!({ "id": id, "status": "pending" }))
}

async fn cancel_task(
    Path(id): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    info!(id = %id, "cancelling task");
    StatusCode::NO_CONTENT
}

// ── Memory ────────────────────────────────────────────────────────────────────

async fn list_memory(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({ "entries": [] }))
}

async fn store_memory(
    State(_state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let id = neuraos_types::MemoryId::new();
    (StatusCode::CREATED, Json(serde_json::json!({ "id": id.0 })))
}

async fn get_memory(
    Path(id): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    Json(serde_json::json!({ "id": id }))
}

async fn delete_memory(
    Path(id): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

// ── LLM ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ChatRequest {
    messages: Vec<neuraos_types::Message>,
    model:    Option<String>,
}

async fn chat_completion(
    State(_state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> impl IntoResponse {
    // TODO: delegate to kernel.llm_router()
    Json(serde_json::json!({
        "choices": [{
            "message": { "role": "assistant", "content": "Hello from NeuraOS!" }
        }]
    }))
}

// ── Error handling ────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ApiError {
    error:   String,
    code:    u16,
}

impl ApiError {
    fn new(msg: impl Into<String>, code: u16) -> (StatusCode, Json<Self>) {
        let status = StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(Self { error: msg.into(), code }))
    }
}
