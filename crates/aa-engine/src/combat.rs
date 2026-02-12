//! Combat resolution engine.
//!
//! Handles battle resolution including:
//! - AA fire, shore bombardment, submarine surprise strikes
//! - Attack/defense rolls, hit calculation
//! - Casualty selection, retreat, submerge
//! - Combat pairing bonuses (infantry+artillery, tac bomber+tank/fighter)
//! - Multi-hit units (battleships, carriers)

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::action::GameEvent;
use crate::dice::DeterministicRng;
use crate::error::EngineError;
use crate::movement;
use crate::phase::PhaseState;
use crate::power::Power;
use crate::state::GameState;
use crate::territory::{RegionId, TerritoryId};
use crate::unit::{get_unit_stats, UnitDomain, UnitId, UnitInstance, UnitType};

/// Sub-phase within a single battle.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export)]
pub enum CombatSubPhase {
    /// AA guns fire at air units (pre-battle).
    AAFire,
    /// Attacker selects AA casualties.
    AAFireCasualties,
    /// Shore bombardment rolls (amphibious assaults).
    ShoreBombardment,
    /// Defender selects shore bombardment casualties.
    ShoreBombardmentCasualties,
    /// Submarine surprise strike - attacker subs fire.
    AttackerSubmarineStrike,
    /// Defender selects surprise strike casualties.
    DefenderSubmarineStrikeCasualties,
    /// Submarine surprise strike - defender subs fire.
    DefenderSubmarineStrike,
    /// Attacker selects surprise strike casualties.
    AttackerSubmarineStrikeCasualties,
    /// Main attack roll phase.
    AttackerRolls,
    /// Main defense roll phase.
    DefenderRolls,
    /// Defender selects casualties from attack hits.
    DefenderSelectsCasualties,
    /// Attacker selects casualties from defense hits.
    AttackerSelectsCasualties,
    /// Attacker chooses: continue, retreat, or submerge.
    AttackerDecision,
    /// Battle is over.
    BattleOver,
}

/// Tracks the state of an active battle.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ActiveCombat {
    /// Location where the battle takes place.
    pub location: RegionId,
    /// The attacking power.
    pub attacker: Power,
    /// Units fighting for the attacker (by ID).
    pub attacker_units: Vec<UnitId>,
    /// Units fighting for the defender.
    pub defender_units: Vec<UnitId>,
    /// The defending power (primary).
    pub defender: Power,
    /// Current round number (starts at 1).
    pub round: u32,
    /// Current sub-phase within this combat round.
    pub sub_phase: CombatSubPhase,
    /// Hits pending allocation to the defender.
    pub pending_attacker_hits: u32,
    /// Hits pending allocation to the attacker.
    pub pending_defender_hits: u32,
    /// Whether this is an amphibious assault.
    pub is_amphibious: bool,
    /// Units eligible for shore bombardment (by ID).
    pub bombardment_units: Vec<UnitId>,
    /// Dice results from the last roll (for UI display).
    pub last_roll: Vec<u8>,
    /// Regions the attacker can retreat to.
    pub retreat_options: Vec<RegionId>,
    /// Whether enemy has a destroyer (cancels sub abilities).
    pub enemy_has_destroyer: bool,
    /// Units that have been submerged this battle.
    pub submerged_units: Vec<UnitId>,
}

impl ActiveCombat {
    /// Create a new active combat.
    pub fn new(
        location: RegionId,
        attacker: Power,
        attacker_units: Vec<UnitId>,
        defender: Power,
        defender_units: Vec<UnitId>,
    ) -> Self {
        ActiveCombat {
            location,
            attacker,
            attacker_units,
            defender,
            defender_units,
            round: 1,
            sub_phase: CombatSubPhase::AAFire,
            pending_attacker_hits: 0,
            pending_defender_hits: 0,
            is_amphibious: false,
            bombardment_units: Vec::new(),
            last_roll: Vec::new(),
            retreat_options: Vec::new(),
            enemy_has_destroyer: false,
            submerged_units: Vec::new(),
        }
    }
}

// =========================================================================
// Combat Resolution Functions
// =========================================================================

/// Start a battle at the given location. Sets up the ActiveCombat and determines
/// the initial sub-phase.
pub fn start_battle(
    state: &mut GameState,
    location: RegionId,
    attacker: Power,
) -> Result<ActiveCombat, EngineError> {
    let (attacker_units, defender_units, defender) = gather_combatants(state, location, attacker)?;

    let mut combat = ActiveCombat::new(
        location,
        attacker,
        attacker_units.clone(),
        defender,
        defender_units.clone(),
    );

    // Check if enemy has destroyers (cancels sub abilities)
    combat.enemy_has_destroyer = check_for_destroyers(state, &defender_units) ||
                                  check_for_destroyers(state, &attacker_units);

    // Calculate retreat options (where attacker's units came from)
    combat.retreat_options = calculate_retreat_options(state, location, attacker);

    // Determine initial sub-phase
    combat.sub_phase = determine_initial_sub_phase(state, &combat);

    Ok(combat)
}

/// Gather attacker and defender units at a location.
fn gather_combatants(
    state: &GameState,
    location: RegionId,
    attacker: Power,
) -> Result<(Vec<UnitId>, Vec<UnitId>, Power), EngineError> {
    let units = get_units_at(state, location);

    let mut attacker_units = Vec::new();
    let mut defender_units = Vec::new();
    let mut defender_power: Option<Power> = None;

    for unit in &units {
        if unit.owner == attacker || state.political.are_friendly(attacker, unit.owner) {
            attacker_units.push(unit.id);
        } else if state.political.are_at_war(attacker, unit.owner) {
            if defender_power.is_none() {
                defender_power = Some(unit.owner);
            }
            defender_units.push(unit.id);
        }
    }

    let defender = defender_power.ok_or(EngineError::InvalidAction {
        reason: "No enemy units at battle location".into(),
    })?;

    Ok((attacker_units, defender_units, defender))
}

