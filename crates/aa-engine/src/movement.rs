//! Movement validation helpers for Combat Movement and Non-Combat Movement phases.
//!
//! Covers land, sea, air movement, blitzing, transport loading/unloading,
//! and strait/canal passage checks.

use crate::data::GameMap;
use crate::error::EngineError;
use crate::power::Power;
use crate::state::GameState;
use crate::territory::{RegionId, SeaZoneId, TerritoryId, TerritoryType};
use crate::unit::{get_unit_stats, UnitDomain, UnitId, UnitInstance, UnitType, SpecialAbility};

/// Find a unit by ID across all territories and sea zones. Returns (RegionId, &UnitInstance).
pub fn find_unit<'a>(state: &'a GameState, unit_id: UnitId) -> Option<(RegionId, &'a UnitInstance)> {
    for (i, t) in state.territories.iter().enumerate() {
        if let Some(u) = t.units.iter().find(|u| u.id == unit_id) {
            return Some((RegionId::Land(i as TerritoryId), u));
        }
    }
    for (i, sz) in state.sea_zones.iter().enumerate() {
        if let Some(u) = sz.units.iter().find(|u| u.id == unit_id) {
            return Some((RegionId::Sea(i as SeaZoneId), u));
        }
    }
    None
}

/// Find a unit mutably by ID. Returns (RegionId, &mut UnitInstance).
pub fn find_unit_mut<'a>(state: &'a mut GameState, unit_id: UnitId) -> Option<(RegionId, &'a mut UnitInstance)> {
    for i in 0..state.territories.len() {
        if let Some(pos) = state.territories[i].units.iter().position(|u| u.id == unit_id) {
            let region = RegionId::Land(i as TerritoryId);
            let unit = &mut state.territories[i].units[pos];
            return Some((region, unit));
        }
    }
    for i in 0..state.sea_zones.len() {
        if let Some(pos) = state.sea_zones[i].units.iter().position(|u| u.id == unit_id) {
            let region = RegionId::Sea(i as SeaZoneId);
            let unit = &mut state.sea_zones[i].units[pos];
            return Some((region, unit));
        }
    }
    None
}

/// Remove a unit from its current location and return it.
pub fn remove_unit(state: &mut GameState, unit_id: UnitId) -> Option<(RegionId, UnitInstance)> {
    for i in 0..state.territories.len() {
        if let Some(pos) = state.territories[i].units.iter().position(|u| u.id == unit_id) {
            let unit = state.territories[i].units.remove(pos);
            return Some((RegionId::Land(i as TerritoryId), unit));
        }
    }
    for i in 0..state.sea_zones.len() {
        if let Some(pos) = state.sea_zones[i].units.iter().position(|u| u.id == unit_id) {
            let unit = state.sea_zones[i].units.remove(pos);
            return Some((RegionId::Sea(i as SeaZoneId), unit));
        }
    }
    None
}

/// Place a unit at a region.
pub fn place_unit_at(state: &mut GameState, region: RegionId, unit: UnitInstance) {
    match region {
        RegionId::Land(tid) => {
            state.territories[tid as usize].units.push(unit);
        }
        RegionId::Sea(sid) => {
            state.sea_zones[sid as usize].units.push(unit);
        }
    }
}

/// Check if a territory is friendly to the given power.
pub fn is_friendly_territory(state: &GameState, tid: TerritoryId, power: Power) -> bool {
    match state.territories[tid as usize].owner {
        Some(owner) => state.political.are_friendly(power, owner),
        None => false, // Unowned/neutral is not friendly
    }
}

/// Check if a territory is enemy to the given power.
pub fn is_enemy_territory(state: &GameState, tid: TerritoryId, power: Power) -> bool {
    match state.territories[tid as usize].owner {
        Some(owner) => state.political.are_at_war(power, owner),
        None => false,
    }
}

/// Check if a territory contains enemy units for the given power.
pub fn has_enemy_units(state: &GameState, tid: TerritoryId, power: Power) -> bool {
    state.territories[tid as usize].units.iter().any(|u| state.political.are_at_war(power, u.owner))
}

