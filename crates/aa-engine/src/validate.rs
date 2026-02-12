//! Action validation dispatcher.
//!
//! Validates that an action is legal given the current game state.
//! Detailed validation for each phase will be implemented in later phases.

use crate::action::{Action, InverseAction};
use crate::data::GameMap;
use crate::error::EngineError;
use crate::movement;
use crate::phase::{Phase, PhaseState};
use crate::power::Power;
use crate::state::GameState;
use crate::territory::{RegionId, TerritoryId};
use crate::unit::{get_unit_stats, UnitDomain, UnitType};

/// Validate that an action is legal in the current game state.
pub fn validate_action(state: &GameState, action: &Action) -> Result<(), EngineError> {
    validate_action_with_map(state, action, None)
}

/// Validate that an action is legal, with optional map for movement validation.
pub fn validate_action_with_map(state: &GameState, action: &Action, map: Option<&GameMap>) -> Result<(), EngineError> {
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

    // Detailed per-action validation
    match action {
        Action::PurchaseUnit { unit_type, count } => {
            validate_purchase_unit(state, *unit_type, *count)?;
        }
        Action::RemovePurchase { unit_type, count } => {
            validate_remove_purchase(state, *unit_type, *count)?;
        }
        Action::RepairFacility {
            territory_id,
            damage_to_repair,
        } => {
            validate_repair_facility(state, *territory_id, *damage_to_repair)?;
        }
        Action::ConfirmPurchases => {
            // Always valid if in correct phase (already checked above)
        }
        Action::MoveUnit { unit_id, path } => {
            validate_move_unit(state, map, *unit_id, path)?;
        }
        Action::UndoMove { unit_id } => {
            validate_undo_move(state, *unit_id)?;
        }
        Action::ConfirmCombatMovement => {
            validate_confirm_combat_movement(state, map)?;
        }
        Action::MoveUnitNonCombat { unit_id, path } => {
            validate_move_unit_noncombat(state, map, *unit_id, path)?;
        }
        Action::LandAirUnit { unit_id, territory_id } => {
            validate_land_air_unit(state, map, *unit_id, *territory_id)?;
        }
        Action::ConfirmNonCombatMovement => {
            validate_confirm_noncombat_movement(state)?;
        }
        _ => {}
    }

    Ok(())
}

/// Get the current power's available IPCs (total minus already spent in this phase).
fn available_ipcs(state: &GameState) -> u32 {
    let power_idx = state.current_power as usize;
    let total = state.powers[power_idx].ipcs;
    if let PhaseState::Purchase(ref ps) = state.phase_state {
        total.saturating_sub(ps.ipcs_spent)
    } else {
        total
    }
}

/// Validate a PurchaseUnit action.
fn validate_purchase_unit(
    state: &GameState,
    unit_type: UnitType,
    count: u32,
) -> Result<(), EngineError> {
    if count == 0 {
        return Err(EngineError::InvalidAction {
            reason: "Cannot purchase 0 units".into(),
        });
    }

    // China can only buy Infantry
    if state.current_power == Power::China && unit_type != UnitType::Infantry {
        return Err(EngineError::InvalidAction {
            reason: "China can only purchase Infantry".into(),
        });
    }

    let stats = get_unit_stats(unit_type);
    let total_cost = stats.cost * count;
    let available = available_ipcs(state);

    if total_cost > available {
        return Err(EngineError::InsufficientIPCs {
            needed: total_cost,
            available,
        });
    }

    Ok(())
}

