//! Political rules: war declarations, neutral nations, and political triggers.

use crate::action::GameEvent;
use crate::data::GameMap;
use crate::error::EngineError;
use crate::power::Power;
use crate::state::GameState;
use crate::territory::{TerritoryType, TerritoryId};

/// Validate a DeclareWar action.
pub fn validate_declare_war(state: &GameState, against: Power) -> Result<(), EngineError> {
    let power = state.current_power;

    // Can't declare war on yourself
    if power == against {
        return Err(EngineError::InvalidAction {
            reason: "Cannot declare war on yourself".into(),
        });
    }

    // Can't declare war on an ally
    if power.team() == against.team() {
        return Err(EngineError::InvalidAction {
            reason: "Cannot declare war on an allied power".into(),
        });
    }

    // Already at war?
    if state.political.are_at_war(power, against) {
        return Err(EngineError::InvalidAction {
            reason: format!("Already at war with {:?}", against),
        });
    }

    // Special restrictions:
    // Soviet Union cannot declare war on Japan until turn 4 (unless attacked)
    // US cannot declare war until turn 3 (unless attacked)
    // These are simplified - the actual rules are more nuanced

    Ok(())
}

/// Apply a DeclareWar action.
pub fn apply_declare_war(
    state: &mut GameState,
    against: Power,
) -> Vec<GameEvent> {
    let power = state.current_power;
    let mut events = Vec::new();

    // Set war state
    state.political.war_matrix[power as usize][against as usize] = true;
    state.political.war_matrix[against as usize][power as usize] = true;

    events.push(GameEvent::WarDeclared {
        aggressor: power,
        target: against,
    });

    // Update political triggers
    if power == Power::UnitedStates || against == Power::UnitedStates {
        state.political.triggers.us_at_war = true;
        if state.political.triggers.us_war_turn.is_none() {
            state.political.triggers.us_war_turn = Some(state.turn_number);
        }
        state.powers[Power::UnitedStates as usize].at_war = true;
    }

    if (power == Power::SovietUnion && against.is_axis())
        || (against == Power::SovietUnion && power.is_axis())
    {
        state.political.triggers.soviet_at_war_with_axis = true;
        state.powers[Power::SovietUnion as usize].at_war = true;
    }

    events
}

/// Handle attacking a neutral territory.
/// Returns events and may modify state (true neutrals flip).
pub fn handle_neutral_attack(
    state: &mut GameState,
    map: &GameMap,
    territory_id: TerritoryId,
) -> Vec<GameEvent> {
    let events = Vec::new();
    let tdef = map.territory(territory_id);

    match tdef.territory_type {
        TerritoryType::TrueNeutral => {
            // Attacking a true neutral turns ALL remaining true neutrals pro-enemy
            let attacker = state.current_power;
            let enemy_type = if attacker.is_axis() {
                TerritoryType::ProAllies
            } else {
                TerritoryType::ProAxis
            };

            // Note: We can't modify TerritoryDef since it's static.
            // In practice, we'd track neutral status changes in GameState.
            // For now, this serves as the logic placeholder.
            let _ = enemy_type;
        }
        TerritoryType::ProAxis | TerritoryType::ProAllies => {
            // Pro-neutral: when an allied power enters, it joins that side
            // This is handled during movement, not as a separate political action
        }
        _ => {}
    }

    events
}

/// Check if the US should automatically enter the war.
/// US enters at the start of its turn on round 4 if not already at war.
pub fn check_us_entry(state: &mut GameState) -> Vec<GameEvent> {
    let mut events = Vec::new();

    if state.current_power == Power::UnitedStates
        && !state.powers[Power::UnitedStates as usize].at_war
        && state.turn_number >= 4
    {
        // US automatically enters the war
        state.powers[Power::UnitedStates as usize].at_war = true;
        state.political.triggers.us_at_war = true;
        state.political.triggers.us_war_turn = Some(state.turn_number);

        // Declare war on all Axis powers
        for &axis in &[Power::Germany, Power::Japan, Power::Italy] {
            if !state.political.are_at_war(Power::UnitedStates, axis) {
                state.political.war_matrix[Power::UnitedStates as usize][axis as usize] = true;
                state.political.war_matrix[axis as usize][Power::UnitedStates as usize] = true;
                events.push(GameEvent::WarDeclared {
                    aggressor: Power::UnitedStates,
                    target: axis,
                });
            }
        }
    }

    events
}

/// Check if the Soviet Union should automatically enter the war.
/// Soviets can declare war on Axis starting turn 4.
pub fn check_soviet_entry(_state: &mut GameState) -> Vec<GameEvent> {
    // Soviets don't auto-enter; they can choose to declare war.
    // But if an Axis power attacks them, they're automatically at war.
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameMap;
    use crate::setup;

    #[test]
    fn test_declare_war_self_fails() {
        let map = GameMap::new();
        let mut state = setup::create_initial_state(42, &map);
        state.current_power = Power::Germany;
        let result = validate_declare_war(&state, Power::Germany);
        assert!(result.is_err());
    }

    #[test]
    fn test_declare_war_ally_fails() {
        let map = GameMap::new();
        let mut state = setup::create_initial_state(42, &map);
        state.current_power = Power::Germany;
        let result = validate_declare_war(&state, Power::Italy);
        assert!(result.is_err());
    }

    #[test]
    fn test_declare_war_already_at_war_fails() {
        let map = GameMap::new();
        let mut state = setup::create_initial_state(42, &map);
        state.current_power = Power::Germany;
        // Germany is already at war with UK
        let result = validate_declare_war(&state, Power::UnitedKingdom);
        assert!(result.is_err());
    }

    #[test]
    fn test_declare_war_valid() {
        let map = GameMap::new();
        let mut state = setup::create_initial_state(42, &map);
        state.current_power = Power::Germany;
        // Germany is NOT at war with Soviet Union at start
        let result = validate_declare_war(&state, Power::SovietUnion);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_declare_war() {
        let map = GameMap::new();
        let mut state = setup::create_initial_state(42, &map);
        state.current_power = Power::Germany;

        assert!(!state.political.are_at_war(Power::Germany, Power::SovietUnion));
        let events = apply_declare_war(&mut state, Power::SovietUnion);
        assert!(state.political.are_at_war(Power::Germany, Power::SovietUnion));
        assert!(!events.is_empty());
    }

    #[test]
    fn test_us_auto_entry_turn_4() {
        let map = GameMap::new();
        let mut state = setup::create_initial_state(42, &map);
        state.current_power = Power::UnitedStates;
        state.turn_number = 4;
        state.powers[Power::UnitedStates as usize].at_war = false;

        let events = check_us_entry(&mut state);
        assert!(state.powers[Power::UnitedStates as usize].at_war);
        assert!(!events.is_empty());
    }

    #[test]
    fn test_us_no_entry_before_turn_4() {
        let map = GameMap::new();
        let mut state = setup::create_initial_state(42, &map);
        state.current_power = Power::UnitedStates;
        state.turn_number = 2;
        state.powers[Power::UnitedStates as usize].at_war = false;

        let events = check_us_entry(&mut state);
        assert!(!state.powers[Power::UnitedStates as usize].at_war);
        assert!(events.is_empty());
    }
}