/// Check if a sea zone contains enemy units (warships) for the given power.
pub fn has_enemy_warships(state: &GameState, sid: SeaZoneId, power: Power) -> bool {
    state.sea_zones[sid as usize].units.iter().any(|u| {
        state.political.are_at_war(power, u.owner) && u.unit_type != UnitType::Transport
    })
}

/// Check if a territory is unoccupied enemy territory (for blitzing).
pub fn is_unoccupied_enemy(state: &GameState, tid: TerritoryId, power: Power) -> bool {
    is_enemy_territory(state, tid, power) && !has_enemy_units(state, tid, power)
}

/// Check if a strait/canal is passable by the given power.
pub fn is_strait_passable(state: &GameState, map: &GameMap, strait_id: u8, power: Power) -> bool {
    map.strait_is_passable(strait_id, |tid| is_friendly_territory(state, tid, power))
}

/// Validate a path for land movement during combat movement.
/// Returns the number of movement points consumed or an error.
pub fn validate_land_combat_path(
    state: &GameState,
    map: &GameMap,
    power: Power,
    unit: &UnitInstance,
    path: &[RegionId],
) -> Result<u8, EngineError> {
    if path.len() < 2 {
        return Err(EngineError::IllegalMove {
            reason: "Path must have at least 2 regions (from and to)".into(),
        });
    }

    let stats = get_unit_stats(unit.unit_type);
    let max_move = stats.movement;
    let can_blitz = stats.special_abilities.contains(&SpecialAbility::Blitz);

    let mut movement_used: u8 = 0;

    for i in 0..path.len() - 1 {
        let from = path[i];
        let to = path[i + 1];

        // Land units must move through land
        let (from_tid, to_tid) = match (from, to) {
            (RegionId::Land(f), RegionId::Land(t)) => (f, t),
            _ => {
                return Err(EngineError::IllegalMove {
                    reason: "Land units can only move through land territories".into(),
                });
            }
        };

        // Check adjacency
        if !map.is_land_adjacent(from_tid, to_tid) {
            return Err(EngineError::IllegalMove {
                reason: format!("Territories are not adjacent"),
            });
        }

        // Check territory type
        let to_def = map.territory(to_tid);
        if to_def.territory_type == TerritoryType::Impassable {
            return Err(EngineError::IllegalMove {
                reason: "Cannot move into impassable territory".into(),
            });
        }

        movement_used += 1;

        // Check intermediate territories for blitzing
        if i < path.len() - 2 {
            // Intermediate territory: must be friendly or unoccupied enemy (blitz)
            if is_enemy_territory(state, to_tid, power) {
                if can_blitz && !has_enemy_units(state, to_tid, power) {
                    // OK - blitzing through
                } else {
                    return Err(EngineError::IllegalMove {
                        reason: "Cannot pass through enemy territory without blitzing".into(),
                    });
                }
            } else if !is_friendly_territory(state, to_tid, power) {
                // Neutral territory - cannot pass through
                return Err(EngineError::IllegalMove {
                    reason: "Cannot pass through neutral territory".into(),
                });
            }
        }
    }

    if movement_used > max_move {
        return Err(EngineError::IllegalMove {
            reason: format!(
                "Insufficient movement: need {}, have {}",
                movement_used, max_move
            ),
        });
    }

    Ok(movement_used)
}

