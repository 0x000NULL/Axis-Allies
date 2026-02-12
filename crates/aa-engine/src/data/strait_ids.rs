//! Strait and canal definitions for Global 1940 2nd Edition.
//!
//! Straits/canals restrict naval movement — a power must control the listed
//! territory to allow ships through. Some also enable a land connection
//! across the strait.

use crate::territory::{StraitId, TerritoryId, SeaZoneId};

/// Total number of straits / canals.
pub const STRAIT_COUNT: usize = 4;

pub const STRAIT_TURKISH: StraitId = 0;
pub const STRAIT_SUEZ: StraitId = 1;
pub const STRAIT_PANAMA: StraitId = 2;
pub const STRAIT_DANISH: StraitId = 3;

/// Static definition of a strait or canal.
#[derive(Clone, Debug)]
pub struct StraitDef {
    pub id: StraitId,
    pub name: &'static str,
    /// Territory that must be controlled to allow passage.
    pub controlled_by: TerritoryId,
    /// Sea zones connected by this strait (ships can pass when controller is friendly).
    pub connects_seas: (SeaZoneId, SeaZoneId),
    /// If Some, this strait also provides a land connection between two territories.
    pub connects_land: Option<(TerritoryId, TerritoryId)>,
}

/// Build the 4 strait/canal definitions.
pub fn build_strait_defs() -> Vec<StraitDef> {
    use super::territory_ids as t;
    use super::sea_zone_ids as sz;

    vec![
        // Turkish Straits: Turkey controls passage between Black Sea and Aegean.
        // Also provides a land bridge (European Turkey ↔ Anatolia represented as single Turkey territory).
        StraitDef {
            id: STRAIT_TURKISH,
            name: "Turkish Straits",
            controlled_by: t::TURKEY,
            connects_seas: (sz::SZ_BLACK_SEA, sz::SZ_AEGEAN_SEA),
            connects_land: None, // Turkey is one territory; no separate land connection needed
        },
        // Suez Canal: Egypt controls passage between Eastern Med and Red Sea.
        StraitDef {
            id: STRAIT_SUEZ,
            name: "Suez Canal",
            controlled_by: t::EGYPT,
            connects_seas: (sz::SZ_OFF_EGYPT, sz::SZ_RED_SEA),
            connects_land: None,
        },
        // Panama Canal: Central America controls passage between Caribbean and Pacific.
        StraitDef {
            id: STRAIT_PANAMA,
            name: "Panama Canal",
            controlled_by: t::CENTRAL_AMERICA,
            connects_seas: (sz::SZ_CARIBBEAN, sz::SZ_OFF_WESTERN_US),
            connects_land: None,
        },
        // Danish Straits: Denmark controls passage between Baltic Sea and North Sea.
        StraitDef {
            id: STRAIT_DANISH,
            name: "Danish Straits",
            controlled_by: t::DENMARK,
            connects_seas: (sz::SZ_BALTIC_SEA, sz::SZ_SKAGERRAK),
            connects_land: None,
        },
    ]
}
