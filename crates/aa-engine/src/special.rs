//! Special rules: China, Kamikaze, straits/canals, capital capture/liberation.

use crate::data::GameMap;
use crate::action::GameEvent;
use crate::error::EngineError;
use crate::power::Power;
use crate::state::GameState;
use crate::territory::{RegionId, TerritoryId};
use crate::unit::{UnitInstance, UnitType};

// =========================================================================
// China special rules
// =========================================================================

/// Chinese territory IDs (all territories China can control).
pub const CHINESE_TERRITORIES: &[TerritoryId] = &[
    95,  // SZECHWAN
    96,  // YUNNAN
    97,  // KWEICHOW
    98,  // HUNAN
    99,  // KIANGSI
    100, // KWANGSI
    101, // SIKANG
    102, // TSINGHAI
    103, // SUIYUAN
    104, // SHENSI
    105, // KANSU
    106, // KIANGSU
    107, // SHANTUNG
    108, // JEHOL
    109, // KWANGTUNG
    110, // ANHWE
    111, // CHAHAR
    112, // HOPEI
    113, // MANCHURIA
];

/// Burma - Chinese units can enter Burma (Burma Road exception).
pub const BURMA: TerritoryId = 116;

/// Check if a territory is a Chinese territory.
pub fn is_chinese_territory(territory_id: TerritoryId) -> bool {
    CHINESE_TERRITORIES.contains(&territory_id)
}

/// Validate Chinese unit movement restrictions.
/// Chinese units can only be in Chinese territories or Burma.
pub fn validate_china_movement(
    _state: &GameState,
    unit: &UnitInstance,
    destination: RegionId,
) -> Result<(), EngineError> {
    if unit.owner != Power::China {
        return Ok(());
    }

    match destination {
        RegionId::Land(tid) => {
            if !is_chinese_territory(tid) && tid != BURMA {
                return Err(EngineError::IllegalMove {
                    reason: "Chinese units cannot leave China (except Burma)".into(),
                });
            }
        }
        RegionId::Sea(_) => {
            return Err(EngineError::IllegalMove {
                reason: "Chinese units cannot enter sea zones".into(),
            });
        }
    }

    Ok(())
}

/// Validate Chinese purchase restrictions.
/// China can only purchase Infantry.
pub fn validate_china_purchase(unit_type: UnitType) -> Result<(), EngineError> {
    if unit_type != UnitType::Infantry {
        return Err(EngineError::InvalidAction {
            reason: "China can only purchase Infantry".into(),
        });
    }
    Ok(())
}

// =========================================================================
// Kamikaze
// =========================================================================

/// Sea zones where Kamikaze attacks can be used.
pub const KAMIKAZE_ZONES: &[u16] = &[
    0,  // SZ_JAPAN_EAST (SZ 6)
    1,  // SZ_SEA_OF_JAPAN (SZ 5)
    30, // SZ_OFF_IWO_JIMA (SZ 7)
    31, // SZ_OFF_OKINAWA (SZ 8)
    4,  // SZ_OFF_FORMOSA (SZ 20)
    25, // SZ_OFF_PHILIPPINES (SZ 35)
];

/// Maximum number of Kamikaze tokens.
pub const MAX_KAMIKAZE_TOKENS: u32 = 6;

/// Check if Kamikaze can be used in a sea zone.
pub fn can_use_kamikaze(sea_zone_id: u16) -> bool {
    KAMIKAZE_ZONES.contains(&sea_zone_id)
}

// =========================================================================
// Strait/Canal control
// =========================================================================

/// Check if a power can pass through a strait/canal.
pub fn can_pass_strait(
    state: &GameState,
    map: &GameMap,
    power: Power,
    strait_id: u8,
) -> bool {
    map.strait_is_passable(strait_id, |tid| {
        let territory = &state.territories[tid as usize];
        match territory.owner {
            Some(owner) => {
                // Friendly = same power or allied and not at war
                owner == power
                    || (owner.team() == power.team()
                        && !state.political.are_at_war(power, owner))
            }
            None => false,
        }
    })
}

// =========================================================================
// Capital capture and liberation
// =========================================================================

/// Handle capital capture: attacker seizes defender's IPC treasury.
pub fn apply_capital_capture(
    state: &mut GameState,
    map: &GameMap,
    territory_id: TerritoryId,
    conquering_power: Power,
) -> Vec<GameEvent> {
    let mut events = Vec::new();
    let tdef = map.territory(territory_id);

    if let Some(capital_power) = tdef.is_capital {
        // Don't capture your own capital
        if capital_power == conquering_power {
            return events;
        }

        // Seize the treasury
        let seized = state.powers[capital_power as usize].ipcs;
        state.powers[capital_power as usize].ipcs = 0;
        state.powers[conquering_power as usize].ipcs += seized;
        state.powers[capital_power as usize].capital_captured = true;

        events.push(GameEvent::CapitalCaptured {
            territory_id,
            by: conquering_power,
        });
    }

    events
}

/// Handle capital liberation: original owner regains control.
pub fn apply_capital_liberation(
    state: &mut GameState,
    map: &GameMap,
    territory_id: TerritoryId,
    liberating_power: Power,
) -> Vec<GameEvent> {
    let mut events = Vec::new();
    let tdef = map.territory(territory_id);

    if let Some(original_power) = tdef.is_capital {
        // Only liberate if the original owner is on the same team as the liberator
        if original_power.team() == liberating_power.team() && original_power != liberating_power {
            // Return territory to original owner
            state.territories[territory_id as usize].owner = Some(original_power);
            state.powers[original_power as usize].capital_captured = false;

            events.push(GameEvent::TerritoryLiberated {
                territory_id,
                to: original_power,
            });
        }
    }

    events
}

