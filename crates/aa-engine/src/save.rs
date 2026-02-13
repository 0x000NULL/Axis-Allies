//! Phase 13: Save/Load System
//!
//! Full game state serialization to/from JSON and MessagePack formats.
//! Includes save file metadata (version, timestamp, summary) and validation.

use serde::{Deserialize, Serialize};

use crate::error::EngineError;
use crate::state::GameState;

/// Current save file format version.
pub const SAVE_FORMAT_VERSION: u32 = 1;

/// A complete save file with metadata and game state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveFile {
    /// Format version for forward compatibility.
    pub version: u32,
    /// Save file metadata.
    pub metadata: SaveMetadata,
    /// The full game state.
    pub state: GameState,
}

/// Metadata about a save file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveMetadata {
    /// Human-readable save name.
    pub name: String,
    /// Unix timestamp when saved (seconds since epoch).
    pub timestamp: u64,
    /// Turn summary string.
    pub summary: String,
    /// Total number of actions taken.
    pub action_count: usize,
}

impl SaveFile {
    /// Create a new save file from the current engine state.
    pub fn from_state(state: &GameState, name: String, timestamp: u64) -> Self {
        let summary = format!(
            "Turn {} - {:?} - {:?}",
            state.turn_number, state.current_power, state.current_phase
        );

        SaveFile {
            version: SAVE_FORMAT_VERSION,
            metadata: SaveMetadata {
                name,
                timestamp,
                summary,
                action_count: state.action_log.len(),
            },
            state: state.clone(),
        }
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, EngineError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| EngineError::Serialization(e.to_string()))
    }

    /// Deserialize from JSON string.
    pub fn from_json(json: &str) -> Result<Self, EngineError> {
        let save: SaveFile = serde_json::from_str(json)
            .map_err(|e| EngineError::Deserialization(e.to_string()))?;
        save.validate()?;
        Ok(save)
    }

    /// Serialize to compact JSON (no pretty printing).
    pub fn to_json_compact(&self) -> Result<String, EngineError> {
        serde_json::to_string(self)
            .map_err(|e| EngineError::Serialization(e.to_string()))
    }

    /// Serialize to MessagePack bytes.
    pub fn to_msgpack(&self) -> Result<Vec<u8>, EngineError> {
        rmp_serde::to_vec(self)
            .map_err(|e| EngineError::Serialization(e.to_string()))
    }

    /// Deserialize from MessagePack bytes.
    pub fn from_msgpack(data: &[u8]) -> Result<Self, EngineError> {
        let save: SaveFile = rmp_serde::from_slice(data)
            .map_err(|e| EngineError::Deserialization(e.to_string()))?;
        save.validate()?;
        Ok(save)
    }

    /// Validate save file integrity.
    pub fn validate(&self) -> Result<(), EngineError> {
        if self.version == 0 || self.version > SAVE_FORMAT_VERSION {
            return Err(EngineError::Deserialization(format!(
                "Unsupported save format version: {} (supported: 1-{})",
                self.version, SAVE_FORMAT_VERSION
            )));
        }

        if self.state.turn_number == 0 {
            return Err(EngineError::Deserialization(
                "Invalid save: turn_number is 0".into(),
            ));
        }

        if self.state.territories.is_empty() {
            return Err(EngineError::Deserialization(
                "Invalid save: no territories".into(),
            ));
        }

        Ok(())
    }

    /// Extract just the metadata without fully deserializing state (JSON only).
    /// Useful for save file browsers.
    pub fn peek_metadata_json(json: &str) -> Result<SaveMetadata, EngineError> {
        // Deserialize just enough to get metadata
        #[derive(Deserialize)]
        struct SaveHeader {
            version: u32,
            metadata: SaveMetadata,
        }

        let header: SaveHeader = serde_json::from_str(json)
            .map_err(|e| EngineError::Deserialization(e.to_string()))?;
        Ok(header.metadata)
    }
}

/// Convenience: serialize just the game state to JSON.
pub fn state_to_json(state: &GameState) -> Result<String, EngineError> {
    serde_json::to_string_pretty(state)
        .map_err(|e| EngineError::Serialization(e.to_string()))
}

