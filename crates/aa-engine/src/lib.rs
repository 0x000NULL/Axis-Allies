//! Axis & Allies Global 1940 2nd Edition - Game Engine
//!
//! This crate contains ALL game logic for the digital edition.
//! It compiles to native Rust (for server/Tauri) and to WASM (for browser).
//! No game logic should exist outside this crate.

pub mod action;
pub mod apply;
pub mod combat;
pub mod dice;
pub mod error;
pub mod phase;
pub mod power;
pub mod setup;
pub mod state;
pub mod territory;
pub mod unit;
pub mod validate;

pub mod data;

use action::{Action, ActionResult, GameEvent, LegalAction};
use data::GameMap;
use error::EngineError;
use state::GameState;

/// The main game engine. Single source of truth for all game rules.
pub struct Engine {
    state: GameState,
    map: GameMap,
}

impl Engine {
    /// Create a new game with default Global 1940 2nd Edition setup.
    pub fn new_game(seed: u64) -> Self {
        let map = GameMap::new();
        let state = setup::create_initial_state(seed, &map);
        Engine { state, map }
    }

    /// Restore an engine from a previously serialized game state.
    pub fn from_state(state: GameState) -> Self {
        let map = GameMap::new();
        Engine { state, map }
    }

    /// Get a reference to the static game map.
    pub fn map(&self) -> &GameMap {
        &self.map
    }

    /// Submit a player action. The engine validates, applies, and returns the result.
    pub fn submit_action(&mut self, action: Action) -> Result<ActionResult, EngineError> {
        validate::validate_action(&self.state, &action)?;
        let result = apply::apply_action(&mut self.state, action)?;
        Ok(result)
    }

    /// Get a reference to the current game state.
    pub fn state(&self) -> &GameState {
        &self.state
    }

    /// Check whether the last action can be undone.
    pub fn can_undo(&self) -> bool {
        self.state
            .action_log
            .last()
            .map(|a| !matches!(a.inverse, action::InverseAction::Irreversible))
            .unwrap_or(false)
    }

    /// Validate whether an action is legal without applying it.
    pub fn is_action_legal(&self, action: &Action) -> Result<(), EngineError> {
        validate::validate_action(&self.state, action)
    }

    /// Get the list of currently legal actions.
    pub fn legal_actions(&self) -> Vec<LegalAction> {
        let mut actions = Vec::new();

        // Add the phase-specific confirm action
        let (confirm_action, description) = match self.state.current_phase {
            phase::Phase::PurchaseAndRepair => (
                Action::ConfirmPurchases,
                "Confirm purchases and advance to Combat Movement",
            ),
            phase::Phase::CombatMovement => (
                Action::ConfirmCombatMovement,
                "Confirm combat moves and advance to Conduct Combat",
            ),
            phase::Phase::ConductCombat => (
                Action::ConfirmPhase,
                "Confirm combat results and advance to Non-Combat Movement",
            ),
            phase::Phase::NonCombatMovement => (
                Action::ConfirmNonCombatMovement,
                "Confirm non-combat moves and advance to Mobilize",
            ),
            phase::Phase::Mobilize => (
                Action::ConfirmMobilization,
                "Confirm unit placements and advance to Collect Income",
            ),
            phase::Phase::CollectIncome => (
                Action::ConfirmIncome,
                "Collect income and end turn",
            ),
        };
        actions.push(LegalAction {
            action: confirm_action,
            description: description.to_string(),
        });

        // Add Undo if available
        if self.can_undo() {
            actions.push(LegalAction {
                action: Action::Undo,
                description: "Undo the last action".to_string(),
            });
        }

        actions
    }

    /// Check if a victory condition has been met.
    pub fn check_victory(&self) -> Option<GameEvent> {
        None // Phase 8
    }

    /// Serialize the game state to JSON (for WASM bridge).
    pub fn serialize_state_json(&self) -> Result<String, EngineError> {
        serde_json::to_string(&self.state).map_err(|e| EngineError::Serialization(e.to_string()))
    }

    /// Serialize the game state to MessagePack bytes (for save files).
    pub fn serialize_state(&self) -> Result<Vec<u8>, EngineError> {
        rmp_serde::to_vec(&self.state).map_err(|e| EngineError::Serialization(e.to_string()))
    }

    /// Deserialize a game state from MessagePack bytes.
    pub fn deserialize_state(data: &[u8]) -> Result<GameState, EngineError> {
        rmp_serde::from_slice(data).map_err(|e| EngineError::Deserialization(e.to_string()))
    }

