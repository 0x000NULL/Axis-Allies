//! Tauri commands exposed to the frontend.
//! These provide desktop-specific functionality (file dialogs, etc.).

/// Get the engine version string.
#[tauri::command]
pub fn get_engine_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
