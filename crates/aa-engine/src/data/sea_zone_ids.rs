//! Named constants for all sea zone IDs in Global 1940 2nd Edition.
//!
//! IDs are contiguous `u16` values used as Vec indices.
//! The board prints sea zones with numbers like "SZ 110" — use `board_number()`
//! to convert an internal ID to the display number.

use crate::territory::SeaZoneId;

/// Total number of sea zones.
pub const SEA_ZONE_COUNT: usize = 80;

// ===== PACIFIC OCEAN =====

/// Off Japan (east coast) — Board SZ 6
pub const SZ_JAPAN_EAST: SeaZoneId = 0;
/// Sea of Japan — Board SZ 5
pub const SZ_SEA_OF_JAPAN: SeaZoneId = 1;
/// Yellow Sea / Off Korea — Board SZ 4
pub const SZ_YELLOW_SEA: SeaZoneId = 2;
/// East China Sea / Off Kiangsu — Board SZ 3
pub const SZ_EAST_CHINA_SEA: SeaZoneId = 3;
/// Off Formosa — Board SZ 20
pub const SZ_OFF_FORMOSA: SeaZoneId = 4;
/// South China Sea / Off Kwangtung — Board SZ 19
pub const SZ_SOUTH_CHINA_SEA: SeaZoneId = 5;
/// Off French Indochina — Board SZ 21
pub const SZ_OFF_FRENCH_INDOCHINA: SeaZoneId = 6;
/// Gulf of Siam / Off Malaya — Board SZ 22
pub const SZ_OFF_MALAYA: SeaZoneId = 7;
/// Off Borneo — Board SZ 23
pub const SZ_OFF_BORNEO: SeaZoneId = 8;
/// Off Celebes — Board SZ 24
pub const SZ_OFF_CELEBES: SeaZoneId = 9;
/// Off Java — Board SZ 25
pub const SZ_OFF_JAVA: SeaZoneId = 10;
/// Off Sumatra — Board SZ 26
pub const SZ_OFF_SUMATRA: SeaZoneId = 11;
/// Bay of Bengal / Off Burma — Board SZ 37
pub const SZ_BAY_OF_BENGAL: SeaZoneId = 12;
/// Off India / Arabian Sea — Board SZ 38
pub const SZ_OFF_INDIA: SeaZoneId = 13;
/// Off West India — Board SZ 39
pub const SZ_OFF_WEST_INDIA: SeaZoneId = 14;
/// Off Ceylon — Board SZ 40
pub const SZ_OFF_CEYLON: SeaZoneId = 15;
/// Central Indian Ocean — Board SZ 41
pub const SZ_CENTRAL_INDIAN_OCEAN: SeaZoneId = 16;
/// Off Western Australia — Board SZ 42
pub const SZ_OFF_WESTERN_AUSTRALIA: SeaZoneId = 17;
/// Off South Australia — Board SZ 43
pub const SZ_OFF_SOUTH_AUSTRALIA: SeaZoneId = 18;
/// Off New South Wales — Board SZ 44
pub const SZ_OFF_NEW_SOUTH_WALES: SeaZoneId = 19;
/// Off Queensland / Coral Sea — Board SZ 45
pub const SZ_CORAL_SEA: SeaZoneId = 20;
/// Off New Guinea — Board SZ 46
pub const SZ_OFF_NEW_GUINEA: SeaZoneId = 21;
/// Off New Britain / Solomons — Board SZ 47
pub const SZ_OFF_NEW_BRITAIN: SeaZoneId = 22;
/// Off Caroline Islands — Board SZ 33
pub const SZ_OFF_CAROLINES: SeaZoneId = 23;
/// Off Palau — Board SZ 34
pub const SZ_OFF_PALAU: SeaZoneId = 24;
/// Off Philippines — Board SZ 35
pub const SZ_OFF_PHILIPPINES: SeaZoneId = 25;
/// Off Guam — Board SZ 32
pub const SZ_OFF_GUAM: SeaZoneId = 26;
/// Off Marshall Islands — Board SZ 31
pub const SZ_OFF_MARSHALLS: SeaZoneId = 27;
/// Off Wake Island — Board SZ 30
pub const SZ_OFF_WAKE: SeaZoneId = 28;
/// Off Midway — Board SZ 29
pub const SZ_OFF_MIDWAY: SeaZoneId = 29;
/// Off Iwo Jima — Board SZ 7
pub const SZ_OFF_IWO_JIMA: SeaZoneId = 30;
/// Off Okinawa — Board SZ 8
pub const SZ_OFF_OKINAWA: SeaZoneId = 31;
/// North Pacific / Off Japan north — Board SZ 9
pub const SZ_NORTH_PACIFIC: SeaZoneId = 32;
/// Sea of Okhotsk / Off Soviet Far East — Board SZ 10
pub const SZ_SEA_OF_OKHOTSK: SeaZoneId = 33;
/// Off Alaska — Board SZ 1
pub const SZ_OFF_ALASKA: SeaZoneId = 34;
/// Northeast Pacific — Board SZ 2
pub const SZ_NE_PACIFIC: SeaZoneId = 35;
/// Off Western United States — Board SZ 11
pub const SZ_OFF_WESTERN_US: SeaZoneId = 36;
/// Off Hawaii — Board SZ 28
pub const SZ_OFF_HAWAII: SeaZoneId = 37;
/// Off Johnston Island — Board SZ 27
pub const SZ_OFF_JOHNSTON: SeaZoneId = 38;
/// Off Line Islands — Board SZ 50
pub const SZ_OFF_LINE_ISLANDS: SeaZoneId = 39;
/// Central Pacific South — Board SZ 51
pub const SZ_CENTRAL_PACIFIC_SOUTH: SeaZoneId = 40;
/// Off Fiji — Board SZ 52
pub const SZ_OFF_FIJI: SeaZoneId = 41;
/// Off New Zealand — Board SZ 53
pub const SZ_OFF_NEW_ZEALAND: SeaZoneId = 42;
/// South Pacific — Board SZ 54
pub const SZ_SOUTH_PACIFIC: SeaZoneId = 43;
/// Off Northern Territory — Board SZ 48
pub const SZ_OFF_NORTHERN_TERRITORY: SeaZoneId = 44;
/// Off Manchuria — Board SZ 12
pub const SZ_OFF_MANCHURIA: SeaZoneId = 45;

