//! Territory and sea zone definitions and state.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::power::Power;
use crate::unit::UnitInstance;

/// Unique identifier for a land territory.
pub type TerritoryId = u16;

/// Unique identifier for a sea zone.
pub type SeaZoneId = u16;

/// A region is either a land territory or a sea zone.
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub enum RegionId {
    Land(TerritoryId),
    Sea(SeaZoneId),
}

/// The type of a territory (affects political rules).
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub enum TerritoryType {
    /// Normal controlled territory
    Normal,
    /// Starts friendly to Axis (joins when an Axis power enters)
    ProAxis,
    /// Starts friendly to Allies (joins when an Allied power enters)
    ProAllies,
    /// True neutral - attacking one turns ALL true neutrals pro-enemy
    TrueNeutral,
    /// Impassable territory (Sahara, Himalayas)
    Impassable,
}

/// Identifier for a strait/canal.
pub type StraitId = u8;

/// Static definition of a land territory (never changes during a game).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerritoryDef {
    pub id: TerritoryId,
    pub name: String,
    pub ipc_value: u32,
    pub is_capital: Option<Power>,
    pub is_victory_city: bool,
    pub original_owner: Option<Power>,
    pub territory_type: TerritoryType,
    pub adjacent_land: Vec<TerritoryId>,
    pub adjacent_sea: Vec<SeaZoneId>,
    pub strait_connections: Vec<(TerritoryId, StraitId)>,
    pub convoys_from: Vec<SeaZoneId>,
    pub is_island: bool,
}

/// Static definition of a sea zone (never changes during a game).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SeaZoneDef {
    pub id: SeaZoneId,
    pub name: String,
    pub adjacent_sea: Vec<SeaZoneId>,
    pub adjacent_land: Vec<TerritoryId>,
    pub is_convoy_zone: bool,
}

/// Facility types that can exist in land territories.
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub enum FacilityType {
    MinorIndustrialComplex,
    MajorIndustrialComplex,
    AirBase,
    NavalBase,
}

/// A facility in a territory.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Facility {
    pub facility_type: FacilityType,
    pub damage: u32,
    pub max_damage: u32,
    pub operational: bool,
}

impl Facility {
    pub fn new(facility_type: FacilityType, territory_ipc: u32) -> Self {
        let max_damage = match facility_type {
            FacilityType::MinorIndustrialComplex | FacilityType::MajorIndustrialComplex => {
                territory_ipc * 2
            }
            FacilityType::AirBase | FacilityType::NavalBase => 6,
        };
        Facility {
            facility_type,
            damage: 0,
            max_damage,
            operational: true,
        }
    }

    /// Maximum production capacity for industrial complexes.
    pub fn production_capacity(&self, territory_ipc: u32) -> u32 {
        if self.damage > 0 {
            // Can't produce more than (max - damage)
            let base = match self.facility_type {
                FacilityType::MajorIndustrialComplex => territory_ipc.min(10),
                FacilityType::MinorIndustrialComplex => territory_ipc.min(3),
                _ => 0,
            };
            base.saturating_sub(self.damage)
        } else {
            match self.facility_type {
                FacilityType::MajorIndustrialComplex => territory_ipc.min(10),
                FacilityType::MinorIndustrialComplex => territory_ipc.min(3),
                _ => 0,
            }
        }
    }
}

/// Mutable state of a land territory during a game.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TerritoryState {
    pub owner: Option<Power>,
    pub units: Vec<UnitInstance>,
    pub facilities: Vec<Facility>,
    pub just_captured: bool,
}

impl TerritoryState {
    pub fn new(owner: Option<Power>) -> Self {
        TerritoryState {
            owner,
            units: Vec::new(),
            facilities: Vec::new(),
            just_captured: false,
        }
    }
}

/// Mutable state of a sea zone during a game.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SeaZoneState {
    pub units: Vec<UnitInstance>,
}

impl SeaZoneState {
    pub fn new() -> Self {
        SeaZoneState { units: Vec::new() }
    }
}

impl Default for SeaZoneState {
    fn default() -> Self {
        Self::new()
    }
}