/// Validate a RemovePurchase action.
fn validate_remove_purchase(
    state: &GameState,
    unit_type: UnitType,
    count: u32,
) -> Result<(), EngineError> {
    if count == 0 {
        return Err(EngineError::InvalidAction {
            reason: "Cannot remove 0 units from purchase".into(),
        });
    }

    let ps = match &state.phase_state {
        PhaseState::Purchase(ps) => ps,
        _ => {
            return Err(EngineError::WrongPhase {
                expected: "PurchaseAndRepair".into(),
                actual: format!("{:?}", state.current_phase),
            });
        }
    };

    // Check that enough of this unit type have been purchased
    let purchased = ps
        .purchases
        .iter()
        .find(|(ut, _)| *ut == unit_type)
        .map(|(_, c)| *c)
        .unwrap_or(0);

    if count > purchased {
        return Err(EngineError::InvalidAction {
            reason: format!(
                "Cannot remove {} {:?} — only {} purchased",
                count, unit_type, purchased
            ),
        });
    }

    Ok(())
}

/// Validate a RepairFacility action.
fn validate_repair_facility(
    state: &GameState,
    territory_id: TerritoryId,
    damage_to_repair: u32,
) -> Result<(), EngineError> {
    if damage_to_repair == 0 {
        return Err(EngineError::InvalidAction {
            reason: "Cannot repair 0 damage".into(),
        });
    }

    // Check territory exists
    let territory = state
        .territories
        .get(territory_id as usize)
        .ok_or(EngineError::TerritoryNotFound { territory_id })?;

    // Check territory is owned by current power
    if territory.owner != Some(state.current_power) {
        return Err(EngineError::InvalidAction {
            reason: "Cannot repair facility in territory you don't own".into(),
        });
    }

    // Find a damaged facility (industrial complex, air base, or naval base)
    let damaged_facility = territory
        .facilities
        .iter()
        .find(|f| f.damage > 0);

    let facility = damaged_facility.ok_or(EngineError::InvalidAction {
        reason: "No damaged facility in this territory".into(),
    })?;

    // Check repair doesn't exceed current damage
    // Also account for repairs already queued this phase
    let already_repaired = if let PhaseState::Purchase(ref ps) = state.phase_state {
        ps.repairs
            .iter()
            .filter(|(tid, _)| *tid == territory_id)
            .map(|(_, r)| *r)
            .sum::<u32>()
    } else {
        0
    };

    let remaining_damage = facility.damage.saturating_sub(already_repaired);
    if damage_to_repair > remaining_damage {
        return Err(EngineError::InvalidAction {
            reason: format!(
                "Cannot repair {} damage — only {} damage remaining",
                damage_to_repair, remaining_damage
            ),
        });
    }

    // Each point of repair costs 1 IPC
    let available = available_ipcs(state);
    if damage_to_repair > available {
        return Err(EngineError::InsufficientIPCs {
            needed: damage_to_repair,
            available,
        });
    }

    Ok(())
}

// =========================================================================
// Movement validation
// =========================================================================

/// Validate a MoveUnit action during Combat Movement.
fn validate_move_unit(
    state: &GameState,
    map: Option<&GameMap>,
    unit_id: u32,
    path: &[RegionId],
) -> Result<(), EngineError> {
    let map = map.ok_or(EngineError::Internal("Map required for movement validation".into()))?;

    let (current_region, unit) = movement::find_unit(state, unit_id)
        .ok_or(EngineError::UnitNotFound { unit_id })?;

    // Unit must belong to current power
    if unit.owner != state.current_power {
        return Err(EngineError::InvalidAction {
            reason: "Unit does not belong to current power".into(),
        });
    }

    // Unit must not have already moved
    if unit.moved_this_turn {
        return Err(EngineError::InvalidAction {
            reason: "Unit has already moved this turn".into(),
        });
    }

    // Path must start at unit's current location
    if path.is_empty() || path[0] != current_region {
        return Err(EngineError::IllegalMove {
            reason: "Path must start at unit's current location".into(),
        });
    }

    // Validate the path
    movement::validate_combat_move(state, map, state.current_power, unit, path)?;

    Ok(())
}