/// Check if a territory should be liberated (returned to original owner).
/// This happens when an allied power captures a territory that was originally
/// owned by another allied power whose capital is not captured.
pub fn check_liberation(
    state: &GameState,
    map: &GameMap,
    territory_id: TerritoryId,
    capturing_power: Power,
) -> Option<Power> {
    let tdef = map.territory(territory_id);

    if let Some(original_owner) = tdef.original_owner {
        // If original owner is on the same team, and their capital is free
        if original_owner != capturing_power
            && original_owner.team() == capturing_power.team()
            && !state.powers[original_owner as usize].capital_captured
        {
            return Some(original_owner);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameMap;
    use crate::setup;

    #[test]
    fn test_is_chinese_territory() {
        assert!(is_chinese_territory(95)); // SZECHWAN
        assert!(is_chinese_territory(113)); // MANCHURIA
        assert!(!is_chinese_territory(0)); // GERMANY
        assert!(!is_chinese_territory(126)); // JAPAN
    }

    #[test]
    fn test_chinese_movement_restriction() {
        let state = GameState::new(42);
        let unit = UnitInstance::new(1, UnitType::Infantry, Power::China);

        // Can move to Chinese territory
        let result = validate_china_movement(&state, &unit, RegionId::Land(95));
        assert!(result.is_ok());

        // Can move to Burma
        let result = validate_china_movement(&state, &unit, RegionId::Land(BURMA));
        assert!(result.is_ok());

        // Cannot move to non-Chinese territory
        let result = validate_china_movement(&state, &unit, RegionId::Land(0)); // Germany
        assert!(result.is_err());

        // Cannot enter sea zones
        let result = validate_china_movement(&state, &unit, RegionId::Sea(0));
        assert!(result.is_err());
    }

    #[test]
    fn test_non_chinese_unit_no_restriction() {
        let state = GameState::new(42);
        let unit = UnitInstance::new(1, UnitType::Infantry, Power::Germany);
        let result = validate_china_movement(&state, &unit, RegionId::Land(0));
        assert!(result.is_ok());
    }

    #[test]
    fn test_china_purchase_restriction() {
        assert!(validate_china_purchase(UnitType::Infantry).is_ok());
        assert!(validate_china_purchase(UnitType::Tank).is_err());
        assert!(validate_china_purchase(UnitType::Fighter).is_err());
    }

    #[test]
    fn test_kamikaze_zones() {
        assert!(can_use_kamikaze(0)); // SZ_JAPAN_EAST
        assert!(can_use_kamikaze(31)); // SZ_OFF_OKINAWA
        assert!(!can_use_kamikaze(65)); // SZ_NORTH_SEA
    }

    #[test]
    fn test_strait_passage() {
        let map = GameMap::new();
        let mut state = setup::create_initial_state(42, &map);

        // Turkey is neutral at start, so straits shouldn't be passable by Germany
        let result = can_pass_strait(&state, &map, Power::Germany, 0); // Turkish Straits
        // Turkey has no owner at start, so not passable
        assert!(!result);

        // If UK controls Egypt, Suez is passable for UK
        state.territories[40].owner = Some(Power::UnitedKingdom);
        let result = can_pass_strait(&state, &map, Power::UnitedKingdom, 1); // Suez
        assert!(result);
    }

    #[test]
    fn test_capital_capture() {
        let map = GameMap::new();
        let mut state = setup::create_initial_state(42, &map);

        // Germany captures Moscow (territory 69 = RUSSIA)
        let initial_soviet_ipcs = state.powers[Power::SovietUnion as usize].ipcs;
        let initial_german_ipcs = state.powers[Power::Germany as usize].ipcs;

        state.current_power = Power::Germany;
        let events = apply_capital_capture(&mut state, &map, 69, Power::Germany);

        assert_eq!(state.powers[Power::SovietUnion as usize].ipcs, 0);
        assert_eq!(
            state.powers[Power::Germany as usize].ipcs,
            initial_german_ipcs + initial_soviet_ipcs
        );
        assert!(state.powers[Power::SovietUnion as usize].capital_captured);
        assert!(!events.is_empty());
    }

    #[test]
    fn test_capital_liberation() {
        let map = GameMap::new();
        let mut state = setup::create_initial_state(42, &map);

        // First capture the Soviet capital
        state.territories[69].owner = Some(Power::Germany);
        state.powers[Power::SovietUnion as usize].capital_captured = true;

        // UK liberates Moscow
        let events = apply_capital_liberation(&mut state, &map, 69, Power::UnitedKingdom);
        assert_eq!(state.territories[69].owner, Some(Power::SovietUnion));
        assert!(!state.powers[Power::SovietUnion as usize].capital_captured);
        assert!(!events.is_empty());
    }

    #[test]
    fn test_check_liberation() {
        let map = GameMap::new();
        let mut state = setup::create_initial_state(42, &map);

        // France (territory 5) captured by Germany
        state.territories[5].owner = Some(Power::Germany);
        state.powers[Power::France as usize].capital_captured = false;

        // UK liberates France - should return to France
        let result = check_liberation(&state, &map, 5, Power::UnitedKingdom);
        assert_eq!(result, Some(Power::France));
    }

    #[test]
    fn test_check_liberation_capital_captured() {
        let map = GameMap::new();
        let mut state = setup::create_initial_state(42, &map);

        // France captured and capital is captured
        state.territories[5].owner = Some(Power::Germany);
        state.powers[Power::France as usize].capital_captured = true;

        // UK liberates - should NOT return to France since capital is captured
        let result = check_liberation(&state, &map, 5, Power::UnitedKingdom);
        assert_eq!(result, None);
    }
}

