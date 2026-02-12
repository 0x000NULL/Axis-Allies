//! Static game data compiled into the binary.
//!
//! Contains all territory definitions, sea zones, adjacency graphs, and
//! strait/canal definitions for Global 1940 2nd Edition.

pub mod territory_ids;
pub mod sea_zone_ids;
pub mod strait_ids;
pub mod territories;
pub mod sea_zones;

use std::collections::VecDeque;

use crate::territory::{TerritoryDef, SeaZoneDef, TerritoryId, SeaZoneId, RegionId};
use strait_ids::StraitDef;

/// The complete static game map. Constructed once and stored on `Engine`.
/// Not serialized — rebuilt on load from the compiled-in data.
pub struct GameMap {
    pub territories: Vec<TerritoryDef>,
    pub sea_zones: Vec<SeaZoneDef>,
    pub straits: Vec<StraitDef>,
}

impl GameMap {
    /// Construct the full map from compiled-in data.
    pub fn new() -> Self {
        GameMap {
            territories: territories::build_territory_defs(),
            sea_zones: sea_zones::build_sea_zone_defs(),
            straits: strait_ids::build_strait_defs(),
        }
    }

    // ------------------------------------------------------------------
    // Lookups
    // ------------------------------------------------------------------

    /// Get a territory definition by ID.
    pub fn territory(&self, id: TerritoryId) -> &TerritoryDef {
        &self.territories[id as usize]
    }

    /// Get a sea zone definition by ID.
    pub fn sea_zone(&self, id: SeaZoneId) -> &SeaZoneDef {
        &self.sea_zones[id as usize]
    }

    // ------------------------------------------------------------------
    // Adjacency queries
    // ------------------------------------------------------------------

    /// Are two territories land-adjacent?
    pub fn is_land_adjacent(&self, a: TerritoryId, b: TerritoryId) -> bool {
        self.territories[a as usize].adjacent_land.contains(&b)
    }

    /// Are two sea zones sea-adjacent?
    pub fn is_sea_adjacent(&self, a: SeaZoneId, b: SeaZoneId) -> bool {
        self.sea_zones[a as usize].adjacent_sea.contains(&b)
    }

    /// Is a territory coastal (adjacent to at least one sea zone)?
    pub fn is_coastal(&self, id: TerritoryId) -> bool {
        !self.territories[id as usize].adjacent_sea.is_empty()
    }

    /// Are two regions adjacent (land↔land, sea↔sea, or land↔sea)?
    pub fn is_adjacent(&self, a: RegionId, b: RegionId) -> bool {
        match (a, b) {
            (RegionId::Land(la), RegionId::Land(lb)) => self.is_land_adjacent(la, lb),
            (RegionId::Sea(sa), RegionId::Sea(sb)) => self.is_sea_adjacent(sa, sb),
            (RegionId::Land(l), RegionId::Sea(s)) | (RegionId::Sea(s), RegionId::Land(l)) => {
                self.territories[l as usize].adjacent_sea.contains(&s)
            }
        }
    }

    // ------------------------------------------------------------------
    // Neighbor lists
    // ------------------------------------------------------------------

    /// Land neighbors of a territory.
    pub fn land_neighbors(&self, id: TerritoryId) -> &[TerritoryId] {
        &self.territories[id as usize].adjacent_land
    }

    /// Sea neighbors of a sea zone.
    pub fn sea_neighbors(&self, id: SeaZoneId) -> &[SeaZoneId] {
        &self.sea_zones[id as usize].adjacent_sea
    }

    /// Sea zones adjacent to a territory (its coastal zones).
    pub fn coastal_zones(&self, id: TerritoryId) -> &[SeaZoneId] {
        &self.territories[id as usize].adjacent_sea
    }

    /// Land territories adjacent to a sea zone.
    pub fn coastal_territories(&self, id: SeaZoneId) -> &[TerritoryId] {
        &self.sea_zones[id as usize].adjacent_land
    }

    // ------------------------------------------------------------------
    // Pathfinding (BFS — all edges weight 1)
    // ------------------------------------------------------------------

    /// BFS shortest land path from `from` to `to`. Returns `None` if unreachable.
    /// The returned path includes both endpoints.
    pub fn find_land_path(&self, from: TerritoryId, to: TerritoryId) -> Option<Vec<TerritoryId>> {
        if from == to {
            return Some(vec![from]);
        }
        let n = self.territories.len();
        let mut visited = vec![false; n];
        let mut parent: Vec<Option<TerritoryId>> = vec![None; n];
        let mut queue = VecDeque::new();

        visited[from as usize] = true;
        queue.push_back(from);

        while let Some(current) = queue.pop_front() {
            for &neighbor in &self.territories[current as usize].adjacent_land {
                if !visited[neighbor as usize] {
                    visited[neighbor as usize] = true;
                    parent[neighbor as usize] = Some(current);
                    if neighbor == to {
                        // Reconstruct path
                        let mut path = vec![to];
                        let mut cur = to;
                        while let Some(p) = parent[cur as usize] {
                            path.push(p);
                            cur = p;
                        }
                        path.reverse();
                        return Some(path);
                    }
                    queue.push_back(neighbor);
                }
            }
        }
        None
    }

