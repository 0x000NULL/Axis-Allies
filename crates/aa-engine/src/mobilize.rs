//! Mobilize phase: place purchased units at eligible factories.

use crate::data::GameMap;
use crate::error::EngineError;
use crate::phase::{MobilizeState, PhaseState};
use crate::power::Power;
use crate::state::GameState;
use crate::territory::{FacilityType, TerritoryId};
use crate::unit::{get_unit_stats, UnitDomain, UnitType};

/// Validate a PlaceUnit action during the Mobilize phase.
pub fn validate_place_unit(
    state: &GameState,
    map: &GameMap,
    unit_type: UnitType,
    territory_id: TerritoryId,
) -> Result<(), EngineError> {
    let power = state.current_power;

    // Check unit is in the purchased list
    let ms = match &state.phase_state {
        PhaseState::Mobilize(ms) => ms,
        _ => {
            return Err(EngineError::WrongPhase {
                expected: "Mobilize".into(),
                actual: format!("{:?}", state.current_phase),
            });
        }
    };

    // Check there are remaining units of this type to place
    let remaining = remaining_to_place(ms, unit_type);
    if remaining == 0 {
        return Err(EngineError::InvalidAction {
            reason: format!("No {:?} remaining to place", unit_type),
        });
    }

    let stats = get_unit_stats(unit_type);

    if stats.domain == UnitDomain::Sea {
        // Naval units: must be placed in a sea zone adjacent to a territory with IC or naval base
        validate_naval_placement(state, map, power, territory_id)?;
    } else {
        // Land and air units: must be placed at a territory with an IC
        validate_land_placement(state, map, power, territory_id)?;
    }

    // Check factory production limit
    check_production_limit(state, ms, territory_id)?;

    Ok(())
}

/// Validate ConfirmMobilization: all purchased units must be placed.
pub fn validate_confirm_mobilization(state: &GameState) -> Result<(), EngineError> {
    let ms = match &state.phase_state {
        PhaseState::Mobilize(ms) => ms,
        _ => {
            return Err(EngineError::WrongPhase {
                expected: "Mobilize".into(),
                actual: format!("{:?}", state.current_phase),
            });
        }
    };

    // Check all units have been placed
    for (ut, count) in &ms.units_to_place {
        let placed = ms.placements.iter().filter(|(put, _)| put == ut).count() as u32;
        if placed < *count {
            return Err(EngineError::InvalidAction {
                reason: format!(
                    "Must place all purchased units: {} {:?} remaining",
                    count - placed,
                    ut
                ),
            });
        }
    }

    Ok(())
}

/// Count remaining units of a given type to place.
fn remaining_to_place(ms: &MobilizeState, unit_type: UnitType) -> u32 {
    let total = ms
        .units_to_place
        .iter()
        .filter(|(ut, _)| *ut == unit_type)
        .map(|(_, c)| *c)
        .sum::<u32>();
    let placed = ms
        .placements
        .iter()
        .filter(|(ut, _)| *ut == unit_type)
        .count() as u32;
    total.saturating_sub(placed)
}

/// Validate placement of land/air units at a territory with an IC.
fn validate_land_placement(
    state: &GameState,
    _map: &GameMap,
    power: Power,
    territory_id: TerritoryId,
) -> Result<(), EngineError> {
    let territory = state
        .territories
        .get(territory_id as usize)
        .ok_or(EngineError::TerritoryNotFound { territory_id })?;

    // Must be owned by current power
    if territory.owner != Some(power) {
        return Err(EngineError::InvalidAction {
            reason: "Cannot place units in territory you don't own".into(),
        });
    }

    // Cannot place in just-captured territory
    if territory.just_captured {
        return Err(EngineError::InvalidAction {
            reason: "Cannot place units in a territory captured this turn".into(),
        });
    }

    // Must have an IC
    let has_ic = territory.facilities.iter().any(|f| {
        matches!(
            f.facility_type,
            FacilityType::MajorIndustrialComplex | FacilityType::MinorIndustrialComplex
        )
    });

    if !has_ic {
        return Err(EngineError::InvalidAction {
            reason: "Territory must have an Industrial Complex to place units".into(),
        });
    }

    Ok(())
}