/// Validate a path for sea movement during combat movement.
pub fn validate_sea_combat_path(
    state: &GameState,
    map: &GameMap,
    power: Power,
    unit: &UnitInstance,
    path: &[RegionId],
) -> Result<u8, EngineError> {
    if path.len() < 2 {
        return Err(EngineError::IllegalMove {
            reason: "Path must have at least 2 regions".into(),
        });
    }

    let stats = get_unit_stats(unit.unit_type);
    let max_move = stats.movement;
    let mut movement_used: u8 = 0;

    for i in 0..path.len() - 1 {
        let from = path[i];
        let to = path[i + 1];

        match (from, to) {
            (RegionId::Sea(f), RegionId::Sea(t)) => {
                if !map.is_sea_adjacent(f, t) {
                    // Check strait connections
                    let mut passable = false;
                    for strait in &map.straits {
                        let (a, b) = strait.connects_seas;
                        if (f == a && t == b) || (f == b && t == a) {
                            if is_strait_passable(state, map, strait.id, power) {
                                passable = true;
                                break;
                            } else {
                                return Err(EngineError::IllegalMove {
                                    reason: format!("Strait '{}' is blocked", strait.name),
                                });
                            }
                        }
                    }
                    if !passable {
                        return Err(EngineError::IllegalMove {
                            reason: "Sea zones are not adjacent".into(),
                        });
                    }
                }
            }
            _ => {
                return Err(EngineError::IllegalMove {
                    reason: "Naval units must move through sea zones".into(),
                });
            }
        }

        movement_used += 1;
    }

    if movement_used > max_move {
        return Err(EngineError::IllegalMove {
            reason: format!(
                "Insufficient movement: need {}, have {}",
                movement_used, max_move
            ),
        });
    }

    Ok(movement_used)
}

/// Validate a path for air movement during combat movement.
/// Air units can fly over any territory type except impassable.
/// Path steps can be Land or Sea regions.
pub fn validate_air_combat_path(
    _state: &GameState,
    map: &GameMap,
    _power: Power,
    unit: &UnitInstance,
    path: &[RegionId],
) -> Result<u8, EngineError> {
    if path.len() < 2 {
        return Err(EngineError::IllegalMove {
            reason: "Path must have at least 2 regions".into(),
        });
    }

    let stats = get_unit_stats(unit.unit_type);
    let max_move = stats.movement;
    let mut movement_used: u8 = 0;

    for i in 0..path.len() - 1 {
        let from = path[i];
        let to = path[i + 1];

        // Air units can move between any adjacent regions
        if !map.is_adjacent(from, to) {
            return Err(EngineError::IllegalMove {
                reason: "Regions are not adjacent for air movement".into(),
            });
        }

        // Check for impassable
        if let RegionId::Land(tid) = to {
            let def = map.territory(tid);
            if def.territory_type == TerritoryType::Impassable {
                return Err(EngineError::IllegalMove {
                    reason: "Cannot fly over impassable territory".into(),
                });
            }
        }

        movement_used += 1;
    }

    if movement_used > max_move {
        return Err(EngineError::IllegalMove {
            reason: format!(
                "Insufficient movement: need {}, have {}",
                movement_used, max_move
            ),
        });
    }

    Ok(movement_used)
}

/// Validate a combat movement path based on unit domain.
pub fn validate_combat_move(
    state: &GameState,
    map: &GameMap,
    power: Power,
    unit: &UnitInstance,
    path: &[RegionId],
) -> Result<u8, EngineError> {
    let stats = get_unit_stats(unit.unit_type);
    match stats.domain {
        UnitDomain::Land => validate_land_combat_path(state, map, power, unit, path),
        UnitDomain::Sea => validate_sea_combat_path(state, map, power, unit, path),
        UnitDomain::Air => validate_air_combat_path(state, map, power, unit, path),
    }
}