// ===== INDIAN OCEAN =====

/// Persian Gulf — Board SZ 80
pub const SZ_PERSIAN_GULF: SeaZoneId = 46;
/// Red Sea — Board SZ 81
pub const SZ_RED_SEA: SeaZoneId = 47;
/// Off East Africa — Board SZ 82
pub const SZ_OFF_EAST_AFRICA: SeaZoneId = 48;
/// Off Madagascar — Board SZ 83
pub const SZ_OFF_MADAGASCAR: SeaZoneId = 49;
/// Off Mozambique — Board SZ 84
pub const SZ_OFF_MOZAMBIQUE: SeaZoneId = 50;
/// Off South Africa (Indian) — Board SZ 85
pub const SZ_OFF_SOUTH_AFRICA_EAST: SeaZoneId = 51;

// ===== ATLANTIC OCEAN =====

/// Off South Africa (Atlantic) — Board SZ 86
pub const SZ_OFF_SOUTH_AFRICA_WEST: SeaZoneId = 52;
/// Off Southwest Africa — Board SZ 87
pub const SZ_OFF_SOUTHWEST_AFRICA: SeaZoneId = 53;
/// Off Angola — Board SZ 88
pub const SZ_OFF_ANGOLA: SeaZoneId = 54;
/// Off French West Africa — Board SZ 89
pub const SZ_OFF_FRENCH_WEST_AFRICA: SeaZoneId = 55;
/// Central Atlantic — Board SZ 90
pub const SZ_CENTRAL_ATLANTIC: SeaZoneId = 56;
/// Off Brazil — Board SZ 91
pub const SZ_OFF_BRAZIL: SeaZoneId = 57;
/// Caribbean Sea — Board SZ 92
pub const SZ_CARIBBEAN: SeaZoneId = 58;
/// Gulf of Mexico — Board SZ 93
pub const SZ_GULF_OF_MEXICO: SeaZoneId = 59;
/// Off Eastern United States — Board SZ 101
pub const SZ_OFF_EASTERN_US: SeaZoneId = 60;
/// Off Labrador / North Atlantic — Board SZ 102
pub const SZ_NORTH_ATLANTIC: SeaZoneId = 61;
/// Off Greenland — Board SZ 103
pub const SZ_OFF_GREENLAND: SeaZoneId = 62;
/// Off Iceland — Board SZ 104
pub const SZ_OFF_ICELAND: SeaZoneId = 63;

