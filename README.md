# Axis & Allies Global 1940 — Digital Edition

A fully playable digital board game implementation of **Axis & Allies Global 1940 2nd Edition**, built in Rust with a web-based UI.

![Game Screenshot](docs/screenshot.png)

## Features

- **Complete game engine** — All 14 phases implemented (purchase, movement, combat, mobilization, income, etc.)
- **AI opponent** — Play against computer-controlled powers at Easy, Normal, or Hard difficulty
- **Interactive world map** — Canvas-based map with all territories, sea zones, and unit displays
- **Save/Load** — Save games to JSON files, auto-save to browser storage
- **Hot seat multiplayer** — Pass-and-play on one device
- **Sound effects** — Audio feedback for combat, purchases, and phase transitions
- **214 unit tests** — Comprehensive test coverage for all game logic

## Quick Start (Web Client)

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

### Build & Run

```bash
# Install wasm-pack if needed
cargo install wasm-pack

# Build the WASM module
wasm-pack build crates/aa-wasm --target web --out-dir ../../web/pkg --dev

# Serve the web client (any static server works)
cd web
python3 -m http.server 8080
# or: npx serve .
```

Then open [http://localhost:8080](http://localhost:8080) in your browser.

### Production Build (optimized)

```bash
wasm-pack build crates/aa-wasm --target web --out-dir ../../web/pkg
```

> Note: The optimized build runs `wasm-opt` which can take several minutes.

## Running Tests

```bash
cargo test
```

All 214 tests should pass.

## Project Structure

```
Axis-Allies/
├── crates/
│   ├── aa-engine/     # Core game engine (pure Rust, no dependencies on platform)
│   │   └── src/
│   │       ├── lib.rs          # Engine struct, main API
│   │       ├── action.rs       # Action types, results, events
│   │       ├── ai.rs           # AI opponent (Easy/Normal/Hard)
│   │       ├── apply.rs        # Action application logic
│   │       ├── combat.rs       # Combat resolution
│   │       ├── movement.rs     # Unit movement & validation
│   │       ├── purchase.rs     # Purchase phase
│   │       ├── mobilize.rs     # Unit placement
│   │       ├── income.rs       # Income collection
│   │       ├── save.rs         # Save/load system
│   │       ├── multiplayer.rs  # Multiplayer foundation
│   │       ├── victory.rs      # Victory conditions
│   │       ├── setup.rs        # Initial game setup
│   │       ├── phase.rs        # Phase management
│   │       ├── state.rs        # Game state
│   │       ├── territory.rs    # Territory definitions
│   │       ├── unit.rs         # Unit types & stats
│   │       └── data/           # Static map data
│   ├── aa-wasm/       # WASM bridge (wasm-bindgen)
│   └── aa-server/     # WebSocket multiplayer server
├── web/               # Web client (vanilla JS + Canvas)
│   ├── index.html     # Main page
│   ├── app.js         # Game UI logic
│   ├── map-data.js    # Territory/sea zone coordinates
│   ├── style.css      # Styles
│   └── pkg/           # Built WASM output
├── client/            # React/Tauri client (alternative)
└── src-tauri/         # Tauri desktop app config
```

## How to Play

1. **Start a game** — Choose "vs AI" or "Hot Seat" mode
2. **Pick your side** — Play as Allies or Axis (AI controls the other side)
3. **Each turn** follows 6 phases:
   - **Purchase & Repair** — Buy new units with your IPCs
   - **Combat Movement** — Move units to attack enemy territories
   - **Conduct Combat** — Resolve battles
   - **Non-Combat Movement** — Move remaining units
   - **Mobilize** — Place purchased units on the map
   - **Collect Income** — Earn IPCs from controlled territories
4. **Win** by capturing enough victory cities

### Controls

- **Drag** the map to pan, **scroll** to zoom
- **Click** territories to see info and interact
- **End Phase** button advances to the next phase
- **Save** button (💾) downloads a save file
- **Menu** (☰) for load, stats, and quit options

## Architecture

The game engine (`aa-engine`) is a pure Rust library with zero platform dependencies. It compiles to both native Rust and WebAssembly. All game rules, AI logic, and state management live in this crate.

The WASM bridge (`aa-wasm`) provides a thin JavaScript-friendly API via `wasm-bindgen`.

The web client (`web/`) is a vanilla JavaScript application using HTML5 Canvas for map rendering. No build step needed beyond the WASM compilation.

## License

MIT
