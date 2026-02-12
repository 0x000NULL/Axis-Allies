//! JsValue conversion helpers for the WASM bridge.

/// Create a JSON error string for returning errors across the WASM boundary.
pub fn error_json(message: &str) -> String {
    serde_json::json!({
        "error": true,
        "message": message
    })
    .to_string()
}
