//! Action validation dispatcher.
//!
//! Validates that an action is legal given the current game state.
//! Detailed validation for each phase will be implemented in later phases.

use crate::action::{Action, InverseAction};
use crate::error::EngineError;
use crate::phase::Phase;
use crate::state::GameState;

/// Validate that an action is legal in the current game state.
pub fn validate_action(state: &GameState, action: &Action) -> Result<(), EngineError> {
    // Basic phase validation
    match action {
        Action::PurchaseUnit { .. }
        | Action::RemovePurchase { .. }
        | Action::RepairFacility { .. }
        | Action::ConfirmPurchases => {
            if state.current_phase != Phase::PurchaseAndRepair {
                return Err(EngineError::WrongPhase {
                    expected: "PurchaseAndRepair".into(),
                    actual: format!("{:?}", state.current_phase),
                });
            }
        }
        Action::MoveUnit { .. } | Action::UndoMove { .. } | Action::ConfirmCombatMovement => {
            if state.current_phase != Phase::CombatMovement {
                return Err(EngineError::WrongPhase {
                    expected: "CombatMovement".into(),
                    actual: format!("{:?}", state.current_phase),
                });
            }
        }
        Action::SelectBattle { .. }
        | Action::RollAttack
        | Action::RollDefense
        | Action::SelectCasualties { .. }
        | Action::AttackerRetreat { .. }
        | Action::SubmergeSubmarine { .. }
        | Action::ContinueCombatRound => {
            if state.current_phase != Phase::ConductCombat {
                return Err(EngineError::WrongPhase {
                    expected: "ConductCombat".into(),
                    actual: format!("{:?}", state.current_phase),
                });
            }
        }
        Action::MoveUnitNonCombat { .. }
        | Action::LandAirUnit { .. }
        | Action::ConfirmNonCombatMovement => {
            if state.current_phase != Phase::NonCombatMovement {
                return Err(EngineError::WrongPhase {
                    expected: "NonCombatMovement".into(),
                    actual: format!("{:?}", state.current_phase),
                });
            }
        }
        Action::PlaceUnit { .. } | Action::ConfirmMobilization => {
            if state.current_phase != Phase::Mobilize {
                return Err(EngineError::WrongPhase {
                    expected: "Mobilize".into(),
                    actual: format!("{:?}", state.current_phase),
                });
            }
        }
        Action::ConfirmIncome => {
            if state.current_phase != Phase::CollectIncome {
                return Err(EngineError::WrongPhase {
                    expected: "CollectIncome".into(),
                    actual: format!("{:?}", state.current_phase),
                });
            }
        }
        Action::Undo => {
            if state.action_log.is_empty() {
                return Err(EngineError::CannotUndo {
                    reason: "No actions to undo".into(),
                });
            }
            if matches!(
                state.action_log.last().unwrap().inverse,
                InverseAction::Irreversible
            ) {
                return Err(EngineError::CannotUndo {
                    reason: "Last action cannot be undone".into(),
                });
            }
        }
        // These can be used in any phase
        Action::DeclareWar { .. } | Action::ConfirmPhase => {}
    }

    // TODO: Detailed per-action validation (Phase 5+)
    Ok(())
}
