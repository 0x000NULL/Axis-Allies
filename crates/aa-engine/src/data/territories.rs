//! All ~164 land territory definitions for Global 1940 2nd Edition.
//!
//! Each entry includes static metadata (name, IPC, owner, type) and
//! adjacency lists (land neighbors, adjacent sea zones, strait connections).

use crate::power::Power;
use crate::territory::{TerritoryDef, TerritoryId, TerritoryType, SeaZoneId, StraitId};

use super::territory_ids as t;
use super::sea_zone_ids as sz;

// ---------------------------------------------------------------------------
// Builder helper — keeps each territory definition concise.
// ---------------------------------------------------------------------------

struct TB(TerritoryDef);

impl TB {
    fn new(id: TerritoryId, name: &str, ipc: u32, owner: Option<Power>) -> Self {
        TB(TerritoryDef {
            id,
            name: name.to_string(),
            ipc_value: ipc,
            is_capital: None,
            is_victory_city: false,
            original_owner: owner,
            territory_type: TerritoryType::Normal,
            adjacent_land: vec![],
            adjacent_sea: vec![],
            strait_connections: vec![],
            convoys_from: vec![],
            is_island: false,
        })
    }

    fn capital(mut self, p: Power) -> Self {
        self.0.is_capital = Some(p);
        self
    }
    fn vc(mut self) -> Self {
        self.0.is_victory_city = true;
        self
    }
    fn island(mut self) -> Self {
        self.0.is_island = true;
        self
    }
    fn ttype(mut self, tt: TerritoryType) -> Self {
        self.0.territory_type = tt;
        self
    }
    fn land(mut self, ids: &[TerritoryId]) -> Self {
        self.0.adjacent_land = ids.to_vec();
        self
    }
    fn sea(mut self, ids: &[SeaZoneId]) -> Self {
        self.0.adjacent_sea = ids.to_vec();
        self
    }
    #[allow(dead_code)]
    fn straits(mut self, pairs: &[(TerritoryId, StraitId)]) -> Self {
        self.0.strait_connections = pairs.to_vec();
        self
    }
    fn convoys(mut self, ids: &[SeaZoneId]) -> Self {
        self.0.convoys_from = ids.to_vec();
        self
    }
    fn done(self) -> TerritoryDef {
        self.0
    }
}

// Shorthand aliases for powers
use Power::{Germany as Ge, SovietUnion as Su, Japan as Jp, UnitedStates as Us,
            China as Ch, UnitedKingdom as Uk, Italy as It, ANZAC as An, France as Fr};