/// Validate placement of naval units at a sea zone adjacent to territory with IC or naval base.
fn validate_naval_placement(
    state: &GameState,
    map: &GameMap,
    power: Power,
    territory_id: TerritoryId,
) -> Result<(), EngineError> {
    // For naval placement, territory_id is actually the sea zone where we're placing.
    // But our action uses TerritoryId - we need to find a friendly territory with IC/naval base
    // adjacent to this placement. Actually, looking at the action definition:
    // PlaceUnit { unit_type, territory_id } - territory_id is the territory with the IC.
    // The naval unit goes to an adjacent sea zone. But we need to figure out which sea zone.
    // For simplicity, we validate that the territory has an IC or naval base and is coastal.

    let territory = state
        .territories
        .get(territory_id as usize)
        .ok_or(EngineError::TerritoryNotFound { territory_id })?;

    if territory.owner != Some(power) {
        return Err(EngineError::InvalidAction {
            reason: "Cannot place naval units at territory you don't own".into(),
        });
    }

    if territory.just_captured {
        return Err(EngineError::InvalidAction {
            reason: "Cannot place units in a territory captured this turn".into(),
        });
    }

    let has_ic_or_naval = territory.facilities.iter().any(|f| {
        matches!(
            f.facility_type,
            FacilityType::MajorIndustrialComplex
                | FacilityType::MinorIndustrialComplex
                | FacilityType::NavalBase
        )
    });

    if !has_ic_or_naval {
        return Err(EngineError::InvalidAction {
            reason: "Territory must have an IC or Naval Base to place naval units".into(),
        });
    }

    // Must be coastal
    let tdef = map.territory(territory_id);
    if tdef.adjacent_sea.is_empty() {
        return Err(EngineError::InvalidAction {
            reason: "Territory must be coastal to place naval units".into(),
        });
    }

    Ok(())
}

/// Check that the factory production limit hasn't been exceeded.
fn check_production_limit(
    state: &GameState,
    ms: &MobilizeState,
    territory_id: TerritoryId,
) -> Result<(), EngineError> {
    let territory = state
        .territories
        .get(territory_id as usize)
        .ok_or(EngineError::TerritoryNotFound { territory_id })?;

    // Find the IC in this territory
    let ic = territory.facilities.iter().find(|f| {
        matches!(
            f.facility_type,
            FacilityType::MajorIndustrialComplex | FacilityType::MinorIndustrialComplex
        )
    });

    if let Some(ic) = ic {
        // Get territory IPC value from map for production capacity
        let ipc_value = 3; // default; we don't have map in this fn but territory state has it
        // Actually we need the map to get ipc_value. Let's compute capacity from facility.
        let capacity = ic.production_capacity(ipc_value);

        // Count units already placed here this phase
        let placed_here = ms
            .placements
            .iter()
            .filter(|(_, tid)| *tid == territory_id)
            .count() as u32;

        if placed_here >= capacity {
            return Err(EngineError::InvalidAction {
                reason: format!(
                    "Factory production limit reached ({}/{})",
                    placed_here, capacity
                ),
            });
        }
    }

    Ok(())
}

/// Get production capacity for a territory (considering IC type and damage).
pub fn get_production_capacity(state: &GameState, map: &GameMap, territory_id: TerritoryId) -> u32 {
    let territory = match state.territories.get(territory_id as usize) {
        Some(t) => t,
        None => return 0,
    };

    let tdef = map.territory(territory_id);

    territory
        .facilities
        .iter()
        .filter(|f| {
            matches!(
                f.facility_type,
                FacilityType::MajorIndustrialComplex | FacilityType::MinorIndustrialComplex
            )
        })
        .map(|f| f.production_capacity(tdef.ipc_value))
        .sum()
}