/// Validate a non-combat movement path for land units.
/// Land units cannot move into enemy territory during non-combat movement.
pub fn validate_land_noncombat_path(
    state: &GameState,
    map: &GameMap,
    power: Power,
    unit: &UnitInstance,
    path: &[RegionId],
) -> Result<u8, EngineError> {
    if path.len() < 2 {
        return Err(EngineError::IllegalMove {
            reason: "Path must have at least 2 regions".into(),
        });
    }

    let stats = get_unit_stats(unit.unit_type);
    let max_move = stats.movement;
    let mut movement_used: u8 = 0;

    for i in 0..path.len() - 1 {
        let from = path[i];
        let to = path[i + 1];

        let (from_tid, to_tid) = match (from, to) {
            (RegionId::Land(f), RegionId::Land(t)) => (f, t),
            _ => {
                return Err(EngineError::IllegalMove {
                    reason: "Land units can only move through land territories".into(),
                });
            }
        };

        if !map.is_land_adjacent(from_tid, to_tid) {
            return Err(EngineError::IllegalMove {
                reason: "Territories are not adjacent".into(),
            });
        }

        let to_def = map.territory(to_tid);
        if to_def.territory_type == TerritoryType::Impassable {
            return Err(EngineError::IllegalMove {
                reason: "Cannot move into impassable territory".into(),
            });
        }

        // Non-combat: cannot enter enemy territory
        if is_enemy_territory(state, to_tid, power) || has_enemy_units(state, to_tid, power) {
            return Err(EngineError::IllegalMove {
                reason: "Cannot move into enemy territory during non-combat movement".into(),
            });
        }

        // Must be friendly (not neutral)
        if !is_friendly_territory(state, to_tid, power) {
            return Err(EngineError::IllegalMove {
                reason: "Cannot move into neutral territory during non-combat movement".into(),
            });
        }

        movement_used += 1;
    }

    if movement_used > max_move {
        return Err(EngineError::IllegalMove {
            reason: format!(
                "Insufficient movement: need {}, have {}",
                movement_used, max_move
            ),
        });
    }

    Ok(movement_used)
}

/// Validate a non-combat movement path for sea units.
pub fn validate_sea_noncombat_path(
    state: &GameState,
    map: &GameMap,
    power: Power,
    unit: &UnitInstance,
    path: &[RegionId],
) -> Result<u8, EngineError> {
    if path.len() < 2 {
        return Err(EngineError::IllegalMove {
            reason: "Path must have at least 2 regions".into(),
        });
    }

    let stats = get_unit_stats(unit.unit_type);
    let max_move = stats.movement;
    let mut movement_used: u8 = 0;

    for i in 0..path.len() - 1 {
        let from = path[i];
        let to = path[i + 1];

        match (from, to) {
            (RegionId::Sea(f), RegionId::Sea(t)) => {
                if !map.is_sea_adjacent(f, t) {
                    // Check straits
                    let mut passable = false;
                    for strait in &map.straits {
                        let (a, b) = strait.connects_seas;
                        if (f == a && t == b) || (f == b && t == a) {
                            if is_strait_passable(state, map, strait.id, power) {
                                passable = true;
                                break;
                            } else {
                                return Err(EngineError::IllegalMove {
                                    reason: format!("Strait '{}' is blocked", strait.name),
                                });
                            }
                        }
                    }
                    if !passable {
                        return Err(EngineError::IllegalMove {
                            reason: "Sea zones are not adjacent".into(),
                        });
                    }
                }
                // Non-combat: cannot enter sea zone with enemy warships
                if has_enemy_warships(state, t, power) {
                    return Err(EngineError::IllegalMove {
                        reason: "Cannot move into sea zone with enemy warships during non-combat movement".into(),
                    });
                }
            }
            _ => {
                return Err(EngineError::IllegalMove {
                    reason: "Naval units must move through sea zones".into(),
                });
            }
        }

        movement_used += 1;
    }

    if movement_used > max_move {
        return Err(EngineError::IllegalMove {
            reason: format!(
                "Insufficient movement: need {}, have {}",
                movement_used, max_move
            ),
        });
    }

    Ok(movement_used)
}

/// Validate a non-combat move based on unit domain.
pub fn validate_noncombat_move(
    state: &GameState,
    map: &GameMap,
    power: Power,
    unit: &UnitInstance,
    path: &[RegionId],
) -> Result<u8, EngineError> {
    let stats = get_unit_stats(unit.unit_type);
    match stats.domain {
        UnitDomain::Land => validate_land_noncombat_path(state, map, power, unit, path),
        UnitDomain::Sea => validate_sea_noncombat_path(state, map, power, unit, path),
        UnitDomain::Air => {
            // Air units use LandAirUnit action instead during non-combat
            // But they can also move with MoveUnitNonCombat if they didn't move during combat
            validate_air_combat_path(state, map, power, unit, path)
        }
    }
}

