# Axis & Allies Global 1940 - Digital Edition

## Project Overview

A faithful digital recreation of Axis & Allies Global 1940 2nd Edition (combined Europe + Pacific), built as a desktop app (Tauri) and web app. The game features a tabletop-faithful 3D board rendered with Three.js, a Rust game engine compiled to WASM for client-server code sharing, and support for local hotseat, online multiplayer, and single-player vs AI.

---

## Design Decisions Summary

| Decision | Choice |
|---|---|
| Platform | Desktop (Tauri) + Web |
| 3D Engine | Three.js via React Three Fiber |
| UI Framework | React + TypeScript |
| State Management | Zustand |
| Game Engine | Rust -> WASM (single source of truth) |
| Backend | Rust (axum + tokio + WebSockets) |
| Visual Style | Tabletop faithful |
| Play Modes | Local hotseat, Online multiplayer, Single-player vs AI |
| Rules | Strict official 2nd Edition |
| AI | Basic heuristic-based |
| Map Data | SVG paths -> 3D extrusion |
| Interaction | Click territories + side panels |
| Turn Flow | Strict phase enforcement with undo |
| Save/Load | Yes (MessagePack format) |
| Audio | Not now |

---

## Tech Stack

- **Frontend**: React 18+, TypeScript, Vite
- **3D Rendering**: Three.js via `@react-three/fiber` + `@react-three/drei`
- **State Management**: Zustand
- **Game Engine**: Rust (`aa-engine` crate), compiled to WASM via `wasm-pack` + `wasm-bindgen`
- **Desktop Shell**: Tauri v2
- **Multiplayer Server**: Rust with `axum` (HTTP + WebSocket) + `tokio` (async runtime)
- **Serialization**: `serde` + `serde_json` (WASM bridge), `rmp-serde` / MessagePack (save files)
- **Type Generation**: `ts-rs` crate (auto-generates TypeScript types from Rust structs)
- **SVG Parsing**: `svg-path-parser` (npm) + `earcut` (triangulation)
- **RNG**: `rand_chacha` (deterministic, seedable PRNG for reproducible dice rolls)

---

## Architecture Overview

```
+------------------+       +------------------+       +------------------+
|   React + R3F    | <---> |   Zustand Store  | <---> |   WASM Engine    |
|   (UI + 3D)      |       |   (gameStore)    |       |   (aa-engine)    |
+------------------+       +------------------+       +------------------+
        |                                                     |
        |  (online mode)                                      |
        v                                                     v
+------------------+                              +------------------+
|  WebSocket Client| <--- Internet --->           | Rust Server      |
|  (browser)       |                              | (aa-server)      |
+------------------+                              | + native engine  |
                                                  +------------------+
```

**Key Principle**: The `aa-engine` Rust crate is the single source of truth for ALL game rules. It compiles to native Rust for the server/Tauri and to WASM for the browser. No game logic lives in TypeScript. The frontend is purely a view and input layer.

**Data Flow**:
1. User clicks territory -> React event handler
2. Handler calls `gameStore.submitAction(action)`
3. Store calls WASM engine's `submitAction(actionJson)`
4. Engine validates, applies, returns `ActionResult` JSON
5. Store updates `gameState` from engine
6. React re-renders UI + 3D scene from new state

**Online Mode**: Client sends actions over WebSocket -> Server validates via its own native `Engine` instance -> Broadcasts `ActionResult` to all clients -> Clients apply result to their local engine copy.

---

## Directory Structure

