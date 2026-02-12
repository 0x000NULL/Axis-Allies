//! Action application dispatcher.
//!
//! Applies a validated action to the game state and returns the result.
//! Detailed application logic will be implemented in later phases.

use crate::action::{Action, ActionResult, AppliedAction, GameEvent, InverseAction};
use crate::data::GameMap;
use crate::error::EngineError;
use crate::movement;
use crate::phase::{
    CombatMoveState, CombatState, CollectIncomeState, MobilizeState, NonCombatMoveState, Phase,
    PhaseState, PlannedMove, PurchaseState,
};
use crate::power;
use crate::state::GameState;
use crate::territory::RegionId;

/// Apply a validated action to the game state.
pub fn apply_action(state: &mut GameState, action: Action, _map: &GameMap) -> Result<ActionResult, EngineError> {
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

            // For ConfirmCombatMovement, identify pending combats
            if matches!(action, Action::ConfirmCombatMovement) {
                let combats = movement::identify_pending_combats(state, state.current_power);
                // Save undo checkpoint at phase boundary
                state.undo_checkpoints.push(state.action_log.len());
                state.current_phase = Phase::ConductCombat;
                let mut combat_state = CombatState::new();
                combat_state.pending_battles = combats;
                state.phase_state = PhaseState::Combat(combat_state);
                events.push(GameEvent::PhaseChanged {
                    from: old_phase,
                    to: Phase::ConductCombat,
                });

                let applied = AppliedAction {
                    action: action.clone(),
                    inverse: InverseAction::Irreversible,
                };
                state.action_log.push(applied.clone());
                return Ok(ActionResult { applied, events });
            }

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

        Action::MoveUnit { unit_id, ref path } => {
            return apply_move_unit(state, *unit_id, path.clone());
        }

        Action::UndoMove { unit_id } => {
            return apply_undo_move(state, *unit_id);
        }

        Action::MoveUnitNonCombat { unit_id, ref path } => {
            return apply_move_unit_noncombat(state, *unit_id, path.clone());
        }

        Action::LandAirUnit { unit_id, territory_id } => {
            return apply_land_air_unit(state, *unit_id, *territory_id);
        }

        Action::SelectBattle { location } => {
            return apply_select_battle_action(state, *location);
        }

        Action::RollAttack => {
            return apply_roll_attack_action(state);
        }

        Action::RollDefense => {
            return apply_roll_defense_action(state);
        }

        Action::SelectCasualties { casualties } => {
            return apply_select_casualties_action(state, casualties.clone());
        }

        Action::AttackerRetreat { to } => {
            return apply_attacker_retreat_action(state, *to);
        }

        Action::SubmergeSubmarine { unit_id } => {
            return apply_submerge_action(state, *unit_id);
        }

        Action::ContinueCombatRound => {
            return apply_continue_combat_action(state);
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

/// Apply a MoveUnit action during Combat Movement.
fn apply_move_unit(
    state: &mut GameState,
    unit_id: u32,
    path: Vec<RegionId>,
) -> Result<ActionResult, EngineError> {
    let from = path[0];
    let to = *path.last().unwrap();

    // Remove unit from current location
    let (_region, mut unit) = movement::remove_unit(state, unit_id)
        .ok_or(EngineError::UnitNotFound { unit_id })?;

    // Mark as moved
    unit.moved_this_turn = true;
    let movement_used = (path.len() as u8).saturating_sub(1);
    unit.movement_remaining = unit.movement_remaining.saturating_sub(movement_used);

    // Place at destination
    movement::place_unit_at(state, to, unit);

    // Record in phase state
    if let PhaseState::CombatMove(ref mut cms) = state.phase_state {
        cms.moves.push(PlannedMove {
            unit_id,
            path: path.clone(),
            from,
            to,
        });
    }

    let applied = AppliedAction {
        action: Action::MoveUnit { unit_id, path },
        inverse: InverseAction::Simple(Action::UndoMove { unit_id }),
    };
    state.action_log.push(applied.clone());

    Ok(ActionResult {
        applied,
        events: Vec::new(),
    })
}

/// Apply an UndoMove action: return unit to its original position.
fn apply_undo_move(
    state: &mut GameState,
    unit_id: u32,
) -> Result<ActionResult, EngineError> {
    // Find the planned move
    let planned = if let PhaseState::CombatMove(ref cms) = state.phase_state {
        cms.moves.iter().find(|m| m.unit_id == unit_id).cloned()
    } else {
        None
    };

    let planned = planned.ok_or(EngineError::InvalidAction {
        reason: "No move found for this unit".into(),
    })?;

    // Remove unit from destination
    let (_region, mut unit) = movement::remove_unit(state, unit_id)
        .ok_or(EngineError::UnitNotFound { unit_id })?;

    // Restore movement
    unit.moved_this_turn = false;
    let stats = crate::unit::get_unit_stats(unit.unit_type);
    unit.movement_remaining = stats.movement;

    // Place back at origin
    movement::place_unit_at(state, planned.from, unit);

    // Remove from phase state
    if let PhaseState::CombatMove(ref mut cms) = state.phase_state {
        cms.moves.retain(|m| m.unit_id != unit_id);
    }

    let inverse_path = planned.path.clone();
    let applied = AppliedAction {
        action: Action::UndoMove { unit_id },
        inverse: InverseAction::Simple(Action::MoveUnit {
            unit_id,
            path: inverse_path,
        }),
    };
    state.action_log.push(applied.clone());

    Ok(ActionResult {
        applied,
        events: Vec::new(),
    })
}

/// Apply a MoveUnitNonCombat action.
fn apply_move_unit_noncombat(
    state: &mut GameState,
    unit_id: u32,
    path: Vec<RegionId>,
) -> Result<ActionResult, EngineError> {
    let from = path[0];
    let to = *path.last().unwrap();

    let (_region, mut unit) = movement::remove_unit(state, unit_id)
        .ok_or(EngineError::UnitNotFound { unit_id })?;

    unit.moved_this_turn = true;
    let movement_used = (path.len() as u8).saturating_sub(1);
    unit.movement_remaining = unit.movement_remaining.saturating_sub(movement_used);

    movement::place_unit_at(state, to, unit);

    // Record in phase state
    if let PhaseState::NonCombatMove(ref mut ncms) = state.phase_state {
        ncms.moves.push(PlannedMove {
            unit_id,
            path: path.clone(),
            from,
            to,
        });
    }

    // Snapshot for undo
    let snapshot = rmp_serde::to_vec(&state.phase_state)
        .map_err(|e| EngineError::Serialization(e.to_string()))?;

    let applied = AppliedAction {
        action: Action::MoveUnitNonCombat { unit_id, path },
        inverse: InverseAction::RestoreSnapshot(snapshot),
    };
    state.action_log.push(applied.clone());

    Ok(ActionResult {
        applied,
        events: Vec::new(),
    })
}

/// Apply a LandAirUnit action.
fn apply_land_air_unit(
    state: &mut GameState,
    unit_id: u32,
    destination: RegionId,
) -> Result<ActionResult, EngineError> {
    let (_region, unit) = movement::remove_unit(state, unit_id)
        .ok_or(EngineError::UnitNotFound { unit_id })?;

    movement::place_unit_at(state, destination, unit);

    let snapshot = rmp_serde::to_vec(&state.phase_state)
        .map_err(|e| EngineError::Serialization(e.to_string()))?;

    let applied = AppliedAction {
        action: Action::LandAirUnit {
            unit_id,
            territory_id: destination,
        },
        inverse: InverseAction::RestoreSnapshot(snapshot),
    };
    state.action_log.push(applied.clone());

    Ok(ActionResult {
        applied,
        events: Vec::new(),
    })
}

// =========================================================================
// Combat phase action handlers
// =========================================================================

use crate::combat;

fn apply_select_battle_action(
    state: &mut GameState,
    location: RegionId,
) -> Result<ActionResult, EngineError> {
    let (active_combat, events) = combat::apply_select_battle(state, location)?;

    if let PhaseState::Combat(ref mut cs) = state.phase_state {
        cs.active_combat = Some(active_combat);
    }

    let applied = AppliedAction {
        action: Action::SelectBattle { location },
        inverse: InverseAction::Irreversible,
    };
    state.action_log.push(applied.clone());
    Ok(ActionResult { applied, events })
}

fn apply_roll_attack_action(
    state: &mut GameState,
) -> Result<ActionResult, EngineError> {
    let mut active_combat = extract_active_combat(state)?;
    let events = combat::apply_roll_attack(state, &mut active_combat)?;
    store_active_combat(state, active_combat);

    let applied = AppliedAction {
        action: Action::RollAttack,
        inverse: InverseAction::Irreversible,
    };
    state.action_log.push(applied.clone());
    Ok(ActionResult { applied, events })
}

fn apply_roll_defense_action(
    state: &mut GameState,
) -> Result<ActionResult, EngineError> {
    let mut active_combat = extract_active_combat(state)?;
    let events = combat::apply_roll_defense(state, &mut active_combat)?;
    store_active_combat(state, active_combat);

    let applied = AppliedAction {
        action: Action::RollDefense,
        inverse: InverseAction::Irreversible,
    };
    state.action_log.push(applied.clone());
    Ok(ActionResult { applied, events })
}

fn apply_select_casualties_action(
    state: &mut GameState,
    casualties: Vec<u32>,
) -> Result<ActionResult, EngineError> {
    let mut active_combat = extract_active_combat(state)?;

    let defender_side = matches!(
        active_combat.sub_phase,
        combat::CombatSubPhase::AAFireCasualties
        | combat::CombatSubPhase::DefenderSelectsCasualties
        | combat::CombatSubPhase::ShoreBombardmentCasualties
        | combat::CombatSubPhase::DefenderSubmarineStrikeCasualties
    );

    let events = combat::apply_casualties(state, &mut active_combat, &casualties, defender_side)?;

    // Check if battle ended after casualties
    if active_combat.sub_phase == combat::CombatSubPhase::BattleOver {
        let battle_events = combat::finalize_battle(state, &active_combat);
        let location = active_combat.location;
        if let PhaseState::Combat(ref mut cs) = state.phase_state {
            cs.resolved_battles.push(location);
            cs.current_battle = None;
            cs.active_combat = None;
        }
        let applied = AppliedAction {
            action: Action::SelectCasualties { casualties },
            inverse: InverseAction::Irreversible,
        };
        state.action_log.push(applied.clone());
        let mut all_events = events;
        all_events.extend(battle_events);
        return Ok(ActionResult { applied, events: all_events });
    }

    store_active_combat(state, active_combat);

    let applied = AppliedAction {
        action: Action::SelectCasualties { casualties },
        inverse: InverseAction::Irreversible,
    };
    state.action_log.push(applied.clone());
    Ok(ActionResult { applied, events })
}

fn apply_attacker_retreat_action(
    state: &mut GameState,
    retreat_to: RegionId,
) -> Result<ActionResult, EngineError> {
    let mut active_combat = extract_active_combat(state)?;
    let mut events = combat::apply_retreat(state, &mut active_combat, retreat_to)?;
    let battle_events = combat::finalize_battle(state, &active_combat);
    events.extend(battle_events);

    let location = active_combat.location;
    if let PhaseState::Combat(ref mut cs) = state.phase_state {
        cs.resolved_battles.push(location);
        cs.current_battle = None;
        cs.active_combat = None;
    }

    let applied = AppliedAction {
        action: Action::AttackerRetreat { to: retreat_to },
        inverse: InverseAction::Irreversible,
    };
    state.action_log.push(applied.clone());
    Ok(ActionResult { applied, events })
}

fn apply_submerge_action(
    state: &mut GameState,
    unit_id: u32,
) -> Result<ActionResult, EngineError> {
    let mut active_combat = extract_active_combat(state)?;
    combat::apply_submerge(state, &mut active_combat, unit_id)?;

    if active_combat.sub_phase == combat::CombatSubPhase::BattleOver {
        let events = combat::finalize_battle(state, &active_combat);
        let location = active_combat.location;
        if let PhaseState::Combat(ref mut cs) = state.phase_state {
            cs.resolved_battles.push(location);
            cs.current_battle = None;
            cs.active_combat = None;
        }
        let applied = AppliedAction {
            action: Action::SubmergeSubmarine { unit_id },
            inverse: InverseAction::Irreversible,
        };
        state.action_log.push(applied.clone());
        return Ok(ActionResult { applied, events });
    }

    store_active_combat(state, active_combat);

    let applied = AppliedAction {
        action: Action::SubmergeSubmarine { unit_id },
        inverse: InverseAction::Irreversible,
    };
    state.action_log.push(applied.clone());
    Ok(ActionResult { applied, events: Vec::new() })
}

fn apply_continue_combat_action(
    state: &mut GameState,
) -> Result<ActionResult, EngineError> {
    let mut active_combat = extract_active_combat(state)?;
    combat::continue_combat_round(state, &mut active_combat);
    store_active_combat(state, active_combat);

    let applied = AppliedAction {
        action: Action::ContinueCombatRound,
        inverse: InverseAction::Irreversible,
    };
    state.action_log.push(applied.clone());
    Ok(ActionResult { applied, events: Vec::new() })
}

/// Extract active combat from phase state.
fn extract_active_combat(state: &mut GameState) -> Result<combat::ActiveCombat, EngineError> {
    if let PhaseState::Combat(ref mut cs) = state.phase_state {
        cs.active_combat.take().ok_or(EngineError::InvalidAction {
            reason: "No active battle".into(),
        })
    } else {
        Err(EngineError::WrongPhase {
            expected: "ConductCombat".into(),
            actual: format!("{:?}", state.current_phase),
        })
    }
}

/// Store active combat back into phase state.
fn store_active_combat(state: &mut GameState, combat: combat::ActiveCombat) {
    if let PhaseState::Combat(ref mut cs) = state.phase_state {
        cs.active_combat = Some(combat);
    }
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
