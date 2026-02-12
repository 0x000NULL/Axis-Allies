//! Strategic and tactical bombing raids.
//!
//! - Strategic bombing raids target Industrial Complexes (reduce production).
//! - Tactical bombing raids target Air Bases and Naval Bases.
//! - Escort fighters vs interceptor fighters.
//! - AA fire from the facility.
//! - Damage applied to facility.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::dice::DeterministicRng;
use crate::error::EngineError;
use crate::movement;
use crate::state::GameState;
use crate::territory::{FacilityType, TerritoryId};
use crate::unit::{get_unit_stats, UnitId, UnitType};

/// The target of a bombing raid.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export)]
pub enum BombingTarget {
    IndustrialComplex(TerritoryId),
    AirBase(TerritoryId),
    NavalBase(TerritoryId),
}

/// Result of a bombing raid.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BombingResult {
    pub target: BombingTarget,
    pub escort_rolls: Vec<u8>,
    pub interceptor_rolls: Vec<u8>,
    pub escort_hits: u32,
    pub interceptor_hits: u32,
    pub aa_rolls: Vec<u8>,
    pub aa_hits: u32,
    pub bombers_lost: Vec<UnitId>,
    pub interceptors_lost: Vec<UnitId>,
    pub damage_rolls: Vec<u8>,
    pub total_damage: u32,
}

/// Resolve escort vs interceptor combat.
/// Escorts and interceptors each roll at attack/defense 1 (simplified).
/// Each hit removes one unit on the opposing side.
pub fn resolve_escort_interceptor(
    state: &mut GameState,
    escorts: &[UnitId],
    interceptors: &[UnitId],
) -> (Vec<u8>, Vec<u8>, u32, u32) {
    let mut rng = DeterministicRng::new(state.rng_seed, state.rng_counter);

    // Escorts roll (fighters attack at 1 in escort role for simplicity; real rules use normal attack)
    let mut escort_rolls = Vec::new();
    let mut escort_hits = 0u32;
    for &eid in escorts {
        if let Some((_, unit)) = movement::find_unit(state, eid) {
            if unit.unit_type == UnitType::Fighter {
                let roll = rng.roll_d6();
                escort_rolls.push(roll);
                // Escorts hit on 1 (or use normal attack value)
                let stats = get_unit_stats(unit.unit_type);
                if roll <= stats.attack {
                    escort_hits += 1;
                }
            }
        }
    }

    // Interceptors roll
    let mut interceptor_rolls = Vec::new();
    let mut interceptor_hits = 0u32;
    for &iid in interceptors {
        if let Some((_, unit)) = movement::find_unit(state, iid) {
            if unit.unit_type == UnitType::Fighter {
                let roll = rng.roll_d6();
                interceptor_rolls.push(roll);
                let stats = get_unit_stats(unit.unit_type);
                if roll <= stats.defense {
                    interceptor_hits += 1;
                }
            }
        }
    }

    state.rng_counter = rng.counter();
    (escort_rolls, interceptor_rolls, escort_hits, interceptor_hits)
}

/// Resolve AA fire from a facility against bombers.
/// Each bomber is fired at once with a roll of 1 to hit.
pub fn resolve_facility_aa(
    state: &mut GameState,
    bombers: &[UnitId],
) -> (Vec<u8>, u32) {
    let mut rng = DeterministicRng::new(state.rng_seed, state.rng_counter);
    let mut rolls = Vec::new();
    let mut hits = 0u32;

    for &_bid in bombers {
        let roll = rng.roll_d6();
        rolls.push(roll);
        if roll == 1 {
            hits += 1;
        }
    }

    state.rng_counter = rng.counter();
    (rolls, hits)
}

