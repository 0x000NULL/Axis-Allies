//! Unit type definitions, stats, and unit instances.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::power::Power;

/// Unique identifier for a unit instance on the board.
pub type UnitId = u32;

/// All unit types in Global 1940 2nd Edition.
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub enum UnitType {
    Infantry,
    MechInfantry,
    Artillery,
    Tank,
    AAA,
    Fighter,
    TacticalBomber,
    StrategicBomber,
    Transport,
    Submarine,
    Destroyer,
    Cruiser,
    Carrier,
    Battleship,
}

/// The domain a unit operates in.
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub enum UnitDomain {
    Land,
    Air,
    Sea,
}

/// Special abilities certain units possess.
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub enum SpecialAbility {
    /// Infantry paired with artillery attacks at 2
    ArtillerySupport,
    /// Mech infantry paired with tank can blitz
    BlitzWithTank,
    /// Tanks can blitz through empty enemy territory
    Blitz,
    /// AA guns fire preemptively at air units (up to 3)
    AntiAir,
    /// Can intercept strategic bombers
    Intercept,
    /// Can escort strategic bombers
    Escort,
    /// Can perform strategic bombing raids
    StrategicBomb,
    /// Can perform tactical bombing (air/naval bases)
    TacticalBomb,
    /// Tactical bombers paired with tanks/fighters attack at 4
    TacticalBoost,
    /// Transports cannot fire and are taken last as casualties
    DefenselessTransport,
    /// Submarines can submerge, surprise strike, convoy disrupt
    SubmarineAbilities,
    /// Destroyers cancel submarine special abilities
    AntiSubmarine,
    /// Shore bombardment capability
    ShoreBombard,
    /// Can carry up to 2 fighter/tactical bombers
    CarryAir,
    /// Two hit points (Battleship, Carrier)
    TwoHitPoints,
}

/// Static stats for each unit type (never changes during game).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitStats {
    pub unit_type: UnitType,
    pub cost: u32,
    pub attack: u8,
    pub defense: u8,
    pub movement: u8,
    pub domain: UnitDomain,
    pub hit_points: u8,
    pub can_bombard: bool,
    pub bombardment_value: u8,
    pub transport_capacity: u8,
    pub can_carry_air: u8,
    pub special_abilities: Vec<SpecialAbility>,
}

/// A specific unit on the board.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UnitInstance {
    pub id: UnitId,
    pub unit_type: UnitType,
    pub owner: Power,
    pub hits_taken: u8,
    pub moved_this_turn: bool,
    pub movement_remaining: u8,
    pub cargo: Vec<UnitId>,
}

impl UnitInstance {
    /// Create a new, healthy unit with full movement.
    pub fn new(id: UnitId, unit_type: UnitType, owner: Power) -> Self {
        let stats = get_unit_stats(unit_type);
        UnitInstance {
            id,
            unit_type,
            owner,
            hits_taken: 0,
            moved_this_turn: false,
            movement_remaining: stats.movement,
            cargo: Vec::new(),
        }
    }

    /// Whether this unit is damaged (has taken at least 1 hit but not destroyed).
    pub fn is_damaged(&self) -> bool {
        self.hits_taken > 0
    }
}

