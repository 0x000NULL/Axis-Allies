//! WebSocket protocol message types.
//!
//! Defines the client->server and server->client message formats.
//! Full room management and game logic will be added in Phase 14.

use serde::{Deserialize, Serialize};

/// Messages sent from client to server.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Ping,
    CreateRoom { player_name: String },
    JoinRoom { room_id: String, player_name: String },
}

/// Messages sent from server to client.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    Welcome { version: String },
    Pong,
    Error { message: String },
    RoomCreated { room_id: String },
}

/// Handle an incoming client message and return a response.
/// This is a placeholder; full implementation in Phase 14.
pub fn handle_message(msg: ClientMessage) -> ServerMessage {
    match msg {
        ClientMessage::Ping => ServerMessage::Pong,
        ClientMessage::CreateRoom { player_name: _ } => {
            // TODO: actual room creation
            ServerMessage::RoomCreated {
                room_id: "PLACEHOLDER".to_string(),
            }
        }
        ClientMessage::JoinRoom { room_id, .. } => ServerMessage::Error {
            message: format!("Room {} not found (not yet implemented)", room_id),
        },
    }
}
