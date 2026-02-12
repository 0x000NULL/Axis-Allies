# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Axis & Allies Global 1940 2nd Edition — a digital board game with a Rust game engine compiled to both native and WASM, a React/Three.js frontend, and a multiplayer WebSocket server. The **aa-engine** crate is the single source of truth for all game logic; the frontend is purely presentational. See `phases.md` for the full design document and phased implementation roadmap.

## Build Commands

### Rust

```bash
cargo build                         # Build all crates (engine, wasm bridge, server)
cargo build -p aa-engine            # Build engine only
cargo test -p aa-engine             # Run engine tests (74 tests)
cargo test -p aa-engine -- test_name # Run a single test by name
cargo test                          # Run all workspace tests
cargo run -p aa-server              # Start multiplayer server on :3001
```

### WASM (must build before running frontend)

```bash
wasm-pack build crates/aa-wasm --target web --out-dir ../../client/src/wasm/pkg
# Or from client/:
cd client && npm run wasm:build
```

### Frontend

```bash
cd client && npm install            # Install dependencies (first time)
cd client && npm run dev            # Vite dev server on :5173
cd client && npm run build          # Production build (tsc + vite)
cd client && npm run type-check     # TypeScript check only (tsc --noEmit)
cd client && npm run lint           # ESLint
```

### Tauri Desktop

```bash
cargo tauri dev                     # Dev mode (wraps Vite + Tauri shell)
cargo tauri build                   # Production desktop binary
```

## Architecture

### Data Flow

```
User click → React handler → gameStore.submitAction(action)
  → WASM engine.submitAction(JSON) → validates → applies → returns ActionResult JSON
  → Zustand store updates gameState → React re-renders
```

Online mode adds: Client → WebSocket → Server (native engine validates) → broadcast to all clients.

### Crate Responsibilities

- **aa-engine** (`crates/aa-engine/`): All game rules, state, validation, phase transitions, dice. Compiles to native and WASM. Never panics — all public methods return `Result<T, EngineError>`.
- **aa-wasm** (`crates/aa-wasm/`): Thin `wasm-bindgen` wrapper. JSON strings cross the WASM boundary. No game logic here. Methods use camelCase names for JavaScript (`submitAction`, `getState`, `legalActions`, etc.).
- **aa-server** (`crates/aa-server/`): Axum WebSocket server. Each game room holds its own `Engine` instance (server-authoritative).

### Engine Action Pipeline

Every player interaction flows through the same path in `Engine::submit_action()`:

1. **`validate.rs`** — `validate_action(state, action)` checks the action is legal: correct phase, valid parameters. Returns `EngineError` on failure.
2. **`apply.rs`** — `apply_action(state, action)` mutates `GameState`, records an `AppliedAction` with its inverse to `action_log`, and returns `ActionResult` with events.
3. **`ActionResult`** contains the `AppliedAction` (for undo tracking) and `Vec<GameEvent>` (for UI feedback: `PhaseChanged`, `TurnChanged`, etc.).

Special case: `Action::Undo` is intercepted in `apply_action` before the normal flow. It pops the last `AppliedAction`, applies its inverse, and returns without logging itself.

### Undo System

- **`AppliedAction`** stores the original action and an `InverseAction` variant:
  - `Simple(Action)` — a reverse action to apply (e.g., `RemovePurchase` undoes `PurchaseUnit`)
  - `RestoreSnapshot(Vec<u8>)` — MessagePack snapshot of `PhaseState` for complex reversals
  - `Irreversible` — cannot be undone (phase transitions, dice rolls)
- **`undo_checkpoints`** — `Vec<usize>` recording `action_log.len()` at each phase boundary. Initialized with `[0]`.
- **`can_undo()`** returns true only if the last action's inverse is not `Irreversible`.
- Phase transitions (all `Confirm*` actions) are `Irreversible` — undo stops at phase boundaries.

### Phase Confirm Actions

Each phase has a specific confirm action. Using the wrong one returns `WrongPhase`:

| Phase | Confirm Action |
|-------|---------------|
| PurchaseAndRepair | `ConfirmPurchases` |
| CombatMovement | `ConfirmCombatMovement` |
| ConductCombat | `ConfirmPhase` |
| NonCombatMovement | `ConfirmNonCombatMovement` |
| Mobilize | `ConfirmMobilization` |
| CollectIncome | `ConfirmIncome` |

After `CollectIncome`, the engine advances to the next power in turn order. When all 9 powers complete, `turn_number` increments.

### Engine Data Module (`crates/aa-engine/src/data/`)