// ===== EUROPE / MEDITERRANEAN =====

/// Off Scotland / Norwegian Sea — Board SZ 105
pub const SZ_NORWEGIAN_SEA: SeaZoneId = 64;
/// North Sea — Board SZ 106
pub const SZ_NORTH_SEA: SeaZoneId = 65;
/// Off Denmark / Skagerrak — Board SZ 107
pub const SZ_SKAGERRAK: SeaZoneId = 66;
/// Baltic Sea — Board SZ 108
pub const SZ_BALTIC_SEA: SeaZoneId = 67;
/// English Channel — Board SZ 109
pub const SZ_ENGLISH_CHANNEL: SeaZoneId = 68;
/// Bay of Biscay — Board SZ 110
pub const SZ_BAY_OF_BISCAY: SeaZoneId = 69;
/// Off Gibraltar — Board SZ 111
pub const SZ_OFF_GIBRALTAR: SeaZoneId = 70;
/// Off Morocco — Board SZ 112
pub const SZ_OFF_MOROCCO: SeaZoneId = 71;
/// Western Mediterranean — Board SZ 113
pub const SZ_WESTERN_MED: SeaZoneId = 72;
/// Off Sardinia / Tyrrhenian Sea — Board SZ 114
pub const SZ_TYRRHENIAN_SEA: SeaZoneId = 73;
/// Off Southern Italy — Board SZ 115
pub const SZ_OFF_SOUTHERN_ITALY: SeaZoneId = 74;
/// Eastern Mediterranean — Board SZ 116
pub const SZ_EASTERN_MED: SeaZoneId = 75;
/// Off Egypt — Board SZ 117
pub const SZ_OFF_EGYPT: SeaZoneId = 76;
/// Aegean Sea — Board SZ 118
pub const SZ_AEGEAN_SEA: SeaZoneId = 77;
/// Black Sea — Board SZ 119
pub const SZ_BLACK_SEA: SeaZoneId = 78;
/// Off Norway / Barents Sea — Board SZ 120
pub const SZ_BARENTS_SEA: SeaZoneId = 79;