/// Build all territory definitions. Indices match territory IDs.
pub fn build_territory_defs() -> Vec<TerritoryDef> {
    vec![
        // =================================================================
        // EUROPE (IDs 0–34)
        // =================================================================

        // 0 — Germany
        TB::new(t::GERMANY, "Germany", 5, Some(Ge))
            .capital(Ge).vc()
            .land(&[t::WESTERN_GERMANY, t::GREATER_SOUTHERN_GERMANY,
                     t::HOLLAND_BELGIUM, t::DENMARK, t::POLAND, t::SLOVAKIA_HUNGARY])
            .sea(&[sz::SZ_BALTIC_SEA])
            .convoys(&[sz::SZ_BALTIC_SEA])
            .done(),

        // 1 — Western Germany
        TB::new(t::WESTERN_GERMANY, "Western Germany", 2, Some(Ge))
            .land(&[t::GERMANY, t::GREATER_SOUTHERN_GERMANY, t::HOLLAND_BELGIUM,
                     t::NORMANDY_BORDEAUX, t::FRANCE])
            .sea(&[sz::SZ_NORTH_SEA])
            .done(),

        // 2 — Greater Southern Germany
        TB::new(t::GREATER_SOUTHERN_GERMANY, "Greater Southern Germany", 2, Some(Ge))
            .land(&[t::GERMANY, t::WESTERN_GERMANY, t::FRANCE,
                     t::SOUTHERN_FRANCE, t::NORTHERN_ITALY, t::SLOVAKIA_HUNGARY])
            .done(),

        // 3 — Holland Belgium
        TB::new(t::HOLLAND_BELGIUM, "Holland Belgium", 2, Some(Ge))
            .land(&[t::GERMANY, t::WESTERN_GERMANY, t::NORMANDY_BORDEAUX, t::FRANCE])
            .sea(&[sz::SZ_NORTH_SEA, sz::SZ_ENGLISH_CHANNEL])
            .done(),

        // 4 — Normandy Bordeaux
        TB::new(t::NORMANDY_BORDEAUX, "Normandy Bordeaux", 2, Some(Fr))
            .land(&[t::HOLLAND_BELGIUM, t::WESTERN_GERMANY, t::FRANCE,
                     t::SOUTHERN_FRANCE, t::SPAIN])
            .sea(&[sz::SZ_ENGLISH_CHANNEL, sz::SZ_BAY_OF_BISCAY])
            .done(),

        // 5 — France
        TB::new(t::FRANCE, "France", 6, Some(Fr))
            .capital(Fr).vc()
            .land(&[t::WESTERN_GERMANY, t::HOLLAND_BELGIUM, t::NORMANDY_BORDEAUX,
                     t::SOUTHERN_FRANCE, t::GREATER_SOUTHERN_GERMANY])
            .done(),

        // 6 — Southern France
        TB::new(t::SOUTHERN_FRANCE, "Southern France", 1, Some(Fr))
            .land(&[t::FRANCE, t::NORMANDY_BORDEAUX, t::GREATER_SOUTHERN_GERMANY,
                     t::NORTHERN_ITALY, t::SPAIN])
            .sea(&[sz::SZ_WESTERN_MED])
            .done(),

        // 7 — Denmark
        TB::new(t::DENMARK, "Denmark", 2, Some(Ge))
            .land(&[t::GERMANY, t::NORWAY, t::SWEDEN])
            .sea(&[sz::SZ_SKAGERRAK, sz::SZ_BALTIC_SEA])
            .done(),

        // 8 — Norway
        TB::new(t::NORWAY, "Norway", 2, Some(Ge))
            .land(&[t::DENMARK, t::SWEDEN, t::FINLAND])
            .sea(&[sz::SZ_NORWEGIAN_SEA, sz::SZ_SKAGERRAK, sz::SZ_BARENTS_SEA, sz::SZ_NORTH_SEA])
            .convoys(&[sz::SZ_NORWEGIAN_SEA, sz::SZ_NORTH_SEA])
            .done(),

        // 9 — Finland
        TB::new(t::FINLAND, "Finland", 1, Some(Ge))
            .land(&[t::NORWAY, t::SWEDEN, t::VYBORG, t::NENETSIA])
            .sea(&[sz::SZ_BALTIC_SEA])
            .done(),

        // 10 — Sweden
        TB::new(t::SWEDEN, "Sweden", 0, None)
            .ttype(TerritoryType::TrueNeutral)
            .land(&[t::NORWAY, t::FINLAND, t::DENMARK])
            .sea(&[sz::SZ_BALTIC_SEA, sz::SZ_SKAGERRAK])
            .done(),

        // 11 — Poland
        TB::new(t::POLAND, "Poland", 2, Some(Ge))
            .land(&[t::GERMANY, t::BALTIC_STATES, t::SLOVAKIA_HUNGARY,
                     t::ROMANIA, t::BELARUS, t::WESTERN_UKRAINE])
            .sea(&[sz::SZ_BALTIC_SEA])
            .done(),

        // 12 — Baltic States
        TB::new(t::BALTIC_STATES, "Baltic States", 2, Some(Ge))
            .land(&[t::POLAND, t::BELARUS, t::NOVGOROD])
            .sea(&[sz::SZ_BALTIC_SEA])
            .done(),

        // 13 — Slovakia Hungary
        TB::new(t::SLOVAKIA_HUNGARY, "Slovakia Hungary", 2, Some(Ge))
            .land(&[t::GERMANY, t::GREATER_SOUTHERN_GERMANY, t::POLAND,
                     t::ROMANIA, t::YUGOSLAVIA, t::NORTHERN_ITALY])
            .done(),

        // 14 — Romania
        TB::new(t::ROMANIA, "Romania", 3, Some(Ge))
            .land(&[t::SLOVAKIA_HUNGARY, t::POLAND, t::YUGOSLAVIA, t::BULGARIA,
                     t::WESTERN_UKRAINE, t::UKRAINE])
            .sea(&[sz::SZ_BLACK_SEA])
            .done(),

        // 15 — Yugoslavia
        TB::new(t::YUGOSLAVIA, "Yugoslavia", 2, Some(Ge))
            .land(&[t::SLOVAKIA_HUNGARY, t::ROMANIA, t::BULGARIA, t::GREECE,
                     t::ALBANIA, t::NORTHERN_ITALY])
            .sea(&[sz::SZ_OFF_SOUTHERN_ITALY])
            .done(),

        // 16 — Bulgaria
        TB::new(t::BULGARIA, "Bulgaria", 2, Some(Ge))
            .land(&[t::ROMANIA, t::YUGOSLAVIA, t::GREECE, t::TURKEY])
            .sea(&[sz::SZ_BLACK_SEA, sz::SZ_AEGEAN_SEA])
            .done(),

        // 17 — Greece
        TB::new(t::GREECE, "Greece", 2, Some(Ge))
            .vc()
            .land(&[t::YUGOSLAVIA, t::BULGARIA, t::ALBANIA])
            .sea(&[sz::SZ_AEGEAN_SEA, sz::SZ_OFF_SOUTHERN_ITALY])
            .done(),

        // 18 — Albania
        TB::new(t::ALBANIA, "Albania", 0, Some(It))
            .land(&[t::YUGOSLAVIA, t::GREECE])
            .sea(&[sz::SZ_OFF_SOUTHERN_ITALY])
            .done(),

        // 19 — Northern Italy
        TB::new(t::NORTHERN_ITALY, "Northern Italy", 4, Some(It))
            .capital(It).vc()
            .land(&[t::GREATER_SOUTHERN_GERMANY, t::SOUTHERN_FRANCE, t::SOUTHERN_ITALY,
                     t::YUGOSLAVIA, t::SLOVAKIA_HUNGARY])
            .sea(&[sz::SZ_TYRRHENIAN_SEA])
            .convoys(&[sz::SZ_TYRRHENIAN_SEA, sz::SZ_OFF_SOUTHERN_ITALY])
            .done(),

        // 20 — Southern Italy
        TB::new(t::SOUTHERN_ITALY, "Southern Italy", 1, Some(It))
            .land(&[t::NORTHERN_ITALY])
            .sea(&[sz::SZ_TYRRHENIAN_SEA, sz::SZ_OFF_SOUTHERN_ITALY])
            .done(),

        // 21 — Sardinia
        TB::new(t::SARDINIA, "Sardinia", 0, Some(It))
            .island()
            .sea(&[sz::SZ_TYRRHENIAN_SEA, sz::SZ_WESTERN_MED])
            .done(),

        // 22 — Sicily
        TB::new(t::SICILY, "Sicily", 0, Some(It))
            .island()
            .sea(&[sz::SZ_TYRRHENIAN_SEA, sz::SZ_OFF_SOUTHERN_ITALY])
            .done(),

        // 23 — Crete
        TB::new(t::CRETE, "Crete", 0, Some(Ge))
            .island()
            .sea(&[sz::SZ_AEGEAN_SEA, sz::SZ_EASTERN_MED])
            .done(),

        // 24 — Malta
        TB::new(t::MALTA, "Malta", 0, Some(Uk))
            .island()
            .sea(&[sz::SZ_OFF_SOUTHERN_ITALY, sz::SZ_EASTERN_MED])
            .done(),

        // 25 — Cyprus
        TB::new(t::CYPRUS, "Cyprus", 0, Some(Uk))
            .island()
            .sea(&[sz::SZ_EASTERN_MED])
            .done(),

        // 26 — United Kingdom
        TB::new(t::UNITED_KINGDOM, "United Kingdom", 6, Some(Uk))
            .capital(Uk).vc()
            .land(&[t::SCOTLAND])
            .sea(&[sz::SZ_ENGLISH_CHANNEL, sz::SZ_NORTH_SEA])
            .convoys(&[sz::SZ_ENGLISH_CHANNEL, sz::SZ_NORTH_SEA, sz::SZ_NORTH_ATLANTIC])
            .island()
            .done(),

        // 27 — Scotland
        TB::new(t::SCOTLAND, "Scotland", 2, Some(Uk))
            .land(&[t::UNITED_KINGDOM])
            .sea(&[sz::SZ_NORTH_SEA, sz::SZ_NORWEGIAN_SEA])
            .island()
            .done(),

        // 28 — Eire
        TB::new(t::EIRE, "Eire", 0, None)
            .ttype(TerritoryType::TrueNeutral)
            .island()
            .sea(&[sz::SZ_BAY_OF_BISCAY, sz::SZ_ENGLISH_CHANNEL])
            .done(),

        // 29 — Gibraltar
        TB::new(t::GIBRALTAR, "Gibraltar", 0, Some(Uk))
            .land(&[t::SPAIN])
            .sea(&[sz::SZ_OFF_GIBRALTAR])
            .done(),

        // 30 — Iceland
        TB::new(t::ICELAND, "Iceland", 0, Some(Uk))
            .island()
            .sea(&[sz::SZ_OFF_ICELAND])
            .done(),

        // 31 — Spain
        TB::new(t::SPAIN, "Spain", 0, None)
            .ttype(TerritoryType::TrueNeutral)
            .land(&[t::NORMANDY_BORDEAUX, t::SOUTHERN_FRANCE, t::PORTUGAL, t::GIBRALTAR])
            .sea(&[sz::SZ_BAY_OF_BISCAY, sz::SZ_OFF_GIBRALTAR, sz::SZ_OFF_MOROCCO])
            .done(),

        // 32 — Portugal
        TB::new(t::PORTUGAL, "Portugal", 0, None)
            .ttype(TerritoryType::TrueNeutral)
            .land(&[t::SPAIN])
            .sea(&[sz::SZ_OFF_GIBRALTAR, sz::SZ_BAY_OF_BISCAY])
            .done(),

        // 33 — Switzerland
        TB::new(t::SWITZERLAND, "Switzerland", 0, None)
            .ttype(TerritoryType::Impassable)
            .done(),

        // 34 — Turkey
        TB::new(t::TURKEY, "Turkey", 0, None)
            .ttype(TerritoryType::TrueNeutral)
            .land(&[t::BULGARIA, t::SYRIA])
            .sea(&[sz::SZ_BLACK_SEA, sz::SZ_AEGEAN_SEA])
            .done(),

        // =================================================================
        // NORTH AFRICA (IDs 35–41)
        // =================================================================

        // 35 — Morocco
        TB::new(t::MOROCCO, "Morocco", 1, Some(Fr))
            .land(&[t::ALGERIA, t::FRENCH_WEST_AFRICA])
            .sea(&[sz::SZ_OFF_MOROCCO, sz::SZ_OFF_GIBRALTAR])
            .done(),

        // 36 — Algeria
        TB::new(t::ALGERIA, "Algeria", 0, Some(Fr))
            .land(&[t::MOROCCO, t::TUNISIA, t::FRENCH_WEST_AFRICA])
            .sea(&[sz::SZ_WESTERN_MED, sz::SZ_OFF_MOROCCO])
            .done(),

        // 37 — Tunisia
        TB::new(t::TUNISIA, "Tunisia", 0, Some(Fr))
            .land(&[t::ALGERIA, t::LIBYA])
            .sea(&[sz::SZ_WESTERN_MED, sz::SZ_TYRRHENIAN_SEA])
            .done(),

        // 38 — Libya
        TB::new(t::LIBYA, "Libya", 1, Some(It))
            .land(&[t::TUNISIA, t::TOBRUK, t::FRENCH_EQUATORIAL_AFRICA])
            .sea(&[sz::SZ_WESTERN_MED, sz::SZ_EASTERN_MED])
            .done(),

        // 39 — Tobruk
        TB::new(t::TOBRUK, "Tobruk", 0, Some(It))
            .land(&[t::LIBYA, t::EGYPT, t::ANGLO_EGYPTIAN_SUDAN])
            .sea(&[sz::SZ_EASTERN_MED, sz::SZ_OFF_EGYPT])
            .done(),

        // 40 — Egypt
        TB::new(t::EGYPT, "Egypt", 2, Some(Uk))
            .vc()
            .land(&[t::TOBRUK, t::ANGLO_EGYPTIAN_SUDAN, t::TRANS_JORDAN])
            .sea(&[sz::SZ_OFF_EGYPT])
            .done(),

        // 41 — Anglo-Egyptian Sudan
        TB::new(t::ANGLO_EGYPTIAN_SUDAN, "Anglo-Egyptian Sudan", 0, Some(Uk))
            .land(&[t::EGYPT, t::TOBRUK, t::KENYA, t::BELGIAN_CONGO,
                     t::FRENCH_EQUATORIAL_AFRICA, t::ETHIOPIA])
            .sea(&[sz::SZ_RED_SEA])
            .done(),

        // =================================================================
        // EAST AND SOUTH AFRICA (IDs 42–56)
        // =================================================================

        // 42 — Ethiopia
        TB::new(t::ETHIOPIA, "Ethiopia", 1, Some(It))
            .land(&[t::ANGLO_EGYPTIAN_SUDAN, t::ITALIAN_SOMALILAND,
                     t::BRITISH_SOMALILAND, t::KENYA])
            .sea(&[sz::SZ_RED_SEA])
            .done(),

        // 43 — Italian Somaliland
        TB::new(t::ITALIAN_SOMALILAND, "Italian Somaliland", 0, Some(It))
            .land(&[t::ETHIOPIA, t::BRITISH_SOMALILAND, t::KENYA])
            .sea(&[sz::SZ_OFF_EAST_AFRICA])
            .done(),

        // 44 — British Somaliland
        TB::new(t::BRITISH_SOMALILAND, "British Somaliland", 0, Some(Uk))
            .land(&[t::ETHIOPIA, t::ITALIAN_SOMALILAND])
            .sea(&[sz::SZ_OFF_EAST_AFRICA])
            .done(),

        // 45 — Kenya
        TB::new(t::KENYA, "Kenya", 0, Some(Uk))
            .land(&[t::ETHIOPIA, t::ITALIAN_SOMALILAND, t::ANGLO_EGYPTIAN_SUDAN,
                     t::BELGIAN_CONGO, t::TANGANYIKA])
            .sea(&[sz::SZ_OFF_EAST_AFRICA])
            .done(),

        // 46 — Belgian Congo
        TB::new(t::BELGIAN_CONGO, "Belgian Congo", 1, Some(Uk))
            .land(&[t::FRENCH_EQUATORIAL_AFRICA, t::ANGLO_EGYPTIAN_SUDAN,
                     t::KENYA, t::TANGANYIKA, t::NORTHERN_RHODESIA, t::ANGOLA])
            .sea(&[sz::SZ_OFF_ANGOLA])
            .done(),

        // 47 — French Equatorial Africa
        TB::new(t::FRENCH_EQUATORIAL_AFRICA, "French Equatorial Africa", 1, Some(Fr))
            .land(&[t::FRENCH_WEST_AFRICA, t::BELGIAN_CONGO,
                     t::ANGLO_EGYPTIAN_SUDAN, t::LIBYA, t::ANGOLA])
            .sea(&[sz::SZ_OFF_ANGOLA])
            .done(),

        // 48 — French West Africa
        TB::new(t::FRENCH_WEST_AFRICA, "French West Africa", 1, Some(Fr))
            .land(&[t::MOROCCO, t::ALGERIA, t::FRENCH_EQUATORIAL_AFRICA])
            .sea(&[sz::SZ_OFF_FRENCH_WEST_AFRICA, sz::SZ_OFF_MOROCCO])
            .done(),

        // 49 — Tanganyika
        TB::new(t::TANGANYIKA, "Tanganyika", 1, Some(Uk))
            .land(&[t::KENYA, t::BELGIAN_CONGO, t::NORTHERN_RHODESIA, t::MOZAMBIQUE])
            .sea(&[sz::SZ_OFF_EAST_AFRICA])
            .done(),

        // 50 — Northern Rhodesia
        TB::new(t::NORTHERN_RHODESIA, "Northern Rhodesia", 0, Some(Uk))
            .land(&[t::BELGIAN_CONGO, t::TANGANYIKA, t::RHODESIA,
                     t::SOUTHWEST_AFRICA, t::MOZAMBIQUE, t::ANGOLA])
            .done(),

        // 51 — Rhodesia
        TB::new(t::RHODESIA, "Rhodesia", 1, Some(Uk))
            .land(&[t::NORTHERN_RHODESIA, t::MOZAMBIQUE, t::SOUTH_AFRICA, t::SOUTHWEST_AFRICA])
            .done(),

        // 52 — South Africa
        TB::new(t::SOUTH_AFRICA, "South Africa", 2, Some(Uk))
            .vc()
            .land(&[t::SOUTHWEST_AFRICA, t::RHODESIA, t::MOZAMBIQUE])
            .sea(&[sz::SZ_OFF_SOUTH_AFRICA_EAST, sz::SZ_OFF_SOUTH_AFRICA_WEST])
            .done(),

        // 53 — Southwest Africa
        TB::new(t::SOUTHWEST_AFRICA, "Southwest Africa", 0, Some(Uk))
            .land(&[t::SOUTH_AFRICA, t::NORTHERN_RHODESIA, t::ANGOLA, t::RHODESIA])
            .sea(&[sz::SZ_OFF_SOUTHWEST_AFRICA])
            .done(),

        // 54 — Mozambique
        TB::new(t::MOZAMBIQUE, "Mozambique", 0, None)
            .ttype(TerritoryType::TrueNeutral)
            .land(&[t::TANGANYIKA, t::NORTHERN_RHODESIA, t::RHODESIA, t::SOUTH_AFRICA])
            .sea(&[sz::SZ_OFF_MOZAMBIQUE])
            .done(),

        // 55 — Madagascar
        TB::new(t::MADAGASCAR, "Madagascar", 1, Some(Fr))
            .island()
            .sea(&[sz::SZ_OFF_MADAGASCAR])
            .done(),

        // 56 — Angola
        TB::new(t::ANGOLA, "Angola", 0, None)
            .ttype(TerritoryType::TrueNeutral)
            .land(&[t::BELGIAN_CONGO, t::NORTHERN_RHODESIA, t::SOUTHWEST_AFRICA,
                     t::FRENCH_EQUATORIAL_AFRICA])
            .sea(&[sz::SZ_OFF_ANGOLA])
            .done(),

        // =================================================================
        // MIDDLE EAST (IDs 57–64)
        // =================================================================

        // 57 — Trans-Jordan
        TB::new(t::TRANS_JORDAN, "Trans-Jordan", 0, Some(Uk))
            .land(&[t::EGYPT, t::IRAQ, t::SYRIA, t::SAUDI_ARABIA])
            .sea(&[sz::SZ_RED_SEA])
            .done(),

        // 58 — Iraq
        TB::new(t::IRAQ, "Iraq", 2, Some(Uk))
            .land(&[t::TRANS_JORDAN, t::SYRIA, t::PERSIA, t::NORTHWEST_PERSIA,
                     t::SAUDI_ARABIA])
            .sea(&[sz::SZ_PERSIAN_GULF])
            .done(),

        // 59 — Syria
        TB::new(t::SYRIA, "Syria", 1, Some(Fr))
            .land(&[t::TRANS_JORDAN, t::IRAQ, t::TURKEY])
            .sea(&[sz::SZ_EASTERN_MED])
            .done(),

        // 60 — Persia
        TB::new(t::PERSIA, "Persia", 2, Some(Uk))
            .land(&[t::IRAQ, t::NORTHWEST_PERSIA, t::EASTERN_PERSIA])
            .sea(&[sz::SZ_PERSIAN_GULF])
            .done(),

        // 61 — Northwest Persia
        TB::new(t::NORTHWEST_PERSIA, "Northwest Persia", 0, Some(Uk))
            .land(&[t::PERSIA, t::EASTERN_PERSIA, t::IRAQ, t::CAUCASUS, t::AFGHANISTAN])
            .done(),

        // 62 — Eastern Persia
        TB::new(t::EASTERN_PERSIA, "Eastern Persia", 0, Some(Uk))
            .land(&[t::PERSIA, t::NORTHWEST_PERSIA, t::AFGHANISTAN, t::WEST_INDIA])
            .done(),

        // 63 — Saudi Arabia
        TB::new(t::SAUDI_ARABIA, "Saudi Arabia", 0, None)
            .ttype(TerritoryType::ProAllies)
            .land(&[t::TRANS_JORDAN, t::IRAQ])
            .sea(&[sz::SZ_PERSIAN_GULF, sz::SZ_RED_SEA])
            .done(),

        // 64 — Afghanistan
        TB::new(t::AFGHANISTAN, "Afghanistan", 0, None)
            .ttype(TerritoryType::TrueNeutral)
            .land(&[t::EASTERN_PERSIA, t::NORTHWEST_PERSIA])
            .done(),

        // =================================================================
        // SOVIET UNION (IDs 65–88)
        // =================================================================

        // 65 — Novgorod (Leningrad)
        TB::new(t::NOVGOROD, "Novgorod", 2, Some(Su))
            .vc()
            .land(&[t::VYBORG, t::BALTIC_STATES, t::BELARUS, t::RUSSIA, t::VOLOGDA, t::ARCHANGEL])
            .sea(&[sz::SZ_BALTIC_SEA])
            .convoys(&[sz::SZ_BALTIC_SEA])
            .done(),

        // 66 — Vyborg (Karelia)
        TB::new(t::VYBORG, "Vyborg", 0, Some(Su))
            .land(&[t::FINLAND, t::NOVGOROD, t::ARCHANGEL, t::NENETSIA])
            .sea(&[sz::SZ_BALTIC_SEA])
            .done(),

        // 67 — Archangel
        TB::new(t::ARCHANGEL, "Archangel", 1, Some(Su))
            .land(&[t::NOVGOROD, t::VYBORG, t::NENETSIA, t::VOLOGDA])
            .sea(&[sz::SZ_BARENTS_SEA])
            .done(),

        // 68 — Nenetsia
        TB::new(t::NENETSIA, "Nenetsia", 0, Some(Su))
            .land(&[t::FINLAND, t::VYBORG, t::ARCHANGEL, t::VOLOGDA, t::URALS])
            .sea(&[sz::SZ_BARENTS_SEA])
            .done(),

        // 69 — Russia (Moscow)
        TB::new(t::RUSSIA, "Russia", 3, Some(Su))
            .capital(Su).vc()
            .land(&[t::NOVGOROD, t::BELARUS, t::WESTERN_UKRAINE, t::TAMBOV, t::VOLOGDA])
            .done(),

        // 70 — Belarus
        TB::new(t::BELARUS, "Belarus", 1, Some(Su))
            .land(&[t::POLAND, t::BALTIC_STATES, t::NOVGOROD, t::RUSSIA, t::WESTERN_UKRAINE])
            .done(),

        // 71 — Western Ukraine
        TB::new(t::WESTERN_UKRAINE, "Western Ukraine", 1, Some(Su))
            .land(&[t::POLAND, t::ROMANIA, t::BELARUS, t::RUSSIA, t::UKRAINE])
            .done(),

        // 72 — Ukraine
        TB::new(t::UKRAINE, "Ukraine", 2, Some(Su))
            .land(&[t::WESTERN_UKRAINE, t::ROMANIA, t::ROSTOV, t::TAMBOV])
            .sea(&[sz::SZ_BLACK_SEA])
            .done(),

        // 73 — Rostov
        TB::new(t::ROSTOV, "Rostov", 1, Some(Su))
            .land(&[t::UKRAINE, t::CAUCASUS, t::VOLGOGRAD, t::TAMBOV])
            .sea(&[sz::SZ_BLACK_SEA])
            .done(),

        // 74 — Volgograd (Stalingrad)
        TB::new(t::VOLGOGRAD, "Volgograd", 2, Some(Su))
            .vc()
            .land(&[t::ROSTOV, t::CAUCASUS, t::TAMBOV, t::URALS, t::KAZAKHSTAN])
            .done(),

        // 75 — Caucasus
        TB::new(t::CAUCASUS, "Caucasus", 4, Some(Su))
            .land(&[t::ROSTOV, t::VOLGOGRAD, t::NORTHWEST_PERSIA])
            .sea(&[sz::SZ_BLACK_SEA])
            .done(),

        // 76 — Tambov
        TB::new(t::TAMBOV, "Tambov", 1, Some(Su))
            .land(&[t::RUSSIA, t::UKRAINE, t::ROSTOV, t::VOLGOGRAD, t::VOLOGDA, t::URALS])
            .done(),

        // 77 — Vologda
        TB::new(t::VOLOGDA, "Vologda", 1, Some(Su))
            .land(&[t::NOVGOROD, t::ARCHANGEL, t::NENETSIA, t::RUSSIA, t::TAMBOV, t::URALS])
            .done(),

        // 78 — Urals
        TB::new(t::URALS, "Urals", 2, Some(Su))
            .land(&[t::VOLOGDA, t::NENETSIA, t::TAMBOV, t::VOLGOGRAD,
                     t::KAZAKHSTAN, t::NOVOSIBIRSK])
            .done(),

        // 79 — Kazakhstan
        TB::new(t::KAZAKHSTAN, "Kazakhstan", 2, Some(Su))
            .land(&[t::VOLGOGRAD, t::URALS, t::NOVOSIBIRSK, t::OLGIY])
            .done(),

        // 80 — Novosibirsk
        TB::new(t::NOVOSIBIRSK, "Novosibirsk", 2, Some(Su))
            .land(&[t::URALS, t::KAZAKHSTAN, t::TIMGUSKA, t::YENISEY])
            .done(),

        // 81 — Timguska
        TB::new(t::TIMGUSKA, "Timguska", 0, Some(Su))
            .land(&[t::NOVOSIBIRSK, t::YENISEY, t::EVENKI_NATIONAL_OKRUG])
            .done(),

        // 82 — Yenisey
        TB::new(t::YENISEY, "Yenisey", 0, Some(Su))
            .land(&[t::NOVOSIBIRSK, t::TIMGUSKA, t::EVENKI_NATIONAL_OKRUG])
            .done(),

        // 83 — Evenki National Okrug
        TB::new(t::EVENKI_NATIONAL_OKRUG, "Evenki National Okrug", 0, Some(Su))
            .land(&[t::TIMGUSKA, t::YENISEY, t::YAKUT_SSR])
            .done(),

        // 84 — Yakut S.S.R.
        TB::new(t::YAKUT_SSR, "Yakut S.S.R.", 0, Some(Su))
            .land(&[t::EVENKI_NATIONAL_OKRUG, t::BURYATIA, t::SAKHA])
            .done(),

        // 85 — Buryatia
        TB::new(t::BURYATIA, "Buryatia", 1, Some(Su))
            .land(&[t::YAKUT_SSR, t::SAKHA, t::AMUR, t::BUYANT_UHAA])
            .done(),

        // 86 — Sakha
        TB::new(t::SAKHA, "Sakha", 0, Some(Su))
            .land(&[t::YAKUT_SSR, t::BURYATIA, t::SOVIET_FAR_EAST])
            .sea(&[sz::SZ_SEA_OF_OKHOTSK])
            .done(),

        // 87 — Soviet Far East
        TB::new(t::SOVIET_FAR_EAST, "Soviet Far East", 1, Some(Su))
            .land(&[t::SAKHA, t::AMUR])
            .sea(&[sz::SZ_SEA_OF_OKHOTSK])
            .done(),

        // 88 — Amur
        TB::new(t::AMUR, "Amur", 1, Some(Su))
            .land(&[t::BURYATIA, t::SOVIET_FAR_EAST, t::MANCHURIA, t::BUYANT_UHAA])
            .sea(&[sz::SZ_OFF_MANCHURIA, sz::SZ_SEA_OF_OKHOTSK])
            .done(),

        // =================================================================
        // MONGOLIA (IDs 89–94) — all True Neutral
        // =================================================================

        // 89 — Olgiy
        TB::new(t::OLGIY, "Olgiy", 0, None)
            .ttype(TerritoryType::TrueNeutral)
            .land(&[t::DZAVHAN, t::TSAGAAN_OLOM, t::KAZAKHSTAN])
            .done(),

        // 90 — Dzavhan
        TB::new(t::DZAVHAN, "Dzavhan", 0, None)
            .ttype(TerritoryType::TrueNeutral)
            .land(&[t::OLGIY, t::TSAGAAN_OLOM, t::CENTRAL_MONGOLIA])
            .done(),

        // 91 — Tsagaan-Olom
        TB::new(t::TSAGAAN_OLOM, "Tsagaan-Olom", 0, None)
            .ttype(TerritoryType::TrueNeutral)
            .land(&[t::OLGIY, t::DZAVHAN, t::CENTRAL_MONGOLIA, t::TSINGHAI, t::SUIYUAN])
            .done(),

        // 92 — Central Mongolia
        TB::new(t::CENTRAL_MONGOLIA, "Central Mongolia", 0, None)
            .ttype(TerritoryType::TrueNeutral)
            .land(&[t::DZAVHAN, t::TSAGAAN_OLOM, t::ULAANBAATAR, t::BUYANT_UHAA, t::CHAHAR])
            .done(),

        // 93 — Ulaanbaatar
        TB::new(t::ULAANBAATAR, "Ulaanbaatar", 0, None)
            .ttype(TerritoryType::TrueNeutral)
            .land(&[t::CENTRAL_MONGOLIA, t::BUYANT_UHAA, t::MANCHURIA, t::CHAHAR])
            .done(),

        // 94 — Buyant-Uhaa
        TB::new(t::BUYANT_UHAA, "Buyant-Uhaa", 0, None)
            .ttype(TerritoryType::TrueNeutral)
            .land(&[t::CENTRAL_MONGOLIA, t::ULAANBAATAR, t::AMUR, t::BURYATIA, t::MANCHURIA])
            .done(),

        // =================================================================
        // CHINA (IDs 95–113)
        // =================================================================

        // 95 — Szechwan (Chungking)
        TB::new(t::SZECHWAN, "Szechwan", 2, Some(Ch))
            .capital(Ch).vc()
            .land(&[t::YUNNAN, t::KWEICHOW, t::HUNAN, t::SIKANG, t::SHENSI, t::TSINGHAI])
            .done(),

        // 96 — Yunnan
        TB::new(t::YUNNAN, "Yunnan", 1, Some(Ch))
            .land(&[t::SZECHWAN, t::KWEICHOW, t::KWANGSI, t::BURMA, t::SHAN_STATE,
                     t::FRENCH_INDOCHINA])
            .done(),

        // 97 — Kweichow
        TB::new(t::KWEICHOW, "Kweichow", 1, Some(Ch))
            .land(&[t::SZECHWAN, t::YUNNAN, t::HUNAN, t::KWANGSI])
            .done(),

        // 98 — Hunan
        TB::new(t::HUNAN, "Hunan", 1, Some(Ch))
            .land(&[t::SZECHWAN, t::KWEICHOW, t::KIANGSI, t::KWANGSI,
                     t::ANHWE, t::SHENSI, t::KIANGSU])
            .done(),

        // 99 — Kiangsi
        TB::new(t::KIANGSI, "Kiangsi", 1, Some(Ch))
            .land(&[t::HUNAN, t::KWANGSI, t::KWANGTUNG, t::ANHWE, t::KIANGSU])
            .done(),

        // 100 — Kwangsi
        TB::new(t::KWANGSI, "Kwangsi", 1, Some(Ch))
            .land(&[t::YUNNAN, t::KWEICHOW, t::HUNAN, t::KIANGSI,
                     t::KWANGTUNG, t::FRENCH_INDOCHINA])
            .done(),

        // 101 — Sikang
        TB::new(t::SIKANG, "Sikang", 1, Some(Ch))
            .land(&[t::SZECHWAN, t::TSINGHAI])
            .done(),

        // 102 — Tsinghai
        TB::new(t::TSINGHAI, "Tsinghai", 1, Some(Ch))
            .land(&[t::SIKANG, t::SZECHWAN, t::SHENSI, t::KANSU,
                     t::SUIYUAN, t::TSAGAAN_OLOM])
            .done(),

        // 103 — Suiyuan
        TB::new(t::SUIYUAN, "Suiyuan", 1, Some(Ch))
            .land(&[t::TSINGHAI, t::SHENSI, t::CHAHAR, t::JEHOL, t::TSAGAAN_OLOM])
            .done(),

        // 104 — Shensi
        TB::new(t::SHENSI, "Shensi", 1, Some(Ch))
            .land(&[t::SZECHWAN, t::HUNAN, t::TSINGHAI, t::SUIYUAN,
                     t::KANSU, t::ANHWE, t::HOPEI])
            .done(),

        // 105 — Kansu
        TB::new(t::KANSU, "Kansu", 0, Some(Ch))
            .land(&[t::TSINGHAI, t::SHENSI])
            .done(),

        // 106 — Kiangsu (Shanghai)
        TB::new(t::KIANGSU, "Kiangsu", 3, Some(Ch))
            .vc()
            .land(&[t::HUNAN, t::KIANGSI, t::ANHWE, t::SHANTUNG, t::HOPEI])
            .sea(&[sz::SZ_EAST_CHINA_SEA])
            .done(),

        // 107 — Shantung
        TB::new(t::SHANTUNG, "Shantung", 1, Some(Ch))
            .land(&[t::KIANGSU, t::HOPEI, t::ANHWE, t::JEHOL])
            .sea(&[sz::SZ_YELLOW_SEA])
            .done(),

        // 108 — Jehol
        TB::new(t::JEHOL, "Jehol", 1, Some(Ch))
            .land(&[t::SUIYUAN, t::CHAHAR, t::HOPEI, t::SHANTUNG, t::MANCHURIA])
            .done(),

        // 109 — Kwangtung
        TB::new(t::KWANGTUNG, "Kwangtung", 3, Some(Ch))
            .land(&[t::KIANGSI, t::KWANGSI])
            .sea(&[sz::SZ_SOUTH_CHINA_SEA])
            .done(),

        // 110 — Anhwe
        TB::new(t::ANHWE, "Anhwe", 1, Some(Ch))
            .land(&[t::HUNAN, t::KIANGSI, t::KIANGSU, t::SHANTUNG, t::HOPEI, t::SHENSI])
            .done(),

        // 111 — Chahar
        TB::new(t::CHAHAR, "Chahar", 0, Some(Ch))
            .land(&[t::SUIYUAN, t::JEHOL, t::HOPEI, t::CENTRAL_MONGOLIA, t::ULAANBAATAR])
            .done(),

        // 112 — Hopei
        TB::new(t::HOPEI, "Hopei", 1, Some(Ch))
            .land(&[t::SHENSI, t::ANHWE, t::KIANGSU, t::SHANTUNG, t::JEHOL, t::CHAHAR])
            .done(),

        // 113 — Manchuria
        TB::new(t::MANCHURIA, "Manchuria", 3, Some(Jp))
            .land(&[t::KOREA, t::JEHOL, t::AMUR, t::ULAANBAATAR, t::BUYANT_UHAA])
            .sea(&[sz::SZ_OFF_MANCHURIA])
            .done(),

        // =================================================================
        // INDIA AND SOUTHEAST ASIA (IDs 114–121)
        // =================================================================

        // 114 — India (Calcutta)
        TB::new(t::INDIA, "India", 3, Some(Uk))
            .capital(Uk).vc()
            .land(&[t::WEST_INDIA, t::BURMA])
            .sea(&[sz::SZ_OFF_INDIA, sz::SZ_BAY_OF_BENGAL])
            .convoys(&[sz::SZ_OFF_INDIA, sz::SZ_BAY_OF_BENGAL])
            .done(),

        // 115 — West India
        TB::new(t::WEST_INDIA, "West India", 1, Some(Uk))
            .land(&[t::INDIA, t::EASTERN_PERSIA])
            .sea(&[sz::SZ_OFF_WEST_INDIA])
            .done(),

        // 116 — Burma
        TB::new(t::BURMA, "Burma", 1, Some(Uk))
            .land(&[t::INDIA, t::SHAN_STATE, t::YUNNAN])
            .sea(&[sz::SZ_BAY_OF_BENGAL])
            .done(),

        // 117 — Shan State
        TB::new(t::SHAN_STATE, "Shan State", 1, Some(Uk))
            .land(&[t::BURMA, t::YUNNAN, t::SIAM, t::FRENCH_INDOCHINA, t::MALAYA])
            .done(),

        // 118 — Malaya (Singapore)
        TB::new(t::MALAYA, "Malaya", 3, Some(Uk))
            .vc()
            .land(&[t::SHAN_STATE, t::SIAM])
            .sea(&[sz::SZ_OFF_MALAYA])
            .done(),

        // 119 — Siam (Thailand)
        TB::new(t::SIAM, "Siam", 0, Some(Jp))
            .land(&[t::SHAN_STATE, t::MALAYA, t::FRENCH_INDOCHINA])
            .sea(&[sz::SZ_OFF_MALAYA, sz::SZ_OFF_FRENCH_INDOCHINA])
            .done(),

        // 120 — French Indochina
        TB::new(t::FRENCH_INDOCHINA, "French Indochina", 1, Some(Jp))
            .land(&[t::SHAN_STATE, t::SIAM, t::KWANGSI, t::YUNNAN])
            .sea(&[sz::SZ_OFF_FRENCH_INDOCHINA, sz::SZ_SOUTH_CHINA_SEA])
            .done(),

        // 121 — Ceylon
        TB::new(t::CEYLON, "Ceylon", 1, Some(Uk))
            .island()
            .sea(&[sz::SZ_OFF_CEYLON])
            .done(),

        // =================================================================
        // DUTCH EAST INDIES (IDs 122–125) — all islands
        // =================================================================

        // 122 — Borneo
        TB::new(t::BORNEO, "Borneo", 4, Some(Uk))
            .island()
            .sea(&[sz::SZ_OFF_BORNEO, sz::SZ_OFF_CELEBES])
            .done(),

        // 123 — Java
        TB::new(t::JAVA, "Java", 4, Some(Uk))
            .island()
            .sea(&[sz::SZ_OFF_JAVA])
            .done(),

        // 124 — Sumatra
        TB::new(t::SUMATRA, "Sumatra", 4, Some(Uk))
            .island()
            .sea(&[sz::SZ_OFF_SUMATRA, sz::SZ_OFF_MALAYA])
            .done(),

        // 125 — Celebes
        TB::new(t::CELEBES, "Celebes", 3, Some(Uk))
            .island()
            .sea(&[sz::SZ_OFF_CELEBES])
            .done(),

        // =================================================================
        // JAPAN AND PACIFIC ISLANDS (IDs 126–140)
        // =================================================================

        // 126 — Japan (Tokyo)
        TB::new(t::JAPAN, "Japan", 8, Some(Jp))
            .capital(Jp).vc()
            .island()
            .sea(&[sz::SZ_JAPAN_EAST, sz::SZ_SEA_OF_JAPAN])
            .convoys(&[sz::SZ_JAPAN_EAST, sz::SZ_SEA_OF_JAPAN])
            .done(),

        // 127 — Okinawa
        TB::new(t::OKINAWA, "Okinawa", 0, Some(Jp))
            .island()
            .sea(&[sz::SZ_OFF_OKINAWA])
            .done(),

        // 128 — Iwo Jima
        TB::new(t::IWO_JIMA, "Iwo Jima", 0, Some(Jp))
            .island()
            .sea(&[sz::SZ_OFF_IWO_JIMA])
            .done(),

        // 129 — Korea
        TB::new(t::KOREA, "Korea", 3, Some(Jp))
            .land(&[t::MANCHURIA])
            .sea(&[sz::SZ_SEA_OF_JAPAN, sz::SZ_YELLOW_SEA])
            .done(),

        // 130 — Formosa
        TB::new(t::FORMOSA, "Formosa", 1, Some(Jp))
            .island()
            .sea(&[sz::SZ_OFF_FORMOSA])
            .done(),

        // 131 — Caroline Islands
        TB::new(t::CAROLINE_ISLANDS, "Caroline Islands", 0, Some(Jp))
            .island()
            .sea(&[sz::SZ_OFF_CAROLINES])
            .done(),

        // 132 — Marshall Islands
        TB::new(t::MARSHALL_ISLANDS, "Marshall Islands", 0, Some(Jp))
            .island()
            .sea(&[sz::SZ_OFF_MARSHALLS])
            .done(),

        // 133 — Palau Island
        TB::new(t::PALAU_ISLAND, "Palau Island", 0, Some(Jp))
            .island()
            .sea(&[sz::SZ_OFF_PALAU])
            .done(),

        // 134 — Midway
        TB::new(t::MIDWAY, "Midway", 0, Some(Us))
            .island()
            .sea(&[sz::SZ_OFF_MIDWAY])
            .done(),

        // 135 — Wake Island
        TB::new(t::WAKE_ISLAND, "Wake Island", 0, Some(Jp))
            .island()
            .sea(&[sz::SZ_OFF_WAKE])
            .done(),

        // 136 — Hawaiian Islands (Honolulu)
        TB::new(t::HAWAIIAN_ISLANDS, "Hawaiian Islands", 1, Some(Us))
            .vc()
            .island()
            .sea(&[sz::SZ_OFF_HAWAII])
            .done(),

        // 137 — Johnston Island
        TB::new(t::JOHNSTON_ISLAND, "Johnston Island", 0, Some(Us))
            .island()
            .sea(&[sz::SZ_OFF_JOHNSTON])
            .done(),

        // 138 — Line Islands
        TB::new(t::LINE_ISLANDS, "Line Islands", 0, Some(Us))
            .island()
            .sea(&[sz::SZ_OFF_LINE_ISLANDS])
            .done(),

        // 139 — Guam
        TB::new(t::GUAM, "Guam", 0, Some(Us))
            .island()
            .sea(&[sz::SZ_OFF_GUAM])
            .done(),

        // 140 — Philippines (Manila)
        TB::new(t::PHILIPPINES, "Philippines", 2, Some(Us))
            .vc()
            .island()
            .sea(&[sz::SZ_OFF_PHILIPPINES])
            .done(),

        // =================================================================
        // AUSTRALIA AND NEW ZEALAND (IDs 141–150)
        // =================================================================

        // 141 — New South Wales (Sydney)
        TB::new(t::NEW_SOUTH_WALES, "New South Wales", 2, Some(An))
            .capital(An).vc()
            .land(&[t::QUEENSLAND, t::SOUTH_AUSTRALIA])
            .sea(&[sz::SZ_OFF_NEW_SOUTH_WALES])
            .convoys(&[sz::SZ_OFF_NEW_SOUTH_WALES, sz::SZ_CORAL_SEA])
            .done(),

        // 142 — Queensland
        TB::new(t::QUEENSLAND, "Queensland", 1, Some(An))
            .land(&[t::NEW_SOUTH_WALES, t::NORTHERN_TERRITORY])
            .sea(&[sz::SZ_CORAL_SEA])
            .done(),

        // 143 — South Australia
        TB::new(t::SOUTH_AUSTRALIA, "South Australia", 0, Some(An))
            .land(&[t::NEW_SOUTH_WALES, t::WESTERN_AUSTRALIA, t::NORTHERN_TERRITORY])
            .sea(&[sz::SZ_OFF_SOUTH_AUSTRALIA])
            .done(),

        // 144 — Western Australia
        TB::new(t::WESTERN_AUSTRALIA, "Western Australia", 1, Some(An))
            .land(&[t::SOUTH_AUSTRALIA, t::NORTHERN_TERRITORY])
            .sea(&[sz::SZ_OFF_WESTERN_AUSTRALIA, sz::SZ_OFF_SOUTH_AUSTRALIA])
            .done(),

        // 145 — Northern Territory
        TB::new(t::NORTHERN_TERRITORY, "Northern Territory", 0, Some(An))
            .land(&[t::QUEENSLAND, t::WESTERN_AUSTRALIA, t::SOUTH_AUSTRALIA])
            .sea(&[sz::SZ_OFF_NORTHERN_TERRITORY])
            .done(),

        // 146 — New Zealand (Auckland)
        TB::new(t::NEW_ZEALAND, "New Zealand", 2, Some(An))
            .vc()
            .island()
            .sea(&[sz::SZ_OFF_NEW_ZEALAND])
            .done(),

        // 147 — New Guinea
        TB::new(t::NEW_GUINEA, "New Guinea", 1, Some(An))
            .island()
            .sea(&[sz::SZ_OFF_NEW_GUINEA])
            .done(),

        // 148 — New Britain
        TB::new(t::NEW_BRITAIN, "New Britain", 0, Some(An))
            .island()
            .sea(&[sz::SZ_OFF_NEW_BRITAIN])
            .done(),

        // 149 — Solomon Islands
        TB::new(t::SOLOMON_ISLANDS, "Solomon Islands", 0, Some(An))
            .island()
            .sea(&[sz::SZ_OFF_NEW_BRITAIN])
            .done(),

        // 150 — Fiji
        TB::new(t::FIJI, "Fiji", 0, Some(An))
            .island()
            .sea(&[sz::SZ_OFF_FIJI])
            .done(),

        // =================================================================
        // AMERICAS (IDs 151–163)
        // =================================================================

        // 151 — Eastern United States (Washington)
        TB::new(t::EASTERN_UNITED_STATES, "Eastern United States", 20, Some(Us))
            .capital(Us).vc()
            .land(&[t::CENTRAL_UNITED_STATES])
            .sea(&[sz::SZ_OFF_EASTERN_US, sz::SZ_CARIBBEAN])
            .convoys(&[sz::SZ_OFF_EASTERN_US, sz::SZ_NORTH_ATLANTIC])
            .done(),

        // 152 — Central United States
        TB::new(t::CENTRAL_UNITED_STATES, "Central United States", 1, Some(Us))
            .land(&[t::EASTERN_UNITED_STATES, t::WESTERN_UNITED_STATES, t::MEXICO])
            .sea(&[sz::SZ_GULF_OF_MEXICO])
            .done(),

        // 153 — Western United States (San Francisco)
        TB::new(t::WESTERN_UNITED_STATES, "Western United States", 10, Some(Us))
            .vc()
            .land(&[t::CENTRAL_UNITED_STATES, t::MEXICO, t::ALASKA])
            .sea(&[sz::SZ_OFF_WESTERN_US])
            .done(),

        // 154 — Alaska
        TB::new(t::ALASKA, "Alaska", 2, Some(Us))
            .land(&[t::WESTERN_UNITED_STATES])
            .sea(&[sz::SZ_OFF_ALASKA])
            .done(),

        // 155 — Mexico
        TB::new(t::MEXICO, "Mexico", 0, None)
            .ttype(TerritoryType::ProAllies)
            .land(&[t::CENTRAL_UNITED_STATES, t::WESTERN_UNITED_STATES, t::CENTRAL_AMERICA])
            .sea(&[sz::SZ_GULF_OF_MEXICO, sz::SZ_OFF_WESTERN_US])
            .done(),

        // 156 — Central America
        TB::new(t::CENTRAL_AMERICA, "Central America", 0, Some(Us))
            .land(&[t::MEXICO, t::COLOMBIA_ECUADOR])
            .sea(&[sz::SZ_CARIBBEAN, sz::SZ_OFF_WESTERN_US])
            .done(),

        // 157 — West Indies
        TB::new(t::WEST_INDIES, "West Indies", 1, Some(Us))
            .island()
            .sea(&[sz::SZ_CARIBBEAN])
            .done(),

        // 158 — Greenland
        TB::new(t::GREENLAND, "Greenland", 0, Some(Us))
            .island()
            .sea(&[sz::SZ_OFF_GREENLAND])
            .done(),

        // 159 — Brazil
        TB::new(t::BRAZIL, "Brazil", 0, None)
            .ttype(TerritoryType::ProAllies)
            .land(&[t::VENEZUELA])
            .sea(&[sz::SZ_OFF_BRAZIL])
            .done(),

        // 160 — Venezuela
        TB::new(t::VENEZUELA, "Venezuela", 0, None)
            .ttype(TerritoryType::ProAllies)
            .land(&[t::COLOMBIA_ECUADOR, t::BRAZIL])
            .sea(&[sz::SZ_CARIBBEAN, sz::SZ_OFF_BRAZIL])
            .done(),

        // 161 — Colombia/Ecuador
        TB::new(t::COLOMBIA_ECUADOR, "Colombia/Ecuador", 0, None)
            .ttype(TerritoryType::ProAllies)
            .land(&[t::VENEZUELA, t::CENTRAL_AMERICA, t::PERU])
            .sea(&[sz::SZ_CARIBBEAN])
            .done(),

        // 162 — Argentina/Chile
        TB::new(t::ARGENTINA_CHILE, "Argentina/Chile", 0, None)
            .ttype(TerritoryType::TrueNeutral)
            .land(&[t::PERU])
            .sea(&[sz::SZ_SOUTH_PACIFIC, sz::SZ_OFF_BRAZIL])
            .done(),

        // 163 — Peru
        TB::new(t::PERU, "Peru", 0, None)
            .ttype(TerritoryType::TrueNeutral)
            .land(&[t::COLOMBIA_ECUADOR, t::ARGENTINA_CHILE])
            .sea(&[sz::SZ_CENTRAL_PACIFIC_SOUTH])
            .done(),
    ]
}
