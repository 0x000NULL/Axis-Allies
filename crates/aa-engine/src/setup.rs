//! Initial game setup: creates the starting game state for Global 1940 2nd Edition.
//!
//! Full OOB (Order of Battle) unit placements will be added in Phase 10.
//! For now, creates territories and sea zones from the GameMap data.

use crate::data::GameMap;
use crate::state::GameState;
use crate::territory::{SeaZoneState, TerritoryState};

/// Create the initial game state for a new Global 1940 2nd Edition game.
///
/// Territories and sea zones are created from the static `GameMap` data.
/// Phase 10 will add the full OOB with all starting units for all 9 powers.
pub fn create_initial_state(seed: u64, map: &GameMap) -> GameState {
    let mut state = GameState::new(seed);

    // Create territory states from map definitions
    for def in &map.territories {
        state.territories.push(TerritoryState::new(def.original_owner));
    }

    // Create sea zone states from map definitions
    for _def in &map.sea_zones {
        state.sea_zones.push(SeaZoneState::new());
    }

    state
}