/// Get all units at a region.
fn get_units_at(state: &GameState, region: RegionId) -> Vec<UnitInstance> {
    match region {
        RegionId::Land(tid) => state.territories[tid as usize].units.clone(),
        RegionId::Sea(sid) => state.sea_zones[sid as usize].units.clone(),
    }
}

/// Check if any units in a list are destroyers.
fn check_for_destroyers(state: &GameState, unit_ids: &[UnitId]) -> bool {
    for &uid in unit_ids {
        if let Some((_, unit)) = movement::find_unit(state, uid) {
            if unit.unit_type == UnitType::Destroyer {
                return true;
            }
        }
    }
    false
}

/// Calculate retreat options for the attacker.
fn calculate_retreat_options(
    state: &GameState,
    location: RegionId,
    attacker: Power,
) -> Vec<RegionId> {
    // Attacker can retreat to any region from which their units came.
    // For simplicity, find friendly regions adjacent to the battle.
    let options = Vec::new();

    match location {
        RegionId::Land(tid) => {
            // Look at combat move state for origins
            if let PhaseState::Combat(ref cs) = state.phase_state {
                // Fall through to adjacent friendly territories
                let _ = cs;
            }
            // For land battles, adjacent friendly territories the attacker controls
            // In a real implementation we'd track where units came from.
            // For now, use adjacent friendly territories.
            for (i, t) in state.territories.iter().enumerate() {
                let tid2 = i as TerritoryId;
                if tid2 != tid && movement::is_friendly_territory(state, tid2, attacker) {
                    if t.units.iter().any(|u| u.owner == attacker) || movement::is_friendly_territory(state, tid2, attacker) {
                        // Check adjacency via simple index comparison (we don't have map here)
                        // We'll store retreat options when starting the battle
                    }
                }
            }
        }
        RegionId::Sea(_sid) => {
            // Naval: retreat to adjacent sea zone with no enemies
        }
    }

    options
}

/// Determine what the first sub-phase should be.
fn determine_initial_sub_phase(state: &GameState, combat: &ActiveCombat) -> CombatSubPhase {
    // Check for AA guns in land combat
    if let RegionId::Land(_tid) = combat.location {
        let has_aaa = combat.defender_units.iter().any(|&uid| {
            movement::find_unit(state, uid)
                .map(|(_, u)| u.unit_type == UnitType::AAA)
                .unwrap_or(false)
        });
        let has_air_attackers = combat.attacker_units.iter().any(|&uid| {
            movement::find_unit(state, uid)
                .map(|(_, u)| get_unit_stats(u.unit_type).domain == UnitDomain::Air)
                .unwrap_or(false)
        });
        if has_aaa && has_air_attackers {
            return CombatSubPhase::AAFire;
        }
    }

    // Check for submarine surprise strike
    if has_submarines_and_no_destroyer(state, combat, true) ||
       has_submarines_and_no_destroyer(state, combat, false) {
        return CombatSubPhase::AttackerSubmarineStrike;
    }

    CombatSubPhase::AttackerRolls
}

/// Check if one side has subs and the other lacks destroyers.
fn has_submarines_and_no_destroyer(state: &GameState, combat: &ActiveCombat, attacker_side: bool) -> bool {
    let (sub_units, opp_units) = if attacker_side {
        (&combat.attacker_units, &combat.defender_units)
    } else {
        (&combat.defender_units, &combat.attacker_units)
    };

    let has_subs = sub_units.iter().any(|&uid| {
        movement::find_unit(state, uid)
            .map(|(_, u)| u.unit_type == UnitType::Submarine)
            .unwrap_or(false)
    });
    let opp_has_destroyer = opp_units.iter().any(|&uid| {
        movement::find_unit(state, uid)
            .map(|(_, u)| u.unit_type == UnitType::Destroyer)
            .unwrap_or(false)
    });

    has_subs && !opp_has_destroyer
}

/// Process AA fire: AAA units fire at air units.
/// Each AAA fires up to 3 shots at attacking air units, hitting on 1.
pub fn resolve_aa_fire(
    state: &mut GameState,
    combat: &mut ActiveCombat,
) -> Vec<u8> {
    let mut rng = DeterministicRng::new(state.rng_seed, state.rng_counter);

    // Count AAA units among defenders
    let aaa_count = combat.defender_units.iter().filter(|&&uid| {
        movement::find_unit(state, uid)
            .map(|(_, u)| u.unit_type == UnitType::AAA)
            .unwrap_or(false)
    }).count();

    // Count attacking air units
    let air_count = combat.attacker_units.iter().filter(|&&uid| {
        movement::find_unit(state, uid)
            .map(|(_, u)| get_unit_stats(u.unit_type).domain == UnitDomain::Air)
            .unwrap_or(false)
    }).count();

    // Max shots = min(3 * aaa_count, air_count)
    let max_shots = (aaa_count * 3).min(air_count);
    let rolls = rng.roll_multiple_d6(max_shots);
    let hits = rolls.iter().filter(|&&r| r == 1).count() as u32;

    state.rng_counter = rng.counter();
    combat.pending_attacker_hits += hits;
    combat.last_roll = rolls.clone();

    if hits > 0 {
        combat.sub_phase = CombatSubPhase::AAFireCasualties;
    } else {
        // Skip to next phase
        combat.sub_phase = advance_past_aa(state, combat);
    }

    rolls
}