```
Axis-Allies/
├── Cargo.toml                           # Rust workspace root
├── package.json                         # Node workspace root (if needed)
│
├── crates/
│   ├── aa-engine/                       # Core game logic (Rust, compiles to WASM)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                   # Engine public API
│   │       ├── state.rs                 # GameState top-level container
│   │       ├── power.rs                 # Power enum, Team, turn order
│   │       ├── territory.rs             # TerritoryDef, TerritoryState
│   │       ├── map.rs                   # GameMap graph, adjacency, pathfinding
│   │       ├── unit.rs                  # UnitType, UnitStats, UnitInstance
│   │       ├── phase.rs                 # Phase state machine, PhaseState variants
│   │       ├── action.rs                # Action enum, AppliedAction, InverseAction
│   │       ├── validate.rs              # Action validation dispatcher
│   │       ├── apply.rs                 # Action application dispatcher
│   │       ├── purchase.rs              # Purchase & Repair phase logic
│   │       ├── movement.rs              # Combat & Non-Combat movement validation
│   │       ├── combat.rs                # Combat resolution engine
│   │       ├── bombing.rs              # Strategic & Tactical bombing raids
│   │       ├── amphibious.rs            # Amphibious assault rules
│   │       ├── convoy.rs               # Convoy disruption
│   │       ├── mobilize.rs              # Unit placement / mobilization rules
│   │       ├── income.rs                # Income collection
│   │       ├── politics.rs              # War declarations, neutral nations
│   │       ├── special.rs               # Kamikaze, straits/canals, China rules
│   │       ├── objectives.rs            # National Objectives definitions + eval
│   │       ├── victory.rs               # Victory condition checking
│   │       ├── dice.rs                  # Deterministic RNG (ChaCha8)
│   │       ├── setup.rs                 # Initial game setup (OOB placements)
│   │       ├── serialize.rs             # Serialization helpers
│   │       ├── error.rs                 # Error types
│   │       ├── data/                    # Static game data compiled into binary
│   │       │   ├── mod.rs
│   │       │   ├── territories.rs       # ~150 territory definitions
│   │       │   ├── sea_zones.rs         # Sea zone definitions
│   │       │   ├── adjacency.rs         # Full adjacency graph
│   │       │   ├── units.rs             # Unit stats tables
│   │       │   ├── objectives.rs        # National Objectives data
│   │       │   ├── setup_global1940.rs  # OOB unit placements for all powers
│   │       │   └── straits.rs           # Strait/canal definitions
│   │       └── ai/                      # AI player logic
│   │           ├── mod.rs               # BasicAI entry point
│   │           ├── evaluator.rs         # Board evaluation heuristics
│   │           ├── purchase.rs          # Purchase decisions
│   │           ├── combat_move.rs       # Attack planning
│   │           ├── combat.rs            # Casualty selection
│   │           ├── noncombat_move.rs    # Non-combat movement
│   │           ├── mobilize.rs          # Placement decisions
│   │           └── strategy.rs          # Strategic posture assessment
│   │
│   ├── aa-wasm/                         # WASM bridge (thin wrapper)
│   │   ├── Cargo.toml                   # wasm-bindgen, aa-engine
│   │   └── src/
│   │       ├── lib.rs                   # wasm_bindgen exported functions
│   │       └── conversions.rs           # JsValue conversion helpers
│   │
│   └── aa-server/                       # Multiplayer WebSocket server
│       ├── Cargo.toml                   # axum, tokio, aa-engine
│       └── src/
│           ├── main.rs                  # Server entry point (HTTP + WS)
│           ├── server.rs                # ServerState, room registry
│           ├── room.rs                  # GameRoom (one Engine per room)
│           ├── session.rs               # Player session / connection
│           ├── protocol.rs              # ClientMessage / ServerMessage types
│           ├── auth.rs                  # Room codes, player tokens
│           └── error.rs
│
├── client/                              # React + TypeScript frontend
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   ├── index.html
│   ├── public/
│   │   └── assets/
│   │       ├── map/
│   │       │   ├── territories.svg      # SVG paths for all territory outlines
│   │       │   ├── sea_zones.svg        # SVG paths for sea zone boundaries
│   │       │   └── decorations.svg      # Labels, borders, decorative elements
│   │       └── textures/
│   │           ├── board_background.jpg # Wood/cardboard table texture
│   │           ├── ocean.jpg            # Ocean water texture
│   │           └── unit_icons/          # 2D icons for unit token tops
│   └── src/
│       ├── main.tsx                     # Entry point, WASM init
│       ├── App.tsx                      # Router: MainMenu | GameScreen
│       ├── wasm/
│       │   ├── loader.ts               # Async WASM initialization
│       │   ├── engine.ts               # TypeScript GameEngine wrapper class
│       │   ├── types.ts                # Auto-generated TS types (from ts-rs)
│       │   └── actions.ts              # Action builder helpers
│       ├── stores/
│       │   ├── gameStore.ts            # Core game state synced from WASM
│       │   ├── uiStore.ts             # UI state (selections, panels, hover)
│       │   ├── settingsStore.ts        # App settings
│       │   └── multiplayerStore.ts     # WebSocket connection state
│       ├── hooks/
│       │   ├── usePhaseUI.ts           # Phase-driven UI panel switching
│       │   ├── useCurrentPower.ts      # Current power info shortcut
│       │   ├── useTerritoryInfo.ts     # Territory details for info panel
│       │   ├── useMultiplayer.ts       # WebSocket connection management
│       │   └── useAutoSave.ts          # Auto-save on phase transitions
│       ├── components/
│       │   ├── MainMenu.tsx
│       │   ├── GameSetupScreen.tsx
│       │   ├── GameScreen.tsx          # Main game layout container
│       │   ├── TopBar/
│       │   │   ├── TopBar.tsx
│       │   │   ├── TurnIndicator.tsx   # "Turn 3 - Germany"
│       │   │   ├── PhaseIndicator.tsx  # "Combat Movement"
│       │   │   └── UndoButton.tsx
│       │   ├── SidePanel/
│       │   │   ├── SidePanel.tsx       # Panel switcher by phase
│       │   │   ├── TerritoryInfoPanel.tsx
│       │   │   ├── PurchasePanel.tsx
│       │   │   ├── MovementPanel.tsx
│       │   │   ├── CombatPanel.tsx
│       │   │   ├── MobilizePanel.tsx
│       │   │   └── IncomePanel.tsx
│       │   ├── BottomBar/
│       │   │   ├── BottomBar.tsx
│       │   │   ├── PhaseActionButtons.tsx  # "Confirm Phase", "Undo"
│       │   │   └── PowerIPCDisplay.tsx
│       │   ├── BattleOverlay/
│       │   │   ├── BattleOverlay.tsx   # Modal overlay for combat
│       │   │   ├── BattleHeader.tsx
│       │   │   ├── AttackerPanel.tsx
│       │   │   ├── DefenderPanel.tsx
│       │   │   ├── DiceDisplay.tsx
│       │   │   ├── CasualtySelector.tsx
│       │   │   └── BattleControls.tsx  # Continue / Retreat / Submerge
│       │   ├── EventLog.tsx
│       │   ├── SaveLoadModal.tsx
│       │   └── SettingsModal.tsx
│       ├── scene/                      # React Three Fiber 3D components
│       │   ├── GameScene.tsx           # Top-level R3F scene
│       │   ├── CameraController.tsx    # Orbit/pan/zoom (top-down tilt)
│       │   ├── AmbientLighting.tsx     # Scene lighting
│       │   ├── board/
│       │   │   ├── BoardSurface.tsx    # Ocean plane + table texture
│       │   │   ├── TerritoryMeshGroup.tsx
│       │   │   ├── TerritoryMesh.tsx   # Single territory 3D mesh
│       │   │   └── SeaZoneOverlay.tsx  # Sea zone boundary lines
│       │   ├── units/
│       │   │   ├── UnitTokenGroup.tsx  # Container for all unit stacks
│       │   │   ├── UnitStack.tsx       # Stack of tokens per region+power
│       │   │   └── UnitToken.tsx       # Single unit piece (colored 3D shape)
│       │   ├── effects/
│       │   │   ├── SelectionHighlight.tsx  # Glow on selected territory
│       │   │   ├── MoveArrows.tsx     # Arrows showing planned moves
│       │   │   └── HoverHighlight.tsx
│       │   ├── map/
│       │   │   ├── svgParser.ts       # Parse SVG paths to point arrays
│       │   │   ├── triangulate.ts     # Earcut polygon triangulation
│       │   │   ├── extrude.ts         # Extrude flat mesh into 3D
│       │   │   ├── territoryGeometryCache.ts  # IndexedDB cache
│       │   │   └── mapData.ts         # Territory metadata for rendering
│       │   └── hooks/
│       │       ├── useTerritoryClick.ts
│       │       ├── useTerritoryHover.ts
│       │       └── useUnitClick.ts
│       ├── multiplayer/
│       │   ├── WebSocketClient.ts     # Connection manager
│       │   ├── protocol.ts            # Message types (mirrors Rust)
│       │   └── reconnect.ts           # Exponential backoff reconnection
│       ├── data/
│       │   ├── powerColors.ts         # Color per power (Germany gray, etc.)
│       │   ├── unitDisplayNames.ts
│       │   └── phaseDescriptions.ts
│       ├── types/
│       │   ├── game.ts                # Core game types (mirrors Rust)
│       │   ├── ui.ts                  # UI-specific types
│       │   └── multiplayer.ts
│       └── utils/
│           ├── formatIPC.ts
│           ├── powerUtils.ts
│           └── territoryUtils.ts
│
└── src-tauri/                          # Tauri desktop shell
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── build.rs
    └── src/
        ├── main.rs                    # Tauri app entry
        └── commands.rs                # Tauri commands (file save/load dialogs)
```

