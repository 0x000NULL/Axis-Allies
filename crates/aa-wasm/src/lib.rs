//! WASM bridge for the Axis & Allies game engine.
//!
//! This is a thin wrapper that exposes the `Engine` API to JavaScript
//! via `wasm-bindgen`. All data crosses the boundary as JSON strings.

mod conversions;

use wasm_bindgen::prelude::*;
use aa_engine::{Engine, action::Action, state::GameState};

/// Set up panic hook for better error messages in the browser console.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// The WASM-exposed game engine wrapper.
#[wasm_bindgen]
pub struct WasmEngine {
    engine: Engine,
}

#[wasm_bindgen]
impl WasmEngine {
    /// Create a new game with the given RNG seed.
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u64) -> WasmEngine {
        WasmEngine {
            engine: Engine::new_game(seed),
        }
    }

    /// Restore an engine from a JSON-serialized game state.
    #[wasm_bindgen(js_name = fromState)]
    pub fn from_state(state_json: &str) -> Result<WasmEngine, JsValue> {
        let state: GameState = serde_json::from_str(state_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse state: {}", e)))?;
        Ok(WasmEngine {
            engine: Engine::from_state(state),
        })
    }

    /// Submit a player action (JSON-encoded). Returns JSON result or error.
    #[wasm_bindgen(js_name = submitAction)]
    pub fn submit_action(&mut self, action_json: &str) -> String {
        let action: Action = match serde_json::from_str(action_json) {
            Ok(a) => a,
            Err(e) => return conversions::error_json(&format!("Invalid action JSON: {}", e)),
        };

        match self.engine.submit_action(action) {
            Ok(result) => serde_json::to_string(&result).unwrap_or_else(|e| {
                conversions::error_json(&format!("Failed to serialize result: {}", e))
            }),
            Err(e) => conversions::error_json(&format!("{}", e)),
        }
    }

    /// Get the current game state as JSON.
    #[wasm_bindgen(js_name = getState)]
    pub fn get_state(&self) -> String {
        self.engine
            .serialize_state_json()
            .unwrap_or_else(|e| conversions::error_json(&format!("Failed to serialize state: {}", e)))
    }

    /// Check if the last action can be undone.
    #[wasm_bindgen(js_name = canUndo)]
    pub fn can_undo(&self) -> bool {
        self.engine.can_undo()
    }

    /// Get a summary string for the current turn state.
    #[wasm_bindgen(js_name = turnSummary)]
    pub fn turn_summary(&self) -> String {
        self.engine.turn_summary()
    }

    /// Serialize the game state to MessagePack bytes (for save files).
    #[wasm_bindgen(js_name = serializeForSave)]
    pub fn serialize_for_save(&self) -> Result<Vec<u8>, JsValue> {
        self.engine
            .serialize_state()
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }

    /// Load a game from MessagePack bytes.
    #[wasm_bindgen(js_name = loadFromSave)]
    pub fn load_from_save(data: &[u8]) -> Result<WasmEngine, JsValue> {
        let state = Engine::deserialize_state(data)
            .map_err(|e| JsValue::from_str(&format!("Deserialization failed: {}", e)))?;
        Ok(WasmEngine {
            engine: Engine::from_state(state),
        })
    }

    /// Get the list of currently legal actions as JSON.
    #[wasm_bindgen(js_name = legalActions)]
    pub fn legal_actions(&self) -> String {
        let actions = self.engine.legal_actions();
        serde_json::to_string(&actions).unwrap_or_else(|e| {
            conversions::error_json(&format!("Failed to serialize legal actions: {}", e))
        })
    }

    /// Check if a victory condition has been met. Returns JSON (null or GameEvent).
    #[wasm_bindgen(js_name = checkVictory)]
    pub fn check_victory(&self) -> String {
        let result = self.engine.check_victory();
        serde_json::to_string(&result).unwrap_or_else(|e| {
            conversions::error_json(&format!("Failed to serialize victory check: {}", e))
        })
    }

    /// Get the engine version string.
    #[wasm_bindgen(js_name = engineVersion)]
    pub fn engine_version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}