    /// Get a summary string for the current game state (for save file headers, etc.)
    pub fn turn_summary(&self) -> String {
        format!(
            "Turn {} - {:?} - {:?}",
            self.state.turn_number, self.state.current_power, self.state.current_phase
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use action::Action;
    use phase::Phase;
    use power::Power;

    /// Helper: advance a power through all 6 phases (Purchase → CollectIncome).
    fn advance_through_phases(engine: &mut Engine) {
        engine.submit_action(Action::ConfirmPurchases).unwrap();
        engine.submit_action(Action::ConfirmCombatMovement).unwrap();
        engine.submit_action(Action::ConfirmPhase).unwrap(); // ConductCombat
        engine.submit_action(Action::ConfirmNonCombatMovement).unwrap();
        engine.submit_action(Action::ConfirmMobilization).unwrap();
        engine.submit_action(Action::ConfirmIncome).unwrap();
    }

    #[test]
    fn test_new_game_creates_valid_state() {
        let engine = Engine::new_game(42);
        let state = engine.state();
        assert_eq!(state.turn_number, 1);
        assert_eq!(state.current_power, Power::Germany);
        assert_eq!(state.current_phase, Phase::PurchaseAndRepair);
    }

    #[test]
    fn test_serialize_deserialize_json() {
        let engine = Engine::new_game(42);
        let json = engine.serialize_state_json().unwrap();
        assert!(!json.is_empty());
        let parsed: GameState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.turn_number, 1);
    }

    #[test]
    fn test_serialize_deserialize_messagepack() {
        let engine = Engine::new_game(42);
        let bytes = engine.serialize_state().unwrap();
        assert!(!bytes.is_empty());
        let restored = Engine::deserialize_state(&bytes).unwrap();
        assert_eq!(restored.turn_number, 1);
    }

    #[test]
    fn test_full_phase_cycle_one_power() {
        let mut engine = Engine::new_game(42);
        assert_eq!(engine.state().current_power, Power::Germany);
        assert_eq!(engine.state().current_phase, Phase::PurchaseAndRepair);

        engine.submit_action(Action::ConfirmPurchases).unwrap();
        assert_eq!(engine.state().current_phase, Phase::CombatMovement);

        engine.submit_action(Action::ConfirmCombatMovement).unwrap();
        assert_eq!(engine.state().current_phase, Phase::ConductCombat);

        engine.submit_action(Action::ConfirmPhase).unwrap();
        assert_eq!(engine.state().current_phase, Phase::NonCombatMovement);

        engine.submit_action(Action::ConfirmNonCombatMovement).unwrap();
        assert_eq!(engine.state().current_phase, Phase::Mobilize);

        engine.submit_action(Action::ConfirmMobilization).unwrap();
        assert_eq!(engine.state().current_phase, Phase::CollectIncome);

        engine.submit_action(Action::ConfirmIncome).unwrap();
        // Should advance to SovietUnion
        assert_eq!(engine.state().current_power, Power::SovietUnion);
        assert_eq!(engine.state().current_phase, Phase::PurchaseAndRepair);
    }

    #[test]
    fn test_full_turn_cycle_all_powers() {
        let mut engine = Engine::new_game(42);
        assert_eq!(engine.state().turn_number, 1);

        for &expected_power in power::TURN_ORDER.iter() {
            assert_eq!(engine.state().current_power, expected_power);
            advance_through_phases(&mut engine);
        }

        // After all 9 powers, back to Germany with turn_number incremented
        assert_eq!(engine.state().current_power, Power::Germany);
        assert_eq!(engine.state().turn_number, 2);
    }

    #[test]
    fn test_multi_turn() {
        let mut engine = Engine::new_game(42);

        for _ in 0..3 {
            for _ in 0..9 {
                advance_through_phases(&mut engine);
            }
        }

        assert_eq!(engine.state().turn_number, 4);
        assert_eq!(engine.state().current_power, Power::Germany);
    }

    #[test]
    fn test_wrong_phase_rejected() {
        let engine = Engine::new_game(42);
        // We're in PurchaseAndRepair, so ConfirmCombatMovement should fail
        let result = engine.is_action_legal(&Action::ConfirmCombatMovement);
        assert!(result.is_err());
        match result.unwrap_err() {
            error::EngineError::WrongPhase { expected, actual } => {
                assert_eq!(expected, "CombatMovement");
                assert_eq!(actual, "PurchaseAndRepair");
            }
            other => panic!("Expected WrongPhase, got {:?}", other),
        }
    }

    #[test]
    fn test_undo_not_available() {
        let mut engine = Engine::new_game(42);
        // No actions yet
        assert!(!engine.can_undo());

        // After a phase transition (Irreversible), still can't undo
        engine.submit_action(Action::ConfirmPurchases).unwrap();
        assert!(!engine.can_undo());
    }

    #[test]
    fn test_undo_fails_on_irreversible() {
        let mut engine = Engine::new_game(42);
        engine.submit_action(Action::ConfirmPurchases).unwrap();

        let result = engine.submit_action(Action::Undo);
        assert!(result.is_err());
        match result.unwrap_err() {
            error::EngineError::CannotUndo { reason } => {
                assert!(reason.contains("cannot be undone"));
            }
            other => panic!("Expected CannotUndo, got {:?}", other),
        }
    }

    #[test]
    fn test_phase_change_events() {
        let mut engine = Engine::new_game(42);
        let result = engine.submit_action(Action::ConfirmPurchases).unwrap();

        assert_eq!(result.events.len(), 1);
        match &result.events[0] {
            GameEvent::PhaseChanged { from, to } => {
                assert_eq!(*from, Phase::PurchaseAndRepair);
                assert_eq!(*to, Phase::CombatMovement);
            }
            other => panic!("Expected PhaseChanged, got {:?}", other),
        }
    }

    #[test]
    fn test_turn_change_events() {
        let mut engine = Engine::new_game(42);
        // Advance through 5 phases to get to CollectIncome
        engine.submit_action(Action::ConfirmPurchases).unwrap();
        engine.submit_action(Action::ConfirmCombatMovement).unwrap();
        engine.submit_action(Action::ConfirmPhase).unwrap();
        engine.submit_action(Action::ConfirmNonCombatMovement).unwrap();
        engine.submit_action(Action::ConfirmMobilization).unwrap();

        let result = engine.submit_action(Action::ConfirmIncome).unwrap();
        assert_eq!(result.events.len(), 2);

        match &result.events[0] {
            GameEvent::PhaseChanged { from, to } => {
                assert_eq!(*from, Phase::CollectIncome);
                assert_eq!(*to, Phase::PurchaseAndRepair);
            }
            other => panic!("Expected PhaseChanged, got {:?}", other),
        }

        match &result.events[1] {
            GameEvent::TurnChanged { power, turn } => {
                assert_eq!(*power, Power::SovietUnion);
                assert_eq!(*turn, 1);
            }
            other => panic!("Expected TurnChanged, got {:?}", other),
        }
    }

    #[test]
    fn test_undo_checkpoints_recorded() {
        let mut engine = Engine::new_game(42);
        // Initial checkpoint
        assert_eq!(engine.state().undo_checkpoints.len(), 1);
        assert_eq!(engine.state().undo_checkpoints[0], 0);

        engine.submit_action(Action::ConfirmPurchases).unwrap();
        assert_eq!(engine.state().undo_checkpoints.len(), 2);

        engine.submit_action(Action::ConfirmCombatMovement).unwrap();
        assert_eq!(engine.state().undo_checkpoints.len(), 3);
    }

    #[test]
    fn test_legal_actions_per_phase() {
        let mut engine = Engine::new_game(42);

        // PurchaseAndRepair phase
        let actions = engine.legal_actions();
        assert_eq!(actions.len(), 1); // Just ConfirmPurchases (no undo available)
        assert!(matches!(actions[0].action, Action::ConfirmPurchases));

        // Advance to CombatMovement
        engine.submit_action(Action::ConfirmPurchases).unwrap();
        let actions = engine.legal_actions();
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0].action, Action::ConfirmCombatMovement));

        // Advance to ConductCombat
        engine.submit_action(Action::ConfirmCombatMovement).unwrap();
        let actions = engine.legal_actions();
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0].action, Action::ConfirmPhase));

        // Advance to NonCombatMovement
        engine.submit_action(Action::ConfirmPhase).unwrap();
        let actions = engine.legal_actions();
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0].action, Action::ConfirmNonCombatMovement));

        // Advance to Mobilize
        engine.submit_action(Action::ConfirmNonCombatMovement).unwrap();
        let actions = engine.legal_actions();
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0].action, Action::ConfirmMobilization));

        // Advance to CollectIncome
        engine.submit_action(Action::ConfirmMobilization).unwrap();
        let actions = engine.legal_actions();
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0].action, Action::ConfirmIncome));
    }
}