---

## Data Model (Rust)

### Powers & Teams

```rust
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Power {
    Germany = 0,
    SovietUnion = 1,
    Japan = 2,
    UnitedStates = 3,
    China = 4,
    UnitedKingdom = 5,   // Single turn, split economy (Europe + Pacific)
    Italy = 6,
    ANZAC = 7,
    France = 8,
}

pub const TURN_ORDER: [Power; 9] = [
    Power::Germany, Power::SovietUnion, Power::Japan,
    Power::UnitedStates, Power::China, Power::UnitedKingdom,
    Power::Italy, Power::ANZAC, Power::France,
];

pub enum Team { Axis, Allies }
// Axis: Germany, Japan, Italy
// Allies: Soviet Union, USA, China, UK, ANZAC, France
```

### Game State

```rust
pub struct GameState {
    pub turn_number: u32,
    pub current_power: Power,
    pub current_phase: Phase,
    pub phase_state: PhaseState,          // Sub-state for current phase

    pub territories: Vec<TerritoryState>, // Indexed by TerritoryId
    pub sea_zones: Vec<SeaZoneState>,     // Indexed by SeaZoneId
    pub powers: [PowerState; 9],          // Per-power state

    pub political: PoliticalState,        // War declarations, neutrals
    pub pending_combats: Vec<PendingCombat>,
    pub current_combat: Option<ActiveCombat>,

    pub action_log: Vec<AppliedAction>,   // For undo
    pub undo_checkpoints: Vec<usize>,     // Phase boundary indices

    pub victory_cities: [(TerritoryId, Option<Team>); 18],
    pub rng_seed: u64,                    // Deterministic RNG seed
    pub rng_counter: u64,                 // RNG consumption counter
}
```

