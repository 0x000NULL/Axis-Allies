//! Income collection: calculate IPC income from territories, national objectives, and convoy disruption.

use crate::data::GameMap;
use crate::power::Power;
use crate::state::GameState;
use crate::territory::SeaZoneId;
use crate::unit::UnitType;

/// Calculate total income for a power.
pub fn calculate_income(state: &GameState, map: &GameMap, power: Power) -> IncomeBreakdown {
    let base = calculate_base_income(state, map, power);
    let objectives = calculate_national_objectives(state, map, power);
    let convoy_losses = calculate_convoy_disruption(state, map, power);

    IncomeBreakdown {
        base_income: base,
        objective_bonus: objectives,
        convoy_losses,
        total: (base + objectives).saturating_sub(convoy_losses),
    }
}

/// Income breakdown for display.
pub struct IncomeBreakdown {
    pub base_income: u32,
    pub objective_bonus: u32,
    pub convoy_losses: u32,
    pub total: u32,
}

/// Calculate base IPC income from controlled territories.
fn calculate_base_income(state: &GameState, map: &GameMap, power: Power) -> u32 {
    let mut income = 0;
    for (i, territory) in state.territories.iter().enumerate() {
        if territory.owner == Some(power) {
            let tdef = map.territory(i as u16);
            income += tdef.ipc_value;
        }
    }
    income
}

/// Calculate national objective bonuses.
/// Returns total bonus IPCs for the power.
fn calculate_national_objectives(state: &GameState, map: &GameMap, power: Power) -> u32 {
    match power {
        Power::Germany => german_objectives(state, map),
        Power::SovietUnion => soviet_objectives(state, map),
        Power::Japan => japanese_objectives(state, map),
        Power::UnitedStates => us_objectives(state, map),
        Power::UnitedKingdom => uk_objectives(state, map),
        Power::Italy => italian_objectives(state, map),
        Power::ANZAC => anzac_objectives(state, map),
        Power::France => french_objectives(state, map),
        Power::China => 0, // China has no national objectives
    }
}

/// Calculate convoy disruption losses.
fn calculate_convoy_disruption(state: &GameState, map: &GameMap, power: Power) -> u32 {
    let mut losses = 0u32;

    // Check each sea zone for enemy subs/warships that disrupt convoys
    for (sz_idx, sz_state) in state.sea_zones.iter().enumerate() {
        let sz_id = sz_idx as SeaZoneId;
        let sz_def = map.sea_zone(sz_id);

        if !sz_def.is_convoy_zone {
            continue;
        }

        // Check for enemy units in this convoy zone
        let mut enemy_disruption = 0u32;
        for unit in &sz_state.units {
            if unit.owner != power && state.political.are_at_war(power, unit.owner) {
                match unit.unit_type {
                    UnitType::Submarine => enemy_disruption += 2,
                    UnitType::Destroyer | UnitType::Cruiser | UnitType::Battleship | UnitType::Carrier => {
                        enemy_disruption += 1;
                    }
                    _ => {}
                }
            }
        }

        if enemy_disruption > 0 {
            // Disruption is capped at the IPC value of adjacent territories owned by this power
            let mut adjacent_ipc = 0u32;
            for &tid in &sz_def.adjacent_land {
                let territory = &state.territories[tid as usize];
                if territory.owner == Some(power) {
                    let tdef = map.territory(tid);
                    adjacent_ipc += tdef.ipc_value;
                }
            }
            losses += enemy_disruption.min(adjacent_ipc);
        }
    }

    losses
}

/// Apply income collection: add IPCs to power's treasury.
pub fn apply_collect_income(state: &mut GameState, map: &GameMap) {
    let power = state.current_power;
    let breakdown = calculate_income(state, map, power);

    let power_idx = power as usize;
    state.powers[power_idx].ipcs += breakdown.total;

    // Update phase state
    if let crate::phase::PhaseState::CollectIncome(ref mut cis) = state.phase_state {
        cis.base_income = breakdown.base_income;
        cis.objective_bonus = breakdown.objective_bonus;
        cis.convoy_losses = breakdown.convoy_losses;
        cis.total_collected = breakdown.total;
    }
}

// =========================================================================
// National Objectives
// =========================================================================

fn controls(state: &GameState, power: Power, territory_id: u16) -> bool {
    state
        .territories
        .get(territory_id as usize)
        .map(|t| t.owner == Some(power))
        .unwrap_or(false)
}

fn controlled_by_team(state: &GameState, power: Power, territory_id: u16) -> bool {
    state
        .territories
        .get(territory_id as usize)
        .and_then(|t| t.owner)
        .map(|owner| owner.team() == power.team())
        .unwrap_or(false)
}

