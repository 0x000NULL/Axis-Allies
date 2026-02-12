//! Named constants for all land territory IDs in Global 1940 2nd Edition.
//!
//! IDs are contiguous `u16` values used as Vec indices.

use crate::territory::TerritoryId;

/// Total number of land territories.
pub const TERRITORY_COUNT: usize = 164;

// ===== EUROPE =====

pub const GERMANY: TerritoryId = 0;
pub const WESTERN_GERMANY: TerritoryId = 1;
pub const GREATER_SOUTHERN_GERMANY: TerritoryId = 2;
pub const HOLLAND_BELGIUM: TerritoryId = 3;
pub const NORMANDY_BORDEAUX: TerritoryId = 4;
pub const FRANCE: TerritoryId = 5;
pub const SOUTHERN_FRANCE: TerritoryId = 6;
pub const DENMARK: TerritoryId = 7;
pub const NORWAY: TerritoryId = 8;
pub const FINLAND: TerritoryId = 9;
pub const SWEDEN: TerritoryId = 10;
pub const POLAND: TerritoryId = 11;
pub const BALTIC_STATES: TerritoryId = 12;
pub const SLOVAKIA_HUNGARY: TerritoryId = 13;
pub const ROMANIA: TerritoryId = 14;
pub const YUGOSLAVIA: TerritoryId = 15;
pub const BULGARIA: TerritoryId = 16;
pub const GREECE: TerritoryId = 17;
pub const ALBANIA: TerritoryId = 18;
pub const NORTHERN_ITALY: TerritoryId = 19;
pub const SOUTHERN_ITALY: TerritoryId = 20;
pub const SARDINIA: TerritoryId = 21;
pub const SICILY: TerritoryId = 22;
pub const CRETE: TerritoryId = 23;
pub const MALTA: TerritoryId = 24;
pub const CYPRUS: TerritoryId = 25;
pub const UNITED_KINGDOM: TerritoryId = 26;
pub const SCOTLAND: TerritoryId = 27;
pub const EIRE: TerritoryId = 28;
pub const GIBRALTAR: TerritoryId = 29;
pub const ICELAND: TerritoryId = 30;
pub const SPAIN: TerritoryId = 31;
pub const PORTUGAL: TerritoryId = 32;
pub const SWITZERLAND: TerritoryId = 33;
pub const TURKEY: TerritoryId = 34;

// ===== NORTH AFRICA =====

pub const MOROCCO: TerritoryId = 35;
pub const ALGERIA: TerritoryId = 36;
pub const TUNISIA: TerritoryId = 37;
pub const LIBYA: TerritoryId = 38;
pub const TOBRUK: TerritoryId = 39;
pub const EGYPT: TerritoryId = 40;
pub const ANGLO_EGYPTIAN_SUDAN: TerritoryId = 41;

// ===== EAST AND SOUTH AFRICA =====

pub const ETHIOPIA: TerritoryId = 42;
pub const ITALIAN_SOMALILAND: TerritoryId = 43;
pub const BRITISH_SOMALILAND: TerritoryId = 44;
pub const KENYA: TerritoryId = 45;
pub const BELGIAN_CONGO: TerritoryId = 46;
pub const FRENCH_EQUATORIAL_AFRICA: TerritoryId = 47;
pub const FRENCH_WEST_AFRICA: TerritoryId = 48;
pub const TANGANYIKA: TerritoryId = 49;
pub const NORTHERN_RHODESIA: TerritoryId = 50;
pub const RHODESIA: TerritoryId = 51;
pub const SOUTH_AFRICA: TerritoryId = 52;
pub const SOUTHWEST_AFRICA: TerritoryId = 53;
pub const MOZAMBIQUE: TerritoryId = 54;
pub const MADAGASCAR: TerritoryId = 55;
pub const ANGOLA: TerritoryId = 56;

// ===== MIDDLE EAST =====

pub const TRANS_JORDAN: TerritoryId = 57;
pub const IRAQ: TerritoryId = 58;
pub const SYRIA: TerritoryId = 59;
pub const PERSIA: TerritoryId = 60;
pub const NORTHWEST_PERSIA: TerritoryId = 61;
pub const EASTERN_PERSIA: TerritoryId = 62;
pub const SAUDI_ARABIA: TerritoryId = 63;
pub const AFGHANISTAN: TerritoryId = 64;

// ===== SOVIET UNION =====

pub const NOVGOROD: TerritoryId = 65;
pub const VYBORG: TerritoryId = 66;
pub const ARCHANGEL: TerritoryId = 67;
pub const NENETSIA: TerritoryId = 68;
pub const RUSSIA: TerritoryId = 69;
pub const BELARUS: TerritoryId = 70;
pub const WESTERN_UKRAINE: TerritoryId = 71;
pub const UKRAINE: TerritoryId = 72;
pub const ROSTOV: TerritoryId = 73;
pub const VOLGOGRAD: TerritoryId = 74;
pub const CAUCASUS: TerritoryId = 75;
pub const TAMBOV: TerritoryId = 76;
pub const VOLOGDA: TerritoryId = 77;
pub const URALS: TerritoryId = 78;
pub const KAZAKHSTAN: TerritoryId = 79;
pub const NOVOSIBIRSK: TerritoryId = 80;
pub const TIMGUSKA: TerritoryId = 81;
pub const YENISEY: TerritoryId = 82;
pub const EVENKI_NATIONAL_OKRUG: TerritoryId = 83;
pub const YAKUT_SSR: TerritoryId = 84;
pub const BURYATIA: TerritoryId = 85;
pub const SAKHA: TerritoryId = 86;
pub const SOVIET_FAR_EAST: TerritoryId = 87;
pub const AMUR: TerritoryId = 88;

