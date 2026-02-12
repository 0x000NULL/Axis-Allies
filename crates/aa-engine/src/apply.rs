//! Action application dispatcher.
//!
//! Applies a validated action to the game state and returns the result.
//! Detailed application logic will be implemented in later phases.

use crate::action::{Action, ActionResult, AppliedAction, GameEvent, InverseAction};
use crate::error::EngineError;
use crate::phase::{
    CombatMoveState, CombatState, CollectIncomeState, MobilizeState, NonCombatMoveState, Phase,
    PhaseState, PurchaseState,
};
use crate::power;
use crate::state::GameState;

/// Apply a validated action to the game state.
pub fn apply_action(state: &mut GameState, action: Action) -> Result<ActionResult, EngineError> {
    // Undo is handled separately — it must NOT be pushed to the action_log
    if matches!(action, Action::Undo) {
        return apply_undo(state);
    }

    let mut events = Vec::new();

    match &action {
        Action::ConfirmPhase | Action::ConfirmPurchases | Action::ConfirmCombatMovement
        | Action::ConfirmNonCombatMovement | Action::ConfirmMobilization | Action::ConfirmIncome => {
            let old_phase = state.current_phase;

            // Save undo checkpoint at phase boundary
            state.undo_checkpoints.push(state.action_log.len());

            if let Some(next_phase) = state.current_phase.next() {
                state.current_phase = next_phase;
                state.phase_state = create_phase_state(next_phase);
                events.push(GameEvent::PhaseChanged {
                    from: old_phase,
                    to: next_phase,
                });
            } else {
                // End of turn: advance to next power
                let next = power::next_power(state.current_power);
                let old_power = state.current_power;
                state.current_power = next;
                state.current_phase = Phase::PurchaseAndRepair;
                state.phase_state = PhaseState::Purchase(PurchaseState::new());

                if next == power::TURN_ORDER[0] {
                    state.turn_number += 1;
                }

                events.push(GameEvent::PhaseChanged {
                    from: old_phase,
                    to: Phase::PurchaseAndRepair,
                });
                events.push(GameEvent::TurnChanged {
                    power: next,
                    turn: state.turn_number,
                });
                let _ = old_power; // suppress unused warning
            }
        }
        // TODO: Apply other action types (Phase 5+)
        _ => {
            // Placeholder: record but don't yet apply detailed logic
        }
    }

    let applied = AppliedAction {
        action: action.clone(),
        inverse: InverseAction::Irreversible, // TODO: proper undo
    };
    state.action_log.push(applied.clone());

    Ok(ActionResult { applied, events })
}

/// Apply an undo operation by popping the last action and reversing it.
fn apply_undo(state: &mut GameState) -> Result<ActionResult, EngineError> {
    let applied = state.action_log.pop().ok_or(EngineError::CannotUndo {
        reason: "No actions to undo".into(),
    })?;

    match applied.inverse {
        InverseAction::Simple(inverse_action) => {
            apply_inverse_simple(state, inverse_action)?;
        }
        InverseAction::RestoreSnapshot(bytes) => {
            apply_inverse_snapshot(state, &bytes)?;
        }
        InverseAction::Irreversible => {
            // Push it back since we can't undo it
            state.action_log.push(applied);
            return Err(EngineError::CannotUndo {
                reason: "Last action cannot be undone".into(),
            });
        }
    }

    Ok(ActionResult {
        applied: AppliedAction {
            action: Action::Undo,
            inverse: InverseAction::Irreversible,
        },
        events: Vec::new(),
    })
}

/// Apply a simple inverse action to reverse a previous action.
/// Populated in Phase 5+ when within-phase actions are implemented.
fn apply_inverse_simple(state: &mut GameState, _action: Action) -> Result<(), EngineError> {
    let _ = state;
    Ok(())
}

/// Restore phase state from a MessagePack snapshot.
fn apply_inverse_snapshot(state: &mut GameState, bytes: &[u8]) -> Result<(), EngineError> {
    let phase_state: PhaseState =
        rmp_serde::from_slice(bytes).map_err(|e| EngineError::Deserialization(e.to_string()))?;
    state.phase_state = phase_state;
    Ok(())
}

/// Create the default PhaseState for a given phase.
fn create_phase_state(phase: Phase) -> PhaseState {
    match phase {
        Phase::PurchaseAndRepair => PhaseState::Purchase(PurchaseState::new()),
        Phase::CombatMovement => PhaseState::CombatMove(CombatMoveState::new()),
        Phase::ConductCombat => PhaseState::Combat(CombatState::new()),
        Phase::NonCombatMovement => PhaseState::NonCombatMove(NonCombatMoveState::new()),
        Phase::Mobilize => PhaseState::Mobilize(MobilizeState::new()),
        Phase::CollectIncome => PhaseState::CollectIncome(CollectIncomeState::new()),
    }
}
