//! Axis & Allies multiplayer WebSocket server.
//!
//! Provides HTTP + WebSocket endpoints for online multiplayer.
//! Each game room has its own authoritative Engine instance.

mod protocol;

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/ws", get(ws_handler))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("Failed to bind to port 3001");

    tracing::info!("Axis & Allies server listening on 0.0.0.0:3001");
    axum::serve(listener, app)
        .await
        .expect("Server failed");
}

async fn index() -> &'static str {
    "Axis & Allies Global 1940 - Multiplayer Server"
}

async fn health() -> &'static str {
    "ok"
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    // Send a welcome message
    let welcome = protocol::ServerMessage::Welcome {
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    let msg = serde_json::to_string(&welcome).unwrap();
    if socket.send(Message::Text(msg.into())).await.is_err() {
        return;
    }

    // Basic message loop (will be expanded in Phase 14)
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                match serde_json::from_str::<protocol::ClientMessage>(&text) {
                    Ok(client_msg) => {
                        let response = protocol::handle_message(client_msg);
                        let resp_json = serde_json::to_string(&response).unwrap();
                        if socket.send(Message::Text(resp_json.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let error = protocol::ServerMessage::Error {
                            message: format!("Invalid message: {}", e),
                        };
                        let resp_json = serde_json::to_string(&error).unwrap();
                        if socket.send(Message::Text(resp_json.into())).await.is_err() {
                            break;
                        }
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}