### Phases

```rust
pub enum Phase {
    PurchaseAndRepair,
    CombatMovement,
    ConductCombat,
    NonCombatMovement,
    Mobilize,
    CollectIncome,
}

// Each phase has sub-state tracking what's been done and what's pending
pub enum PhaseState {
    Purchase(PurchaseState),        // Queued purchases, repairs, IPCs spent
    CombatMove(CombatMoveState),    // Moved units, pending orders
    Combat(CombatState),            // Battles remaining, current battle
    NonCombatMove(NonCombatMoveState),
    Mobilize(MobilizeState),        // Units to place, where placed so far
    CollectIncome(CollectIncomeState),
}
```

### Territories & Map

```rust
pub type TerritoryId = u16;
pub type SeaZoneId = u16;
pub enum RegionId { Land(TerritoryId), Sea(SeaZoneId) }

// Static definition (never changes during game)
pub struct TerritoryDef {
    pub id: TerritoryId,
    pub name: String,
    pub ipc_value: u32,
    pub is_capital: Option<Power>,
    pub is_victory_city: bool,
    pub original_owner: Power,
    pub territory_type: TerritoryType,  // Normal, ProNeutral, TrueNeutral, etc.
    pub adjacent_land: Vec<TerritoryId>,
    pub adjacent_sea: Vec<SeaZoneId>,
    pub strait_connections: Vec<(TerritoryId, StraitId)>,
    pub convoys_from: Vec<SeaZoneId>,
    pub is_island: bool,
}

// Mutable state during a game
pub struct TerritoryState {
    pub owner: Option<Power>,
    pub units: Vec<UnitInstance>,
    pub facilities: Vec<Facility>,
    pub just_captured: bool,           // Can't mobilize here this turn
}
```

### Units

```rust
pub type UnitId = u32;

pub enum UnitType {
    Infantry, MechInfantry, Artillery, Tank, AAA,
    Fighter, TacticalBomber, StrategicBomber,
    Transport, Submarine, Destroyer, Cruiser, Carrier, Battleship,
}

pub struct UnitStats {
    pub cost: u32,
    pub attack: u8,
    pub defense: u8,
    pub movement: u8,
    pub domain: UnitDomain,          // Land, Air, Sea
    pub hit_points: u8,              // 2 for Battleship & Carrier
    pub can_bombard: bool,
    pub bombardment_value: u8,
    pub transport_capacity: u8,
    pub can_carry_air: u8,           // Carrier: 2
    pub special_abilities: Vec<SpecialAbility>,
}

pub struct UnitInstance {
    pub id: UnitId,
    pub unit_type: UnitType,
    pub owner: Power,
    pub hits_taken: u8,              // 0 = healthy, 1 = damaged (BB/CV)
    pub moved_this_turn: bool,
    pub movement_remaining: u8,
    pub cargo: Vec<UnitId>,          // For transports/carriers
}
```

### Facilities

```rust
pub enum FacilityType {
    MinorIndustrialComplex,  // Produces up to territory IPC value (max 3)
    MajorIndustrialComplex,  // Produces up to territory IPC value (max 10)
    AirBase,                 // +1 movement for air, scramble up to 3
    NavalBase,               // +1 movement for naval, repair BB/CV
}

pub struct Facility {
    pub facility_type: FacilityType,
    pub damage: u32,                 // From bombing raids
    pub max_damage: u32,             // IC: 2x territory value; Bases: 6
    pub operational: bool,
}
```

### Political State

```rust
pub struct PoliticalState {
    pub war_state: WarMatrix,        // Symmetric who-is-at-war-with-whom
    pub triggers: PoliticalTriggers, // US entry, Soviet war, Mongolia, etc.
    pub neutrals: Vec<NeutralState>,
}

pub struct PoliticalTriggers {
    pub us_at_war: bool,
    pub us_war_turn: Option<u32>,
    pub soviet_at_war_with_axis: bool,
    pub japan_attacked_uk_anzac: bool,
    pub japan_attacked_soviet_or_mongolia: bool,
    pub london_captured: bool,
    pub paris_captured: bool,
    // ... etc
}
```

---

## Action / Command System (Undo Support)

Every player interaction is an `Action`. The engine validates, applies, and records an inverse for undo.

### Action Types

