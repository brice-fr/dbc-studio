mod commands;
mod dbc_model;

use commands::{file::*, model::*, validate::*};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // File I/O
            open_dbc,
            save_dbc,
            // Model helpers
            default_message,
            default_signal,
            default_node,
            app_version,
            // Validation
            validate_dbc,
        ])
        .run(tauri::generate_context!())
        .expect("error while running dbc-studio");
}