    /// BFS shortest sea path from `from` to `to`. Returns `None` if unreachable.
    /// The returned path includes both endpoints.
    pub fn find_sea_path(&self, from: SeaZoneId, to: SeaZoneId) -> Option<Vec<SeaZoneId>> {
        if from == to {
            return Some(vec![from]);
        }
        let n = self.sea_zones.len();
        let mut visited = vec![false; n];
        let mut parent: Vec<Option<SeaZoneId>> = vec![None; n];
        let mut queue = VecDeque::new();

        visited[from as usize] = true;
        queue.push_back(from);

        while let Some(current) = queue.pop_front() {
            for &neighbor in &self.sea_zones[current as usize].adjacent_sea {
                if !visited[neighbor as usize] {
                    visited[neighbor as usize] = true;
                    parent[neighbor as usize] = Some(current);
                    if neighbor == to {
                        let mut path = vec![to];
                        let mut cur = to;
                        while let Some(p) = parent[cur as usize] {
                            path.push(p);
                            cur = p;
                        }
                        path.reverse();
                        return Some(path);
                    }
                    queue.push_back(neighbor);
                }
            }
        }
        None
    }

    // ------------------------------------------------------------------
    // Reachability
    // ------------------------------------------------------------------

    /// All territories reachable from `origin` within `max_dist` land hops.
    /// Includes `origin` itself (distance 0).
    pub fn land_reachable_within(&self, origin: TerritoryId, max_dist: u32) -> Vec<TerritoryId> {
        let n = self.territories.len();
        let mut dist = vec![u32::MAX; n];
        let mut queue = VecDeque::new();

        dist[origin as usize] = 0;
        queue.push_back(origin);

        let mut result = vec![origin];

        while let Some(current) = queue.pop_front() {
            let d = dist[current as usize];
            if d >= max_dist {
                continue;
            }
            for &neighbor in &self.territories[current as usize].adjacent_land {
                if dist[neighbor as usize] == u32::MAX {
                    dist[neighbor as usize] = d + 1;
                    result.push(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }
        result
    }

    /// All sea zones reachable from `origin` within `max_dist` hops.
    /// Includes `origin` itself (distance 0).
    pub fn sea_reachable_within(&self, origin: SeaZoneId, max_dist: u32) -> Vec<SeaZoneId> {
        let n = self.sea_zones.len();
        let mut dist = vec![u32::MAX; n];
        let mut queue = VecDeque::new();

        dist[origin as usize] = 0;
        queue.push_back(origin);

        let mut result = vec![origin];

        while let Some(current) = queue.pop_front() {
            let d = dist[current as usize];
            if d >= max_dist {
                continue;
            }
            for &neighbor in &self.sea_zones[current as usize].adjacent_sea {
                if dist[neighbor as usize] == u32::MAX {
                    dist[neighbor as usize] = d + 1;
                    result.push(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }
        result
    }

    // ------------------------------------------------------------------
    // Distance
    // ------------------------------------------------------------------

    /// Shortest land distance between two territories.
    /// Returns `None` if not connected by land.
    pub fn land_distance(&self, from: TerritoryId, to: TerritoryId) -> Option<u32> {
        if from == to {
            return Some(0);
        }
        self.find_land_path(from, to)
            .map(|path| (path.len() - 1) as u32)
    }

    /// Shortest sea distance between two sea zones.
    /// Returns `None` if not connected by sea.
    pub fn sea_distance(&self, from: SeaZoneId, to: SeaZoneId) -> Option<u32> {
        if from == to {
            return Some(0);
        }
        self.find_sea_path(from, to)
            .map(|path| (path.len() - 1) as u32)
    }

    // ------------------------------------------------------------------
    // Straits / Canals
    // ------------------------------------------------------------------

    /// Check if a strait is passable given a control predicate.
    /// `controls_fn` takes a TerritoryId and returns whether the relevant power
    /// controls (or is friendly to the controller of) that territory.
    pub fn strait_is_passable<F>(&self, strait_id: u8, controls_fn: F) -> bool
    where
        F: Fn(TerritoryId) -> bool,
    {
        if let Some(strait) = self.straits.get(strait_id as usize) {
            controls_fn(strait.controlled_by)
        } else {
            false
        }
    }
}

impl Default for GameMap {
    fn default() -> Self {
        Self::new()
    }
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::power::Power;
    use territory_ids as t;
    use sea_zone_ids as sz;

    fn map() -> GameMap {
        GameMap::new()
    }

    // ----- Data integrity -----

    #[test]
    fn territory_count_matches() {
        let m = map();
        assert_eq!(m.territories.len(), territory_ids::TERRITORY_COUNT);
    }

    #[test]
    fn sea_zone_count_matches() {
        let m = map();
        assert_eq!(m.sea_zones.len(), sea_zone_ids::SEA_ZONE_COUNT);
    }

    #[test]
    fn territory_ids_match_indices() {
        let m = map();
        for (i, def) in m.territories.iter().enumerate() {
            assert_eq!(
                def.id as usize, i,
                "Territory '{}' has id {} but is at index {}",
                def.name, def.id, i
            );
        }
    }

    #[test]
    fn sea_zone_ids_match_indices() {
        let m = map();
        for (i, def) in m.sea_zones.iter().enumerate() {
            assert_eq!(
                def.id as usize, i,
                "Sea zone '{}' has id {} but is at index {}",
                def.name, def.id, i
            );
        }
    }

    #[test]
    fn strait_count_matches() {
        let m = map();
        assert_eq!(m.straits.len(), strait_ids::STRAIT_COUNT);
    }

    // ----- Adjacency symmetry -----

    #[test]
    fn land_adjacency_is_symmetric() {
        let m = map();
        for def in &m.territories {
            for &neighbor in &def.adjacent_land {
                let neighbor_def = &m.territories[neighbor as usize];
                assert!(
                    neighbor_def.adjacent_land.contains(&def.id),
                    "Land adjacency not symmetric: '{}' lists '{}' but not vice versa",
                    def.name, neighbor_def.name
                );
            }
        }
    }

    #[test]
    fn sea_adjacency_is_symmetric() {
        let m = map();
        for def in &m.sea_zones {
            for &neighbor in &def.adjacent_sea {
                let neighbor_def = &m.sea_zones[neighbor as usize];
                assert!(
                    neighbor_def.adjacent_sea.contains(&def.id),
                    "Sea adjacency not symmetric: '{}' lists '{}' but not vice versa",
                    def.name, neighbor_def.name
                );
            }
        }
    }

    #[test]
    fn land_sea_adjacency_is_symmetric() {
        let m = map();
        // If a territory lists a sea zone, that sea zone must list the territory
        for tdef in &m.territories {
            for &sz_id in &tdef.adjacent_sea {
                let sz_def = &m.sea_zones[sz_id as usize];
                assert!(
                    sz_def.adjacent_land.contains(&tdef.id),
                    "Territory '{}' lists sea zone '{}' but sea zone doesn't list territory back",
                    tdef.name, sz_def.name
                );
            }
        }
        // And vice versa
        for sz_def in &m.sea_zones {
            for &t_id in &sz_def.adjacent_land {
                let tdef = &m.territories[t_id as usize];
                assert!(
                    tdef.adjacent_sea.contains(&sz_def.id),
                    "Sea zone '{}' lists territory '{}' but territory doesn't list sea zone back",
                    sz_def.name, tdef.name
                );
            }
        }
    }

    #[test]
    fn no_self_adjacency() {
        let m = map();
        for def in &m.territories {
            assert!(
                !def.adjacent_land.contains(&def.id),
                "Territory '{}' is self-adjacent (land)",
                def.name
            );
        }
        for def in &m.sea_zones {
            assert!(
                !def.adjacent_sea.contains(&def.id),
                "Sea zone '{}' is self-adjacent",
                def.name
            );
        }
    }

    // ----- Spot checks -----

    #[test]
    fn germany_correct_properties() {
        let m = map();
        let de = m.territory(t::GERMANY);
        assert_eq!(de.name, "Germany");
        assert_eq!(de.ipc_value, 5);
        assert_eq!(de.is_capital, Some(Power::Germany));
        assert!(de.is_victory_city);
        assert_eq!(de.original_owner, Some(Power::Germany));
        // Germany should border Western Germany, among others
        assert!(de.adjacent_land.contains(&t::WESTERN_GERMANY));
        assert!(de.adjacent_land.contains(&t::DENMARK));
    }

    #[test]
    fn france_is_capital_and_vc() {
        let m = map();
        let fr = m.territory(t::FRANCE);
        assert_eq!(fr.ipc_value, 6);
        assert_eq!(fr.is_capital, Some(Power::France));
        assert!(fr.is_victory_city);
        assert_eq!(fr.original_owner, Some(Power::France));
    }

    #[test]
    fn united_kingdom_is_island() {
        let m = map();
        let uk = m.territory(t::UNITED_KINGDOM);
        assert!(uk.is_island);
        // UK is land-adjacent to Scotland (same island) but NOT to France
        assert!(uk.adjacent_land.contains(&t::SCOTLAND));
        assert!(!uk.adjacent_land.contains(&t::FRANCE));
    }

    #[test]
    fn all_powers_have_capital() {
        let m = map();
        for power in Power::all() {
            let has_capital = m
                .territories
                .iter()
                .any(|t| t.is_capital == Some(*power));
            assert!(
                has_capital,
                "Power {:?} has no capital territory",
                power
            );
        }
    }

    #[test]
    fn victory_city_count() {
        let m = map();
        let vc_count = m.territories.iter().filter(|t| t.is_victory_city).count();
        // Global 1940 has at least 14 victory cities
        assert!(
            vc_count >= 14,
            "Expected at least 14 victory cities, found {}",
            vc_count
        );
    }

    #[test]
    fn neutrals_have_no_owner() {
        let m = map();
        let neutrals = [t::SWEDEN, t::SWITZERLAND, t::TURKEY, t::SPAIN, t::PORTUGAL, t::EIRE];
        for &id in &neutrals {
            let def = m.territory(id);
            assert_eq!(
                def.original_owner, None,
                "Neutral territory '{}' should have no original_owner",
                def.name
            );
        }
    }

    #[test]
    fn switzerland_is_impassable() {
        let m = map();
        let ch = m.territory(t::SWITZERLAND);
        assert_eq!(ch.territory_type, crate::territory::TerritoryType::Impassable);
        assert!(ch.adjacent_land.is_empty());
        assert!(ch.adjacent_sea.is_empty());
    }

    // ----- Pathfinding -----

    #[test]
    fn adjacent_territories_have_distance_one() {
        let m = map();
        assert_eq!(m.land_distance(t::GERMANY, t::WESTERN_GERMANY), Some(1));
    }

    #[test]
    fn same_territory_has_distance_zero() {
        let m = map();
        assert_eq!(m.land_distance(t::GERMANY, t::GERMANY), Some(0));
        assert_eq!(m.sea_distance(sz::SZ_BALTIC_SEA, sz::SZ_BALTIC_SEA), Some(0));
    }

    #[test]
    fn islands_unreachable_by_land() {
        let m = map();
        // Iceland is an island with no land connections
        assert_eq!(m.find_land_path(t::GERMANY, t::ICELAND), None);
        // Japan is an island
        assert_eq!(m.find_land_path(t::GERMANY, t::JAPAN), None);
    }

    #[test]
    fn land_reachable_within_returns_correct_set() {
        let m = map();
        let reachable = m.land_reachable_within(t::GERMANY, 1);
        // Should include Germany itself and all direct land neighbors
        assert!(reachable.contains(&t::GERMANY));
        assert!(reachable.contains(&t::WESTERN_GERMANY));
        assert!(reachable.contains(&t::DENMARK));
        // Should NOT include territories 2+ hops away
        assert!(!reachable.contains(&t::FRANCE));
    }

    #[test]
    fn sea_reachable_within_returns_correct_set() {
        let m = map();
        let reachable = m.sea_reachable_within(sz::SZ_BALTIC_SEA, 1);
        assert!(reachable.contains(&sz::SZ_BALTIC_SEA));
        assert!(reachable.contains(&sz::SZ_SKAGERRAK));
        // Baltic only has 1 sea neighbor (Skagerrak), so size should be 2
        assert_eq!(reachable.len(), 2);
    }

    #[test]
    fn sea_path_exists_between_connected_zones() {
        let m = map();
        // Baltic → Skagerrak → North Sea
        let path = m.find_sea_path(sz::SZ_BALTIC_SEA, sz::SZ_NORTH_SEA);
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], sz::SZ_BALTIC_SEA);
        assert_eq!(path[2], sz::SZ_NORTH_SEA);
    }

    // ----- Straits / Canals -----

    #[test]
    fn four_straits_defined() {
        let m = map();
        assert_eq!(m.straits.len(), 4);
    }

    #[test]
    fn turkish_straits_connects_black_and_aegean() {
        let m = map();
        let strait = &m.straits[strait_ids::STRAIT_TURKISH as usize];
        assert_eq!(strait.connects_seas.0, sz::SZ_BLACK_SEA);
        assert_eq!(strait.connects_seas.1, sz::SZ_AEGEAN_SEA);
    }

    #[test]
    fn suez_canal_has_no_land_connection() {
        let m = map();
        let suez = &m.straits[strait_ids::STRAIT_SUEZ as usize];
        assert!(suez.connects_land.is_none());
    }

    #[test]
    fn strait_passable_when_controlled() {
        let m = map();
        // If we control Egypt, Suez is passable
        assert!(m.strait_is_passable(strait_ids::STRAIT_SUEZ, |tid| tid == t::EGYPT));
        // If we don't control Egypt, Suez is not passable
        assert!(!m.strait_is_passable(strait_ids::STRAIT_SUEZ, |_| false));
    }

}