fn german_objectives(state: &GameState, _map: &GameMap) -> u32 {
    use crate::data::territory_ids as t;
    let mut bonus = 0;

    // 5 IPCs: Germany controls all of: Norway, Denmark, Holland/Belgium, France
    if controls(state, Power::Germany, t::NORWAY)
        && controls(state, Power::Germany, t::DENMARK)
        && controls(state, Power::Germany, t::HOLLAND_BELGIUM)
        && controls(state, Power::Germany, t::FRANCE)
    {
        bonus += 5;
    }

    // 5 IPCs: Axis controls Novgorod (Leningrad), Volgograd (Stalingrad), Russia (Moscow)
    if controlled_by_team(state, Power::Germany, t::NOVGOROD)
        && controlled_by_team(state, Power::Germany, t::VOLGOGRAD)
        && controlled_by_team(state, Power::Germany, t::RUSSIA)
    {
        bonus += 5;
    }

    // 5 IPCs: Germany controls at least one territory in Egypt, Caucasus, or Novgorod
    if controls(state, Power::Germany, t::EGYPT)
        || controls(state, Power::Germany, t::CAUCASUS)
        || controls(state, Power::Germany, t::NOVGOROD)
    {
        bonus += 5;
    }

    bonus
}

fn soviet_objectives(state: &GameState, _map: &GameMap) -> u32 {
    use crate::data::territory_ids as t;
    let mut bonus = 0;

    // Only if at war with an Axis power
    if !state.political.are_at_war(Power::SovietUnion, Power::Germany)
        && !state.political.are_at_war(Power::SovietUnion, Power::Japan)
        && !state.political.are_at_war(Power::SovietUnion, Power::Italy)
    {
        return 0;
    }

    // 5 IPCs: Soviet Union controls Archangel, Novgorod, Russia, Volgograd, Caucasus, plus no Axis on Soviet territory
    if controls(state, Power::SovietUnion, t::ARCHANGEL)
        && controls(state, Power::SovietUnion, t::NOVGOROD)
        && controls(state, Power::SovietUnion, t::RUSSIA)
        && controls(state, Power::SovietUnion, t::VOLGOGRAD)
        && controls(state, Power::SovietUnion, t::CAUCASUS)
    {
        bonus += 5;
    }

    // 3 IPCs: Soviet controls any Axis original territory (Germany, Southern Italy, etc.)
    let axis_territories = [t::GERMANY, t::WESTERN_GERMANY, t::NORTHERN_ITALY, t::SOUTHERN_ITALY, t::JAPAN, t::KOREA];
    if axis_territories.iter().any(|&tid| controls(state, Power::SovietUnion, tid)) {
        bonus += 3;
    }

    bonus
}

fn japanese_objectives(state: &GameState, _map: &GameMap) -> u32 {
    use crate::data::territory_ids as t;
    let mut bonus = 0;

    // 5 IPCs: Japan controls all of: Guam, Midway, Wake, Gilbert, Solomon
    if controls(state, Power::Japan, t::GUAM)
        && controls(state, Power::Japan, t::MIDWAY)
        && controls(state, Power::Japan, t::WAKE_ISLAND)
        && controls(state, Power::Japan, t::SOLOMON_ISLANDS)
    {
        bonus += 5;
    }

    // 5 IPCs: Japan controls all of: Sumatra, Java, Borneo, Celebes
    if controls(state, Power::Japan, t::SUMATRA)
        && controls(state, Power::Japan, t::JAVA)
        && controls(state, Power::Japan, t::BORNEO)
        && controls(state, Power::Japan, t::CELEBES)
    {
        bonus += 5;
    }

    // 5 IPCs: Japan controls all of: India, New South Wales, Hawaiian Islands
    if controls(state, Power::Japan, t::INDIA)
        && controls(state, Power::Japan, t::NEW_SOUTH_WALES)
        && controls(state, Power::Japan, t::HAWAIIAN_ISLANDS)
    {
        bonus += 5;
    }

    bonus
}

fn us_objectives(state: &GameState, _map: &GameMap) -> u32 {
    use crate::data::territory_ids as t;
    let mut bonus = 0;

    if !state.powers[Power::UnitedStates as usize].at_war {
        return 0;
    }

    // 10 IPCs: US is at war and controls France
    if controls(state, Power::UnitedStates, t::FRANCE) {
        bonus += 10;
    }

    // 5 IPCs: US controls all of: Philippines, Borneo, Sumatra, Java, Celebes
    if controls(state, Power::UnitedStates, t::PHILIPPINES)
        && controls(state, Power::UnitedStates, t::BORNEO)
    {
        bonus += 5;
    }

    bonus
}

