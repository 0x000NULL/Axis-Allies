//! All 80 sea zone definitions for Global 1940 2nd Edition.
//!
//! Each entry includes adjacent sea zones, adjacent land territories,
//! and convoy zone status.

use crate::territory::{SeaZoneDef, SeaZoneId, TerritoryId};

use super::territory_ids as t;
use super::sea_zone_ids as sz;

// ---------------------------------------------------------------------------
// Builder helper
// ---------------------------------------------------------------------------

struct SZB(SeaZoneDef);

impl SZB {
    fn new(id: SeaZoneId, name: &str) -> Self {
        SZB(SeaZoneDef {
            id,
            name: name.to_string(),
            adjacent_sea: vec![],
            adjacent_land: vec![],
            is_convoy_zone: false,
        })
    }

    fn adj_sea(mut self, ids: &[SeaZoneId]) -> Self {
        self.0.adjacent_sea = ids.to_vec();
        self
    }
    fn adj_land(mut self, ids: &[TerritoryId]) -> Self {
        self.0.adjacent_land = ids.to_vec();
        self
    }
    fn convoy(mut self) -> Self {
        self.0.is_convoy_zone = true;
        self
    }
    fn done(self) -> SeaZoneDef {
        self.0
    }
}

/// Build all sea zone definitions. Indices match sea zone IDs.
pub fn build_sea_zone_defs() -> Vec<SeaZoneDef> {
    vec![
        // =================================================================
        // PACIFIC OCEAN (IDs 0–45)
        // =================================================================

        // 0 — Off Japan (east coast) — Board SZ 6
        SZB::new(sz::SZ_JAPAN_EAST, "Sea Zone 6")
            .adj_sea(&[sz::SZ_SEA_OF_JAPAN, sz::SZ_OFF_IWO_JIMA, sz::SZ_NORTH_PACIFIC,
                        sz::SZ_OFF_OKINAWA])
            .adj_land(&[t::JAPAN])
            .convoy()
            .done(),

        // 1 — Sea of Japan — Board SZ 5
        SZB::new(sz::SZ_SEA_OF_JAPAN, "Sea Zone 5")
            .adj_sea(&[sz::SZ_JAPAN_EAST, sz::SZ_YELLOW_SEA, sz::SZ_OFF_MANCHURIA,
                        sz::SZ_NORTH_PACIFIC, sz::SZ_SEA_OF_OKHOTSK])
            .adj_land(&[t::JAPAN, t::KOREA])
            .convoy()
            .done(),

        // 2 — Yellow Sea — Board SZ 4
        SZB::new(sz::SZ_YELLOW_SEA, "Sea Zone 4")
            .adj_sea(&[sz::SZ_SEA_OF_JAPAN, sz::SZ_EAST_CHINA_SEA, sz::SZ_OFF_FORMOSA])
            .adj_land(&[t::KOREA, t::SHANTUNG])
            .done(),

        // 3 — East China Sea — Board SZ 3
        SZB::new(sz::SZ_EAST_CHINA_SEA, "Sea Zone 3")
            .adj_sea(&[sz::SZ_YELLOW_SEA, sz::SZ_OFF_FORMOSA, sz::SZ_SOUTH_CHINA_SEA])
            .adj_land(&[t::KIANGSU])
            .done(),

        // 4 — Off Formosa — Board SZ 20
        SZB::new(sz::SZ_OFF_FORMOSA, "Sea Zone 20")
            .adj_sea(&[sz::SZ_YELLOW_SEA, sz::SZ_EAST_CHINA_SEA, sz::SZ_SOUTH_CHINA_SEA,
                        sz::SZ_OFF_OKINAWA, sz::SZ_OFF_PHILIPPINES])
            .adj_land(&[t::FORMOSA])
            .done(),

        // 5 — South China Sea — Board SZ 19
        SZB::new(sz::SZ_SOUTH_CHINA_SEA, "Sea Zone 19")
            .adj_sea(&[sz::SZ_EAST_CHINA_SEA, sz::SZ_OFF_FORMOSA,
                        sz::SZ_OFF_FRENCH_INDOCHINA, sz::SZ_OFF_BORNEO,
                        sz::SZ_OFF_PHILIPPINES])
            .adj_land(&[t::KWANGTUNG, t::FRENCH_INDOCHINA])
            .done(),

        // 6 — Off French Indochina — Board SZ 21
        SZB::new(sz::SZ_OFF_FRENCH_INDOCHINA, "Sea Zone 21")
            .adj_sea(&[sz::SZ_SOUTH_CHINA_SEA, sz::SZ_OFF_MALAYA, sz::SZ_OFF_BORNEO])
            .adj_land(&[t::FRENCH_INDOCHINA, t::SIAM])
            .done(),

        // 7 — Off Malaya — Board SZ 22
        SZB::new(sz::SZ_OFF_MALAYA, "Sea Zone 22")
            .adj_sea(&[sz::SZ_OFF_FRENCH_INDOCHINA, sz::SZ_OFF_BORNEO,
                        sz::SZ_OFF_JAVA, sz::SZ_OFF_SUMATRA, sz::SZ_BAY_OF_BENGAL])
            .adj_land(&[t::MALAYA, t::SIAM, t::SUMATRA])
            .done(),

        // 8 — Off Borneo — Board SZ 23
        SZB::new(sz::SZ_OFF_BORNEO, "Sea Zone 23")
            .adj_sea(&[sz::SZ_SOUTH_CHINA_SEA, sz::SZ_OFF_FRENCH_INDOCHINA,
                        sz::SZ_OFF_MALAYA, sz::SZ_OFF_CELEBES, sz::SZ_OFF_JAVA,
                        sz::SZ_OFF_PHILIPPINES])
            .adj_land(&[t::BORNEO])
            .done(),

        // 9 — Off Celebes — Board SZ 24
        SZB::new(sz::SZ_OFF_CELEBES, "Sea Zone 24")
            .adj_sea(&[sz::SZ_OFF_BORNEO, sz::SZ_OFF_JAVA, sz::SZ_OFF_NEW_GUINEA,
                        sz::SZ_OFF_PALAU, sz::SZ_OFF_PHILIPPINES])
            .adj_land(&[t::CELEBES, t::BORNEO])
            .done(),

        // 10 — Off Java — Board SZ 25
        SZB::new(sz::SZ_OFF_JAVA, "Sea Zone 25")
            .adj_sea(&[sz::SZ_OFF_MALAYA, sz::SZ_OFF_BORNEO, sz::SZ_OFF_CELEBES,
                        sz::SZ_OFF_SUMATRA, sz::SZ_CENTRAL_INDIAN_OCEAN])
            .adj_land(&[t::JAVA])
            .done(),

        // 11 — Off Sumatra — Board SZ 26
        SZB::new(sz::SZ_OFF_SUMATRA, "Sea Zone 26")
            .adj_sea(&[sz::SZ_OFF_MALAYA, sz::SZ_OFF_JAVA, sz::SZ_BAY_OF_BENGAL,
                        sz::SZ_OFF_CEYLON, sz::SZ_CENTRAL_INDIAN_OCEAN])
            .adj_land(&[t::SUMATRA])
            .done(),

        // 12 — Bay of Bengal — Board SZ 37
        SZB::new(sz::SZ_BAY_OF_BENGAL, "Sea Zone 37")
            .adj_sea(&[sz::SZ_OFF_MALAYA, sz::SZ_OFF_SUMATRA, sz::SZ_OFF_INDIA,
                        sz::SZ_OFF_CEYLON])
            .adj_land(&[t::INDIA, t::BURMA])
            .convoy()
            .done(),

        // 13 — Off India — Board SZ 38
        SZB::new(sz::SZ_OFF_INDIA, "Sea Zone 38")
            .adj_sea(&[sz::SZ_BAY_OF_BENGAL, sz::SZ_OFF_WEST_INDIA, sz::SZ_OFF_CEYLON])
            .adj_land(&[t::INDIA])
            .convoy()
            .done(),

        // 14 — Off West India — Board SZ 39
        SZB::new(sz::SZ_OFF_WEST_INDIA, "Sea Zone 39")
            .adj_sea(&[sz::SZ_OFF_INDIA, sz::SZ_OFF_CEYLON, sz::SZ_PERSIAN_GULF,
                        sz::SZ_CENTRAL_INDIAN_OCEAN])
            .adj_land(&[t::WEST_INDIA])
            .done(),

        // 15 — Off Ceylon — Board SZ 40
        SZB::new(sz::SZ_OFF_CEYLON, "Sea Zone 40")
            .adj_sea(&[sz::SZ_BAY_OF_BENGAL, sz::SZ_OFF_INDIA, sz::SZ_OFF_WEST_INDIA,
                        sz::SZ_OFF_SUMATRA, sz::SZ_CENTRAL_INDIAN_OCEAN])
            .adj_land(&[t::CEYLON])
            .done(),

        // 16 — Central Indian Ocean — Board SZ 41
        SZB::new(sz::SZ_CENTRAL_INDIAN_OCEAN, "Sea Zone 41")
            .adj_sea(&[sz::SZ_OFF_CEYLON, sz::SZ_OFF_WEST_INDIA, sz::SZ_OFF_SUMATRA,
                        sz::SZ_OFF_JAVA, sz::SZ_OFF_WESTERN_AUSTRALIA,
                        sz::SZ_OFF_MADAGASCAR, sz::SZ_OFF_MOZAMBIQUE,
                        sz::SZ_OFF_SOUTH_AFRICA_EAST])
            .done(),

        // 17 — Off Western Australia — Board SZ 42
        SZB::new(sz::SZ_OFF_WESTERN_AUSTRALIA, "Sea Zone 42")
            .adj_sea(&[sz::SZ_CENTRAL_INDIAN_OCEAN, sz::SZ_OFF_SOUTH_AUSTRALIA,
                        sz::SZ_OFF_NORTHERN_TERRITORY])
            .adj_land(&[t::WESTERN_AUSTRALIA])
            .done(),

        // 18 — Off South Australia — Board SZ 43
        SZB::new(sz::SZ_OFF_SOUTH_AUSTRALIA, "Sea Zone 43")
            .adj_sea(&[sz::SZ_OFF_WESTERN_AUSTRALIA, sz::SZ_OFF_NEW_SOUTH_WALES,
                        sz::SZ_OFF_NEW_ZEALAND, sz::SZ_SOUTH_PACIFIC])
            .adj_land(&[t::SOUTH_AUSTRALIA, t::WESTERN_AUSTRALIA])
            .done(),

        // 19 — Off New South Wales — Board SZ 44
        SZB::new(sz::SZ_OFF_NEW_SOUTH_WALES, "Sea Zone 44")
            .adj_sea(&[sz::SZ_OFF_SOUTH_AUSTRALIA, sz::SZ_CORAL_SEA,
                        sz::SZ_OFF_NEW_ZEALAND])
            .adj_land(&[t::NEW_SOUTH_WALES])
            .convoy()
            .done(),

        // 20 — Coral Sea — Board SZ 45
        SZB::new(sz::SZ_CORAL_SEA, "Sea Zone 45")
            .adj_sea(&[sz::SZ_OFF_NEW_SOUTH_WALES, sz::SZ_OFF_NEW_GUINEA,
                        sz::SZ_OFF_NEW_BRITAIN, sz::SZ_OFF_FIJI,
                        sz::SZ_OFF_NORTHERN_TERRITORY])
            .adj_land(&[t::QUEENSLAND])
            .convoy()
            .done(),

        // 21 — Off New Guinea — Board SZ 46
        SZB::new(sz::SZ_OFF_NEW_GUINEA, "Sea Zone 46")
            .adj_sea(&[sz::SZ_CORAL_SEA, sz::SZ_OFF_NEW_BRITAIN, sz::SZ_OFF_CELEBES,
                        sz::SZ_OFF_NORTHERN_TERRITORY, sz::SZ_OFF_PALAU])
            .adj_land(&[t::NEW_GUINEA])
            .done(),

        // 22 — Off New Britain / Solomons — Board SZ 47
        SZB::new(sz::SZ_OFF_NEW_BRITAIN, "Sea Zone 47")
            .adj_sea(&[sz::SZ_CORAL_SEA, sz::SZ_OFF_NEW_GUINEA, sz::SZ_OFF_CAROLINES,
                        sz::SZ_OFF_MARSHALLS, sz::SZ_OFF_FIJI])
            .adj_land(&[t::NEW_BRITAIN, t::SOLOMON_ISLANDS])
            .done(),

        // 23 — Off Caroline Islands — Board SZ 33
        SZB::new(sz::SZ_OFF_CAROLINES, "Sea Zone 33")
            .adj_sea(&[sz::SZ_OFF_NEW_BRITAIN, sz::SZ_OFF_PALAU, sz::SZ_OFF_MARSHALLS,
                        sz::SZ_OFF_GUAM])
            .adj_land(&[t::CAROLINE_ISLANDS])
            .done(),

        // 24 — Off Palau — Board SZ 34
        SZB::new(sz::SZ_OFF_PALAU, "Sea Zone 34")
            .adj_sea(&[sz::SZ_OFF_CAROLINES, sz::SZ_OFF_CELEBES, sz::SZ_OFF_PHILIPPINES,
                        sz::SZ_OFF_NEW_GUINEA])
            .adj_land(&[t::PALAU_ISLAND])
            .done(),

        // 25 — Off Philippines — Board SZ 35
        SZB::new(sz::SZ_OFF_PHILIPPINES, "Sea Zone 35")
            .adj_sea(&[sz::SZ_OFF_FORMOSA, sz::SZ_SOUTH_CHINA_SEA, sz::SZ_OFF_BORNEO,
                        sz::SZ_OFF_CELEBES, sz::SZ_OFF_PALAU, sz::SZ_OFF_GUAM])
            .adj_land(&[t::PHILIPPINES])
            .done(),

        // 26 — Off Guam — Board SZ 32
        SZB::new(sz::SZ_OFF_GUAM, "Sea Zone 32")
            .adj_sea(&[sz::SZ_OFF_CAROLINES, sz::SZ_OFF_PHILIPPINES, sz::SZ_OFF_MARSHALLS,
                        sz::SZ_OFF_WAKE, sz::SZ_OFF_IWO_JIMA])
            .adj_land(&[t::GUAM])
            .done(),

        // 27 — Off Marshall Islands — Board SZ 31
        SZB::new(sz::SZ_OFF_MARSHALLS, "Sea Zone 31")
            .adj_sea(&[sz::SZ_OFF_CAROLINES, sz::SZ_OFF_NEW_BRITAIN, sz::SZ_OFF_GUAM,
                        sz::SZ_OFF_WAKE, sz::SZ_OFF_FIJI, sz::SZ_OFF_LINE_ISLANDS,
                        sz::SZ_OFF_JOHNSTON])
            .adj_land(&[t::MARSHALL_ISLANDS])
            .done(),

        // 28 — Off Wake Island — Board SZ 30
        SZB::new(sz::SZ_OFF_WAKE, "Sea Zone 30")
            .adj_sea(&[sz::SZ_OFF_GUAM, sz::SZ_OFF_MARSHALLS, sz::SZ_OFF_MIDWAY,
                        sz::SZ_OFF_IWO_JIMA, sz::SZ_OFF_JOHNSTON])
            .adj_land(&[t::WAKE_ISLAND])
            .done(),

        // 29 — Off Midway — Board SZ 29
        SZB::new(sz::SZ_OFF_MIDWAY, "Sea Zone 29")
            .adj_sea(&[sz::SZ_OFF_WAKE, sz::SZ_OFF_HAWAII, sz::SZ_OFF_JOHNSTON,
                        sz::SZ_NORTH_PACIFIC, sz::SZ_NE_PACIFIC])
            .adj_land(&[t::MIDWAY])
            .done(),

        // 30 — Off Iwo Jima — Board SZ 7
        SZB::new(sz::SZ_OFF_IWO_JIMA, "Sea Zone 7")
            .adj_sea(&[sz::SZ_JAPAN_EAST, sz::SZ_OFF_OKINAWA, sz::SZ_OFF_GUAM,
                        sz::SZ_OFF_WAKE, sz::SZ_NORTH_PACIFIC])
            .adj_land(&[t::IWO_JIMA])
            .done(),

        // 31 — Off Okinawa — Board SZ 8
        SZB::new(sz::SZ_OFF_OKINAWA, "Sea Zone 8")
            .adj_sea(&[sz::SZ_JAPAN_EAST, sz::SZ_OFF_IWO_JIMA, sz::SZ_OFF_FORMOSA])
            .adj_land(&[t::OKINAWA])
            .done(),

        // 32 — North Pacific — Board SZ 9
        SZB::new(sz::SZ_NORTH_PACIFIC, "Sea Zone 9")
            .adj_sea(&[sz::SZ_JAPAN_EAST, sz::SZ_SEA_OF_JAPAN, sz::SZ_SEA_OF_OKHOTSK,
                        sz::SZ_OFF_IWO_JIMA, sz::SZ_OFF_MIDWAY, sz::SZ_NE_PACIFIC,
                        sz::SZ_OFF_ALASKA])
            .done(),

        // 33 — Sea of Okhotsk — Board SZ 10
        SZB::new(sz::SZ_SEA_OF_OKHOTSK, "Sea Zone 10")
            .adj_sea(&[sz::SZ_NORTH_PACIFIC, sz::SZ_OFF_MANCHURIA, sz::SZ_SEA_OF_JAPAN,
                        sz::SZ_OFF_ALASKA])
            .adj_land(&[t::SAKHA, t::SOVIET_FAR_EAST, t::AMUR])
            .done(),

        // 34 — Off Alaska — Board SZ 1
        SZB::new(sz::SZ_OFF_ALASKA, "Sea Zone 1")
            .adj_sea(&[sz::SZ_NORTH_PACIFIC, sz::SZ_SEA_OF_OKHOTSK, sz::SZ_NE_PACIFIC])
            .adj_land(&[t::ALASKA])
            .done(),

        // 35 — Northeast Pacific — Board SZ 2
        SZB::new(sz::SZ_NE_PACIFIC, "Sea Zone 2")
            .adj_sea(&[sz::SZ_OFF_ALASKA, sz::SZ_NORTH_PACIFIC, sz::SZ_OFF_MIDWAY,
                        sz::SZ_OFF_WESTERN_US])
            .done(),

        // 36 — Off Western US — Board SZ 11
        SZB::new(sz::SZ_OFF_WESTERN_US, "Sea Zone 11")
            .adj_sea(&[sz::SZ_NE_PACIFIC, sz::SZ_OFF_HAWAII, sz::SZ_GULF_OF_MEXICO,
                        sz::SZ_CARIBBEAN])
            .adj_land(&[t::WESTERN_UNITED_STATES, t::MEXICO, t::CENTRAL_AMERICA])
            .done(),

        // 37 — Off Hawaii — Board SZ 28
        SZB::new(sz::SZ_OFF_HAWAII, "Sea Zone 28")
            .adj_sea(&[sz::SZ_OFF_WESTERN_US, sz::SZ_OFF_MIDWAY, sz::SZ_OFF_JOHNSTON,
                        sz::SZ_OFF_LINE_ISLANDS])
            .adj_land(&[t::HAWAIIAN_ISLANDS])
            .done(),

        // 38 — Off Johnston Island — Board SZ 27
        SZB::new(sz::SZ_OFF_JOHNSTON, "Sea Zone 27")
            .adj_sea(&[sz::SZ_OFF_HAWAII, sz::SZ_OFF_WAKE, sz::SZ_OFF_MIDWAY,
                        sz::SZ_OFF_LINE_ISLANDS, sz::SZ_OFF_MARSHALLS])
            .adj_land(&[t::JOHNSTON_ISLAND])
            .done(),

        // 39 — Off Line Islands — Board SZ 50
        SZB::new(sz::SZ_OFF_LINE_ISLANDS, "Sea Zone 50")
            .adj_sea(&[sz::SZ_OFF_HAWAII, sz::SZ_OFF_JOHNSTON, sz::SZ_OFF_MARSHALLS,
                        sz::SZ_CENTRAL_PACIFIC_SOUTH, sz::SZ_OFF_FIJI])
            .adj_land(&[t::LINE_ISLANDS])
            .done(),

        // 40 — Central Pacific South — Board SZ 51
        SZB::new(sz::SZ_CENTRAL_PACIFIC_SOUTH, "Sea Zone 51")
            .adj_sea(&[sz::SZ_OFF_LINE_ISLANDS, sz::SZ_OFF_FIJI, sz::SZ_SOUTH_PACIFIC])
            .adj_land(&[t::PERU])
            .done(),

        // 41 — Off Fiji — Board SZ 52
        SZB::new(sz::SZ_OFF_FIJI, "Sea Zone 52")
            .adj_sea(&[sz::SZ_CORAL_SEA, sz::SZ_OFF_NEW_BRITAIN, sz::SZ_OFF_MARSHALLS,
                        sz::SZ_OFF_LINE_ISLANDS, sz::SZ_CENTRAL_PACIFIC_SOUTH,
                        sz::SZ_OFF_NEW_ZEALAND])
            .adj_land(&[t::FIJI])
            .done(),

        // 42 — Off New Zealand — Board SZ 53
        SZB::new(sz::SZ_OFF_NEW_ZEALAND, "Sea Zone 53")
            .adj_sea(&[sz::SZ_OFF_SOUTH_AUSTRALIA, sz::SZ_OFF_NEW_SOUTH_WALES,
                        sz::SZ_OFF_FIJI, sz::SZ_SOUTH_PACIFIC])
            .adj_land(&[t::NEW_ZEALAND])
            .done(),

        // 43 — South Pacific — Board SZ 54
        SZB::new(sz::SZ_SOUTH_PACIFIC, "Sea Zone 54")
            .adj_sea(&[sz::SZ_OFF_SOUTH_AUSTRALIA, sz::SZ_OFF_NEW_ZEALAND,
                        sz::SZ_CENTRAL_PACIFIC_SOUTH,
                        sz::SZ_OFF_SOUTH_AFRICA_WEST])
            .adj_land(&[t::ARGENTINA_CHILE])
            .done(),

        // 44 — Off Northern Territory — Board SZ 48
        SZB::new(sz::SZ_OFF_NORTHERN_TERRITORY, "Sea Zone 48")
            .adj_sea(&[sz::SZ_OFF_WESTERN_AUSTRALIA, sz::SZ_CORAL_SEA,
                        sz::SZ_OFF_NEW_GUINEA])
            .adj_land(&[t::NORTHERN_TERRITORY])
            .done(),

        // 45 — Off Manchuria — Board SZ 12
        SZB::new(sz::SZ_OFF_MANCHURIA, "Sea Zone 12")
            .adj_sea(&[sz::SZ_SEA_OF_JAPAN, sz::SZ_SEA_OF_OKHOTSK])
            .adj_land(&[t::MANCHURIA, t::AMUR])
            .done(),

        // =================================================================
        // INDIAN OCEAN (IDs 46–51)
        // =================================================================

        // 46 — Persian Gulf — Board SZ 80
        SZB::new(sz::SZ_PERSIAN_GULF, "Sea Zone 80")
            .adj_sea(&[sz::SZ_OFF_WEST_INDIA, sz::SZ_RED_SEA])
            .adj_land(&[t::IRAQ, t::PERSIA, t::SAUDI_ARABIA])
            .done(),

        // 47 — Red Sea — Board SZ 81
        SZB::new(sz::SZ_RED_SEA, "Sea Zone 81")
            .adj_sea(&[sz::SZ_PERSIAN_GULF, sz::SZ_OFF_EGYPT, sz::SZ_OFF_EAST_AFRICA])
            .adj_land(&[t::ANGLO_EGYPTIAN_SUDAN, t::ETHIOPIA, t::TRANS_JORDAN,
                         t::SAUDI_ARABIA])
            .done(),

        // 48 — Off East Africa — Board SZ 82
        SZB::new(sz::SZ_OFF_EAST_AFRICA, "Sea Zone 82")
            .adj_sea(&[sz::SZ_RED_SEA, sz::SZ_OFF_MADAGASCAR, sz::SZ_OFF_MOZAMBIQUE])
            .adj_land(&[t::ITALIAN_SOMALILAND, t::BRITISH_SOMALILAND, t::KENYA,
                         t::TANGANYIKA])
            .done(),

        // 49 — Off Madagascar — Board SZ 83
        SZB::new(sz::SZ_OFF_MADAGASCAR, "Sea Zone 83")
            .adj_sea(&[sz::SZ_OFF_EAST_AFRICA, sz::SZ_OFF_MOZAMBIQUE,
                        sz::SZ_CENTRAL_INDIAN_OCEAN])
            .adj_land(&[t::MADAGASCAR])
            .done(),

        // 50 — Off Mozambique — Board SZ 84
        SZB::new(sz::SZ_OFF_MOZAMBIQUE, "Sea Zone 84")
            .adj_sea(&[sz::SZ_OFF_EAST_AFRICA, sz::SZ_OFF_MADAGASCAR,
                        sz::SZ_OFF_SOUTH_AFRICA_EAST, sz::SZ_CENTRAL_INDIAN_OCEAN])
            .adj_land(&[t::MOZAMBIQUE])
            .done(),

        // 51 — Off South Africa (Indian Ocean side) — Board SZ 85
        SZB::new(sz::SZ_OFF_SOUTH_AFRICA_EAST, "Sea Zone 85")
            .adj_sea(&[sz::SZ_OFF_MOZAMBIQUE, sz::SZ_OFF_SOUTH_AFRICA_WEST,
                        sz::SZ_CENTRAL_INDIAN_OCEAN])
            .adj_land(&[t::SOUTH_AFRICA])
            .done(),

        // =================================================================
        // ATLANTIC OCEAN (IDs 52–63)
        // =================================================================

        // 52 — Off South Africa (Atlantic side) — Board SZ 86
        SZB::new(sz::SZ_OFF_SOUTH_AFRICA_WEST, "Sea Zone 86")
            .adj_sea(&[sz::SZ_OFF_SOUTH_AFRICA_EAST, sz::SZ_OFF_SOUTHWEST_AFRICA,
                        sz::SZ_SOUTH_PACIFIC])
            .adj_land(&[t::SOUTH_AFRICA])
            .done(),

        // 53 — Off Southwest Africa — Board SZ 87
        SZB::new(sz::SZ_OFF_SOUTHWEST_AFRICA, "Sea Zone 87")
            .adj_sea(&[sz::SZ_OFF_SOUTH_AFRICA_WEST, sz::SZ_OFF_ANGOLA])
            .adj_land(&[t::SOUTHWEST_AFRICA])
            .done(),

        // 54 — Off Angola — Board SZ 88
        SZB::new(sz::SZ_OFF_ANGOLA, "Sea Zone 88")
            .adj_sea(&[sz::SZ_OFF_SOUTHWEST_AFRICA, sz::SZ_OFF_FRENCH_WEST_AFRICA,
                        sz::SZ_CENTRAL_ATLANTIC])
            .adj_land(&[t::BELGIAN_CONGO, t::FRENCH_EQUATORIAL_AFRICA, t::ANGOLA])
            .done(),

        // 55 — Off French West Africa — Board SZ 89
        SZB::new(sz::SZ_OFF_FRENCH_WEST_AFRICA, "Sea Zone 89")
            .adj_sea(&[sz::SZ_OFF_ANGOLA, sz::SZ_CENTRAL_ATLANTIC, sz::SZ_OFF_MOROCCO])
            .adj_land(&[t::FRENCH_WEST_AFRICA])
            .done(),

        // 56 — Central Atlantic — Board SZ 90
        SZB::new(sz::SZ_CENTRAL_ATLANTIC, "Sea Zone 90")
            .adj_sea(&[sz::SZ_OFF_ANGOLA, sz::SZ_OFF_FRENCH_WEST_AFRICA,
                        sz::SZ_OFF_BRAZIL, sz::SZ_OFF_MOROCCO])
            .done(),

        // 57 — Off Brazil — Board SZ 91
        SZB::new(sz::SZ_OFF_BRAZIL, "Sea Zone 91")
            .adj_sea(&[sz::SZ_CENTRAL_ATLANTIC, sz::SZ_CARIBBEAN])
            .adj_land(&[t::BRAZIL, t::VENEZUELA, t::ARGENTINA_CHILE])
            .done(),

        // 58 — Caribbean Sea — Board SZ 92
        SZB::new(sz::SZ_CARIBBEAN, "Sea Zone 92")
            .adj_sea(&[sz::SZ_OFF_BRAZIL, sz::SZ_GULF_OF_MEXICO, sz::SZ_OFF_EASTERN_US,
                        sz::SZ_OFF_WESTERN_US])
            .adj_land(&[t::EASTERN_UNITED_STATES, t::CENTRAL_AMERICA, t::WEST_INDIES,
                         t::VENEZUELA, t::COLOMBIA_ECUADOR])
            .done(),

        // 59 — Gulf of Mexico — Board SZ 93
        SZB::new(sz::SZ_GULF_OF_MEXICO, "Sea Zone 93")
            .adj_sea(&[sz::SZ_CARIBBEAN, sz::SZ_OFF_WESTERN_US, sz::SZ_OFF_EASTERN_US])
            .adj_land(&[t::CENTRAL_UNITED_STATES, t::MEXICO])
            .done(),

        // 60 — Off Eastern United States — Board SZ 101
        SZB::new(sz::SZ_OFF_EASTERN_US, "Sea Zone 101")
            .adj_sea(&[sz::SZ_CARIBBEAN, sz::SZ_GULF_OF_MEXICO, sz::SZ_NORTH_ATLANTIC])
            .adj_land(&[t::EASTERN_UNITED_STATES])
            .convoy()
            .done(),

        // 61 — North Atlantic — Board SZ 102
        SZB::new(sz::SZ_NORTH_ATLANTIC, "Sea Zone 102")
            .adj_sea(&[sz::SZ_OFF_EASTERN_US, sz::SZ_OFF_GREENLAND, sz::SZ_OFF_ICELAND,
                        sz::SZ_BAY_OF_BISCAY])
            .convoy()
            .done(),

        // 62 — Off Greenland — Board SZ 103
        SZB::new(sz::SZ_OFF_GREENLAND, "Sea Zone 103")
            .adj_sea(&[sz::SZ_NORTH_ATLANTIC, sz::SZ_OFF_ICELAND])
            .adj_land(&[t::GREENLAND])
            .done(),

        // 63 — Off Iceland — Board SZ 104
        SZB::new(sz::SZ_OFF_ICELAND, "Sea Zone 104")
            .adj_sea(&[sz::SZ_OFF_GREENLAND, sz::SZ_NORTH_ATLANTIC, sz::SZ_NORWEGIAN_SEA])
            .adj_land(&[t::ICELAND])
            .done(),

        // =================================================================
        // EUROPE / MEDITERRANEAN (IDs 64–79)
        // =================================================================

        // 64 — Norwegian Sea — Board SZ 105
        SZB::new(sz::SZ_NORWEGIAN_SEA, "Sea Zone 105")
            .adj_sea(&[sz::SZ_OFF_ICELAND, sz::SZ_NORTH_SEA, sz::SZ_SKAGERRAK,
                        sz::SZ_BARENTS_SEA])
            .adj_land(&[t::SCOTLAND, t::NORWAY])
            .convoy()
            .done(),

        // 65 — North Sea — Board SZ 106
        SZB::new(sz::SZ_NORTH_SEA, "Sea Zone 106")
            .adj_sea(&[sz::SZ_NORWEGIAN_SEA, sz::SZ_SKAGERRAK, sz::SZ_ENGLISH_CHANNEL])
            .adj_land(&[t::UNITED_KINGDOM, t::SCOTLAND, t::WESTERN_GERMANY,
                         t::HOLLAND_BELGIUM, t::NORWAY])
            .convoy()
            .done(),

        // 66 — Skagerrak — Board SZ 107
        SZB::new(sz::SZ_SKAGERRAK, "Sea Zone 107")
            .adj_sea(&[sz::SZ_NORWEGIAN_SEA, sz::SZ_NORTH_SEA, sz::SZ_BALTIC_SEA])
            .adj_land(&[t::DENMARK, t::NORWAY, t::SWEDEN])
            .done(),

        // 67 — Baltic Sea — Board SZ 108
        SZB::new(sz::SZ_BALTIC_SEA, "Sea Zone 108")
            .adj_sea(&[sz::SZ_SKAGERRAK])
            .adj_land(&[t::GERMANY, t::DENMARK, t::SWEDEN, t::FINLAND,
                         t::POLAND, t::BALTIC_STATES, t::NOVGOROD, t::VYBORG])
            .convoy()
            .done(),

        // 68 — English Channel — Board SZ 109
        SZB::new(sz::SZ_ENGLISH_CHANNEL, "Sea Zone 109")
            .adj_sea(&[sz::SZ_NORTH_SEA, sz::SZ_BAY_OF_BISCAY])
            .adj_land(&[t::UNITED_KINGDOM, t::HOLLAND_BELGIUM, t::NORMANDY_BORDEAUX,
                         t::EIRE])
            .convoy()
            .done(),

        // 69 — Bay of Biscay — Board SZ 110
        SZB::new(sz::SZ_BAY_OF_BISCAY, "Sea Zone 110")
            .adj_sea(&[sz::SZ_ENGLISH_CHANNEL, sz::SZ_OFF_GIBRALTAR,
                        sz::SZ_NORTH_ATLANTIC])
            .adj_land(&[t::NORMANDY_BORDEAUX, t::SPAIN, t::PORTUGAL, t::EIRE])
            .done(),

        // 70 — Off Gibraltar — Board SZ 111
        SZB::new(sz::SZ_OFF_GIBRALTAR, "Sea Zone 111")
            .adj_sea(&[sz::SZ_BAY_OF_BISCAY, sz::SZ_OFF_MOROCCO, sz::SZ_WESTERN_MED])
            .adj_land(&[t::GIBRALTAR, t::SPAIN, t::PORTUGAL, t::MOROCCO])
            .done(),

        // 71 — Off Morocco — Board SZ 112
        SZB::new(sz::SZ_OFF_MOROCCO, "Sea Zone 112")
            .adj_sea(&[sz::SZ_OFF_GIBRALTAR, sz::SZ_WESTERN_MED,
                        sz::SZ_OFF_FRENCH_WEST_AFRICA, sz::SZ_CENTRAL_ATLANTIC])
            .adj_land(&[t::MOROCCO, t::ALGERIA, t::FRENCH_WEST_AFRICA, t::SPAIN])
            .done(),

        // 72 — Western Mediterranean — Board SZ 113
        SZB::new(sz::SZ_WESTERN_MED, "Sea Zone 113")
            .adj_sea(&[sz::SZ_OFF_GIBRALTAR, sz::SZ_OFF_MOROCCO, sz::SZ_TYRRHENIAN_SEA])
            .adj_land(&[t::ALGERIA, t::TUNISIA, t::SOUTHERN_FRANCE, t::LIBYA,
                         t::SARDINIA])
            .done(),

        // 73 — Tyrrhenian Sea — Board SZ 114
        SZB::new(sz::SZ_TYRRHENIAN_SEA, "Sea Zone 114")
            .adj_sea(&[sz::SZ_WESTERN_MED, sz::SZ_OFF_SOUTHERN_ITALY])
            .adj_land(&[t::NORTHERN_ITALY, t::SOUTHERN_ITALY, t::SARDINIA, t::SICILY,
                         t::TUNISIA])
            .done(),

        // 74 — Off Southern Italy — Board SZ 115
        SZB::new(sz::SZ_OFF_SOUTHERN_ITALY, "Sea Zone 115")
            .adj_sea(&[sz::SZ_TYRRHENIAN_SEA, sz::SZ_EASTERN_MED, sz::SZ_AEGEAN_SEA])
            .adj_land(&[t::SOUTHERN_ITALY, t::SICILY, t::MALTA, t::YUGOSLAVIA,
                         t::ALBANIA, t::GREECE])
            .done(),

        // 75 — Eastern Mediterranean — Board SZ 116
        SZB::new(sz::SZ_EASTERN_MED, "Sea Zone 116")
            .adj_sea(&[sz::SZ_OFF_SOUTHERN_ITALY, sz::SZ_AEGEAN_SEA, sz::SZ_OFF_EGYPT])
            .adj_land(&[t::LIBYA, t::TOBRUK, t::CRETE, t::MALTA, t::CYPRUS, t::SYRIA])
            .done(),

        // 76 — Off Egypt — Board SZ 117
        SZB::new(sz::SZ_OFF_EGYPT, "Sea Zone 117")
            .adj_sea(&[sz::SZ_EASTERN_MED, sz::SZ_RED_SEA])
            .adj_land(&[t::EGYPT, t::TOBRUK])
            .done(),

        // 77 — Aegean Sea — Board SZ 118
        SZB::new(sz::SZ_AEGEAN_SEA, "Sea Zone 118")
            .adj_sea(&[sz::SZ_OFF_SOUTHERN_ITALY, sz::SZ_EASTERN_MED, sz::SZ_BLACK_SEA])
            .adj_land(&[t::GREECE, t::BULGARIA, t::TURKEY, t::CRETE])
            .done(),

        // 78 — Black Sea — Board SZ 119
        SZB::new(sz::SZ_BLACK_SEA, "Sea Zone 119")
            .adj_sea(&[sz::SZ_AEGEAN_SEA])
            .adj_land(&[t::TURKEY, t::BULGARIA, t::ROMANIA, t::UKRAINE, t::ROSTOV,
                         t::CAUCASUS])
            .done(),

        // 79 — Barents Sea — Board SZ 120
        SZB::new(sz::SZ_BARENTS_SEA, "Sea Zone 120")
            .adj_sea(&[sz::SZ_NORWEGIAN_SEA])
            .adj_land(&[t::NORWAY, t::ARCHANGEL, t::NENETSIA])
            .convoy()
            .done(),
    ]
}