// ===== MONGOLIA =====

pub const OLGIY: TerritoryId = 89;
pub const DZAVHAN: TerritoryId = 90;
pub const TSAGAAN_OLOM: TerritoryId = 91;
pub const CENTRAL_MONGOLIA: TerritoryId = 92;
pub const ULAANBAATAR: TerritoryId = 93;
pub const BUYANT_UHAA: TerritoryId = 94;

// ===== CHINA =====

pub const SZECHWAN: TerritoryId = 95;
pub const YUNNAN: TerritoryId = 96;
pub const KWEICHOW: TerritoryId = 97;
pub const HUNAN: TerritoryId = 98;
pub const KIANGSI: TerritoryId = 99;
pub const KWANGSI: TerritoryId = 100;
pub const SIKANG: TerritoryId = 101;
pub const TSINGHAI: TerritoryId = 102;
pub const SUIYUAN: TerritoryId = 103;
pub const SHENSI: TerritoryId = 104;
pub const KANSU: TerritoryId = 105;
pub const KIANGSU: TerritoryId = 106;
pub const SHANTUNG: TerritoryId = 107;
pub const JEHOL: TerritoryId = 108;
pub const KWANGTUNG: TerritoryId = 109;
pub const ANHWE: TerritoryId = 110;
pub const CHAHAR: TerritoryId = 111;
pub const HOPEI: TerritoryId = 112;
pub const MANCHURIA: TerritoryId = 113;

// ===== INDIA AND SOUTHEAST ASIA =====

pub const INDIA: TerritoryId = 114;
pub const WEST_INDIA: TerritoryId = 115;
pub const BURMA: TerritoryId = 116;
pub const SHAN_STATE: TerritoryId = 117;
pub const MALAYA: TerritoryId = 118;
pub const SIAM: TerritoryId = 119;
pub const FRENCH_INDOCHINA: TerritoryId = 120;
pub const CEYLON: TerritoryId = 121;

// ===== DUTCH EAST INDIES =====

pub const BORNEO: TerritoryId = 122;
pub const JAVA: TerritoryId = 123;
pub const SUMATRA: TerritoryId = 124;
pub const CELEBES: TerritoryId = 125;

// ===== JAPAN AND PACIFIC ISLANDS =====

pub const JAPAN: TerritoryId = 126;
pub const OKINAWA: TerritoryId = 127;
pub const IWO_JIMA: TerritoryId = 128;
pub const KOREA: TerritoryId = 129;
pub const FORMOSA: TerritoryId = 130;
pub const CAROLINE_ISLANDS: TerritoryId = 131;
pub const MARSHALL_ISLANDS: TerritoryId = 132;
pub const PALAU_ISLAND: TerritoryId = 133;
pub const MIDWAY: TerritoryId = 134;
pub const WAKE_ISLAND: TerritoryId = 135;
pub const HAWAIIAN_ISLANDS: TerritoryId = 136;
pub const JOHNSTON_ISLAND: TerritoryId = 137;
pub const LINE_ISLANDS: TerritoryId = 138;
pub const GUAM: TerritoryId = 139;
pub const PHILIPPINES: TerritoryId = 140;

// ===== AUSTRALIA AND NEW ZEALAND =====

pub const NEW_SOUTH_WALES: TerritoryId = 141;
pub const QUEENSLAND: TerritoryId = 142;
pub const SOUTH_AUSTRALIA: TerritoryId = 143;
pub const WESTERN_AUSTRALIA: TerritoryId = 144;
pub const NORTHERN_TERRITORY: TerritoryId = 145;
pub const NEW_ZEALAND: TerritoryId = 146;
pub const NEW_GUINEA: TerritoryId = 147;
pub const NEW_BRITAIN: TerritoryId = 148;
pub const SOLOMON_ISLANDS: TerritoryId = 149;
pub const FIJI: TerritoryId = 150;

// ===== AMERICAS =====

pub const EASTERN_UNITED_STATES: TerritoryId = 151;
pub const CENTRAL_UNITED_STATES: TerritoryId = 152;
pub const WESTERN_UNITED_STATES: TerritoryId = 153;
pub const ALASKA: TerritoryId = 154;
pub const MEXICO: TerritoryId = 155;
pub const CENTRAL_AMERICA: TerritoryId = 156;
pub const WEST_INDIES: TerritoryId = 157;
pub const GREENLAND: TerritoryId = 158;
pub const BRAZIL: TerritoryId = 159;
pub const VENEZUELA: TerritoryId = 160;
pub const COLOMBIA_ECUADOR: TerritoryId = 161;
pub const ARGENTINA_CHILE: TerritoryId = 162;
pub const PERU: TerritoryId = 163;