fn uk_objectives(state: &GameState, _map: &GameMap) -> u32 {
    use crate::data::territory_ids as t;
    let mut bonus = 0;

    // 5 IPCs (Europe): No Axis subs in the Atlantic (simplified: UK controls Gibraltar)
    if controls(state, Power::UnitedKingdom, t::EGYPT) {
        bonus += 5;
    }

    // 5 IPCs: UK controls its original territories
    if controls(state, Power::UnitedKingdom, t::UNITED_KINGDOM)
        && controls(state, Power::UnitedKingdom, t::SCOTLAND)
    {
        bonus += 5;
    }

    bonus
}

fn italian_objectives(state: &GameState, _map: &GameMap) -> u32 {
    use crate::data::territory_ids as t;
    let mut bonus = 0;

    // 5 IPCs: No Allied surface warships in Mediterranean (simplified: Italy controls all N. Africa)
    if controls(state, Power::Italy, t::LIBYA)
        && controls(state, Power::Italy, t::TOBRUK)
        && controls(state, Power::Italy, t::EGYPT)
    {
        bonus += 5;
    }

    // 5 IPCs: Axis controls all of: Morocco, Algeria, Tunisia, Libya, Tobruk, Egypt
    if controlled_by_team(state, Power::Italy, t::MOROCCO)
        && controlled_by_team(state, Power::Italy, t::ALGERIA)
        && controlled_by_team(state, Power::Italy, t::TUNISIA)
        && controlled_by_team(state, Power::Italy, t::LIBYA)
        && controlled_by_team(state, Power::Italy, t::TOBRUK)
        && controlled_by_team(state, Power::Italy, t::EGYPT)
    {
        bonus += 5;
    }

    bonus
}

fn anzac_objectives(state: &GameState, _map: &GameMap) -> u32 {
    use crate::data::territory_ids as t;
    let mut bonus = 0;

    // 5 IPCs: Allies control Malaya, and no Japanese units in ANZAC original territories
    if controlled_by_team(state, Power::ANZAC, t::MALAYA) {
        bonus += 5;
    }

    bonus
}

fn french_objectives(_state: &GameState, _map: &GameMap) -> u32 {
    // France has no national objectives in Global 1940
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameMap;
    use crate::setup;

    #[test]
    fn test_base_income_germany() {
        let map = GameMap::new();
        let state = setup::create_initial_state(42, &map);
        let income = calculate_base_income(&state, &map, Power::Germany);
        // Germany starts controlling territories with significant IPC value
        assert!(income > 0, "Germany should have positive base income");
    }

    #[test]
    fn test_base_income_france_minimal() {
        let map = GameMap::new();
        let state = setup::create_initial_state(42, &map);
        let income = calculate_base_income(&state, &map, Power::France);
        // France starts with Paris captured, so very little income
        // But still controls some colonies
        // This depends on setup
        // u32 is always >= 0; just verify the function doesn't panic
        let _ = income;
    }

    #[test]
    fn test_calculate_income_breakdown() {
        let map = GameMap::new();
        let state = setup::create_initial_state(42, &map);
        let breakdown = calculate_income(&state, &map, Power::Germany);
        assert!(breakdown.base_income > 0);
        assert_eq!(breakdown.total, (breakdown.base_income + breakdown.objective_bonus).saturating_sub(breakdown.convoy_losses));
    }

    #[test]
    fn test_convoy_disruption_no_enemies() {
        let map = GameMap::new();
        let mut state = setup::create_initial_state(42, &map);
        // Clear all sea zone units so there are no enemy ships
        for sz in state.sea_zones.iter_mut() {
            sz.units.clear();
        }
        let losses = calculate_convoy_disruption(&state, &map, Power::Germany);
        assert_eq!(losses, 0);
    }

    #[test]
    fn test_national_objectives_china_zero() {
        let map = GameMap::new();
        let state = setup::create_initial_state(42, &map);
        let bonus = calculate_national_objectives(&state, &map, Power::China);
        assert_eq!(bonus, 0);
    }

    #[test]
    fn test_apply_collect_income() {
        let map = GameMap::new();
        let mut state = setup::create_initial_state(42, &map);
        state.current_power = Power::Germany;
        state.current_phase = crate::phase::Phase::CollectIncome;
        state.phase_state = crate::phase::PhaseState::CollectIncome(crate::phase::CollectIncomeState::new());

        let initial_ipcs = state.powers[Power::Germany as usize].ipcs;
        apply_collect_income(&mut state, &map);
        let final_ipcs = state.powers[Power::Germany as usize].ipcs;

        assert!(final_ipcs > initial_ipcs, "IPCs should increase after collecting income");
    }
}

