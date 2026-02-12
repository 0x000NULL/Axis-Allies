//! Initial game setup: creates the starting game state for Global 1940 2nd Edition.
//!
//! Contains the full Order of Battle (OOB) with all starting units, facilities,
//! territory ownership, political state, and IPC treasuries for all 9 powers.

use crate::data::sea_zone_ids as sz;
use crate::data::territory_ids as t;
use crate::data::GameMap;
use crate::power::Power;
use crate::state::GameState;
use crate::territory::{Facility, FacilityType, SeaZoneState, TerritoryState};
use crate::unit::{UnitId, UnitInstance, UnitType};

use Power::{
    China as Ch, France as Fr, Germany as Ge, Italy as It, Japan as Jp, SovietUnion as Su,
    UnitedKingdom as Uk, UnitedStates as Us, ANZAC as An,
};

use UnitType::*;

// ---------------------------------------------------------------------------
// Unit ID generator
// ---------------------------------------------------------------------------

struct IdGen(UnitId);

impl IdGen {
    fn new() -> Self {
        Self(0)
    }
    fn next(&mut self) -> UnitId {
        let id = self.0;
        self.0 += 1;
        id
    }
}

// ---------------------------------------------------------------------------
// Helper: place N units of a type into a territory
// ---------------------------------------------------------------------------

fn place_land(
    state: &mut GameState,
    gen: &mut IdGen,
    territory: u16,
    owner: Power,
    unit_type: UnitType,
    count: u32,
) {
    for _ in 0..count {
        state.territories[territory as usize]
            .units
            .push(UnitInstance::new(gen.next(), unit_type, owner));
    }
}

fn place_sea(
    state: &mut GameState,
    gen: &mut IdGen,
    sea_zone: u16,
    owner: Power,
    unit_type: UnitType,
    count: u32,
) {
    for _ in 0..count {
        state.sea_zones[sea_zone as usize]
            .units
            .push(UnitInstance::new(gen.next(), unit_type, owner));
    }
}

fn add_facility(
    state: &mut GameState,
    map: &GameMap,
    territory: u16,
    facility_type: FacilityType,
) {
    let ipc = map.territory(territory).ipc_value;
    state.territories[territory as usize]
        .facilities
        .push(Facility::new(facility_type, ipc));
}

// ---------------------------------------------------------------------------
// Macro for concise unit placement
// ---------------------------------------------------------------------------

/// Place multiple unit types in a single territory.
/// Usage: land!(state, gen, TERRITORY, Owner; 3 Infantry, 1 Artillery, 2 Tank);
macro_rules! land {
    ($state:expr, $gen:expr, $terr:expr, $owner:expr; $( $count:literal $utype:ident ),+ $(,)?) => {
        $( place_land($state, $gen, $terr, $owner, $utype, $count); )+
    };
}