/// Convenience: deserialize just a game state from JSON.
pub fn state_from_json(json: &str) -> Result<GameState, EngineError> {
    serde_json::from_str(json)
        .map_err(|e| EngineError::Deserialization(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Engine;
    use crate::phase::Phase;
    use crate::power::Power;

    #[test]
    fn test_save_file_json_roundtrip() {
        let engine = Engine::new_game(42);
        let save = SaveFile::from_state(engine.state(), "Test Save".into(), 1700000000);

        let json = save.to_json().unwrap();
        assert!(json.contains("Test Save"));
        assert!(json.contains("version"));

        let loaded = SaveFile::from_json(&json).unwrap();
        assert_eq!(loaded.version, SAVE_FORMAT_VERSION);
        assert_eq!(loaded.metadata.name, "Test Save");
        assert_eq!(loaded.metadata.timestamp, 1700000000);
        assert_eq!(loaded.state.turn_number, 1);
        assert_eq!(loaded.state.current_power, Power::Germany);
    }

    #[test]
    fn test_save_file_msgpack_roundtrip() {
        let engine = Engine::new_game(42);
        let save = SaveFile::from_state(engine.state(), "MsgPack Test".into(), 1700000000);

        let bytes = save.to_msgpack().unwrap();
        assert!(!bytes.is_empty());

        let loaded = SaveFile::from_msgpack(&bytes).unwrap();
        assert_eq!(loaded.metadata.name, "MsgPack Test");
        assert_eq!(loaded.state.turn_number, 1);
    }

    #[test]
    fn test_save_file_compact_json() {
        let engine = Engine::new_game(42);
        let save = SaveFile::from_state(engine.state(), "Compact".into(), 0);

        let compact = save.to_json_compact().unwrap();
        let pretty = save.to_json().unwrap();

        // Compact should be shorter (no whitespace)
        assert!(compact.len() < pretty.len());

        // Both should deserialize to the same thing
        let loaded = SaveFile::from_json(&compact).unwrap();
        assert_eq!(loaded.state.turn_number, 1);
    }

    #[test]
    fn test_save_after_actions() {
        let mut engine = Engine::new_game(42);

        // Make some purchases
        engine
            .submit_action(crate::action::Action::PurchaseUnit {
                unit_type: crate::unit::UnitType::Infantry,
                count: 5,
            })
            .unwrap();
        engine
            .submit_action(crate::action::Action::ConfirmPurchases)
            .unwrap();

        let save = SaveFile::from_state(engine.state(), "Mid-turn".into(), 1700000000);
        assert_eq!(save.metadata.action_count, 2);

        let json = save.to_json().unwrap();
        let loaded = SaveFile::from_json(&json).unwrap();
        assert_eq!(loaded.state.current_phase, Phase::CombatMovement);
        assert_eq!(loaded.metadata.action_count, 2);
    }

    #[test]
    fn test_load_and_resume() {
        let mut engine = Engine::new_game(42);
        engine
            .submit_action(crate::action::Action::ConfirmPurchases)
            .unwrap();

        let save = SaveFile::from_state(engine.state(), "Resume Test".into(), 0);
        let json = save.to_json().unwrap();

        // Load into a new engine
        let loaded = SaveFile::from_json(&json).unwrap();
        let mut engine2 = Engine::from_state(loaded.state);

        // Should be able to continue playing
        assert_eq!(engine2.state().current_phase, Phase::CombatMovement);
        engine2
            .submit_action(crate::action::Action::ConfirmCombatMovement)
            .unwrap();
        assert_eq!(engine2.state().current_phase, Phase::ConductCombat);
    }

    #[test]
    fn test_invalid_version_rejected() {
        let engine = Engine::new_game(42);
        let mut save = SaveFile::from_state(engine.state(), "Bad".into(), 0);
        save.version = 999;

        let json = serde_json::to_string(&save).unwrap();
        let result = SaveFile::from_json(&json);
        assert!(result.is_err());
    }

    #[test]
    fn test_peek_metadata() {
        let engine = Engine::new_game(42);
        let save = SaveFile::from_state(engine.state(), "Peek Test".into(), 1234567890);
        let json = save.to_json().unwrap();

        let metadata = SaveFile::peek_metadata_json(&json).unwrap();
        assert_eq!(metadata.name, "Peek Test");
        assert_eq!(metadata.timestamp, 1234567890);
    }

    #[test]
    fn test_state_json_roundtrip() {
        let engine = Engine::new_game(42);
        let json = state_to_json(engine.state()).unwrap();
        let loaded = state_from_json(&json).unwrap();
        assert_eq!(loaded.turn_number, 1);
        assert_eq!(loaded.current_power, Power::Germany);
        assert_eq!(loaded.territories.len(), engine.state().territories.len());
    }

    #[test]
    fn test_save_preserves_all_state() {
        let engine = Engine::new_game(42);
        let original = engine.state();

        let save = SaveFile::from_state(original, "Full State".into(), 0);
        let json = save.to_json().unwrap();
        let loaded = SaveFile::from_json(&json).unwrap();

        // Verify key state fields
        assert_eq!(loaded.state.turn_number, original.turn_number);
        assert_eq!(loaded.state.current_power, original.current_power);
        assert_eq!(loaded.state.current_phase, original.current_phase);
        assert_eq!(loaded.state.territories.len(), original.territories.len());
        assert_eq!(loaded.state.sea_zones.len(), original.sea_zones.len());
        assert_eq!(loaded.state.powers.len(), original.powers.len());
        assert_eq!(loaded.state.rng_seed, original.rng_seed);

        // Verify unit counts match
        let original_units: usize = original
            .territories
            .iter()
            .map(|t| t.units.len())
            .sum();
        let loaded_units: usize = loaded
            .state
            .territories
            .iter()
            .map(|t| t.units.len())
            .sum();
        assert_eq!(loaded_units, original_units);

        // Verify IPCs match
        for i in 0..9 {
            assert_eq!(loaded.state.powers[i].ipcs, original.powers[i].ipcs);
        }
    }

    #[test]
    fn test_msgpack_smaller_than_json() {
        let engine = Engine::new_game(42);
        let save = SaveFile::from_state(engine.state(), "Size Test".into(), 0);

        let json_bytes = save.to_json_compact().unwrap().len();
        let msgpack_bytes = save.to_msgpack().unwrap().len();

        assert!(
            msgpack_bytes < json_bytes,
            "MsgPack ({}) should be smaller than JSON ({})",
            msgpack_bytes,
            json_bytes
        );
    }
}