/// Get the static stats for a unit type.
pub fn get_unit_stats(unit_type: UnitType) -> UnitStats {
    match unit_type {
        UnitType::Infantry => UnitStats {
            unit_type,
            cost: 3,
            attack: 1,
            defense: 2,
            movement: 1,
            domain: UnitDomain::Land,
            hit_points: 1,
            can_bombard: false,
            bombardment_value: 0,
            transport_capacity: 0,
            can_carry_air: 0,
            special_abilities: vec![],
        },
        UnitType::MechInfantry => UnitStats {
            unit_type,
            cost: 4,
            attack: 1,
            defense: 2,
            movement: 2,
            domain: UnitDomain::Land,
            hit_points: 1,
            can_bombard: false,
            bombardment_value: 0,
            transport_capacity: 0,
            can_carry_air: 0,
            special_abilities: vec![SpecialAbility::BlitzWithTank],
        },
        UnitType::Artillery => UnitStats {
            unit_type,
            cost: 4,
            attack: 2,
            defense: 2,
            movement: 1,
            domain: UnitDomain::Land,
            hit_points: 1,
            can_bombard: false,
            bombardment_value: 0,
            transport_capacity: 0,
            can_carry_air: 0,
            special_abilities: vec![SpecialAbility::ArtillerySupport],
        },
        UnitType::Tank => UnitStats {
            unit_type,
            cost: 6,
            attack: 3,
            defense: 3,
            movement: 2,
            domain: UnitDomain::Land,
            hit_points: 1,
            can_bombard: false,
            bombardment_value: 0,
            transport_capacity: 0,
            can_carry_air: 0,
            special_abilities: vec![SpecialAbility::Blitz],
        },
        UnitType::AAA => UnitStats {
            unit_type,
            cost: 5,
            attack: 0,
            defense: 0,
            movement: 1,
            domain: UnitDomain::Land,
            hit_points: 1,
            can_bombard: false,
            bombardment_value: 0,
            transport_capacity: 0,
            can_carry_air: 0,
            special_abilities: vec![SpecialAbility::AntiAir],
        },
        UnitType::Fighter => UnitStats {
            unit_type,
            cost: 10,
            attack: 3,
            defense: 4,
            movement: 4,
            domain: UnitDomain::Air,
            hit_points: 1,
            can_bombard: false,
            bombardment_value: 0,
            transport_capacity: 0,
            can_carry_air: 0,
            special_abilities: vec![SpecialAbility::Intercept, SpecialAbility::Escort],
        },
        UnitType::TacticalBomber => UnitStats {
            unit_type,
            cost: 11,
            attack: 3,
            defense: 3,
            movement: 4,
            domain: UnitDomain::Air,
            hit_points: 1,
            can_bombard: false,
            bombardment_value: 0,
            transport_capacity: 0,
            can_carry_air: 0,
            special_abilities: vec![
                SpecialAbility::TacticalBoost,
                SpecialAbility::TacticalBomb,
            ],
        },
        UnitType::StrategicBomber => UnitStats {
            unit_type,
            cost: 12,
            attack: 4,
            defense: 1,
            movement: 6,
            domain: UnitDomain::Air,
            hit_points: 1,
            can_bombard: false,
            bombardment_value: 0,
            transport_capacity: 0,
            can_carry_air: 0,
            special_abilities: vec![SpecialAbility::StrategicBomb],
        },
        UnitType::Transport => UnitStats {
            unit_type,
            cost: 7,
            attack: 0,
            defense: 0,
            movement: 2,
            domain: UnitDomain::Sea,
            hit_points: 1,
            can_bombard: false,
            bombardment_value: 0,
            transport_capacity: 2,
            can_carry_air: 0,
            special_abilities: vec![SpecialAbility::DefenselessTransport],
        },
        UnitType::Submarine => UnitStats {
            unit_type,
            cost: 6,
            attack: 2,
            defense: 1,
            movement: 2,
            domain: UnitDomain::Sea,
            hit_points: 1,
            can_bombard: false,
            bombardment_value: 0,
            transport_capacity: 0,
            can_carry_air: 0,
            special_abilities: vec![SpecialAbility::SubmarineAbilities],
        },
        UnitType::Destroyer => UnitStats {
            unit_type,
            cost: 8,
            attack: 2,
            defense: 2,
            movement: 2,
            domain: UnitDomain::Sea,
            hit_points: 1,
            can_bombard: false,
            bombardment_value: 0,
            transport_capacity: 0,
            can_carry_air: 0,
            special_abilities: vec![SpecialAbility::AntiSubmarine],
        },
        UnitType::Cruiser => UnitStats {
            unit_type,
            cost: 12,
            attack: 3,
            defense: 3,
            movement: 2,
            domain: UnitDomain::Sea,
            hit_points: 1,
            can_bombard: true,
            bombardment_value: 3,
            transport_capacity: 0,
            can_carry_air: 0,
            special_abilities: vec![SpecialAbility::ShoreBombard],
        },
        UnitType::Carrier => UnitStats {
            unit_type,
            cost: 16,
            attack: 0,
            defense: 2,
            movement: 2,
            domain: UnitDomain::Sea,
            hit_points: 2,
            can_bombard: false,
            bombardment_value: 0,
            transport_capacity: 0,
            can_carry_air: 2,
            special_abilities: vec![SpecialAbility::CarryAir, SpecialAbility::TwoHitPoints],
        },
        UnitType::Battleship => UnitStats {
            unit_type,
            cost: 20,
            attack: 4,
            defense: 4,
            movement: 2,
            domain: UnitDomain::Sea,
            hit_points: 2,
            can_bombard: true,
            bombardment_value: 4,
            transport_capacity: 0,
            can_carry_air: 0,
            special_abilities: vec![SpecialAbility::ShoreBombard, SpecialAbility::TwoHitPoints],
        },
    }
}

impl UnitType {
    /// Returns all unit types as a slice.
    pub fn all() -> &'static [UnitType] {
        &[
            UnitType::Infantry,
            UnitType::MechInfantry,
            UnitType::Artillery,
            UnitType::Tank,
            UnitType::AAA,
            UnitType::Fighter,
            UnitType::TacticalBomber,
            UnitType::StrategicBomber,
            UnitType::Transport,
            UnitType::Submarine,
            UnitType::Destroyer,
            UnitType::Cruiser,
            UnitType::Carrier,
            UnitType::Battleship,
        ]
    }
}
