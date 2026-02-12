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

            // For ConfirmPurchases, capture purchases for later mobilization
            let _purchased_units = if matches!(action, Action::ConfirmPurchases) {
                if let PhaseState::Purchase(ref ps) = state.phase_state {
                    ps.purchases.clone()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

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
        Action::PurchaseUnit { unit_type, count } => {
            let stats = crate::unit::get_unit_stats(*unit_type);
            let cost = stats.cost * count;

            if let PhaseState::Purchase(ref mut ps) = state.phase_state {
                // Add to or update existing purchase entry
                if let Some(entry) = ps.purchases.iter_mut().find(|(ut, _)| *ut == *unit_type) {
                    entry.1 += count;
                } else {
                    ps.purchases.push((*unit_type, *count));
                }
                ps.ipcs_spent += cost;
            }

            // Deduct IPCs from the power
            let power_idx = state.current_power as usize;
            state.powers[power_idx].ipcs -= cost;

            events.push(GameEvent::UnitsPurchased {
                unit_type: *unit_type,
                count: *count,
                cost,
            });

            let applied = AppliedAction {
                action: action.clone(),
                inverse: InverseAction::Simple(Action::RemovePurchase {
                    unit_type: *unit_type,
                    count: *count,
                }),
            };
            state.action_log.push(applied.clone());
            return Ok(ActionResult { applied, events });
        }

        Action::RemovePurchase { unit_type, count } => {
            let stats = crate::unit::get_unit_stats(*unit_type);
            let refund = stats.cost * count;

            if let PhaseState::Purchase(ref mut ps) = state.phase_state {
                if let Some(entry) = ps.purchases.iter_mut().find(|(ut, _)| *ut == *unit_type) {
                    entry.1 = entry.1.saturating_sub(*count);
                    if entry.1 == 0 {
                        ps.purchases.retain(|(ut, _)| *ut != *unit_type);
                    }
                }
                ps.ipcs_spent = ps.ipcs_spent.saturating_sub(refund);
            }

            // Refund IPCs
            let power_idx = state.current_power as usize;
            state.powers[power_idx].ipcs += refund;

            let applied = AppliedAction {
                action: action.clone(),
                inverse: InverseAction::Simple(Action::PurchaseUnit {
                    unit_type: *unit_type,
                    count: *count,
                }),
            };
            state.action_log.push(applied.clone());
            return Ok(ActionResult { applied, events });
        }

        Action::RepairFacility {
            territory_id,
            damage_to_repair,
        } => {
            let cost = *damage_to_repair; // 1 IPC per damage point

            // Capture old facility damage for undo
            let old_damage = state
                .territories
                .get(*territory_id as usize)
                .and_then(|t| t.facilities.iter().find(|f| f.damage > 0))
                .map(|f| f.damage)
                .unwrap_or(0);

            // Record in phase state
            if let PhaseState::Purchase(ref mut ps) = state.phase_state {
                ps.repairs.push((*territory_id, *damage_to_repair));
                ps.ipcs_spent += cost;
            }

            // Apply damage reduction to the facility
            if let Some(territory) = state.territories.get_mut(*territory_id as usize) {
                if let Some(facility) = territory.facilities.iter_mut().find(|f| f.damage > 0 || f.damage == old_damage.saturating_sub(*damage_to_repair)) {
                    facility.damage = old_damage.saturating_sub(*damage_to_repair);
                }
            }

            // Deduct IPCs
            let power_idx = state.current_power as usize;
            state.powers[power_idx].ipcs -= cost;

            let applied = AppliedAction {
                action: action.clone(),
                inverse: InverseAction::Irreversible, // Repair undo is complex; mark irreversible for now
            };
            state.action_log.push(applied.clone());
            return Ok(ActionResult { applied, events });
        }

        // Other actions not yet implemented
        _ => {}
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
fn apply_inverse_simple(state: &mut GameState, action: Action) -> Result<(), EngineError> {
    match &action {
        Action::RemovePurchase { unit_type, count } => {
            // This is the inverse of PurchaseUnit — remove from queue, refund IPCs
            let stats = crate::unit::get_unit_stats(*unit_type);
            let refund = stats.cost * count;

            if let PhaseState::Purchase(ref mut ps) = state.phase_state {
                if let Some(entry) = ps.purchases.iter_mut().find(|(ut, _)| *ut == *unit_type) {
                    entry.1 = entry.1.saturating_sub(*count);
                    if entry.1 == 0 {
                        ps.purchases.retain(|(ut, _)| *ut != *unit_type);
                    }
                }
                ps.ipcs_spent = ps.ipcs_spent.saturating_sub(refund);
            }

            let power_idx = state.current_power as usize;
            state.powers[power_idx].ipcs += refund;
        }
        Action::PurchaseUnit { unit_type, count } => {
            // This is the inverse of RemovePurchase — re-add to queue, deduct IPCs
            let stats = crate::unit::get_unit_stats(*unit_type);
            let cost = stats.cost * count;

            if let PhaseState::Purchase(ref mut ps) = state.phase_state {
                if let Some(entry) = ps.purchases.iter_mut().find(|(ut, _)| *ut == *unit_type) {
                    entry.1 += count;
                } else {
                    ps.purchases.push((*unit_type, *count));
                }
                ps.ipcs_spent += cost;
            }

            let power_idx = state.current_power as usize;
            state.powers[power_idx].ipcs -= cost;
        }
        _ => {
            // Other inverse actions not yet implemented
        }
    }
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
