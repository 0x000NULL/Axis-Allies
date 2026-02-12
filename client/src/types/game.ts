/**
 * Core game types mirroring the Rust engine's data model.
 * These will be auto-generated from Rust via ts-rs in later phases.
 * For now, manually defined for scaffolding.
 */

export type Power =
  | 'Germany'
  | 'SovietUnion'
  | 'Japan'
  | 'UnitedStates'
  | 'China'
  | 'UnitedKingdom'
  | 'Italy'
  | 'ANZAC'
  | 'France';

export type Team = 'Axis' | 'Allies';

export type Phase =
  | 'PurchaseAndRepair'
  | 'CombatMovement'
  | 'ConductCombat'
  | 'NonCombatMovement'
  | 'Mobilize'
  | 'CollectIncome';

export type UnitType =
  | 'Infantry'
  | 'MechInfantry'
  | 'Artillery'
  | 'Tank'
  | 'AAA'
  | 'Fighter'
  | 'TacticalBomber'
  | 'StrategicBomber'
  | 'Transport'
  | 'Submarine'
  | 'Destroyer'
  | 'Cruiser'
  | 'Carrier'
  | 'Battleship';

export interface UnitInstance {
  id: number;
  unit_type: UnitType;
  owner: Power;
  hits_taken: number;
  moved_this_turn: boolean;
  movement_remaining: number;
  cargo: number[];
}

export interface TerritoryState {
  owner: Power | null;
  units: UnitInstance[];
  facilities: Facility[];
  just_captured: boolean;
}

export interface Facility {
  facility_type: string;
  damage: number;
  max_damage: number;
  operational: boolean;
}

export interface SeaZoneState {
  units: UnitInstance[];
}

export interface PowerState {
  power: Power;
  ipcs: number;
  ipcs_europe: number;
  ipcs_pacific: number;
  at_war: boolean;
  capital_captured: boolean;
  researched_techs: string[];
}

export interface PhaseState {
  [key: string]: unknown;
}

export interface GameState {
  turn_number: number;
  current_power: Power;
  current_phase: Phase;
  phase_state: PhaseState;
  territories: TerritoryState[];
  sea_zones: SeaZoneState[];
  powers: PowerState[];
  political: PoliticalState;
  action_log: unknown[];
  undo_checkpoints: number[];
  rng_seed: number;
  rng_counter: number;
}

export interface PoliticalState {
  war_matrix: boolean[][];
  triggers: PoliticalTriggers;
}

export interface PoliticalTriggers {
  us_at_war: boolean;
  us_war_turn: number | null;
  soviet_at_war_with_axis: boolean;
  japan_attacked_uk_anzac: boolean;
  london_captured: boolean;
  paris_captured: boolean;
}

export type RegionId = { Land: number } | { Sea: number };

// Action types — serde serializes unit variants (no fields) as plain strings
export type Action =
  | 'ConfirmPhase'
  | 'ConfirmPurchases'
  | 'ConfirmCombatMovement'
  | 'ConfirmNonCombatMovement'
  | 'ConfirmMobilization'
  | 'ConfirmIncome'
  | 'Undo'
  | { PurchaseUnit: { unit_type: UnitType; count: number } }
  | { RemovePurchase: { unit_type: UnitType; count: number } }
  | { RepairFacility: { territory_id: number; damage_to_repair: number } }
  | { MoveUnit: { unit_id: number; path: RegionId[] } }
  | { PlaceUnit: { unit_type: UnitType; territory_id: number } }
  | { DeclareWar: { against: Power } };

export interface LegalAction {
  action: Action;
  description: string;
}

export interface ActionResult {
  applied: {
    action: Action;
  };
  events: GameEvent[];
}

export type GameEvent =
  | { PhaseChanged: { from: Phase; to: Phase } }
  | { TurnChanged: { power: Power; turn: number } }
  | { WarDeclared: { aggressor: Power; target: Power } }
  | { VictoryAchieved: { winner: Team } }
  | { UnitsPurchased: { unit_type: UnitType; count: number; cost: number } }
  | { UnitsPlaced: { unit_type: UnitType; territory_id: number } }
  | { IncomeCollected: { power: Power; amount: number } }
  | { BattleStarted: { location: RegionId } }
  | { BattleEnded: { location: RegionId; attacker_won: boolean } }
  | { CapitalCaptured: { territory_id: number; by: Power } }
  | { TerritoryLiberated: { territory_id: number; to: Power } }
  | { ConvoyDisrupted: { zone: number; power: Power; lost_ipcs: number } };
