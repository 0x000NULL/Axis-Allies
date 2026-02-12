//! Top-level game state container.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::action::AppliedAction;
use crate::phase::{Phase, PhaseState, PurchaseState};
use crate::power::Power;
use crate::territory::{SeaZoneState, TerritoryState};

/// Per-power mutable state.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PowerState {
    pub power: Power,
    pub ipcs: u32,
    pub ipcs_europe: u32,
    pub ipcs_pacific: u32,
    pub at_war: bool,
    pub capital_captured: bool,
    pub researched_techs: Vec<String>,
}

impl PowerState {
    pub fn new(power: Power, starting_ipcs: u32) -> Self {
        PowerState {
            power,
            ipcs: starting_ipcs,
            ipcs_europe: 0,
            ipcs_pacific: 0,
            at_war: matches!(
                power,
                Power::Germany | Power::Japan | Power::Italy | Power::UnitedKingdom | Power::France
            ),
            capital_captured: false,
            researched_techs: Vec::new(),
        }
    }
}

/// Political state tracking wars, triggers, and neutrals.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PoliticalState {
    /// war_matrix[a][b] == true means power a is at war with power b
    pub war_matrix: [[bool; 9]; 9],
    pub triggers: PoliticalTriggers,
}

impl PoliticalState {
    pub fn new() -> Self {
        let mut state = PoliticalState {
            war_matrix: [[false; 9]; 9],
            triggers: PoliticalTriggers::new(),
        };

        // Initial wars at game start: Axis vs UK, France; Germany vs Soviet Union is NOT at war yet
        // Europe: Germany + Italy at war with UK, France
        // Pacific: Japan at war with UK, ANZAC, China
        let axis_europe = [Power::Germany as usize, Power::Italy as usize];
        let allies_europe = [Power::UnitedKingdom as usize, Power::France as usize];

        for &a in &axis_europe {
            for &b in &allies_europe {
                state.war_matrix[a][b] = true;
                state.war_matrix[b][a] = true;
            }
        }

        let japan = Power::Japan as usize;
        let pacific_allies = [
            Power::UnitedKingdom as usize,
            Power::China as usize,
            Power::ANZAC as usize,
        ];
        for &b in &pacific_allies {
            state.war_matrix[japan][b] = true;
            state.war_matrix[b][japan] = true;
        }

        state
    }

    /// Check if two powers are at war.
    pub fn are_at_war(&self, a: Power, b: Power) -> bool {
        self.war_matrix[a as usize][b as usize]
    }

    /// Check if two powers are on the same team or not at war.
    pub fn are_friendly(&self, a: Power, b: Power) -> bool {
        a == b || (a.team() == b.team() && !self.are_at_war(a, b))
    }
}

impl Default for PoliticalState {
    fn default() -> Self {
        Self::new()
    }
}

/// Political triggers that affect gameplay.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PoliticalTriggers {
    pub us_at_war: bool,
    pub us_war_turn: Option<u32>,
    pub soviet_at_war_with_axis: bool,
    pub japan_attacked_uk_anzac: bool,
    pub london_captured: bool,
    pub paris_captured: bool,
}

impl PoliticalTriggers {
    pub fn new() -> Self {
        PoliticalTriggers {
            us_at_war: false,
            us_war_turn: None,
            soviet_at_war_with_axis: false,
            japan_attacked_uk_anzac: true, // Japan starts at war with UK/ANZAC in Pacific
            london_captured: false,
            paris_captured: true, // Paris starts captured by Germany
        }
    }
}

impl Default for PoliticalTriggers {
    fn default() -> Self {
        Self::new()
    }
}

/// The complete game state. This is the single source of truth.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GameState {
    pub turn_number: u32,
    pub current_power: Power,
    pub current_phase: Phase,
    pub phase_state: PhaseState,

    pub territories: Vec<TerritoryState>,
    pub sea_zones: Vec<SeaZoneState>,
    pub powers: Vec<PowerState>,

    pub political: PoliticalState,

    #[ts(skip)]
    pub action_log: Vec<AppliedAction>,
    #[ts(skip)]
    pub undo_checkpoints: Vec<usize>,

    /// Units purchased this turn, to be placed during Mobilize phase.
    pub pending_purchases: Vec<(crate::unit::UnitType, u32)>,

    pub rng_seed: u64,
    pub rng_counter: u64,
}

impl GameState {
    /// Create a minimal initial state (actual setup done in setup.rs).
    pub fn new(seed: u64) -> Self {
        let powers = vec![
            PowerState::new(Power::Germany, 30),
            PowerState::new(Power::SovietUnion, 37),
            PowerState::new(Power::Japan, 26),
            PowerState::new(Power::UnitedStates, 52),
            PowerState::new(Power::China, 12),
            PowerState::new(Power::UnitedKingdom, 28),
            PowerState::new(Power::Italy, 10),
            PowerState::new(Power::ANZAC, 10),
            PowerState::new(Power::France, 0), // France starts with Paris captured
        ];

        GameState {
            turn_number: 1,
            current_power: Power::Germany,
            current_phase: Phase::PurchaseAndRepair,
            phase_state: PhaseState::Purchase(PurchaseState::new()),
            territories: Vec::new(), // Populated by setup
            sea_zones: Vec::new(),   // Populated by setup
            powers,
            political: PoliticalState::new(),
            pending_purchases: Vec::new(),
            action_log: Vec::new(),
            undo_checkpoints: vec![0],
            rng_seed: seed,
            rng_counter: 0,
        }
    }
}