```rust
pub enum Action {
    // Purchase Phase
    PurchaseUnit { unit_type: UnitType, count: u32 },
    RemovePurchase { unit_type: UnitType, count: u32 },
    RepairFacility { territory_id: TerritoryId, damage_to_repair: u32 },
    ConfirmPurchases,

    // Combat Movement Phase
    MoveUnit { unit_id: UnitId, path: Vec<RegionId> },
    SetAmphibiousSource { unit_id: UnitId, transport_id: UnitId },
    SetBombingRaidTarget { unit_id: UnitId, target: BombingTarget },
    UndoMove { unit_id: UnitId },
    ConfirmCombatMovement,

    // Combat Phase
    SelectBattleOrder { location: RegionId },
    RollAttack,
    RollDefense,
    SelectCasualties { casualties: Vec<UnitId> },
    AttackerRetreat { to: RegionId },
    SubmergeSubmarine { unit_id: UnitId },
    ContinueCombatRound,

    // Non-Combat Movement Phase
    MoveUnitNonCombat { unit_id: UnitId, path: Vec<RegionId> },
    LandAirUnit { unit_id: UnitId, territory_id: RegionId },
    ConfirmNonCombatMovement,

    // Mobilize Phase
    PlaceUnit { unit_type: UnitType, territory_id: TerritoryId },
    ConfirmMobilization,

    // Collect Income (mostly automatic)
    ConfirmIncome,

    // Political
    DeclareWar { against: Power },

    // Meta
    Undo,
    ConfirmPhase,
}
```

### Undo Rules

- **Freely undoable**: Purchases, movement orders, unit placements (within current phase)
- **NOT undoable**: Dice rolls, combat resolution steps, phase transitions
- **Phase reset**: "Reset Phase" button reverts all actions back to the phase start checkpoint
- Each `AppliedAction` stores an `InverseAction` (either a `RestorePartial` snapshot or a `Simple` reverse action)
- `undo_checkpoints` records action_log indices at each phase boundary

---

## Engine Public API

```rust
pub struct Engine {
    state: GameState,
}

impl Engine {
    // Lifecycle
    pub fn new_game(setup: GameSetup, seed: u64) -> Self;
    pub fn from_state(state: GameState) -> Self;

    // Core loop
    pub fn submit_action(&mut self, action: Action) -> Result<ActionResult, EngineError>;
    pub fn state(&self) -> &GameState;

    // UI Queries
    pub fn legal_actions(&self) -> Vec<LegalAction>;
    pub fn is_action_legal(&self, action: &Action) -> Result<(), EngineError>;
    pub fn legal_moves_for_unit(&self, unit_id: UnitId) -> Vec<RegionId>;
    pub fn purchasable_units(&self) -> Vec<(UnitType, u32)>;
    pub fn legal_placement_locations(&self, unit_type: UnitType) -> Vec<TerritoryId>;
    pub fn can_undo(&self) -> bool;
    pub fn check_victory(&self) -> Option<VictoryResult>;

    // Serialization
    pub fn serialize_state(&self) -> Vec<u8>;        // MessagePack
    pub fn serialize_state_json(&self) -> String;     // JSON for WASM
    pub fn deserialize_state(data: &[u8]) -> Result<GameState, SerializeError>;

    // AI
    pub fn run_ai_turn(&mut self) -> Vec<Action>;
}

// Returned after every action
pub struct ActionResult {
    pub applied: AppliedAction,
    pub state_updates: Vec<StateUpdate>,
    pub events: Vec<GameEvent>,
}

// Narrative events for the event log
pub enum GameEvent {
    PhaseChanged { from: Phase, to: Phase },
    TurnChanged { power: Power, turn: u32 },
    WarDeclared { aggressor: Power, target: Power },
    BattleStarted { location: RegionId },
    BattleEnded { location: RegionId, outcome: BattleOutcome },
    CapitalCaptured { capital: TerritoryId, by: Power },
    TerritoryLiberated { territory: TerritoryId, to: Power },
    NationalObjectiveAchieved { power: Power, objective: ObjectiveId },
    ConvoyDisrupted { zone: SeaZoneId, power: Power, lost_ipcs: u32 },
    VictoryAchieved { winner: Team },
}
```

---

## WASM Bridge

### Strategy
- JSON serialization across the WASM boundary (via `serde_json`)
- Acceptable performance for a turn-based game (state is ~50-100KB JSON, changes only on player actions)
- TypeScript types auto-generated from Rust using the `ts-rs` crate

### WASM API (`aa-wasm/src/lib.rs`)

```rust
#[wasm_bindgen]
pub struct WasmEngine { engine: Engine }

#[wasm_bindgen]
impl WasmEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u64) -> WasmEngine;

    pub fn from_state(state_json: &str) -> Result<WasmEngine, JsValue>;
    pub fn submit_action(&mut self, action_json: &str) -> String;  // JSON result
    pub fn get_state(&self) -> String;                              // JSON state
    pub fn legal_actions(&self) -> String;
    pub fn legal_moves_for_unit(&self, unit_id: u32) -> String;
    pub fn purchasable_units(&self) -> String;
    pub fn legal_placement_locations(&self, unit_type_json: &str) -> String;
    pub fn can_undo(&self) -> bool;
    pub fn check_victory(&self) -> String;
    pub fn serialize_for_save(&self) -> Vec<u8>;
    pub fn load_from_save(data: &[u8]) -> Result<WasmEngine, JsValue>;
    pub fn run_ai_turn(&mut self) -> String;
}
```

