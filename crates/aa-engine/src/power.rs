//! Power (nation) definitions, teams, and turn order.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The nine playable powers in Global 1940 2nd Edition.
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, TS)]
#[ts(export)]
#[repr(u8)]
pub enum Power {
    Germany = 0,
    SovietUnion = 1,
    Japan = 2,
    UnitedStates = 3,
    China = 4,
    UnitedKingdom = 5,
    Italy = 6,
    ANZAC = 7,
    France = 8,
}

impl Power {
    /// Returns the team this power belongs to.
    pub fn team(&self) -> Team {
        match self {
            Power::Germany | Power::Japan | Power::Italy => Team::Axis,
            _ => Team::Allies,
        }
    }

    /// Returns true if this power is on the Axis team.
    pub fn is_axis(&self) -> bool {
        self.team() == Team::Axis
    }

    /// Returns true if this power is on the Allies team.
    pub fn is_allies(&self) -> bool {
        self.team() == Team::Allies
    }

    /// Display name for UI.
    pub fn name(&self) -> &'static str {
        match self {
            Power::Germany => "Germany",
            Power::SovietUnion => "Soviet Union",
            Power::Japan => "Japan",
            Power::UnitedStates => "United States",
            Power::China => "China",
            Power::UnitedKingdom => "United Kingdom",
            Power::Italy => "Italy",
            Power::ANZAC => "ANZAC",
            Power::France => "France",
        }
    }

    /// Returns all powers in an array.
    pub fn all() -> &'static [Power; 9] {
        &TURN_ORDER
    }
}

/// The two teams in the game.
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub enum Team {
    Axis,
    Allies,
}

/// The official turn order for Global 1940 2nd Edition.
pub const TURN_ORDER: [Power; 9] = [
    Power::Germany,
    Power::SovietUnion,
    Power::Japan,
    Power::UnitedStates,
    Power::China,
    Power::UnitedKingdom,
    Power::Italy,
    Power::ANZAC,
    Power::France,
];

/// Returns the next power in turn order after the given power.
pub fn next_power(current: Power) -> Power {
    let idx = TURN_ORDER.iter().position(|&p| p == current).unwrap();
    TURN_ORDER[(idx + 1) % TURN_ORDER.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_teams() {
        assert_eq!(Power::Germany.team(), Team::Axis);
        assert_eq!(Power::Japan.team(), Team::Axis);
        assert_eq!(Power::Italy.team(), Team::Axis);
        assert_eq!(Power::SovietUnion.team(), Team::Allies);
        assert_eq!(Power::UnitedStates.team(), Team::Allies);
        assert_eq!(Power::China.team(), Team::Allies);
        assert_eq!(Power::UnitedKingdom.team(), Team::Allies);
        assert_eq!(Power::ANZAC.team(), Team::Allies);
        assert_eq!(Power::France.team(), Team::Allies);
    }

    #[test]
    fn test_turn_order() {
        assert_eq!(next_power(Power::Germany), Power::SovietUnion);
        assert_eq!(next_power(Power::France), Power::Germany);
    }
}