/// Advance past AA fire to the next applicable sub-phase.
fn advance_past_aa(state: &GameState, combat: &ActiveCombat) -> CombatSubPhase {
    if combat.is_amphibious && !combat.bombardment_units.is_empty() {
        return CombatSubPhase::ShoreBombardment;
    }
    if has_submarines_and_no_destroyer(state, combat, true) {
        return CombatSubPhase::AttackerSubmarineStrike;
    }
    CombatSubPhase::AttackerRolls
}

/// Resolve shore bombardment.
pub fn resolve_shore_bombardment(
    state: &mut GameState,
    combat: &mut ActiveCombat,
) -> Vec<u8> {
    let mut rng = DeterministicRng::new(state.rng_seed, state.rng_counter);
    let mut all_rolls = Vec::new();
    let mut hits = 0u32;

    for &uid in &combat.bombardment_units {
        if let Some((_, unit)) = movement::find_unit(state, uid) {
            let stats = get_unit_stats(unit.unit_type);
            if stats.can_bombard {
                let roll = rng.roll_d6();
                all_rolls.push(roll);
                if roll <= stats.bombardment_value {
                    hits += 1;
                }
            }
        }
    }

    state.rng_counter = rng.counter();
    combat.pending_defender_hits = hits;
    combat.last_roll = all_rolls.clone();

    if hits > 0 {
        combat.sub_phase = CombatSubPhase::ShoreBombardmentCasualties;
    } else {
        combat.sub_phase = if has_submarines_and_no_destroyer(state, combat, true) {
            CombatSubPhase::AttackerSubmarineStrike
        } else {
            CombatSubPhase::AttackerRolls
        };
    }

    all_rolls
}

/// Resolve submarine surprise strike for one side.
pub fn resolve_submarine_strike(
    state: &mut GameState,
    combat: &mut ActiveCombat,
    attacker_side: bool,
) -> Vec<u8> {
    let mut rng = DeterministicRng::new(state.rng_seed, state.rng_counter);
    let sub_units = if attacker_side {
        &combat.attacker_units
    } else {
        &combat.defender_units
    };

    let mut rolls = Vec::new();
    let mut hits = 0u32;

    for &uid in sub_units {
        if let Some((_, unit)) = movement::find_unit(state, uid) {
            if unit.unit_type == UnitType::Submarine {
                let stats = get_unit_stats(unit.unit_type);
                let value = if attacker_side { stats.attack } else { stats.defense };
                let roll = rng.roll_d6();
                rolls.push(roll);
                if roll <= value {
                    hits += 1;
                }
            }
        }
    }

    state.rng_counter = rng.counter();

    if attacker_side {
        combat.pending_attacker_hits += hits;
        if hits > 0 {
            combat.sub_phase = CombatSubPhase::DefenderSubmarineStrikeCasualties;
        } else if has_submarines_and_no_destroyer(state, combat, false) {
            combat.sub_phase = CombatSubPhase::DefenderSubmarineStrike;
        } else {
            combat.sub_phase = CombatSubPhase::AttackerRolls;
        }
    } else {
        combat.pending_defender_hits += hits;
        if hits > 0 {
            combat.sub_phase = CombatSubPhase::AttackerSubmarineStrikeCasualties;
        } else {
            combat.sub_phase = CombatSubPhase::AttackerRolls;
        }
    }

    combat.last_roll = rolls.clone();
    rolls
}

/// Calculate attack value for a unit, including pairing bonuses.
pub fn effective_attack_value(state: &GameState, unit: &UnitInstance, friendly_units: &[UnitId]) -> u8 {
    let stats = get_unit_stats(unit.unit_type);
    let mut value = stats.attack;

    match unit.unit_type {
        UnitType::Infantry | UnitType::MechInfantry => {
            // Infantry/MechInfantry paired with Artillery: +1 attack
            let artillery_count = friendly_units.iter().filter(|&&uid| {
                movement::find_unit(state, uid)
                    .map(|(_, u)| u.unit_type == UnitType::Artillery)
                    .unwrap_or(false)
            }).count();
            let inf_mech_count = friendly_units.iter().filter(|&&uid| {
                movement::find_unit(state, uid)
                    .map(|(_, u)| matches!(u.unit_type, UnitType::Infantry | UnitType::MechInfantry))
                    .unwrap_or(false)
            }).count();
            // Each artillery supports one infantry/mech
            // We need to figure out if THIS unit gets the bonus.
            // Simple approach: count inf/mech before this unit, see if artillery covers them.
            let position = friendly_units.iter()
                .filter(|&&uid| {
                    movement::find_unit(state, uid)
                        .map(|(_, u)| matches!(u.unit_type, UnitType::Infantry | UnitType::MechInfantry))
                        .unwrap_or(false)
                })
                .position(|&uid| uid == unit.id)
                .unwrap_or(inf_mech_count);
            if position < artillery_count {
                value += 1;
            }
        }
        UnitType::TacticalBomber => {
            // Tactical bomber paired with tank or fighter: +1 attack (attacks at 4)
            let has_tank_or_fighter = friendly_units.iter().any(|&uid| {
                movement::find_unit(state, uid)
                    .map(|(_, u)| matches!(u.unit_type, UnitType::Tank | UnitType::Fighter))
                    .unwrap_or(false)
            });
            if has_tank_or_fighter {
                value += 1;
            }
        }
        _ => {}
    }

    value
}