/// Map an internal sea zone ID to the display board number.
pub const fn board_number(id: SeaZoneId) -> u16 {
    // Board numbers for display only — no gameplay effect.
    const BOARD_NUMBERS: [u16; SEA_ZONE_COUNT] = [
        6,   // 0  SZ_JAPAN_EAST
        5,   // 1  SZ_SEA_OF_JAPAN
        4,   // 2  SZ_YELLOW_SEA
        3,   // 3  SZ_EAST_CHINA_SEA
        20,  // 4  SZ_OFF_FORMOSA
        19,  // 5  SZ_SOUTH_CHINA_SEA
        21,  // 6  SZ_OFF_FRENCH_INDOCHINA
        22,  // 7  SZ_OFF_MALAYA
        23,  // 8  SZ_OFF_BORNEO
        24,  // 9  SZ_OFF_CELEBES
        25,  // 10 SZ_OFF_JAVA
        26,  // 11 SZ_OFF_SUMATRA
        37,  // 12 SZ_BAY_OF_BENGAL
        38,  // 13 SZ_OFF_INDIA
        39,  // 14 SZ_OFF_WEST_INDIA
        40,  // 15 SZ_OFF_CEYLON
        41,  // 16 SZ_CENTRAL_INDIAN_OCEAN
        42,  // 17 SZ_OFF_WESTERN_AUSTRALIA
        43,  // 18 SZ_OFF_SOUTH_AUSTRALIA
        44,  // 19 SZ_OFF_NEW_SOUTH_WALES
        45,  // 20 SZ_CORAL_SEA
        46,  // 21 SZ_OFF_NEW_GUINEA
        47,  // 22 SZ_OFF_NEW_BRITAIN
        33,  // 23 SZ_OFF_CAROLINES
        34,  // 24 SZ_OFF_PALAU
        35,  // 25 SZ_OFF_PHILIPPINES
        32,  // 26 SZ_OFF_GUAM
        31,  // 27 SZ_OFF_MARSHALLS
        30,  // 28 SZ_OFF_WAKE
        29,  // 29 SZ_OFF_MIDWAY
        7,   // 30 SZ_OFF_IWO_JIMA
        8,   // 31 SZ_OFF_OKINAWA
        9,   // 32 SZ_NORTH_PACIFIC
        10,  // 33 SZ_SEA_OF_OKHOTSK
        1,   // 34 SZ_OFF_ALASKA
        2,   // 35 SZ_NE_PACIFIC
        11,  // 36 SZ_OFF_WESTERN_US
        28,  // 37 SZ_OFF_HAWAII
        27,  // 38 SZ_OFF_JOHNSTON
        50,  // 39 SZ_OFF_LINE_ISLANDS
        51,  // 40 SZ_CENTRAL_PACIFIC_SOUTH
        52,  // 41 SZ_OFF_FIJI
        53,  // 42 SZ_OFF_NEW_ZEALAND
        54,  // 43 SZ_SOUTH_PACIFIC
        48,  // 44 SZ_OFF_NORTHERN_TERRITORY
        12,  // 45 SZ_OFF_MANCHURIA
        80,  // 46 SZ_PERSIAN_GULF
        81,  // 47 SZ_RED_SEA
        82,  // 48 SZ_OFF_EAST_AFRICA
        83,  // 49 SZ_OFF_MADAGASCAR
        84,  // 50 SZ_OFF_MOZAMBIQUE
        85,  // 51 SZ_OFF_SOUTH_AFRICA_EAST
        86,  // 52 SZ_OFF_SOUTH_AFRICA_WEST
        87,  // 53 SZ_OFF_SOUTHWEST_AFRICA
        88,  // 54 SZ_OFF_ANGOLA
        89,  // 55 SZ_OFF_FRENCH_WEST_AFRICA
        90,  // 56 SZ_CENTRAL_ATLANTIC
        91,  // 57 SZ_OFF_BRAZIL
        92,  // 58 SZ_CARIBBEAN
        93,  // 59 SZ_GULF_OF_MEXICO
        101, // 60 SZ_OFF_EASTERN_US
        102, // 61 SZ_NORTH_ATLANTIC
        103, // 62 SZ_OFF_GREENLAND
        104, // 63 SZ_OFF_ICELAND
        105, // 64 SZ_NORWEGIAN_SEA
        106, // 65 SZ_NORTH_SEA
        107, // 66 SZ_SKAGERRAK
        108, // 67 SZ_BALTIC_SEA
        109, // 68 SZ_ENGLISH_CHANNEL
        110, // 69 SZ_BAY_OF_BISCAY
        111, // 70 SZ_OFF_GIBRALTAR
        112, // 71 SZ_OFF_MOROCCO
        113, // 72 SZ_WESTERN_MED
        114, // 73 SZ_TYRRHENIAN_SEA
        115, // 74 SZ_OFF_SOUTHERN_ITALY
        116, // 75 SZ_EASTERN_MED
        117, // 76 SZ_OFF_EGYPT
        118, // 77 SZ_AEGEAN_SEA
        119, // 78 SZ_BLACK_SEA
        120, // 79 SZ_BARENTS_SEA
    ];
    BOARD_NUMBERS[id as usize]
}
