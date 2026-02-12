//! Tauri desktop shell for Axis & Allies Global 1940.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_engine_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Axis & Allies");
}