/// Roll attack for all attacker units.
pub fn resolve_attack_roll(
    state: &mut GameState,
    combat: &mut ActiveCombat,
) -> Vec<u8> {
    let mut rng = DeterministicRng::new(state.rng_seed, state.rng_counter);
    let mut rolls = Vec::new();
    let mut hits = 0u32;

    // Skip submarines (already fired in surprise strike if applicable) if they had surprise strike
    let skip_subs = has_submarines_and_no_destroyer(state, combat, true);

    let attacker_units = combat.attacker_units.clone();
    for &uid in &attacker_units {
        if combat.submerged_units.contains(&uid) {
            continue;
        }
        if let Some((_, unit)) = movement::find_unit(state, uid) {
            if skip_subs && unit.unit_type == UnitType::Submarine {
                continue; // Already fired in surprise strike
            }
            if unit.unit_type == UnitType::AAA || unit.unit_type == UnitType::Transport {
                continue; // AAA and transports don't attack
            }
            let attack_value = effective_attack_value(state, unit, &attacker_units);
            let roll = rng.roll_d6();
            rolls.push(roll);
            if roll <= attack_value {
                hits += 1;
            }
        }
    }

    state.rng_counter = rng.counter();
    combat.pending_attacker_hits += hits;
    combat.last_roll = rolls.clone();
    combat.sub_phase = CombatSubPhase::DefenderRolls;

    rolls
}

/// Roll defense for all defender units.
pub fn resolve_defense_roll(
    state: &mut GameState,
    combat: &mut ActiveCombat,
) -> Vec<u8> {
    let mut rng = DeterministicRng::new(state.rng_seed, state.rng_counter);
    let mut rolls = Vec::new();
    let mut hits = 0u32;

    let skip_subs = has_submarines_and_no_defender_destroyer(state, combat);

    let defender_units = combat.defender_units.clone();
    for &uid in &defender_units {
        if combat.submerged_units.contains(&uid) {
            continue;
        }
        if let Some((_, unit)) = movement::find_unit(state, uid) {
            if skip_subs && unit.unit_type == UnitType::Submarine {
                continue;
            }
            if unit.unit_type == UnitType::Transport {
                continue; // Transports don't defend
            }
            // AAA doesn't fire in normal combat (only pre-battle)
            if unit.unit_type == UnitType::AAA {
                continue;
            }
            let stats = get_unit_stats(unit.unit_type);
            let roll = rng.roll_d6();
            rolls.push(roll);
            if roll <= stats.defense {
                hits += 1;
            }
        }
    }

    state.rng_counter = rng.counter();
    combat.pending_defender_hits += hits;
    combat.last_roll = rolls.clone();

    // Now both sides need to select casualties
    if combat.pending_attacker_hits > 0 {
        combat.sub_phase = CombatSubPhase::DefenderSelectsCasualties;
    } else if combat.pending_defender_hits > 0 {
        combat.sub_phase = CombatSubPhase::AttackerSelectsCasualties;
    } else {
        combat.sub_phase = CombatSubPhase::AttackerDecision;
    }

    rolls
}

/// Check if defender has subs and attacker has no destroyer.
fn has_submarines_and_no_defender_destroyer(state: &GameState, combat: &ActiveCombat) -> bool {
    has_submarines_and_no_destroyer(state, combat, false)
}

/// Apply casualties selected by a player.
pub fn apply_casualties(
    state: &mut GameState,
    combat: &mut ActiveCombat,
    casualties: &[UnitId],
    defender_side: bool,
) -> Result<Vec<GameEvent>, EngineError> {
    let events = Vec::new();
    let required_hits = if defender_side {
        combat.pending_attacker_hits
    } else {
        combat.pending_defender_hits
    };

    // Validate casualty count
    let mut hits_absorbed = 0u32;
    for &uid in casualties {
        if let Some((_, unit)) = movement::find_unit(state, uid) {
            let stats = get_unit_stats(unit.unit_type);
            if stats.hit_points > 1 && unit.hits_taken == 0 {
                // Multi-hit unit taking first hit (damaged, not destroyed)
                hits_absorbed += 1;
            } else {
                hits_absorbed += 1;
            }
        }
    }

    // Allow fewer casualties if not enough units remain
    let remaining_units = if defender_side {
        &combat.defender_units
    } else {
        &combat.attacker_units
    };
    let max_possible = remaining_units.len() as u32;
    let expected = required_hits.min(max_possible);

    if hits_absorbed < expected {
        return Err(EngineError::InvalidAction {
            reason: format!(
                "Must select {} casualties, but only {} selected",
                expected, hits_absorbed
            ),
        });
    }

    // Apply casualties
    for &uid in casualties {
        let unit_info = movement::find_unit(state, uid).map(|(_, u)| (u.unit_type, u.hits_taken));
        if let Some((unit_type, hits_taken)) = unit_info {
            let stats = get_unit_stats(unit_type);
            if stats.hit_points > 1 && hits_taken == 0 {
                // Damage the unit (first hit on a multi-hit unit)
                if let Some((_, unit_mut)) = movement::find_unit_mut(state, uid) {
                    unit_mut.hits_taken = 1;
                }
            } else {
                // Destroy the unit
                let _ = movement::remove_unit(state, uid);
                // Remove from combat tracking
                if defender_side {
                    combat.defender_units.retain(|&id| id != uid);
                } else {
                    combat.attacker_units.retain(|&id| id != uid);
                }
            }
        }
    }

    // Clear pending hits
    if defender_side {
        combat.pending_attacker_hits = 0;
    } else {
        combat.pending_defender_hits = 0;
    }

    // Advance sub-phase
    if defender_side {
        // Defender just selected casualties from attacker's hits
        if combat.pending_defender_hits > 0 {
            combat.sub_phase = CombatSubPhase::AttackerSelectsCasualties;
        } else {
            // Check if battle is over
            if check_battle_end(combat) {
                combat.sub_phase = CombatSubPhase::BattleOver;
            } else {
                combat.sub_phase = CombatSubPhase::AttackerDecision;
            }
        }
    } else {
        // Attacker just selected casualties from defender's hits
        if check_battle_end(combat) {
            combat.sub_phase = CombatSubPhase::BattleOver;
        } else {
            combat.sub_phase = CombatSubPhase::AttackerDecision;
        }
    }

    Ok(events)
}

