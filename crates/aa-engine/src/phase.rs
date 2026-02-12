//! Turn phase definitions and phase state machine.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::territory::{RegionId, TerritoryId};
use crate::unit::{UnitId, UnitType};

/// The six phases of each power's turn.
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub enum Phase {
    PurchaseAndRepair,
    CombatMovement,
    ConductCombat,
    NonCombatMovement,
    Mobilize,
    CollectIncome,
}

impl Phase {
    /// Returns the next phase in the sequence.
    pub fn next(&self) -> Option<Phase> {
        match self {
            Phase::PurchaseAndRepair => Some(Phase::CombatMovement),
            Phase::CombatMovement => Some(Phase::ConductCombat),
            Phase::ConductCombat => Some(Phase::NonCombatMovement),
            Phase::NonCombatMovement => Some(Phase::Mobilize),
            Phase::Mobilize => Some(Phase::CollectIncome),
            Phase::CollectIncome => None, // Turn ends
        }
    }

    /// Human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            Phase::PurchaseAndRepair => "Purchase & Repair Units",
            Phase::CombatMovement => "Combat Movement",
            Phase::ConductCombat => "Conduct Combat",
            Phase::NonCombatMovement => "Non-Combat Movement",
            Phase::Mobilize => "Mobilize New Units",
            Phase::CollectIncome => "Collect Income",
        }
    }
}

/// Per-phase sub-state. Tracks what has been done in the current phase.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum PhaseState {
    Purchase(PurchaseState),
    CombatMove(CombatMoveState),
    Combat(CombatState),
    NonCombatMove(NonCombatMoveState),
    Mobilize(MobilizeState),
    CollectIncome(CollectIncomeState),
}

/// State for the Purchase & Repair phase.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PurchaseState {
    pub purchases: Vec<(UnitType, u32)>,
    pub repairs: Vec<(TerritoryId, u32)>,
    pub ipcs_spent: u32,
}

impl PurchaseState {
    pub fn new() -> Self {
        PurchaseState {
            purchases: Vec::new(),
            repairs: Vec::new(),
            ipcs_spent: 0,
        }
    }
}

impl Default for PurchaseState {
    fn default() -> Self {
        Self::new()
    }
}

/// State for the Combat Movement phase.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CombatMoveState {
    pub moves: Vec<PlannedMove>,
}

/// A planned unit move during combat movement.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PlannedMove {
    pub unit_id: UnitId,
    pub path: Vec<RegionId>,
    pub from: RegionId,
    pub to: RegionId,
}

impl CombatMoveState {
    pub fn new() -> Self {
        CombatMoveState { moves: Vec::new() }
    }
}

impl Default for CombatMoveState {
    fn default() -> Self {
        Self::new()
    }
}

/// State for the Conduct Combat phase.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CombatState {
    pub pending_battles: Vec<RegionId>,
    pub resolved_battles: Vec<RegionId>,
    pub current_battle: Option<RegionId>,
    /// The active combat being resolved (if any).
    #[ts(skip)]
    pub active_combat: Option<crate::combat::ActiveCombat>,
}

impl CombatState {
    pub fn new() -> Self {
        CombatState {
            pending_battles: Vec::new(),
            resolved_battles: Vec::new(),
            current_battle: None,
            active_combat: None,
        }
    }
}

impl Default for CombatState {
    fn default() -> Self {
        Self::new()
    }
}

/// State for the Non-Combat Movement phase.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NonCombatMoveState {
    pub moves: Vec<PlannedMove>,
}

impl NonCombatMoveState {
    pub fn new() -> Self {
        NonCombatMoveState { moves: Vec::new() }
    }
}

impl Default for NonCombatMoveState {
    fn default() -> Self {
        Self::new()
    }
}

/// State for the Mobilize phase.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MobilizeState {
    pub placements: Vec<(UnitType, TerritoryId)>,
    pub units_to_place: Vec<(UnitType, u32)>,
}

impl MobilizeState {
    pub fn new() -> Self {
        MobilizeState {
            placements: Vec::new(),
            units_to_place: Vec::new(),
        }
    }
}

impl Default for MobilizeState {
    fn default() -> Self {
        Self::new()
    }
}

/// State for the Collect Income phase.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CollectIncomeState {
    pub base_income: u32,
    pub objective_bonus: u32,
    pub convoy_losses: u32,
    pub total_collected: u32,
}

impl CollectIncomeState {
    pub fn new() -> Self {
        CollectIncomeState {
            base_income: 0,
            objective_bonus: 0,
            convoy_losses: 0,
            total_collected: 0,
        }
    }
}

impl Default for CollectIncomeState {
    fn default() -> Self {
        Self::new()
    }
}