/// Validate an UndoMove action.
fn validate_undo_move(state: &GameState, unit_id: u32) -> Result<(), EngineError> {
    // Check that this unit has a recorded move in the phase state
    let cms = match &state.phase_state {
        PhaseState::CombatMove(cms) => cms,
        _ => {
            return Err(EngineError::WrongPhase {
                expected: "CombatMovement".into(),
                actual: format!("{:?}", state.current_phase),
            });
        }
    };

    if !cms.moves.iter().any(|m| m.unit_id == unit_id) {
        return Err(EngineError::InvalidAction {
            reason: "Unit has no recorded move to undo".into(),
        });
    }

    Ok(())
}

/// Validate ConfirmCombatMovement: check that all air units have potential landing spots.
fn validate_confirm_combat_movement(
    state: &GameState,
    map: Option<&GameMap>,
) -> Result<(), EngineError> {
    let map = map.ok_or(EngineError::Internal("Map required for movement validation".into()))?;
    let power = state.current_power;

    let cms = match &state.phase_state {
        PhaseState::CombatMove(cms) => cms,
        _ => return Ok(()),
    };

    // For each moved air unit, check it has a potential landing spot
    for planned in &cms.moves {
        // Find the unit at its destination
        if let Some((_region, unit)) = movement::find_unit(state, planned.unit_id) {
            let stats = get_unit_stats(unit.unit_type);
            if stats.domain == UnitDomain::Air {
                let movement_used = (planned.path.len() as u8).saturating_sub(1);
                if !movement::air_unit_has_potential_landing(
                    state,
                    map,
                    power,
                    unit,
                    planned.to,
                    movement_used,
                ) {
                    return Err(EngineError::IllegalMove {
                        reason: format!(
                            "Air unit {} has no potential landing spot",
                            planned.unit_id
                        ),
                    });
                }
            }
        }
    }

    Ok(())
}

/// Validate a MoveUnitNonCombat action.
fn validate_move_unit_noncombat(
    state: &GameState,
    map: Option<&GameMap>,
    unit_id: u32,
    path: &[RegionId],
) -> Result<(), EngineError> {
    let map = map.ok_or(EngineError::Internal("Map required for movement validation".into()))?;

    let (current_region, unit) = movement::find_unit(state, unit_id)
        .ok_or(EngineError::UnitNotFound { unit_id })?;

    // Unit must belong to current power
    if unit.owner != state.current_power {
        return Err(EngineError::InvalidAction {
            reason: "Unit does not belong to current power".into(),
        });
    }

    // Unit must not have already moved this turn (in combat movement)
    // Exception: units that didn't move in combat can move in non-combat
    if unit.moved_this_turn {
        return Err(EngineError::InvalidAction {
            reason: "Unit has already moved this turn".into(),
        });
    }

    // Path must start at unit's current location
    if path.is_empty() || path[0] != current_region {
        return Err(EngineError::IllegalMove {
            reason: "Path must start at unit's current location".into(),
        });
    }

    movement::validate_noncombat_move(state, map, state.current_power, unit, path)?;

    Ok(())
}

/// Validate a LandAirUnit action.
fn validate_land_air_unit(
    state: &GameState,
    map: Option<&GameMap>,
    unit_id: u32,
    destination: RegionId,
) -> Result<(), EngineError> {
    let map = map.ok_or(EngineError::Internal("Map required for movement validation".into()))?;

    let (_current_region, unit) = movement::find_unit(state, unit_id)
        .ok_or(EngineError::UnitNotFound { unit_id })?;

    if unit.owner != state.current_power {
        return Err(EngineError::InvalidAction {
            reason: "Unit does not belong to current power".into(),
        });
    }

    movement::validate_air_landing(state, map, state.current_power, unit, destination)?;

    Ok(())
}

/// Validate ConfirmNonCombatMovement: all air units that moved must be landed.
fn validate_confirm_noncombat_movement(_state: &GameState) -> Result<(), EngineError> {
    // Check that no air units are "in flight" (moved but not landed)
    // This is simplified - in a full implementation we'd track air units in flight
    // For now, always valid
    Ok(())
}