### TypeScript Wrapper (`client/src/wasm/engine.ts`)

Wraps raw WASM calls with typed TypeScript methods. Parses JSON responses into typed objects. Exposes a clean API that the Zustand store calls.

---

## 3D Rendering Architecture

### SVG-to-3D Pipeline

1. **Source**: `territories.svg` - Each territory is a `<path>` with `id` matching the territory ID
2. **Parse**: `svgParser.ts` converts SVG path commands to polygon point arrays
3. **Triangulate**: `triangulate.ts` uses `earcut` to create triangle meshes from polygons
4. **Extrude**: `extrude.ts` raises land territories slightly above sea level (tabletop look)
5. **Cache**: Generated `BufferGeometry` objects cached in IndexedDB after first load

### Territory Rendering

- Each territory is a `<TerritoryMesh>` R3F component with `MeshStandardMaterial`
- Color = owner's power color (Germany gray, Japan orange, USA green, UK tan, USSR red, Italy brown, ANZAC light gold, France blue, China olive)
- Selected territory: yellow emissive glow
- Hovered territory: subtle highlight
- Click detection via R3F's built-in raycasting

### Unit Token Rendering

- Units are colored 3D tokens (cylinders/discs) with unit-type silhouettes
- Uses `THREE.InstancedMesh` for performance (hundreds of units on board)
- Units grouped into "stacks" per territory per power
- Stack layout: grid pattern within territory bounds, scaled to territory area
- LOD: At far zoom, show single icon with count badge instead of individual tokens

### Camera System

- `OrbitControls` from `@react-three/drei`
- Mostly top-down view (polar angle limited to 10-45 degrees)
- Pan via middle mouse drag or arrow keys
- Zoom via scroll wheel (clamped min/max)
- Double-click territory to smooth-focus camera on it

---

## Multiplayer Architecture

### Server Design

- **Runtime**: `tokio` async runtime
- **HTTP/WS**: `axum` with WebSocket upgrade
- **Authority**: Server is authoritative (has its own `Engine` instance per room)
- **Rooms**: Each game is a `GameRoom` with a unique room code

### Protocol (JSON over WebSocket)

```rust
// Client -> Server
pub enum ClientMessage {
    CreateRoom { config: RoomConfig },
    JoinRoom { room_id: String, player_token: String },
    SubmitAction { action: Action },
    RequestSync,
    Ping,
}

// Server -> Client
pub enum ServerMessage {
    RoomJoined { power: Power, state: GameState, players: Vec<PlayerInfo> },
    ActionApplied { power: Power, result: ActionResult },
    ActionRejected { reason: String },
    StateSync { state: GameState },
    PlayerStatus { power: Power, online: bool },
    Pong,
    Error { message: String },
}
```

### Reconnection

- Players get a UUID `player_token` on join (stored in localStorage)
- On disconnect: client auto-reconnects with exponential backoff
- On reconnect: sends `JoinRoom` with same token; server sends full `StateSync`
- Rooms persist for 24 hours after last connection (configurable)
- Server auto-saves room state periodically

---

## AI Architecture

### Design: Heuristic-based (no ML)

The AI lives inside `aa-engine/src/ai/` and has full access to game state and validation functions. Invoked via `Engine::run_ai_turn()`.

### Board Evaluation (`evaluator.rs`)

Scores the board from a team's perspective using weighted factors:
- IPC income (1.0x weight)
- Total unit value (0.5x)
- Victory city control (3.0x)
- Capital safety (2.0x)
- National objectives met (1.5x)
- Strategic positioning (0.8x)

### Decision Modules

| Module | Logic |
|---|---|
| `purchase.rs` | Assess strategic posture (offensive/defensive/naval), buy units in weighted ratios |
| `combat_move.rs` | Identify attack opportunities, compute battle odds, greedily assign units to attacks with >60% win probability |
| `combat.rs` | Select cheapest casualties first (infantry before tanks), keep at least one capturing unit |
| `noncombat_move.rs` | Reinforce threatened territories, stack infantry on front lines, land air units safely |
| `mobilize.rs` | Place units near the front lines, prioritize threatened capitals |
| `strategy.rs` | Assess overall posture per power (Germany: attack Russia vs defend France, Japan: push India vs island-hop, etc.) |

---

## Save/Load System

### Format: MessagePack (binary, compact)

```rust
pub struct SaveFile {
    pub header: SaveHeader,
    pub game_state: GameState,
}

pub struct SaveHeader {
    pub version: u32,
    pub engine_version: String,
    pub game_name: String,
    pub created_at: u64,
    pub turn_summary: String,         // "Turn 3 - Germany - Combat Movement"
    pub player_config: PlayerConfig,
}
```

### File Extension: `.aa1940`

### Flow
- **Save**: User clicks Save -> WASM serializes state to `Uint8Array` -> Tauri file dialog writes to disk
- **Load**: User clicks Load -> Tauri file dialog reads bytes -> WASM deserializes -> Game store updates
- **Auto-save**: After every phase transition, save to IndexedDB (rolling buffer of 3 auto-saves)
- **Version migration**: Save header includes format version; engine migrates old saves forward

