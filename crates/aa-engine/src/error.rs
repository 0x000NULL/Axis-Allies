//! Engine error types. The engine never panics; all errors are returned as Results.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

/// All possible errors from the game engine.
#[derive(Clone, Debug, Error, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum EngineError {
    #[error("Not your turn: current power is {current:?}")]
    NotYourTurn { current: String },

    #[error("Wrong phase: expected {expected}, got {actual}")]
    WrongPhase { expected: String, actual: String },

    #[error("Insufficient IPCs: need {needed}, have {available}")]
    InsufficientIPCs { needed: u32, available: u32 },

    #[error("Illegal move: {reason}")]
    IllegalMove { reason: String },

    #[error("Invalid action: {reason}")]
    InvalidAction { reason: String },

    #[error("Unit not found: {unit_id}")]
    UnitNotFound { unit_id: u32 },

    #[error("Territory not found: {territory_id}")]
    TerritoryNotFound { territory_id: u16 },

    #[error("Cannot undo: {reason}")]
    CannotUndo { reason: String },

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Game setup error: {0}")]
    SetupError(String),

    #[error("Internal engine error: {0}")]
    Internal(String),
}