/// Roll bombing damage for surviving bombers.
/// Strategic bombers roll 1d6+2 damage each.
/// Tactical bombers roll 1d6 damage each.
pub fn resolve_bombing_damage(
    state: &mut GameState,
    bombers: &[UnitId],
    target: &BombingTarget,
) -> (Vec<u8>, u32) {
    let mut rng = DeterministicRng::new(state.rng_seed, state.rng_counter);
    let mut rolls = Vec::new();
    let mut total_damage = 0u32;

    // Calculate max damage
    let max_damage = match target {
        BombingTarget::IndustrialComplex(tid) => {
            // Max damage = territory IPC value x 2 (for IC)
            // We'd need the territory def for IPC value, use a reasonable default
            // In practice, this would look up the territory IPC value
            let territory = &state.territories[*tid as usize];
            // We don't have the map here, so use facility max_damage
            territory.facilities.iter()
                .find(|f| matches!(f.facility_type, FacilityType::MinorIndustrialComplex | FacilityType::MajorIndustrialComplex))
                .map(|f| f.max_damage)
                .unwrap_or(20)
        }
        BombingTarget::AirBase(_) | BombingTarget::NavalBase(_) => 6,
    };

    for &bid in bombers {
        if let Some((_, unit)) = movement::find_unit(state, bid) {
            let roll = rng.roll_d6();
            rolls.push(roll);
            let damage = match unit.unit_type {
                UnitType::StrategicBomber => (roll as u32) + 2,
                UnitType::TacticalBomber => roll as u32,
                _ => 0,
            };
            total_damage += damage;
        }
    }

    // Cap at max damage
    total_damage = total_damage.min(max_damage);

    state.rng_counter = rng.counter();
    (rolls, total_damage)
}

/// Apply bombing damage to a facility.
pub fn apply_bombing_damage(
    state: &mut GameState,
    target: &BombingTarget,
    damage: u32,
) -> Result<(), EngineError> {
    let (tid, facility_type) = match target {
        BombingTarget::IndustrialComplex(tid) => (*tid, None::<FacilityType>), // Match IC
        BombingTarget::AirBase(tid) => (*tid, Some(FacilityType::AirBase)),
        BombingTarget::NavalBase(tid) => (*tid, Some(FacilityType::NavalBase)),
    };

    let territory = state.territories.get_mut(tid as usize)
        .ok_or(EngineError::TerritoryNotFound { territory_id: tid })?;

    let facility = if let Some(ft) = facility_type {
        territory.facilities.iter_mut().find(|f| f.facility_type == ft)
    } else {
        territory.facilities.iter_mut().find(|f| {
            matches!(f.facility_type, FacilityType::MinorIndustrialComplex | FacilityType::MajorIndustrialComplex)
        })
    };

    if let Some(f) = facility {
        f.damage = (f.damage + damage).min(f.max_damage);
        f.operational = f.damage < f.max_damage;
    } else {
        return Err(EngineError::InvalidAction {
            reason: "No matching facility found at target territory".into(),
        });
    }

    Ok(())
}