---

## Special Rules Implementation

### UK Split Economy (`purchase.rs`, `income.rs`)
- UK has two IPC pools: Europe and Pacific
- During UK's turn, purchases handled in two sub-phases (Europe, then Pacific)
- Income collected separately per theater based on territory locations

### China (`special.rs`)
- Can only purchase infantry (exception: receives 1 Fighter from US)
- Chinese units cannot leave Chinese territories (exception: Burma Road)
- No capturable capital (IPCs are lost if China has 0 territories, not captured)
- Collects IPCs for controlled Chinese territories + Burma

### Straits & Canals (`special.rs`)
- **Turkish Straits**: Requires friendly control of Turkey
- **Suez Canal**: Requires friendly control of Egypt + Trans-Jordan
- **Panama Canal**: Requires friendly control of Central America
- **Danish Straits**: Requires friendly control of Denmark
- Each strait connects specific sea zones; passage blocked if controller is hostile

### Kamikaze (`special.rs`)
- Japan only, 6 tokens
- Available in specific Pacific sea zones
- Used during combat in those zones, adds 1 hit at defense value 2

### Neutral Nations (`politics.rs`)
- **Pro-Axis/Pro-Allied**: Join their side when an ally power enters their territory
- **True Neutral**: Join the opposing side if attacked; attacking triggers ALL true neutrals to become pro-enemy
- **Strict Neutral**: Cannot be entered (Sahara, Himalayas = impassable)

---

## UI Flow Details

### Combat Resolution Flow
1. Engine identifies all battles -> UI shows battle list in CombatPanel
2. Player selects a battle to resolve
3. **BattleOverlay** modal opens showing attackers (left) vs defenders (right)
4. Pre-battle: AA fire / shore bombardment / submarine surprise strike (auto-resolved, shown to player)
5. Each round: Roll Attack -> show dice -> Roll Defense -> show dice -> Select casualties (both sides) -> Continue or Retreat
6. Battle ends when one side eliminated or attacker retreats
7. Territory control updates, next battle

### Purchase Phase Flow
1. SidePanel shows PurchasePanel with all unit types, +/- buttons, costs
2. Running IPC total updates as units are added/removed
3. UK gets two tabs: "Europe" and "Pacific"
4. "Confirm Purchases" advances to Combat Movement

### Movement Phase Flow
1. Click a territory to see its units in SidePanel
2. Click a unit to select it; legal destinations highlight on board
3. Click a destination to move; arrow appears showing the path
4. Repeat for all units; "Confirm Movement" advances phase

### Mobilize Phase Flow
1. SidePanel shows purchased units not yet placed
2. Click a unit type to "pick up", legal territories highlight on board
3. Click a territory to place (respects factory production limits)
4. Repeat until all placed; "Confirm Mobilization" advances

---

## Build Pipeline

1. `cargo build -p aa-engine` -- compile core engine
2. `wasm-pack build crates/aa-wasm --target web` -- compile WASM + JS bindings
3. `cargo test -p aa-engine` -- runs tests + generates TypeScript types via `ts-rs`
4. Copy generated `.ts` types to `client/src/wasm/types.ts`
5. `npm run dev` (Vite) -- serves frontend with hot reload
6. `cargo tauri dev` -- wraps everything in Tauri desktop shell

---

## Error Handling

- Engine NEVER panics. All public methods return `Result<T, EngineError>`
- WASM bridge converts Rust errors to JSON error objects
- Frontend displays errors as toast notifications
- Error types are descriptive and specific (InsufficientIPCs, IllegalMove, NotYourTurn, WrongPhase, etc.)

---

## Development Phases (Recommended Workflow)

I recommend a **hybrid approach**: phase-by-phase for the foundation, then vertical feature slices once stable. Each phase below represents roughly one conversation session.

### Phase 1: Project Scaffolding
- Initialize Rust workspace with `aa-engine`, `aa-wasm`, `aa-server` crates
- Initialize React + Vite + TypeScript client
- Set up Tauri desktop shell
- Configure `wasm-pack` build pipeline
- Verify end-to-end: Rust -> WASM -> TypeScript -> React renders "Hello"

### Phase 2: Core Data Model
- Implement all data structures: `Power`, `Team`, `UnitType`, `UnitStats`, `TerritoryDef`, `SeaZoneDef`, `GameState`
- Implement the full map data: all ~150 territories, sea zones, adjacency graph
- Implement `GameMap` with pathfinding
- Unit tests for adjacency and pathfinding

### Phase 3: Phase State Machine + Basic Engine
- Implement `Phase`, `PhaseState`, phase transitions
- Implement `Action` enum and `submit_action` dispatcher
- Implement undo infrastructure (`AppliedAction`, `InverseAction`, checkpoints)
- Implement `DeterministicRng` (dice)
- Wire up WASM bridge with basic `new_game()` + `get_state()` + `submit_action()`