macro_rules! sea {
    ($state:expr, $gen:expr, $sz:expr, $owner:expr; $( $count:literal $utype:ident ),+ $(,)?) => {
        $( place_sea($state, $gen, $sz, $owner, $utype, $count); )+
    };
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Create the initial game state for a new Global 1940 2nd Edition game.
pub fn create_initial_state(seed: u64, map: &GameMap) -> GameState {
    let mut state = GameState::new(seed);
    let mut gen = IdGen::new();

    // --- Create territory states from map definitions ---
    for def in &map.territories {
        state.territories.push(TerritoryState::new(def.original_owner));
    }
    for _def in &map.sea_zones {
        state.sea_zones.push(SeaZoneState::new());
    }

    // --- Fix starting IPCs ---
    // France starts with IPCs (Paris not yet captured at game start)
    state.powers[Fr as usize].ipcs = 19;

    // --- Fix political triggers ---
    // Paris is NOT captured at game start (Germany attacks Turn 1)
    state.political.triggers.paris_captured = false;
    // France is at war (with Germany/Italy) but Paris is intact
    state.powers[Fr as usize].at_war = true;
    // China at war
    state.powers[Ch as usize].at_war = true;
    // ANZAC at war (with Japan)
    state.powers[An as usize].at_war = true;
    // US and Soviet Union are NOT at war
    state.powers[Us as usize].at_war = false;
    state.powers[Su as usize].at_war = false;

    // --- Place all facilities ---
    place_facilities(&mut state, map);

    // --- Place all starting units ---
    place_germany(&mut state, &mut gen);
    place_soviet_union(&mut state, &mut gen);
    place_japan(&mut state, &mut gen);
    place_united_states(&mut state, &mut gen);
    place_china(&mut state, &mut gen);
    place_united_kingdom(&mut state, &mut gen);
    place_italy(&mut state, &mut gen);
    place_anzac(&mut state, &mut gen);
    place_france(&mut state, &mut gen);

    state
}

// ---------------------------------------------------------------------------
// Facilities
// ---------------------------------------------------------------------------

fn place_facilities(state: &mut GameState, map: &GameMap) {
    use FacilityType::*;

    // === Major Industrial Complexes ===
    add_facility(state, map, t::GERMANY, MajorIndustrialComplex);
    add_facility(state, map, t::RUSSIA, MajorIndustrialComplex);
    add_facility(state, map, t::UNITED_KINGDOM, MajorIndustrialComplex);
    add_facility(state, map, t::JAPAN, MajorIndustrialComplex);
    add_facility(state, map, t::EASTERN_UNITED_STATES, MajorIndustrialComplex);
    add_facility(state, map, t::WESTERN_UNITED_STATES, MajorIndustrialComplex);
    add_facility(state, map, t::NORTHERN_ITALY, MajorIndustrialComplex);

    // === Minor Industrial Complexes ===
    add_facility(state, map, t::FRANCE, MinorIndustrialComplex);
    add_facility(state, map, t::INDIA, MinorIndustrialComplex);
    add_facility(state, map, t::NEW_SOUTH_WALES, MinorIndustrialComplex);
    add_facility(state, map, t::SOUTH_AFRICA, MinorIndustrialComplex);

    // === Air Bases ===
    add_facility(state, map, t::GERMANY, AirBase);
    add_facility(state, map, t::WESTERN_GERMANY, AirBase);
    add_facility(state, map, t::UNITED_KINGDOM, AirBase);
    add_facility(state, map, t::SCOTLAND, AirBase);
    add_facility(state, map, t::JAPAN, AirBase);
    add_facility(state, map, t::NORTHERN_ITALY, AirBase);
    add_facility(state, map, t::EASTERN_UNITED_STATES, AirBase);
    add_facility(state, map, t::WESTERN_UNITED_STATES, AirBase);
    add_facility(state, map, t::INDIA, AirBase);
    add_facility(state, map, t::GIBRALTAR, AirBase);
    add_facility(state, map, t::MALTA, AirBase);
    add_facility(state, map, t::EGYPT, AirBase);
    add_facility(state, map, t::HAWAIIAN_ISLANDS, AirBase);
    add_facility(state, map, t::FORMOSA, AirBase);
    add_facility(state, map, t::KOREA, AirBase);
    add_facility(state, map, t::NORWAY, AirBase);
    add_facility(state, map, t::ROMANIA, AirBase);
    add_facility(state, map, t::NEW_SOUTH_WALES, AirBase);

    // === Naval Bases ===
    add_facility(state, map, t::GERMANY, NavalBase);
    add_facility(state, map, t::WESTERN_GERMANY, NavalBase);
    add_facility(state, map, t::UNITED_KINGDOM, NavalBase);
    add_facility(state, map, t::JAPAN, NavalBase);
    add_facility(state, map, t::NORTHERN_ITALY, NavalBase);
    add_facility(state, map, t::SOUTHERN_ITALY, NavalBase);
    add_facility(state, map, t::EASTERN_UNITED_STATES, NavalBase);
    add_facility(state, map, t::WESTERN_UNITED_STATES, NavalBase);
    add_facility(state, map, t::INDIA, NavalBase);
    add_facility(state, map, t::PHILIPPINES, NavalBase);
    add_facility(state, map, t::HAWAIIAN_ISLANDS, NavalBase);
    add_facility(state, map, t::GIBRALTAR, NavalBase);
    add_facility(state, map, t::SOUTH_AFRICA, NavalBase);
    add_facility(state, map, t::NEW_SOUTH_WALES, NavalBase);
    add_facility(state, map, t::MALAYA, NavalBase);
    add_facility(state, map, t::NORWAY, NavalBase);
    add_facility(state, map, t::KOREA, NavalBase);
    add_facility(state, map, t::FRANCE, NavalBase);
}

// ===========================================================================
// GERMANY
// ===========================================================================

fn place_germany(state: &mut GameState, gen: &mut IdGen) {
    // --- Land forces ---
    land!(state, gen, t::GERMANY, Ge;
        6 Infantry, 1 Artillery, 1 MechInfantry, 2 Tank, 1 Fighter,
        1 TacticalBomber, 1 AAA
    );
    land!(state, gen, t::WESTERN_GERMANY, Ge;
        4 Infantry, 1 Artillery, 1 MechInfantry, 1 Tank, 1 Fighter,
        1 StrategicBomber
    );
    land!(state, gen, t::GREATER_SOUTHERN_GERMANY, Ge;
        2 Infantry, 1 Artillery, 1 MechInfantry, 1 Tank, 1 AAA
    );
    land!(state, gen, t::HOLLAND_BELGIUM, Ge;
        3 Infantry, 1 Artillery, 1 Fighter
    );
    land!(state, gen, t::DENMARK, Ge;
        1 Infantry
    );
    land!(state, gen, t::NORWAY, Ge;
        3 Infantry, 1 Fighter, 1 AAA
    );
    land!(state, gen, t::FINLAND, Ge;
        3 Infantry, 1 Artillery
    );
    land!(state, gen, t::POLAND, Ge;
        2 Infantry, 1 Artillery, 1 Tank
    );
    land!(state, gen, t::SLOVAKIA_HUNGARY, Ge;
        2 Infantry, 1 Artillery, 1 Tank
    );
    land!(state, gen, t::ROMANIA, Ge;
        3 Infantry, 1 Artillery, 1 Tank, 1 TacticalBomber
    );
    land!(state, gen, t::YUGOSLAVIA, Ge;
        2 Infantry
    );
    land!(state, gen, t::BULGARIA, Ge;
        2 Infantry
    );
    land!(state, gen, t::CRETE, Ge;
        1 Infantry
    );
    land!(state, gen, t::BALTIC_STATES, Ge;
        1 Infantry
    );
    land!(state, gen, t::SOUTHERN_FRANCE, Ge;
        1 Infantry
    );

    // Wait — Southern France is original_owner = France. Germany shouldn't
    // have units there unless Germany controls it. In the actual game,
    // Southern France is French at start. Let me remove the German infantry
    // from S. France and instead this territory is French with French units.
    // Actually the infantry was already placed. Let me clear it.
    state.territories[t::SOUTHERN_FRANCE as usize].units.clear();

    // --- Naval forces ---
    // Baltic Sea (Board SZ 108)
    sea!(state, gen, sz::SZ_BALTIC_SEA, Ge;
        1 Battleship, 1 Cruiser, 2 Transport
    );
    // North Atlantic / Off Greenland (Board SZ 103) — submarine wolfpacks
    sea!(state, gen, sz::SZ_OFF_GREENLAND, Ge;
        2 Submarine
    );
    // Bay of Biscay (Board SZ 110)
    sea!(state, gen, sz::SZ_BAY_OF_BISCAY, Ge;
        1 Submarine
    );
    // Off Morocco / Central Atlantic (Board SZ 112)
    sea!(state, gen, sz::SZ_OFF_MOROCCO, Ge;
        1 Submarine
    );
    // Central Atlantic (Board SZ 90)
    sea!(state, gen, sz::SZ_CENTRAL_ATLANTIC, Ge;
        1 Submarine
    );
    // Western Med (Board SZ 113)
    sea!(state, gen, sz::SZ_WESTERN_MED, Ge;
        1 Submarine
    );
    // Off Egypt (Board SZ 117)
    sea!(state, gen, sz::SZ_OFF_EGYPT, Ge;
        1 Submarine
    );
}

// ===========================================================================
// SOVIET UNION
// ===========================================================================

fn place_soviet_union(state: &mut GameState, gen: &mut IdGen) {
    // --- Land forces ---
    land!(state, gen, t::NOVGOROD, Su;
        3 Infantry, 1 Artillery, 1 MechInfantry, 1 Tank, 1 Fighter, 1 AAA
    );
    land!(state, gen, t::RUSSIA, Su;
        4 Infantry, 1 Artillery, 1 MechInfantry, 2 Tank, 1 Fighter,
        1 TacticalBomber, 1 AAA
    );
    land!(state, gen, t::VYBORG, Su;
        2 Infantry
    );
    land!(state, gen, t::ARCHANGEL, Su;
        1 Infantry
    );
    land!(state, gen, t::BELARUS, Su;
        2 Infantry, 1 Artillery
    );
    land!(state, gen, t::WESTERN_UKRAINE, Su;
        2 Infantry, 1 Artillery, 1 Tank
    );
    land!(state, gen, t::UKRAINE, Su;
        3 Infantry, 1 Artillery, 1 Tank
    );
    land!(state, gen, t::ROSTOV, Su;
        1 Infantry
    );
    land!(state, gen, t::VOLGOGRAD, Su;
        2 Infantry, 1 AAA
    );
    land!(state, gen, t::CAUCASUS, Su;
        2 Infantry, 1 Artillery
    );
    land!(state, gen, t::TAMBOV, Su;
        1 Infantry
    );
    land!(state, gen, t::VOLOGDA, Su;
        1 Infantry
    );
    land!(state, gen, t::URALS, Su;
        1 Infantry
    );
    land!(state, gen, t::KAZAKHSTAN, Su;
        1 Infantry
    );
    land!(state, gen, t::NOVOSIBIRSK, Su;
        1 Infantry
    );
    land!(state, gen, t::AMUR, Su;
        2 Infantry
    );
    land!(state, gen, t::SOVIET_FAR_EAST, Su;
        2 Infantry
    );
    land!(state, gen, t::BURYATIA, Su;
        1 Infantry
    );
    land!(state, gen, t::SAKHA, Su;
        1 Infantry
    );
    land!(state, gen, t::NENETSIA, Su;
        1 Infantry
    );

    // --- Naval forces (minimal) ---
    // Sea of Japan (Board SZ 5) — Soviet Pacific sub
    sea!(state, gen, sz::SZ_SEA_OF_JAPAN, Su;
        1 Submarine
    );
}

// ===========================================================================
// JAPAN
// ===========================================================================

fn place_japan(state: &mut GameState, gen: &mut IdGen) {
    // --- Land forces ---
    land!(state, gen, t::JAPAN, Jp;
        4 Infantry, 1 Artillery, 2 Fighter, 1 TacticalBomber,
        1 StrategicBomber, 3 AAA
    );
    land!(state, gen, t::MANCHURIA, Jp;
        6 Infantry, 1 Artillery, 1 MechInfantry, 2 Tank, 1 Fighter
    );
    land!(state, gen, t::KOREA, Jp;
        3 Infantry, 1 Fighter
    );
    land!(state, gen, t::FORMOSA, Jp;
        1 Infantry, 1 Fighter
    );
    land!(state, gen, t::OKINAWA, Jp;
        1 Infantry
    );
    land!(state, gen, t::IWO_JIMA, Jp;
        1 Infantry
    );
    land!(state, gen, t::CAROLINE_ISLANDS, Jp;
        1 Infantry
    );
    land!(state, gen, t::MARSHALL_ISLANDS, Jp;
        1 Infantry
    );
    land!(state, gen, t::PALAU_ISLAND, Jp;
        1 Infantry
    );
    land!(state, gen, t::WAKE_ISLAND, Jp;
        1 Infantry
    );
    land!(state, gen, t::SIAM, Jp;
        2 Infantry
    );
    land!(state, gen, t::FRENCH_INDOCHINA, Jp;
        2 Infantry, 1 Artillery, 1 Fighter
    );

    // --- Naval forces ---
    // Off Japan East (Board SZ 6) — Main fleet
    sea!(state, gen, sz::SZ_JAPAN_EAST, Jp;
        2 Battleship, 2 Carrier, 1 Cruiser, 2 Destroyer, 2 Transport,
        4 Fighter
    );
    // Sea of Japan (Board SZ 5)
    sea!(state, gen, sz::SZ_SEA_OF_JAPAN, Jp;
        1 Transport
    );
    // Off Carolines (Board SZ 33) — Southern fleet
    sea!(state, gen, sz::SZ_OFF_CAROLINES, Jp;
        1 Battleship, 1 Carrier, 1 Cruiser, 1 Destroyer, 1 Submarine,
        2 Fighter
    );
    // Off Formosa (Board SZ 20) — China Sea task force
    sea!(state, gen, sz::SZ_OFF_FORMOSA, Jp;
        1 Cruiser, 1 Destroyer, 2 Transport
    );
    // South China Sea (Board SZ 19)
    sea!(state, gen, sz::SZ_SOUTH_CHINA_SEA, Jp;
        1 Destroyer, 1 Transport
    );
    // Off Manchuria (Board SZ 12)
    sea!(state, gen, sz::SZ_OFF_MANCHURIA, Jp;
        1 Submarine
    );
}

// ===========================================================================
// UNITED STATES
// ===========================================================================

fn place_united_states(state: &mut GameState, gen: &mut IdGen) {
    // --- Land forces ---
    land!(state, gen, t::EASTERN_UNITED_STATES, Us;
        5 Infantry, 1 Artillery, 1 Tank, 1 MechInfantry, 1 Fighter,
        1 TacticalBomber, 1 StrategicBomber, 1 AAA
    );
    land!(state, gen, t::CENTRAL_UNITED_STATES, Us;
        1 Infantry
    );
    land!(state, gen, t::WESTERN_UNITED_STATES, Us;
        4 Infantry, 1 Artillery, 1 Tank, 1 Fighter, 1 TacticalBomber, 1 AAA
    );
    land!(state, gen, t::PHILIPPINES, Us;
        1 Infantry, 1 Fighter
    );
    land!(state, gen, t::HAWAIIAN_ISLANDS, Us;
        3 Infantry, 1 Fighter, 1 AAA
    );
    land!(state, gen, t::ALASKA, Us;
        1 Infantry
    );
    land!(state, gen, t::MIDWAY, Us;
        1 Infantry
    );
    land!(state, gen, t::GUAM, Us;
        1 Infantry
    );

    // --- Naval forces ---
    // Off Hawaii (Board SZ 28) — Pacific Fleet
    sea!(state, gen, sz::SZ_OFF_HAWAII, Us;
        1 Battleship, 1 Carrier, 1 Cruiser, 1 Destroyer, 1 Transport,
        2 Fighter
    );
    // Off Eastern US (Board SZ 101) — Atlantic Fleet
    sea!(state, gen, sz::SZ_OFF_EASTERN_US, Us;
        1 Battleship, 1 Carrier, 1 Cruiser, 2 Destroyer, 1 Transport,
        2 Fighter
    );
    // Off Philippines (Board SZ 35) — Asiatic Fleet
    sea!(state, gen, sz::SZ_OFF_PHILIPPINES, Us;
        1 Destroyer, 1 Submarine, 1 Transport
    );
    // Off Western US (Board SZ 11)
    sea!(state, gen, sz::SZ_OFF_WESTERN_US, Us;
        1 Destroyer, 1 Submarine
    );
}

// ===========================================================================
// CHINA
// ===========================================================================

fn place_china(state: &mut GameState, gen: &mut IdGen) {
    // China gets infantry plus one AVG Fighter (lent by US)
    land!(state, gen, t::SZECHWAN, Ch;
        4 Infantry, 1 Fighter
    );
    land!(state, gen, t::YUNNAN, Ch;
        2 Infantry
    );
    land!(state, gen, t::KWEICHOW, Ch;
        1 Infantry
    );
    land!(state, gen, t::HUNAN, Ch;
        1 Infantry
    );
    land!(state, gen, t::KIANGSI, Ch;
        1 Infantry
    );
    land!(state, gen, t::SHENSI, Ch;
        1 Infantry
    );
    land!(state, gen, t::SUIYUAN, Ch;
        1 Infantry
    );
    land!(state, gen, t::ANHWE, Ch;
        1 Infantry
    );
}

// ===========================================================================
// UNITED KINGDOM
// ===========================================================================

fn place_united_kingdom(state: &mut GameState, gen: &mut IdGen) {
    // --- Europe ---
    land!(state, gen, t::UNITED_KINGDOM, Uk;
        5 Infantry, 1 Artillery, 1 Tank, 1 Fighter, 1 TacticalBomber, 1 AAA
    );
    land!(state, gen, t::SCOTLAND, Uk;
        1 Infantry, 1 Fighter
    );
    land!(state, gen, t::EGYPT, Uk;
        2 Infantry, 1 Artillery, 1 Tank, 1 MechInfantry, 1 Fighter,
        1 TacticalBomber, 1 AAA
    );
    land!(state, gen, t::TRANS_JORDAN, Uk;
        1 Infantry
    );
    land!(state, gen, t::GIBRALTAR, Uk;
        1 Infantry
    );
    land!(state, gen, t::MALTA, Uk;
        1 Infantry, 1 Fighter
    );
    land!(state, gen, t::SOUTH_AFRICA, Uk;
        2 Infantry, 1 Artillery
    );
    land!(state, gen, t::IRAQ, Uk;
        1 Infantry
    );

    // --- Pacific ---
    land!(state, gen, t::INDIA, Uk;
        5 Infantry, 1 Artillery, 1 Tank, 1 AAA, 1 Fighter
    );
    land!(state, gen, t::BURMA, Uk;
        1 Infantry
    );
    land!(state, gen, t::WEST_INDIA, Uk;
        1 Infantry
    );
    land!(state, gen, t::MALAYA, Uk;
        2 Infantry, 1 Fighter
    );
    land!(state, gen, t::BORNEO, Uk;
        1 Infantry
    );
    land!(state, gen, t::CEYLON, Uk;
        1 Infantry
    );

    // --- Naval forces (Europe) ---
    // English Channel (Board SZ 109)
    sea!(state, gen, sz::SZ_ENGLISH_CHANNEL, Uk;
        1 Battleship, 1 Cruiser, 1 Destroyer, 1 Transport
    );
    // North Sea (Board SZ 106)
    sea!(state, gen, sz::SZ_NORTH_SEA, Uk;
        1 Destroyer, 1 Cruiser
    );
    // Off Gibraltar (Board SZ 111)
    sea!(state, gen, sz::SZ_OFF_GIBRALTAR, Uk;
        1 Battleship, 1 Destroyer
    );
    // Off Egypt (Board SZ 117)
    sea!(state, gen, sz::SZ_OFF_EGYPT, Uk;
        1 Cruiser, 1 Destroyer, 1 Transport
    );
    // North Atlantic (Board SZ 102)
    sea!(state, gen, sz::SZ_NORTH_ATLANTIC, Uk;
        1 Destroyer
    );

    // --- Naval forces (Pacific) ---
    // Off India (Board SZ 38)
    sea!(state, gen, sz::SZ_OFF_INDIA, Uk;
        1 Cruiser, 1 Destroyer, 1 Transport
    );
    // Bay of Bengal (Board SZ 37)
    sea!(state, gen, sz::SZ_BAY_OF_BENGAL, Uk;
        1 Destroyer
    );
    // Off Ceylon (Board SZ 40)
    sea!(state, gen, sz::SZ_OFF_CEYLON, Uk;
        1 Transport
    );
}

// ===========================================================================
// ITALY
// ===========================================================================

fn place_italy(state: &mut GameState, gen: &mut IdGen) {
    // --- Land forces ---
    land!(state, gen, t::NORTHERN_ITALY, It;
        4 Infantry, 1 Artillery, 1 MechInfantry, 1 Tank, 1 Fighter, 1 AAA
    );
    land!(state, gen, t::SOUTHERN_ITALY, It;
        2 Infantry, 1 Artillery
    );
    land!(state, gen, t::LIBYA, It;
        2 Infantry, 1 Artillery, 1 Tank, 1 MechInfantry
    );
    land!(state, gen, t::TOBRUK, It;
        2 Infantry, 1 Artillery, 1 Tank
    );
    land!(state, gen, t::ETHIOPIA, It;
        2 Infantry, 1 Artillery
    );
    land!(state, gen, t::ITALIAN_SOMALILAND, It;
        1 Infantry
    );
    land!(state, gen, t::ALBANIA, It;
        1 Infantry
    );

    // --- Naval forces ---
    // Tyrrhenian Sea (Board SZ 114)
    sea!(state, gen, sz::SZ_TYRRHENIAN_SEA, It;
        1 Battleship, 1 Cruiser, 2 Transport
    );
    // Off Southern Italy (Board SZ 115)
    sea!(state, gen, sz::SZ_OFF_SOUTHERN_ITALY, It;
        1 Cruiser, 1 Destroyer, 1 Submarine
    );
    // Eastern Med (Board SZ 116)
    sea!(state, gen, sz::SZ_EASTERN_MED, It;
        1 Destroyer
    );
}

// ===========================================================================
// ANZAC
// ===========================================================================

fn place_anzac(state: &mut GameState, gen: &mut IdGen) {
    land!(state, gen, t::NEW_SOUTH_WALES, An;
        2 Infantry, 1 Fighter, 1 AAA
    );
    land!(state, gen, t::QUEENSLAND, An;
        1 Infantry
    );
    land!(state, gen, t::NEW_ZEALAND, An;
        1 Infantry, 1 Fighter
    );
    land!(state, gen, t::NEW_GUINEA, An;
        1 Infantry
    );

    // --- Naval forces ---
    // Off New South Wales (Board SZ 44)
    sea!(state, gen, sz::SZ_OFF_NEW_SOUTH_WALES, An;
        1 Cruiser, 1 Destroyer, 1 Transport
    );
}

// ===========================================================================
// FRANCE
// ===========================================================================

fn place_france(state: &mut GameState, gen: &mut IdGen) {
    // France: homeland + scattered colonial garrisons
    land!(state, gen, t::FRANCE, Fr;
        4 Infantry, 1 Artillery, 1 Tank, 1 Fighter, 1 AAA
    );
    land!(state, gen, t::NORMANDY_BORDEAUX, Fr;
        1 Infantry
    );
    land!(state, gen, t::SOUTHERN_FRANCE, Fr;
        1 Infantry
    );
    land!(state, gen, t::MOROCCO, Fr;
        1 Infantry
    );
    land!(state, gen, t::ALGERIA, Fr;
        1 Infantry
    );
    land!(state, gen, t::TUNISIA, Fr;
        1 Infantry
    );
    land!(state, gen, t::SYRIA, Fr;
        1 Infantry
    );
    land!(state, gen, t::MADAGASCAR, Fr;
        1 Infantry
    );
    land!(state, gen, t::FRENCH_EQUATORIAL_AFRICA, Fr;
        1 Infantry
    );
    land!(state, gen, t::FRENCH_WEST_AFRICA, Fr;
        1 Infantry
    );

    // --- Naval forces ---
    // Western Med (Board SZ 113) — French Mediterranean fleet
    sea!(state, gen, sz::SZ_WESTERN_MED, Fr;
        1 Cruiser, 1 Destroyer
    );
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameMap;
    use crate::power::Power;
    use crate::territory::FacilityType;

    fn setup() -> (GameState, GameMap) {
        let map = GameMap::new();
        let state = create_initial_state(42, &map);
        (state, map)
    }

    #[test]
    fn test_territory_count() {
        let (state, _) = setup();
        assert_eq!(state.territories.len(), 164);
    }

    #[test]
    fn test_sea_zone_count() {
        let (state, _) = setup();
        assert_eq!(state.sea_zones.len(), 80);
    }

    #[test]
    fn test_total_unit_count_reasonable() {
        let (state, _) = setup();
        let land_units: usize = state.territories.iter().map(|t| t.units.len()).sum();
        let sea_units: usize = state.sea_zones.iter().map(|sz| sz.units.len()).sum();
        let total = land_units + sea_units;
        // Global 1940 2E has roughly 400-600 starting units
        assert!(
            total >= 300 && total <= 700,
            "Total unit count {} is out of expected range 300-700",
            total
        );
        println!("Total starting units: {} (land: {}, sea: {})", total, land_units, sea_units);
    }

    #[test]
    fn test_each_power_has_units() {
        let (state, _) = setup();
        for power in Power::all() {
            let land_count: usize = state
                .territories
                .iter()
                .flat_map(|t| &t.units)
                .filter(|u| u.owner == *power)
                .count();
            let sea_count: usize = state
                .sea_zones
                .iter()
                .flat_map(|sz| &sz.units)
                .filter(|u| u.owner == *power)
                .count();
            let total = land_count + sea_count;
            assert!(
                total > 0,
                "Power {:?} has no units at game start!",
                power
            );
            println!("{:?}: {} units (land: {}, sea: {})", power, total, land_count, sea_count);
        }
    }

    #[test]
    fn test_germany_owns_germany() {
        let (state, _) = setup();
        assert_eq!(state.territories[t::GERMANY as usize].owner, Some(Ge));
    }

    #[test]
    fn test_france_owns_france() {
        let (state, _) = setup();
        assert_eq!(state.territories[t::FRANCE as usize].owner, Some(Fr));
    }

    #[test]
    fn test_japan_owns_manchuria() {
        let (state, _) = setup();
        assert_eq!(state.territories[t::MANCHURIA as usize].owner, Some(Jp));
    }

    #[test]
    fn test_uk_owns_india() {
        let (state, _) = setup();
        assert_eq!(state.territories[t::INDIA as usize].owner, Some(Uk));
    }

    #[test]
    fn test_us_owns_eastern_us() {
        let (state, _) = setup();
        assert_eq!(
            state.territories[t::EASTERN_UNITED_STATES as usize].owner,
            Some(Us)
        );
    }

    #[test]
    fn test_political_state_initial_wars() {
        let (state, _) = setup();
        // Germany at war with UK and France
        assert!(state.political.are_at_war(Ge, Uk));
        assert!(state.political.are_at_war(Ge, Fr));
        // Italy at war with UK and France
        assert!(state.political.are_at_war(It, Uk));
        assert!(state.political.are_at_war(It, Fr));
        // Japan at war with UK, China, ANZAC
        assert!(state.political.are_at_war(Jp, Uk));
        assert!(state.political.are_at_war(Jp, Ch));
        assert!(state.political.are_at_war(Jp, An));
        // US and Soviet NOT at war
        assert!(!state.political.are_at_war(Us, Ge));
        assert!(!state.political.are_at_war(Us, Jp));
        assert!(!state.political.are_at_war(Su, Ge));
        assert!(!state.political.are_at_war(Su, Jp));
    }

    #[test]
    fn test_us_not_at_war() {
        let (state, _) = setup();
        assert!(!state.powers[Us as usize].at_war);
    }

    #[test]
    fn test_soviet_not_at_war() {
        let (state, _) = setup();
        assert!(!state.powers[Su as usize].at_war);
    }

    #[test]
    fn test_starting_ipcs() {
        let (state, _) = setup();
        assert_eq!(state.powers[Ge as usize].ipcs, 30);
        assert_eq!(state.powers[Su as usize].ipcs, 37);
        assert_eq!(state.powers[Jp as usize].ipcs, 26);
        assert_eq!(state.powers[Us as usize].ipcs, 52);
        assert_eq!(state.powers[Ch as usize].ipcs, 12);
        assert_eq!(state.powers[Uk as usize].ipcs, 28);
        assert_eq!(state.powers[It as usize].ipcs, 10);
        assert_eq!(state.powers[An as usize].ipcs, 10);
        assert_eq!(state.powers[Fr as usize].ipcs, 19);
    }

    #[test]
    fn test_turn_1_germany_first() {
        let (state, _) = setup();
        assert_eq!(state.turn_number, 1);
        assert_eq!(state.current_power, Power::Germany);
    }

    #[test]
    fn test_major_ics_present() {
        let (state, _) = setup();
        let major_ic_territories = [
            t::GERMANY,
            t::RUSSIA,
            t::UNITED_KINGDOM,
            t::JAPAN,
            t::EASTERN_UNITED_STATES,
            t::WESTERN_UNITED_STATES,
            t::NORTHERN_ITALY,
        ];
        for &tid in &major_ic_territories {
            let has_major = state.territories[tid as usize]
                .facilities
                .iter()
                .any(|f| f.facility_type == FacilityType::MajorIndustrialComplex);
            assert!(has_major, "Territory {} should have a Major IC", tid);
        }
    }

    #[test]
    fn test_minor_ics_present() {
        let (state, _) = setup();
        let minor_ic_territories = [t::FRANCE, t::INDIA, t::NEW_SOUTH_WALES, t::SOUTH_AFRICA];
        for &tid in &minor_ic_territories {
            let has_minor = state.territories[tid as usize]
                .facilities
                .iter()
                .any(|f| f.facility_type == FacilityType::MinorIndustrialComplex);
            assert!(has_minor, "Territory {} should have a Minor IC", tid);
        }
    }

    #[test]
    fn test_paris_not_captured() {
        let (state, _) = setup();
        assert!(!state.political.triggers.paris_captured);
    }

    #[test]
    fn test_unit_ids_unique() {
        let (state, _) = setup();
        let mut ids: Vec<UnitId> = Vec::new();
        for t in &state.territories {
            for u in &t.units {
                ids.push(u.id);
            }
        }
        for sz in &state.sea_zones {
            for u in &sz.units {
                ids.push(u.id);
            }
        }
        let total = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(total, ids.len(), "Duplicate unit IDs found!");
    }

    #[test]
    fn test_new_game_creates_playable_state() {
        // Verify Engine::new_game produces a valid state that can advance phases
        let mut engine = crate::Engine::new_game(42);
        assert_eq!(engine.state().turn_number, 1);
        assert_eq!(engine.state().current_power, Power::Germany);

        // Should be able to confirm purchases (empty purchase is valid)
        let result = engine.submit_action(crate::action::Action::ConfirmPurchases);
        assert!(result.is_ok(), "Failed to confirm purchases: {:?}", result.err());
    }

    #[test]
    fn test_germany_has_expected_units() {
        let (state, _) = setup();
        let german_infantry: usize = state
            .territories
            .iter()
            .flat_map(|t| &t.units)
            .filter(|u| u.owner == Ge && u.unit_type == Infantry)
            .count();
        // Germany should have ~35-45 infantry
        assert!(
            german_infantry >= 25 && german_infantry <= 50,
            "German infantry count {} out of expected range",
            german_infantry
        );
    }

    #[test]
    fn test_japan_has_large_navy() {
        let (state, _) = setup();
        let japan_naval: usize = state
            .sea_zones
            .iter()
            .flat_map(|sz| &sz.units)
            .filter(|u| u.owner == Jp)
            .count();
        // Japan should have the largest starting navy (~25-40 ships)
        assert!(
            japan_naval >= 15,
            "Japan naval count {} too low (expected 15+)",
            japan_naval
        );
    }

    #[test]
    fn test_china_infantry_only_plus_fighter() {
        let (state, _) = setup();
        let china_units: Vec<&UnitInstance> = state
            .territories
            .iter()
            .flat_map(|t| &t.units)
            .filter(|u| u.owner == Ch)
            .collect();
        let fighters = china_units.iter().filter(|u| u.unit_type == Fighter).count();
        let infantry = china_units
            .iter()
            .filter(|u| u.unit_type == Infantry)
            .count();
        assert_eq!(fighters, 1, "China should have exactly 1 AVG Fighter");
        assert_eq!(
            infantry + fighters,
            china_units.len(),
            "China should only have infantry and 1 fighter"
        );
    }

    #[test]
    fn test_neutrals_have_no_units() {
        let (state, _) = setup();
        let neutral_territories = [
            t::SWEDEN,
            t::SWITZERLAND,
            t::TURKEY,
            t::SPAIN,
            t::PORTUGAL,
            t::EIRE,
            t::AFGHANISTAN,
        ];
        for &tid in &neutral_territories {
            assert!(
                state.territories[tid as usize].units.is_empty(),
                "Neutral territory {} should have no units",
                tid
            );
        }
    }
}