/// Check if a battle should end.
pub fn check_battle_end(combat: &ActiveCombat) -> bool {
    // Filter out submerged units
    let active_attackers: Vec<_> = combat.attacker_units.iter()
        .filter(|uid| !combat.submerged_units.contains(uid))
        .collect();
    let active_defenders: Vec<_> = combat.defender_units.iter()
        .filter(|uid| !combat.submerged_units.contains(uid))
        .collect();

    active_attackers.is_empty() || active_defenders.is_empty()
}

/// Process attacker retreat: move all attacking units to the retreat destination.
pub fn apply_retreat(
    state: &mut GameState,
    combat: &mut ActiveCombat,
    retreat_to: RegionId,
) -> Result<Vec<GameEvent>, EngineError> {
    let events = Vec::new();

    // Move all attacker units to the retreat destination
    let unit_ids: Vec<UnitId> = combat.attacker_units.clone();
    for uid in unit_ids {
        if combat.submerged_units.contains(&uid) {
            continue;
        }
        if let Some((_, unit)) = movement::remove_unit(state, uid) {
            movement::place_unit_at(state, retreat_to, unit);
        }
    }

    combat.attacker_units.clear();
    combat.sub_phase = CombatSubPhase::BattleOver;

    Ok(events)
}

/// Submerge a submarine.
pub fn apply_submerge(
    state: &mut GameState,
    combat: &mut ActiveCombat,
    unit_id: UnitId,
) -> Result<(), EngineError> {
    // Verify it's a submarine
    let is_sub = movement::find_unit(state, unit_id)
        .map(|(_, u)| u.unit_type == UnitType::Submarine)
        .unwrap_or(false);
    if !is_sub {
        return Err(EngineError::InvalidAction {
            reason: "Only submarines can submerge".into(),
        });
    }

    combat.submerged_units.push(unit_id);

    // If all attacker units are submerged, battle ends
    let active_attackers: Vec<_> = combat.attacker_units.iter()
        .filter(|uid| !combat.submerged_units.contains(uid))
        .collect();
    if active_attackers.is_empty() {
        combat.sub_phase = CombatSubPhase::BattleOver;
    }

    Ok(())
}

/// Continue to the next combat round.
pub fn continue_combat_round(
    state: &GameState,
    combat: &mut ActiveCombat,
) {
    combat.round += 1;
    combat.pending_attacker_hits = 0;
    combat.pending_defender_hits = 0;

    // Determine if sub surprise strike applies this round
    if has_submarines_and_no_destroyer(state, combat, true) {
        combat.sub_phase = CombatSubPhase::AttackerSubmarineStrike;
    } else {
        combat.sub_phase = CombatSubPhase::AttackerRolls;
    }
}

/// Finalize a battle: handle territory capture for land battles.
pub fn finalize_battle(
    state: &mut GameState,
    combat: &ActiveCombat,
) -> Vec<GameEvent> {
    let mut events = Vec::new();

    let active_defenders: Vec<_> = combat.defender_units.iter()
        .filter(|uid| !combat.submerged_units.contains(uid))
        .collect();
    let active_attackers: Vec<_> = combat.attacker_units.iter()
        .filter(|uid| !combat.submerged_units.contains(uid))
        .collect();

    let attacker_won = active_defenders.is_empty() && !active_attackers.is_empty();

    events.push(GameEvent::BattleEnded {
        location: combat.location,
        attacker_won,
    });

    // Handle territory capture
    if attacker_won {
        if let RegionId::Land(tid) = combat.location {
            let has_land_unit = active_attackers.iter().any(|&&uid| {
                movement::find_unit(state, uid)
                    .map(|(_, u)| get_unit_stats(u.unit_type).domain == UnitDomain::Land)
                    .unwrap_or(false)
            });

            if has_land_unit {
                state.territories[tid as usize].owner = Some(combat.attacker);
                state.territories[tid as usize].just_captured = true;
            }
        }
    }

    events
}

// =========================================================================
// Helper: apply SelectBattle action
// =========================================================================

/// Handle the SelectBattle action.
pub fn apply_select_battle(
    state: &mut GameState,
    location: RegionId,
) -> Result<(ActiveCombat, Vec<GameEvent>), EngineError> {
    let attacker = state.current_power;

    // Remove from pending
    if let PhaseState::Combat(ref mut cs) = state.phase_state {
        if !cs.pending_battles.contains(&location) {
            return Err(EngineError::InvalidAction {
                reason: "No pending battle at this location".into(),
            });
        }
        cs.pending_battles.retain(|&r| r != location);
        cs.current_battle = Some(location);
    }

    let combat = start_battle(state, location, attacker)?;

    let events = vec![GameEvent::BattleStarted { location }];
    Ok((combat, events))
}

/// Handle the RollAttack action.
pub fn apply_roll_attack(
    state: &mut GameState,
    combat: &mut ActiveCombat,
) -> Result<Vec<GameEvent>, EngineError> {
    match combat.sub_phase {
        CombatSubPhase::AAFire => {
            resolve_aa_fire(state, combat);
        }
        CombatSubPhase::ShoreBombardment => {
            resolve_shore_bombardment(state, combat);
        }
        CombatSubPhase::AttackerSubmarineStrike => {
            resolve_submarine_strike(state, combat, true);
        }
        CombatSubPhase::AttackerRolls => {
            resolve_attack_roll(state, combat);
        }
        _ => {
            return Err(EngineError::InvalidAction {
                reason: format!("Cannot roll attack in sub-phase {:?}", combat.sub_phase),
            });
        }
    }
    Ok(Vec::new())
}