Static game map data for Global 1940 2nd Edition. All data is compiled into the binary (no external files).

- **`territory_ids.rs`** / **`sea_zone_ids.rs`**: Named `const u16` IDs for 164 territories and 80 sea zones. Contiguous 0-indexed for direct Vec indexing.
- **`territories.rs`**: `build_territory_defs()` — all territory definitions with name, IPC value, owner, type (Normal/ProAxis/ProAllies/TrueNeutral/Impassable), capital/victory city flags, and full adjacency lists (land, sea, strait connections).
- **`sea_zones.rs`**: `build_sea_zone_defs()` — all sea zone definitions with adjacency to other sea zones and coastal territories.
- **`strait_ids.rs`**: 4 strait/canal definitions (Turkish Straits, Suez, Panama, Danish Straits) with control territory and connected regions.
- **`mod.rs`**: `GameMap` struct with query methods (adjacency, coastal, neighbors) and BFS pathfinding (`find_land_path`, `find_sea_path`, `land_reachable_within`, `sea_reachable_within`). Constructed once per `Engine` instance, not serialized.

When adding or modifying territory/sea zone data, run `cargo test -p aa-engine -- adjacency_symmetry` to catch one-directional adjacency entries.

### Engine Core Concepts

- **Phase state machine**: `PurchaseAndRepair → CombatMovement → ConductCombat → NonCombatMovement → Mobilize → CollectIncome`, then next power in turn order.
- **Action system**: Every player interaction is an `Action` enum variant. Validated by `validate.rs`, applied by `apply.rs`. Each applied action records an inverse for undo.
- **9 powers** in fixed turn order: Germany, SovietUnion, Japan, UnitedStates, China, UnitedKingdom, Italy, ANZAC, France. Teams: Axis (Germany/Japan/Italy) vs Allies (rest).
- **Deterministic RNG**: ChaCha8 seeded from `GameState.rng_seed` — same seed + same actions = identical game.
- **Engine public API**: `submit_action()`, `legal_actions()`, `can_undo()`, `is_action_legal()`, `check_victory()`, `state()`, `map()`.

### Frontend Structure

- **Zustand stores**: `gameStore` (mirrors WASM engine state, exposes `submitAction`/`undo`/`canUndo`), `uiStore` (selections, panels, screen)
- **3D rendering**: React Three Fiber + drei. `GameScreen.tsx` hosts the Canvas with OrbitControls.
- **WASM bridge**: `client/src/wasm/engine.ts` wraps `WasmEngine` with typed methods. `App.tsx` loads WASM on startup via dynamic import.
- **Type sync**: Rust structs with `#[derive(TS)]` (ts-rs) export TypeScript types. Manual types in `client/src/types/game.ts` until auto-generation pipeline is wired.

### Serialization

- **WASM boundary**: JSON via `serde_json` (both directions). Serde serializes unit enum variants (no fields) as plain strings, not objects — e.g., `Action::Undo` becomes `"Undo"`, not `{"Undo":{}}`.
- **Save files**: MessagePack via `rmp-serde` (.aa1940 extension)

## Environment Notes

- **Windows toolchain**: A `rustup override` is set for this directory to use `stable-x86_64-pc-windows-gnu` because Git's `link.exe` shadows MSVC's linker on PATH. The `.cargo/config.toml` also sets `target = "x86_64-pc-windows-gnu"`.
- **WASM target**: `wasm32-unknown-unknown` must be installed (`rustup target add wasm32-unknown-unknown`).
- **Vite proxy**: Dev server proxies `/ws` and `/api` to `localhost:3001` (Rust server).
- **WASM .d.ts**: After adding new methods to `crates/aa-wasm/src/lib.rs`, the type declarations in `client/src/wasm/pkg/aa_wasm.d.ts` must be updated (either manually or by running `wasm-pack build`).

## Key Patterns

- **No game logic in TypeScript.** The engine is the single source of truth. Frontend only reads state and submits actions.
- **ts-rs type generation**: Add `#[derive(TS)]` and `#[ts(export)]` to Rust types that cross the WASM boundary. Use `#[ts(skip)]` for engine-internal fields (like `action_log`).
- **Engine never panics**: Use `Result<T, EngineError>`. WASM bridge converts errors to JSON error objects.
- **Phase validation**: Every action is checked against `current_phase` before application. Wrong-phase actions return `EngineError::WrongPhase`.
- **TypeScript Action type**: Unit variants (no fields) are string literals (`'ConfirmPhase'`, `'Undo'`); struct variants are objects (`{ PurchaseUnit: { unit_type, count } }`). This matches serde's default JSON serialization.
