// neuraos-api/src/ws.rs
// WebSocket handler for real-time agent streaming

use axum::{
    extract::{State, WebSocketUpgrade, ws::{Message, WebSocket}},
    response::Response,
};
use crate::AppState;
use futures::{sink::SinkExt, stream::StreamExt};
use tracing::{debug, info, warn};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    info!("WebSocket connection established");

    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                debug!("WS received: {}", text);
                // Echo back with metadata
                let reply = serde_json::json!({
                    "echo": text,
                    "app": state.app_name,
                });
                if socket.send(Message::Text(reply.to_string())).await.is_err() {
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket client disconnected");
                break;
            }
            Err(e) => {
                warn!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}