/// Handle the RollDefense action.
pub fn apply_roll_defense(
    state: &mut GameState,
    combat: &mut ActiveCombat,
) -> Result<Vec<GameEvent>, EngineError> {
    match combat.sub_phase {
        CombatSubPhase::DefenderSubmarineStrike => {
            resolve_submarine_strike(state, combat, false);
        }
        CombatSubPhase::DefenderRolls => {
            resolve_defense_roll(state, combat);
        }
        _ => {
            return Err(EngineError::InvalidAction {
                reason: format!("Cannot roll defense in sub-phase {:?}", combat.sub_phase),
            });
        }
    }
    Ok(Vec::new())
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameMap;
    use crate::data::territory_ids as t;
    use crate::data::sea_zone_ids as sz;
    use crate::phase::CombatState;
    use crate::territory::SeaZoneId;

    /// Helper: set up state in ConductCombat phase with given units at a territory.
    fn setup_land_combat(
        attacker: Power,
        attacker_units: Vec<(UnitId, UnitType)>,
        defender: Power,
        defender_units: Vec<(UnitId, UnitType)>,
        territory: TerritoryId,
    ) -> GameState {
        let map = GameMap::new();
        let mut state = crate::setup::create_initial_state(42, &map);

        // Clear ALL units from ALL regions to avoid ID collisions
        for t in state.territories.iter_mut() {
            t.units.clear();
        }
        for sz in state.sea_zones.iter_mut() {
            sz.units.clear();
        }

        state.territories[territory as usize].owner = Some(defender);

        // Place attacker units (marked as moved)
        for (id, ut) in &attacker_units {
            let mut u = UnitInstance::new(*id, *ut, attacker);
            u.moved_this_turn = true;
            state.territories[territory as usize].units.push(u);
        }

        // Place defender units
        for (id, ut) in &defender_units {
            let u = UnitInstance::new(*id, *ut, defender);
            state.territories[territory as usize].units.push(u);
        }

        // Set up combat phase
        state.current_power = attacker;
        state.current_phase = crate::phase::Phase::ConductCombat;
        let mut cs = CombatState::new();
        cs.pending_battles.push(RegionId::Land(territory));
        state.phase_state = PhaseState::Combat(cs);

        state
    }

    /// Helper: set up naval combat.
    fn setup_naval_combat(
        attacker: Power,
        attacker_units: Vec<(UnitId, UnitType)>,
        defender: Power,
        defender_units: Vec<(UnitId, UnitType)>,
        sea_zone: SeaZoneId,
    ) -> GameState {
        let map = GameMap::new();
        let mut state = crate::setup::create_initial_state(42, &map);

        // Clear ALL units from ALL regions to avoid ID collisions
        for t in state.territories.iter_mut() {
            t.units.clear();
        }
        for sz in state.sea_zones.iter_mut() {
            sz.units.clear();
        }

        // Place attacker units
        for (id, ut) in &attacker_units {
            let mut u = UnitInstance::new(*id, *ut, attacker);
            u.moved_this_turn = true;
            state.sea_zones[sea_zone as usize].units.push(u);
        }

        // Place defender units
        for (id, ut) in &defender_units {
            let u = UnitInstance::new(*id, *ut, defender);
            state.sea_zones[sea_zone as usize].units.push(u);
        }

        // Set up combat phase
        state.current_power = attacker;
        state.current_phase = crate::phase::Phase::ConductCombat;
        let mut cs = CombatState::new();
        cs.pending_battles.push(RegionId::Sea(sea_zone));
        state.phase_state = PhaseState::Combat(cs);

        state
    }

    #[test]
    fn test_basic_land_combat_infantry_vs_infantry() {
        let mut state = setup_land_combat(
            Power::Germany,
            vec![(100, UnitType::Infantry), (101, UnitType::Infantry)],
            Power::UnitedKingdom,
            vec![(200, UnitType::Infantry)],
            t::FRANCE,
        );

        // Select the battle
        let (mut combat, events) = apply_select_battle(&mut state, RegionId::Land(t::FRANCE)).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(combat.attacker, Power::Germany);
        assert_eq!(combat.defender, Power::UnitedKingdom);
        assert_eq!(combat.attacker_units.len(), 2);
        assert_eq!(combat.defender_units.len(), 1);

        // Should go to AttackerRolls (no AA, no subs)
        assert_eq!(combat.sub_phase, CombatSubPhase::AttackerRolls);

        // Roll attack
        let _rolls = resolve_attack_roll(&mut state, &mut combat);
        assert_eq!(combat.sub_phase, CombatSubPhase::DefenderRolls);

        // Roll defense
        let _rolls = resolve_defense_roll(&mut state, &mut combat);
        // Should be in some casualty selection or decision phase
        assert!(matches!(
            combat.sub_phase,
            CombatSubPhase::DefenderSelectsCasualties
                | CombatSubPhase::AttackerSelectsCasualties
                | CombatSubPhase::AttackerDecision
        ));
    }

    #[test]
    fn test_aa_fire() {
        let mut state = setup_land_combat(
            Power::Germany,
            vec![
                (100, UnitType::Infantry),
                (101, UnitType::Fighter),
                (102, UnitType::Fighter),
            ],
            Power::UnitedKingdom,
            vec![
                (200, UnitType::Infantry),
                (201, UnitType::AAA),
            ],
            t::FRANCE,
        );

        let (mut combat, _) = apply_select_battle(&mut state, RegionId::Land(t::FRANCE)).unwrap();
        assert_eq!(combat.sub_phase, CombatSubPhase::AAFire);

        // Resolve AA fire
        let rolls = resolve_aa_fire(&mut state, &mut combat);
        // 1 AAA, 2 air units → min(3, 2) = 2 shots
        assert_eq!(rolls.len(), 2);
    }

    #[test]
    fn test_submarine_surprise_strike() {
        // Attacker has subs, defender has no destroyer
        let mut state = setup_naval_combat(
            Power::Germany,
            vec![(100, UnitType::Submarine)],
            Power::UnitedKingdom,
            vec![(200, UnitType::Cruiser)],
            sz::SZ_NORTH_SEA,
        );

        let (mut combat, _) = apply_select_battle(&mut state, RegionId::Sea(sz::SZ_NORTH_SEA)).unwrap();
        assert_eq!(combat.sub_phase, CombatSubPhase::AttackerSubmarineStrike);

        let rolls = resolve_submarine_strike(&mut state, &mut combat, true);
        assert_eq!(rolls.len(), 1); // One sub
    }

    #[test]
    fn test_submarine_no_surprise_with_destroyer() {
        // Defender has destroyer, so no surprise strike
        let mut state = setup_naval_combat(
            Power::Germany,
            vec![(100, UnitType::Submarine)],
            Power::UnitedKingdom,
            vec![(200, UnitType::Destroyer)],
            sz::SZ_NORTH_SEA,
        );

        let (combat, _) = apply_select_battle(&mut state, RegionId::Sea(sz::SZ_NORTH_SEA)).unwrap();
        // Should skip straight to AttackerRolls since destroyer negates surprise
        assert_eq!(combat.sub_phase, CombatSubPhase::AttackerRolls);
    }

    #[test]
    fn test_casualty_selection() {
        let mut state = setup_land_combat(
            Power::Germany,
            vec![(100, UnitType::Infantry)],
            Power::UnitedKingdom,
            vec![(200, UnitType::Infantry), (201, UnitType::Infantry)],
            t::FRANCE,
        );

        let (mut combat, _) = apply_select_battle(&mut state, RegionId::Land(t::FRANCE)).unwrap();

        // Simulate 1 hit on defenders
        combat.pending_attacker_hits = 1;
        combat.sub_phase = CombatSubPhase::DefenderSelectsCasualties;

        // Defender selects unit 200 as casualty
        let result = apply_casualties(&mut state, &mut combat, &[200], true);
        assert!(result.is_ok());
        assert_eq!(combat.defender_units.len(), 1);
        assert_eq!(combat.defender_units[0], 201);
        assert_eq!(combat.pending_attacker_hits, 0);
    }

    #[test]
    fn test_multi_hit_battleship() {
        let mut state = setup_naval_combat(
            Power::Germany,
            vec![(100, UnitType::Battleship)],
            Power::UnitedKingdom,
            vec![(200, UnitType::Cruiser)],
            sz::SZ_NORTH_SEA,
        );

        let (mut combat, _) = apply_select_battle(&mut state, RegionId::Sea(sz::SZ_NORTH_SEA)).unwrap();

        // Simulate 1 hit on attacker's battleship
        combat.pending_defender_hits = 1;
        combat.sub_phase = CombatSubPhase::AttackerSelectsCasualties;

        // Battleship takes first hit (becomes damaged, not destroyed)
        let result = apply_casualties(&mut state, &mut combat, &[100], false);
        assert!(result.is_ok());
        // Battleship should still be in combat
        assert_eq!(combat.attacker_units.len(), 1);
        // Verify it's damaged
        let (_, unit) = movement::find_unit(&state, 100).unwrap();
        assert_eq!(unit.hits_taken, 1);
    }

    #[test]
    fn test_combat_pairing_infantry_artillery() {
        let state = setup_land_combat(
            Power::Germany,
            vec![
                (100, UnitType::Infantry),
                (101, UnitType::Artillery),
            ],
            Power::UnitedKingdom,
            vec![(200, UnitType::Infantry)],
            t::FRANCE,
        );

        // Check effective attack value
        let (_, inf) = movement::find_unit(&state, 100).unwrap();
        let attack = effective_attack_value(&state, inf, &[100, 101]);
        assert_eq!(attack, 2); // Infantry base 1 + 1 from artillery = 2
    }

    #[test]
    fn test_combat_pairing_tac_bomber_with_tank() {
        let state = setup_land_combat(
            Power::Germany,
            vec![
                (100, UnitType::TacticalBomber),
                (101, UnitType::Tank),
            ],
            Power::UnitedKingdom,
            vec![(200, UnitType::Infantry)],
            t::FRANCE,
        );

        let (_, tac) = movement::find_unit(&state, 100).unwrap();
        let attack = effective_attack_value(&state, tac, &[100, 101]);
        assert_eq!(attack, 4); // TacBomber base 3 + 1 from tank = 4
    }

    #[test]
    fn test_retreat() {
        let mut state = setup_land_combat(
            Power::Germany,
            vec![(100, UnitType::Infantry), (101, UnitType::Infantry)],
            Power::UnitedKingdom,
            vec![(200, UnitType::Infantry)],
            t::FRANCE,
        );

        let (mut combat, _) = apply_select_battle(&mut state, RegionId::Land(t::FRANCE)).unwrap();
        combat.sub_phase = CombatSubPhase::AttackerDecision;

        // Retreat to Normandy-Bordeaux
        let result = apply_retreat(&mut state, &mut combat, RegionId::Land(t::NORMANDY_BORDEAUX));
        assert!(result.is_ok());
        assert_eq!(combat.sub_phase, CombatSubPhase::BattleOver);
        assert!(combat.attacker_units.is_empty());

        // Verify units moved
        let normandy_units = &state.territories[t::NORMANDY_BORDEAUX as usize].units;
        assert!(normandy_units.iter().any(|u| u.id == 100));
        assert!(normandy_units.iter().any(|u| u.id == 101));
    }

    #[test]
    fn test_submerge() {
        let mut state = setup_naval_combat(
            Power::Germany,
            vec![(100, UnitType::Submarine)],
            Power::UnitedKingdom,
            vec![(200, UnitType::Cruiser)],
            sz::SZ_NORTH_SEA,
        );

        let (mut combat, _) = apply_select_battle(&mut state, RegionId::Sea(sz::SZ_NORTH_SEA)).unwrap();
        combat.sub_phase = CombatSubPhase::AttackerDecision;

        // Submerge the sub
        let result = apply_submerge(&mut state, &mut combat, 100);
        assert!(result.is_ok());
        assert!(combat.submerged_units.contains(&100));
        // Only sub was attacking, so battle should be over
        assert_eq!(combat.sub_phase, CombatSubPhase::BattleOver);
    }

    #[test]
    fn test_battle_end_attacker_eliminated() {
        let combat = ActiveCombat {
            attacker_units: vec![],
            defender_units: vec![200],
            submerged_units: vec![],
            location: RegionId::Land(0),
            attacker: Power::Germany,
            defender: Power::UnitedKingdom,
            round: 1,
            sub_phase: CombatSubPhase::AttackerDecision,
            pending_attacker_hits: 0,
            pending_defender_hits: 0,
            is_amphibious: false,
            bombardment_units: vec![],
            last_roll: vec![],
            retreat_options: vec![],
            enemy_has_destroyer: false,
        };
        assert!(check_battle_end(&combat));
    }

    #[test]
    fn test_battle_end_defender_eliminated() {
        let combat = ActiveCombat {
            attacker_units: vec![100],
            defender_units: vec![],
            submerged_units: vec![],
            location: RegionId::Land(0),
            attacker: Power::Germany,
            defender: Power::UnitedKingdom,
            round: 1,
            sub_phase: CombatSubPhase::AttackerDecision,
            pending_attacker_hits: 0,
            pending_defender_hits: 0,
            is_amphibious: false,
            bombardment_units: vec![],
            last_roll: vec![],
            retreat_options: vec![],
            enemy_has_destroyer: false,
        };
        assert!(check_battle_end(&combat));
    }

    #[test]
    fn test_multi_round_combat() {
        let mut state = setup_land_combat(
            Power::Germany,
            vec![(100, UnitType::Infantry), (101, UnitType::Infantry), (102, UnitType::Infantry)],
            Power::UnitedKingdom,
            vec![(200, UnitType::Infantry), (201, UnitType::Infantry)],
            t::FRANCE,
        );

        let (mut combat, _) = apply_select_battle(&mut state, RegionId::Land(t::FRANCE)).unwrap();

        // Round 1
        assert_eq!(combat.round, 1);
        resolve_attack_roll(&mut state, &mut combat);
        resolve_defense_roll(&mut state, &mut combat);

        // Handle casualties if any (auto-select for test)
        if combat.pending_attacker_hits > 0 {
            let defender_ids: Vec<_> = combat.defender_units.clone();
            let to_remove: Vec<_> = defender_ids.iter()
                .take(combat.pending_attacker_hits as usize)
                .copied()
                .collect();
            apply_casualties(&mut state, &mut combat, &to_remove, true).unwrap();
        }
        if combat.pending_defender_hits > 0 {
            let attacker_ids: Vec<_> = combat.attacker_units.clone();
            let to_remove: Vec<_> = attacker_ids.iter()
                .take(combat.pending_defender_hits as usize)
                .copied()
                .collect();
            apply_casualties(&mut state, &mut combat, &to_remove, false).unwrap();
        }

        if !check_battle_end(&combat) {
            // Continue to round 2
            continue_combat_round(&state, &mut combat);
            assert_eq!(combat.round, 2);
            assert!(matches!(
                combat.sub_phase,
                CombatSubPhase::AttackerRolls | CombatSubPhase::AttackerSubmarineStrike
            ));
        }
    }

    #[test]
    fn test_finalize_battle_territory_capture() {
        let mut state = setup_land_combat(
            Power::Germany,
            vec![(100, UnitType::Infantry)],
            Power::UnitedKingdom,
            vec![],  // No defenders left
            t::FRANCE,
        );
        state.territories[t::FRANCE as usize].owner = Some(Power::UnitedKingdom);

        let combat = ActiveCombat {
            location: RegionId::Land(t::FRANCE),
            attacker: Power::Germany,
            attacker_units: vec![100],
            defender: Power::UnitedKingdom,
            defender_units: vec![],
            round: 1,
            sub_phase: CombatSubPhase::BattleOver,
            pending_attacker_hits: 0,
            pending_defender_hits: 0,
            is_amphibious: false,
            bombardment_units: vec![],
            last_roll: vec![],
            retreat_options: vec![],
            enemy_has_destroyer: false,
            submerged_units: vec![],
        };

        let events = finalize_battle(&mut state, &combat);
        assert_eq!(state.territories[t::FRANCE as usize].owner, Some(Power::Germany));
        assert!(state.territories[t::FRANCE as usize].just_captured);
        assert!(events.iter().any(|e| matches!(e, GameEvent::BattleEnded { attacker_won: true, .. })));
    }
}
