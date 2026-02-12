//! Victory condition checking for Global 1940 2nd Edition.

use crate::action::GameEvent;
use crate::data::GameMap;
use crate::power::Team;
use crate::state::GameState;
use crate::territory::TerritoryId;

/// Victory cities and their territory IDs.
pub const VICTORY_CITIES: &[(TerritoryId, &str)] = &[
    // Europe board
    (0, "Berlin"),       // GERMANY
    (5, "Paris"),        // FRANCE
    (26, "London"),      // UNITED_KINGDOM
    (69, "Moscow"),      // RUSSIA
    (19, "Rome"),        // NORTHERN_ITALY
    (65, "Leningrad"),   // NOVGOROD
    (74, "Stalingrad"),  // VOLGOGRAD
    (40, "Cairo"),       // EGYPT
    (151, "Washington"), // EASTERN_UNITED_STATES
    // Pacific board
    (126, "Tokyo"),              // JAPAN
    (114, "Calcutta"),           // INDIA
    (141, "Sydney"),             // NEW_SOUTH_WALES
    (136, "Honolulu"),           // HAWAIIAN_ISLANDS
    (140, "Manila"),             // PHILIPPINES
    (106, "Shanghai"),           // KIANGSU
    (153, "San Francisco"),      // WESTERN_UNITED_STATES
    (118, "Singapore"),          // MALAYA (used for Pacific VC count)
    (95, "Chungking"),           // SZECHWAN (used as China VC stand-in)
];

/// Europe board victory cities (indices into VICTORY_CITIES).
const EUROPE_VC_INDICES: &[usize] = &[0, 1, 2, 3, 4, 5, 6, 7, 8];

/// Pacific board victory cities (indices into VICTORY_CITIES).
const PACIFIC_VC_INDICES: &[usize] = &[9, 10, 11, 12, 13, 14, 15, 16, 17];

/// Result of a victory check.
#[derive(Clone, Debug)]
pub struct VictoryResult {
    pub winner: Team,
    pub reason: String,
}

/// Check if a victory condition has been met.
pub fn check_victory(state: &GameState, _map: &GameMap) -> Option<VictoryResult> {
    // Count Axis VCs on each board
    let mut axis_europe = 0;
    let mut axis_pacific = 0;

    for &idx in EUROPE_VC_INDICES {
        let (tid, _name) = VICTORY_CITIES[idx];
        if let Some(owner) = state.territories.get(tid as usize).and_then(|t| t.owner) {
            if owner.is_axis() {
                axis_europe += 1;
            }
        }
    }

    for &idx in PACIFIC_VC_INDICES {
        let (tid, _name) = VICTORY_CITIES[idx];
        if let Some(owner) = state.territories.get(tid as usize).and_then(|t| t.owner) {
            if owner.is_axis() {
                axis_pacific += 1;
            }
        }
    }

    // Axis wins: 8 VCs on Europe board OR 6 on Pacific board
    if axis_europe >= 8 {
        return Some(VictoryResult {
            winner: Team::Axis,
            reason: format!("Axis controls {} victory cities on the Europe board", axis_europe),
        });
    }

    if axis_pacific >= 6 {
        return Some(VictoryResult {
            winner: Team::Axis,
            reason: format!("Axis controls {} victory cities on the Pacific board", axis_pacific),
        });
    }

    // Allies win: control both Berlin and Tokyo
    let berlin_tid = 0u16; // GERMANY
    let tokyo_tid = 126u16; // JAPAN

    let berlin_allied = state
        .territories
        .get(berlin_tid as usize)
        .and_then(|t| t.owner)
        .map(|o| o.is_allies())
        .unwrap_or(false);

    let tokyo_allied = state
        .territories
        .get(tokyo_tid as usize)
        .and_then(|t| t.owner)
        .map(|o| o.is_allies())
        .unwrap_or(false);

    if berlin_allied && tokyo_allied {
        return Some(VictoryResult {
            winner: Team::Allies,
            reason: "Allies control both Berlin and Tokyo".into(),
        });
    }

    None
}

/// Convert victory result to a game event.
pub fn victory_event(result: &VictoryResult) -> GameEvent {
    GameEvent::VictoryAchieved {
        winner: result.winner,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameMap;
    use crate::power::Power;
    use crate::setup;

    #[test]
    fn test_no_victory_at_start() {
        let map = GameMap::new();
        let state = setup::create_initial_state(42, &map);
        assert!(check_victory(&state, &map).is_none());
    }

    #[test]
    fn test_allied_victory_berlin_and_tokyo() {
        let map = GameMap::new();
        let mut state = setup::create_initial_state(42, &map);

        // Allies capture Berlin and Tokyo
        state.territories[0].owner = Some(Power::UnitedStates); // Germany -> US
        state.territories[126].owner = Some(Power::UnitedStates); // Japan -> US

        let result = check_victory(&state, &map);
        assert!(result.is_some());
        let vr = result.unwrap();
        assert_eq!(vr.winner, Team::Allies);
    }

    #[test]
    fn test_no_victory_only_berlin() {
        let map = GameMap::new();
        let mut state = setup::create_initial_state(42, &map);

        // Allies capture only Berlin
        state.territories[0].owner = Some(Power::UnitedStates);

        let result = check_victory(&state, &map);
        assert!(result.is_none());
    }

    #[test]
    fn test_victory_cities_exist() {
        assert_eq!(VICTORY_CITIES.len(), 18);
    }
}

