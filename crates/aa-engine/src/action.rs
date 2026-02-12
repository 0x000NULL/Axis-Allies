//! Action types, results, and the undo system.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::phase::Phase;
use crate::power::Power;
use crate::territory::{RegionId, SeaZoneId, TerritoryId};
use crate::unit::{UnitId, UnitType};

/// All possible player actions. Every interaction with the engine is an Action.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Action {
    // -- Purchase Phase --
    PurchaseUnit {
        unit_type: UnitType,
        count: u32,
    },
    RemovePurchase {
        unit_type: UnitType,
        count: u32,
    },
    RepairFacility {
        territory_id: TerritoryId,
        damage_to_repair: u32,
    },
    ConfirmPurchases,

    // -- Combat Movement Phase --
    MoveUnit {
        unit_id: UnitId,
        path: Vec<RegionId>,
    },
    UndoMove {
        unit_id: UnitId,
    },
    ConfirmCombatMovement,

    // -- Combat Phase --
    SelectBattle {
        location: RegionId,
    },
    RollAttack,
    RollDefense,
    SelectCasualties {
        casualties: Vec<UnitId>,
    },
    AttackerRetreat {
        to: RegionId,
    },
    SubmergeSubmarine {
        unit_id: UnitId,
    },
    ContinueCombatRound,

    // -- Non-Combat Movement Phase --
    MoveUnitNonCombat {
        unit_id: UnitId,
        path: Vec<RegionId>,
    },
    LandAirUnit {
        unit_id: UnitId,
        territory_id: RegionId,
    },
    ConfirmNonCombatMovement,

    // -- Mobilize Phase --
    PlaceUnit {
        unit_type: UnitType,
        territory_id: TerritoryId,
    },
    ConfirmMobilization,

    // -- Collect Income --
    ConfirmIncome,

    // -- Political --
    DeclareWar {
        against: Power,
    },

    // -- Meta --
    Undo,
    ConfirmPhase,
}

/// The result of successfully applying an action.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ActionResult {
    #[ts(skip)]
    pub applied: AppliedAction,
    pub events: Vec<GameEvent>,
}

/// A record of an applied action (stored for undo).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppliedAction {
    pub action: Action,
    pub inverse: InverseAction,
}

/// How to reverse an applied action.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InverseAction {
    /// Simple reverse (e.g., remove a purchase = re-add it)
    Simple(Action),
    /// Restore a partial state snapshot
    RestoreSnapshot(Vec<u8>),
    /// This action cannot be undone (e.g., dice rolls)
    Irreversible,
}

/// Narrative events for the event log and UI feedback.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum GameEvent {
    PhaseChanged {
        from: Phase,
        to: Phase,
    },
    TurnChanged {
        power: Power,
        turn: u32,
    },
    WarDeclared {
        aggressor: Power,
        target: Power,
    },
    BattleStarted {
        location: RegionId,
    },
    BattleEnded {
        location: RegionId,
        attacker_won: bool,
    },
    CapitalCaptured {
        territory_id: TerritoryId,
        by: Power,
    },
    TerritoryLiberated {
        territory_id: TerritoryId,
        to: Power,
    },
    ConvoyDisrupted {
        zone: SeaZoneId,
        power: Power,
        lost_ipcs: u32,
    },
    VictoryAchieved {
        winner: crate::power::Team,
    },
    UnitsPurchased {
        unit_type: UnitType,
        count: u32,
        cost: u32,
    },
    UnitsPlaced {
        unit_type: UnitType,
        territory_id: TerritoryId,
    },
    IncomeCollected {
        power: Power,
        amount: u32,
    },
}

/// A legal action with a human-readable description.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LegalAction {
    pub action: Action,
    pub description: String,
}