/// Check if an air unit has a potential landing spot within remaining movement.
/// Used during ConfirmCombatMovement to verify air units can land.
pub fn air_unit_has_potential_landing(
    state: &GameState,
    map: &GameMap,
    power: Power,
    unit: &UnitInstance,
    current_location: RegionId,
    movement_used: u8,
) -> bool {
    let stats = get_unit_stats(unit.unit_type);
    let remaining = stats.movement.saturating_sub(movement_used);

    // BFS from current location to find friendly land territories within remaining movement
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back((current_location, 0u8));
    visited.insert(current_location);

    while let Some((region, dist)) = queue.pop_front() {
        if dist > remaining {
            continue;
        }

        // Check if this is a valid landing spot
        if let RegionId::Land(tid) = region {
            if is_friendly_territory(state, tid, power) {
                let def = map.territory(tid);
                if def.territory_type != TerritoryType::Impassable {
                    return true;
                }
            }
        }
        // Carriers in friendly sea zones count for fighters/tac bombers
        if let RegionId::Sea(sid) = region {
            if matches!(unit.unit_type, UnitType::Fighter | UnitType::TacticalBomber) {
                let has_friendly_carrier = state.sea_zones[sid as usize]
                    .units
                    .iter()
                    .any(|u| u.unit_type == UnitType::Carrier && state.political.are_friendly(power, u.owner));
                if has_friendly_carrier {
                    return true;
                }
            }
        }

        if dist < remaining {
            // Expand neighbors
            match region {
                RegionId::Land(tid) => {
                    for &n in map.land_neighbors(tid) {
                        let r = RegionId::Land(n);
                        if !visited.contains(&r) {
                            let def = map.territory(n);
                            if def.territory_type != TerritoryType::Impassable {
                                visited.insert(r);
                                queue.push_back((r, dist + 1));
                            }
                        }
                    }
                    for &sz in map.coastal_zones(tid) {
                        let r = RegionId::Sea(sz);
                        if !visited.contains(&r) {
                            visited.insert(r);
                            queue.push_back((r, dist + 1));
                        }
                    }
                }
                RegionId::Sea(sid) => {
                    for &n in map.sea_neighbors(sid) {
                        let r = RegionId::Sea(n);
                        if !visited.contains(&r) {
                            visited.insert(r);
                            queue.push_back((r, dist + 1));
                        }
                    }
                    for &tid in map.coastal_territories(sid) {
                        let r = RegionId::Land(tid);
                        if !visited.contains(&r) {
                            let def = map.territory(tid);
                            if def.territory_type != TerritoryType::Impassable {
                                visited.insert(r);
                                queue.push_back((r, dist + 1));
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

/// Validate that an air unit can land at the given destination.
pub fn validate_air_landing(
    state: &GameState,
    map: &GameMap,
    power: Power,
    unit: &UnitInstance,
    destination: RegionId,
) -> Result<(), EngineError> {
    let stats = get_unit_stats(unit.unit_type);
    if stats.domain != UnitDomain::Air {
        return Err(EngineError::InvalidAction {
            reason: "Only air units can use LandAirUnit".into(),
        });
    }

    match destination {
        RegionId::Land(tid) => {
            if !is_friendly_territory(state, tid, power) {
                return Err(EngineError::IllegalMove {
                    reason: "Air units can only land in friendly territory".into(),
                });
            }
            let def = map.territory(tid);
            if def.territory_type == TerritoryType::Impassable {
                return Err(EngineError::IllegalMove {
                    reason: "Cannot land in impassable territory".into(),
                });
            }
        }
        RegionId::Sea(sid) => {
            // Only fighters and tac bombers can land on carriers
            if !matches!(unit.unit_type, UnitType::Fighter | UnitType::TacticalBomber) {
                return Err(EngineError::IllegalMove {
                    reason: "Only fighters and tactical bombers can land on carriers".into(),
                });
            }
            let has_carrier_space = state.sea_zones[sid as usize]
                .units
                .iter()
                .any(|u| {
                    u.unit_type == UnitType::Carrier
                        && state.political.are_friendly(power, u.owner)
                        && carrier_has_space(state, u.id, sid)
                });
            if !has_carrier_space {
                return Err(EngineError::IllegalMove {
                    reason: "No carrier with available space at this sea zone".into(),
                });
            }
        }
    }

    Ok(())
}

/// Check if a carrier has space for another air unit.
fn carrier_has_space(state: &GameState, _carrier_id: UnitId, sea_zone_id: SeaZoneId) -> bool {
    // Count air units on carriers in this sea zone
    // Simplified: count fighters+tac bombers vs carrier capacity
    let carrier_capacity: usize = state.sea_zones[sea_zone_id as usize]
        .units
        .iter()
        .filter(|u| u.unit_type == UnitType::Carrier)
        .map(|u| get_unit_stats(u.unit_type).can_carry_air as usize)
        .sum();

    let air_on_carriers: usize = state.sea_zones[sea_zone_id as usize]
        .units
        .iter()
        .filter(|u| matches!(u.unit_type, UnitType::Fighter | UnitType::TacticalBomber))
        .count();

    air_on_carriers < carrier_capacity
}

/// Identify territories where combat should occur after combat movement.
/// A battle occurs where the current power has moved units into a region containing enemy units.
pub fn identify_pending_combats(state: &GameState, power: Power) -> Vec<RegionId> {
    let mut combats = Vec::new();

    // Check land territories
    for (i, t) in state.territories.iter().enumerate() {
        let has_friendly = t.units.iter().any(|u| u.owner == power && u.moved_this_turn);
        let has_enemy = t.units.iter().any(|u| state.political.are_at_war(power, u.owner));
        if has_friendly && has_enemy {
            combats.push(RegionId::Land(i as TerritoryId));
        }
    }

    // Check sea zones
    for (i, sz) in state.sea_zones.iter().enumerate() {
        let has_friendly = sz.units.iter().any(|u| u.owner == power && u.moved_this_turn);
        let has_enemy = sz.units.iter().any(|u| state.political.are_at_war(power, u.owner));
        if has_friendly && has_enemy {
            combats.push(RegionId::Sea(i as SeaZoneId));
        }
    }

    combats
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameMap;
    use crate::data::territory_ids as t;
    use crate::data::sea_zone_ids as sz;
    use crate::unit::UnitInstance;

    fn test_state_and_map() -> (GameState, GameMap) {
        let map = GameMap::new();
        let state = crate::setup::create_initial_state(42, &map);
        (state, map)
    }

    #[test]
    fn test_find_unit_in_territory() {
        let (mut state, _map) = test_state_and_map();
        let unit = UnitInstance::new(1, UnitType::Infantry, Power::Germany);
        state.territories[t::GERMANY as usize].units.push(unit);

        let found = find_unit(&state, 1);
        assert!(found.is_some());
        let (region, u) = found.unwrap();
        assert_eq!(region, RegionId::Land(t::GERMANY));
        assert_eq!(u.unit_type, UnitType::Infantry);
    }

    #[test]
    fn test_find_unit_in_sea_zone() {
        let (mut state, _map) = test_state_and_map();
        let unit = UnitInstance::new(99902, UnitType::Destroyer, Power::Germany);
        state.sea_zones[sz::SZ_BALTIC_SEA as usize].units.push(unit);

        let found = find_unit(&state, 99902);
        assert!(found.is_some());
        let (region, _u) = found.unwrap();
        assert_eq!(region, RegionId::Sea(sz::SZ_BALTIC_SEA));
    }

    #[test]
    fn test_is_friendly_territory() {
        let (state, _map) = test_state_and_map();
        // Germany owns Germany
        assert!(is_friendly_territory(&state, t::GERMANY, Power::Germany));
        // Northern Italy is friendly to Germany (same team)
        assert!(is_friendly_territory(&state, t::NORTHERN_ITALY, Power::Germany));
        // UK territory is enemy to Germany
        assert!(!is_friendly_territory(&state, t::UNITED_KINGDOM, Power::Germany));
    }

    #[test]
    fn test_is_enemy_territory() {
        let (state, _map) = test_state_and_map();
        // Germany is at war with UK
        assert!(is_enemy_territory(&state, t::UNITED_KINGDOM, Power::Germany));
        // Germany is not at war with Soviet Union at start
        assert!(!is_enemy_territory(&state, t::RUSSIA, Power::Germany));
    }

    #[test]
    fn test_land_combat_path_valid() {
        let (state, map) = test_state_and_map();
        let unit = UnitInstance::new(1, UnitType::Infantry, Power::Germany);
        // Germany to Western Germany (adjacent, 1 movement)
        let path = vec![RegionId::Land(t::GERMANY), RegionId::Land(t::WESTERN_GERMANY)];
        let result = validate_land_combat_path(&state, &map, Power::Germany, &unit, &path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_land_combat_path_too_far() {
        let (state, map) = test_state_and_map();
        let unit = UnitInstance::new(1, UnitType::Infantry, Power::Germany);
        // Infantry has movement 1, try to go 2 hops
        let path = vec![
            RegionId::Land(t::GERMANY),
            RegionId::Land(t::WESTERN_GERMANY),
            RegionId::Land(t::HOLLAND_BELGIUM),
        ];
        let result = validate_land_combat_path(&state, &map, Power::Germany, &unit, &path);
        assert!(result.is_err());
    }

    #[test]
    fn test_tank_can_move_two() {
        let (state, map) = test_state_and_map();
        let unit = UnitInstance::new(1, UnitType::Tank, Power::Germany);
        // Tank has movement 2
        let path = vec![
            RegionId::Land(t::GERMANY),
            RegionId::Land(t::WESTERN_GERMANY),
            RegionId::Land(t::HOLLAND_BELGIUM),
        ];
        let result = validate_land_combat_path(&state, &map, Power::Germany, &unit, &path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_remove_and_place_unit() {
        let (mut state, _map) = test_state_and_map();
        // Clear existing units so we can test in isolation
        state.territories[t::GERMANY as usize].units.clear();
        state.territories[t::WESTERN_GERMANY as usize].units.clear();
        let unit = UnitInstance::new(99901, UnitType::Infantry, Power::Germany);
        state.territories[t::GERMANY as usize].units.push(unit);

        let (region, unit) = remove_unit(&mut state, 99901).unwrap();
        assert_eq!(region, RegionId::Land(t::GERMANY));
        assert!(state.territories[t::GERMANY as usize].units.is_empty());

        place_unit_at(&mut state, RegionId::Land(t::WESTERN_GERMANY), unit);
        assert_eq!(state.territories[t::WESTERN_GERMANY as usize].units.len(), 1);
    }

    #[test]
    fn test_noncombat_cannot_enter_enemy() {
        let (state, map) = test_state_and_map();
        let unit = UnitInstance::new(1, UnitType::Infantry, Power::Germany);
        // Try to move into France (owned by France, at war with Germany)
        let path = vec![
            RegionId::Land(t::NORMANDY_BORDEAUX),
            RegionId::Land(t::FRANCE),
        ];
        let result = validate_land_noncombat_path(&state, &map, Power::Germany, &unit, &path);
        // France is enemy, should fail
        assert!(result.is_err());
    }

    #[test]
    fn test_identify_pending_combats_empty() {
        let (state, _map) = test_state_and_map();
        let combats = identify_pending_combats(&state, Power::Germany);
        assert!(combats.is_empty());
    }

    #[test]
    fn test_identify_pending_combats_with_units() {
        let (mut state, _map) = test_state_and_map();
        // Place a German unit that has moved into a territory with a UK unit
        let mut german = UnitInstance::new(1, UnitType::Infantry, Power::Germany);
        german.moved_this_turn = true;
        let british = UnitInstance::new(2, UnitType::Infantry, Power::UnitedKingdom);
        state.territories[t::FRANCE as usize].units.push(german);
        state.territories[t::FRANCE as usize].units.push(british);

        let combats = identify_pending_combats(&state, Power::Germany);
        assert_eq!(combats.len(), 1);
        assert_eq!(combats[0], RegionId::Land(t::FRANCE));
    }
}
