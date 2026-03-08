// neuraos-api/src/router.rs
// Axum router setup

use crate::state::AppState;
use axum::{
    routing::{get, post, delete},
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
    compression::CompressionLayer,
};

use crate::handlers::{
    health::health_check,
    agents::{list_agents, get_agent, create_agent, delete_agent, run_agent},
    tasks::{list_tasks, get_task, cancel_task},
    tools::{list_tools, call_tool},
    memory::{get_memory, set_memory, delete_memory},
};

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health
        .route("/health", get(health_check))
        // Agents
        .route("/api/v1/agents", get(list_agents).post(create_agent))
        .route("/api/v1/agents/:id", get(get_agent).delete(delete_agent))
        .route("/api/v1/agents/:id/run", post(run_agent))
        // Tasks
        .route("/api/v1/tasks", get(list_tasks))
        .route("/api/v1/tasks/:id", get(get_task))
        .route("/api/v1/tasks/:id/cancel", post(cancel_task))
        // Tools
        .route("/api/v1/tools", get(list_tools))
        .route("/api/v1/tools/call", post(call_tool))
        // Memory
        .route("/api/v1/memory/:key", get(get_memory).put(set_memory).delete(delete_memory))
        // Middleware
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .with_state(state)
}