/// Run a complete bombing raid.
pub fn resolve_bombing_raid(
    state: &mut GameState,
    bombers: Vec<UnitId>,
    escorts: Vec<UnitId>,
    interceptors: Vec<UnitId>,
    target: BombingTarget,
) -> Result<BombingResult, EngineError> {
    // Step 1: Escort vs Interceptor combat
    let (escort_rolls, interceptor_rolls, escort_hits, interceptor_hits) =
        resolve_escort_interceptor(state, &escorts, &interceptors);

    // Apply interceptor casualties (from escort hits)
    let mut interceptors_lost = Vec::new();
    let mut remaining_interceptors = interceptors.clone();
    for _ in 0..escort_hits.min(remaining_interceptors.len() as u32) {
        if let Some(iid) = remaining_interceptors.pop() {
            movement::remove_unit(state, iid);
            interceptors_lost.push(iid);
        }
    }

    // Apply escort casualties (from interceptor hits) - escort fighters lost
    // Note: bomber casualties from interceptors also possible, but simplified
    let mut bombers_lost_from_interceptors = Vec::new();
    let mut remaining_escorts = escorts.clone();
    for _ in 0..interceptor_hits {
        if let Some(eid) = remaining_escorts.pop() {
            movement::remove_unit(state, eid);
            bombers_lost_from_interceptors.push(eid);
        }
    }

    // Step 2: AA fire from facility
    let surviving_bombers: Vec<_> = bombers.iter()
        .filter(|b| !bombers_lost_from_interceptors.contains(b))
        .copied()
        .collect();

    let (aa_rolls, aa_hits) = resolve_facility_aa(state, &surviving_bombers);

    // Remove bombers hit by AA
    let mut bombers_lost = bombers_lost_from_interceptors;
    let mut final_bombers = surviving_bombers.clone();
    for _ in 0..aa_hits.min(final_bombers.len() as u32) {
        if let Some(bid) = final_bombers.pop() {
            movement::remove_unit(state, bid);
            bombers_lost.push(bid);
        }
    }

    // Step 3: Roll damage
    let (damage_rolls, total_damage) = resolve_bombing_damage(state, &final_bombers, &target);

    // Step 4: Apply damage
    apply_bombing_damage(state, &target, total_damage)?;

    Ok(BombingResult {
        target,
        escort_rolls,
        interceptor_rolls,
        escort_hits,
        interceptor_hits,
        aa_rolls,
        aa_hits,
        bombers_lost,
        interceptors_lost,
        damage_rolls,
        total_damage,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameMap;
    use crate::data::territory_ids as t;
    use crate::power::Power;
    use crate::territory::{Facility, FacilityType};
    use crate::unit::UnitInstance;

    fn setup_bombing_state() -> GameState {
        let map = GameMap::new();
        let mut state = crate::setup::create_initial_state(42, &map);

        // Set up a territory with an IC and some units
        let tid = t::WESTERN_GERMANY;
        state.territories[tid as usize].owner = Some(Power::Germany);
        state.territories[tid as usize].facilities.push(
            Facility::new(FacilityType::MajorIndustrialComplex, 5),
        );

        // Place bombers (UK attacking)
        let bomber1 = UnitInstance::new(300, UnitType::StrategicBomber, Power::UnitedKingdom);
        let bomber2 = UnitInstance::new(301, UnitType::StrategicBomber, Power::UnitedKingdom);
        state.territories[tid as usize].units.push(bomber1);
        state.territories[tid as usize].units.push(bomber2);

        // Place interceptor
        let interceptor = UnitInstance::new(400, UnitType::Fighter, Power::Germany);
        state.territories[tid as usize].units.push(interceptor);

        state
    }

    #[test]
    fn test_facility_aa_fire() {
        let mut state = setup_bombing_state();
        let bombers = vec![300, 301];
        let (rolls, hits) = resolve_facility_aa(&mut state, &bombers);
        assert_eq!(rolls.len(), 2);
        // Hits should be 0 or more
        assert!(hits <= 2);
    }

    #[test]
    fn test_bombing_damage_rolls() {
        let mut state = setup_bombing_state();
        let target = BombingTarget::IndustrialComplex(t::WESTERN_GERMANY);
        let (rolls, damage) = resolve_bombing_damage(&mut state, &[300, 301], &target);
        assert_eq!(rolls.len(), 2);
        // Strategic bombers roll 1d6+2 each, min 3 max 8 per bomber
        assert!(damage >= 3); // At least one bomber rolling minimum (1+2)
    }

    #[test]
    fn test_apply_bombing_damage() {
        let mut state = setup_bombing_state();
        let target = BombingTarget::IndustrialComplex(t::WESTERN_GERMANY);
        let result = apply_bombing_damage(&mut state, &target, 5);
        assert!(result.is_ok());

        let facility = state.territories[t::WESTERN_GERMANY as usize]
            .facilities.iter()
            .find(|f| matches!(f.facility_type, FacilityType::MajorIndustrialComplex))
            .unwrap();
        assert_eq!(facility.damage, 5);
    }

    #[test]
    fn test_full_bombing_raid() {
        let mut state = setup_bombing_state();

        // Add escort fighter
        let escort = UnitInstance::new(500, UnitType::Fighter, Power::UnitedKingdom);
        state.territories[t::WESTERN_GERMANY as usize].units.push(escort);

        let result = resolve_bombing_raid(
            &mut state,
            vec![300, 301],
            vec![500],
            vec![400],
            BombingTarget::IndustrialComplex(t::WESTERN_GERMANY),
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.escort_rolls.is_empty() || !result.interceptor_rolls.is_empty());
    }
}