### Phase 4: 3D Board Rendering
- Create SVG map with territory outlines (this is a significant art/data task)
- Implement SVG-to-3D pipeline (parse, triangulate, extrude)
- Render the full board with territory meshes colored by owner
- Implement camera controls (orbit, pan, zoom)
- Implement territory click/hover highlighting
- Display territory info in side panel

### Phase 5: Purchase Phase (Full Vertical Slice)
- Engine: Purchase validation, IPC tracking, unit costs
- Engine: Repair facilities logic
- Engine: UK split economy handling
- UI: PurchasePanel with +/- buttons, IPC counter
- UI: Phase bar, confirm button, undo button
- Wire everything end-to-end

### Phase 6: Movement Phases (Full Vertical Slice)
- Engine: Movement validation (land, sea, air range, blitzing)
- Engine: Transport loading/unloading rules
- Engine: Strait/canal passage checks
- Engine: Non-combat movement rules (air unit landing)
- UI: Unit selection, destination highlighting, move arrows
- UI: MovementPanel in side panel

### Phase 7: Combat Resolution (Full Vertical Slice)
- Engine: Battle identification
- Engine: AA fire, shore bombardment, submarine surprise strike
- Engine: Attack/defense rolling, hit calculation
- Engine: Casualty application, retreat/submerge
- Engine: Amphibious assault rules
- Engine: Strategic/tactical bombing raids
- UI: BattleOverlay modal with full combat flow
- UI: DiceDisplay, CasualtySelector, BattleControls

### Phase 8: Mobilize + Income + Victory
- Engine: Unit placement with factory limits
- Engine: Income collection, convoy disruption
- Engine: National objectives evaluation
- Engine: Victory condition checking
- UI: MobilizePanel, IncomePanel
- UI: Victory screen

### Phase 9: Political Rules + Special Rules
- Engine: War declarations, political triggers
- Engine: Neutral nation rules
- Engine: China special rules
- Engine: Kamikaze
- Engine: Capital capture/liberation/IPC seizure

### Phase 10: Initial Game Setup
- Implement OOB (Order of Battle) for Global 1940 2nd Edition
- All starting units for all 9 powers in correct territories
- Starting political state (who is at war with whom at game start)
- Game setup screen UI (choose power assignments)

### Phase 11: 3D Unit Tokens
- Procedural 3D models for each unit type
- Power-colored materials
- Unit stack layout algorithm within territories
- InstancedMesh optimization for performance

### Phase 12: AI Player
- Board evaluator with heuristic scoring
- Purchase AI (posture-based buying)
- Combat movement AI (attack opportunity identification)
- Casualty selection AI (cheapest first)
- Non-combat movement AI (reinforce, land air)
- Mobilization AI (place near front)
- Integration: AI turns run automatically, UI shows results

### Phase 13: Save/Load
- MessagePack serialization of GameState
- Save file format with header + versioning
- Tauri file dialogs for save/load
- Auto-save to IndexedDB on phase transitions
- SaveLoadModal UI

### Phase 14: Online Multiplayer
- Rust WebSocket server with axum + tokio
- GameRoom management
- Client WebSocket connection + reconnection
- Server-authoritative action validation
- State sync on reconnect
- Game lobby / room creation UI

### Phase 15: Polish & Integration Testing
- Full game playthrough testing
- Edge case rule verification
- Performance optimization (instanced rendering, geometry caching)
- UI polish (animations, transitions)
- Event log completeness

---

## Coding Guidelines

- **Rust**: Use `clippy` and `rustfmt`. All public APIs documented with `///` doc comments. Prefer `Result` over `panic`. Use `#[derive(Serialize, Deserialize, Debug, Clone)]` on all game state types. Use `#[derive(TS)]` from `ts-rs` on types that cross the WASM bridge.
- **TypeScript**: Strict mode enabled. No `any` types. Use interfaces for data, types for unions. Components use named exports. Hooks prefixed with `use`.
- **React**: Functional components only. Use `React.memo` for expensive renders (territory meshes). Zustand selectors for fine-grained subscriptions (avoid re-rendering the whole board on every state change).
- **Testing**: Rust unit tests for all game logic (especially combat resolution, movement validation, political rules). React component tests with Vitest + React Testing Library for critical UI flows. Integration test: simulate a full game turn via the engine API.
- **Git**: Feature branches, one feature per branch. Conventional commits (`feat:`, `fix:`, `refactor:`).

---

## Verification Plan

After each development phase, verify with:

1. **Unit tests**: `cargo test -p aa-engine` for all game logic
2. **WASM integration**: Verify TypeScript can call WASM engine and get correct results
3. **Visual verification**: Load the app, verify 3D board renders, territories are clickable, UI panels show correct data
4. **Gameplay test**: Play through at least one full turn (all 9 powers, all 6 phases) after core features are complete
5. **Full game test**: After Phase 15, play a complete game (5+ turns) to verify rules, victory conditions, and edge cases