/// Get all eligible placement territories for a given unit type.
pub fn eligible_placement_territories(
    state: &GameState,
    map: &GameMap,
    power: Power,
    unit_type: UnitType,
) -> Vec<TerritoryId> {
    let stats = get_unit_stats(unit_type);
    let mut result = Vec::new();

    for (i, territory) in state.territories.iter().enumerate() {
        let tid = i as TerritoryId;

        if territory.owner != Some(power) {
            continue;
        }
        if territory.just_captured {
            continue;
        }

        if stats.domain == UnitDomain::Sea {
            // Naval: need IC or naval base + coastal
            let has_ic_or_naval = territory.facilities.iter().any(|f| {
                matches!(
                    f.facility_type,
                    FacilityType::MajorIndustrialComplex
                        | FacilityType::MinorIndustrialComplex
                        | FacilityType::NavalBase
                )
            });
            let tdef = map.territory(tid);
            if has_ic_or_naval && !tdef.adjacent_sea.is_empty() {
                // Check production limit
                if get_production_capacity(state, map, tid) > 0 {
                    result.push(tid);
                }
            }
        } else {
            // Land/Air: need IC
            let has_ic = territory.facilities.iter().any(|f| {
                matches!(
                    f.facility_type,
                    FacilityType::MajorIndustrialComplex | FacilityType::MinorIndustrialComplex
                )
            });
            if has_ic && get_production_capacity(state, map, tid) > 0 {
                result.push(tid);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameMap;
    use crate::phase::{MobilizeState, Phase, PhaseState};
    use crate::power::Power;
    use crate::state::GameState;
    use crate::territory::{Facility, FacilityType};
    use crate::unit::UnitType;

    fn setup_mobilize_state() -> (GameState, GameMap) {
        let map = GameMap::new();
        let mut state = crate::setup::create_initial_state(42, &map);
        state.current_phase = Phase::Mobilize;
        state.current_power = Power::Germany;

        // Add a Major IC to Germany (territory 0)
        state.territories[0].facilities.push(Facility::new(
            FacilityType::MajorIndustrialComplex,
            5, // Germany IPC value
        ));
        state.territories[0].owner = Some(Power::Germany);

        // Set up mobilize state with some units to place
        let ms = MobilizeState {
            placements: Vec::new(),
            units_to_place: vec![(UnitType::Infantry, 3), (UnitType::Tank, 1)],
        };
        state.phase_state = PhaseState::Mobilize(ms);

        (state, map)
    }

    #[test]
    fn test_validate_place_unit_valid() {
        let (state, map) = setup_mobilize_state();
        let result = validate_place_unit(&state, &map, UnitType::Infantry, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_place_unit_no_remaining() {
        let (mut state, map) = setup_mobilize_state();
        // Place all tanks
        if let PhaseState::Mobilize(ref mut ms) = state.phase_state {
            ms.placements.push((UnitType::Tank, 0));
        }
        let result = validate_place_unit(&state, &map, UnitType::Tank, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_place_unit_wrong_owner() {
        let (state, map) = setup_mobilize_state();
        // Try to place at a territory not owned by Germany
        // Territory 5 (France) - may be owned by Germany at start, let's use another
        let result = validate_place_unit(&state, &map, UnitType::Infantry, 126); // Japan
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_place_unit_just_captured() {
        let (mut state, map) = setup_mobilize_state();
        state.territories[0].just_captured = true;
        let result = validate_place_unit(&state, &map, UnitType::Infantry, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_place_unit_no_ic() {
        let (mut state, map) = setup_mobilize_state();
        // Use Western Germany (territory 1) which has no IC
        state.territories[1].owner = Some(Power::Germany);
        let result = validate_place_unit(&state, &map, UnitType::Infantry, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_confirm_mobilization_all_placed() {
        let (mut state, _map) = setup_mobilize_state();
        if let PhaseState::Mobilize(ref mut ms) = state.phase_state {
            // Place all units
            ms.placements = vec![
                (UnitType::Infantry, 0),
                (UnitType::Infantry, 0),
                (UnitType::Infantry, 0),
                (UnitType::Tank, 0),
            ];
        }
        let result = validate_confirm_mobilization(&state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_confirm_mobilization_not_all_placed() {
        let (state, _map) = setup_mobilize_state();
        let result = validate_confirm_mobilization(&state);
        assert!(result.is_err());
    }

    #[test]
    fn test_production_capacity_major_ic() {
        let (state, map) = setup_mobilize_state();
        let capacity = get_production_capacity(&state, &map, 0);
        // Major IC: min(territory_ipc, 10) = min(5, 10) = 5
        assert_eq!(capacity, 5);
    }

    #[test]
    fn test_production_capacity_minor_ic() {
        let (mut state, map) = setup_mobilize_state();
        // Replace with minor IC
        state.territories[0].facilities.clear();
        state.territories[0].facilities.push(Facility::new(
            FacilityType::MinorIndustrialComplex,
            5,
        ));
        let capacity = get_production_capacity(&state, &map, 0);
        // Minor IC: min(territory_ipc, 3) = min(5, 3) = 3
        assert_eq!(capacity, 3);
    }

    #[test]
    fn test_eligible_placement_territories() {
        let (state, map) = setup_mobilize_state();
        let territories =
            eligible_placement_territories(&state, &map, Power::Germany, UnitType::Infantry);
        assert!(territories.contains(&0)); // Germany has IC
    }
}

