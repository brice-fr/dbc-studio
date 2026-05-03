mod commands;
mod dbc_model;
mod dbc_sanitize;

use commands::{file::*, model::*, validate::*};

// Emitter, Manager and RunEvent are needed on macOS for the open-file handler.
#[cfg(target_os = "macos")]
use tauri::{Emitter, Manager, RunEvent};

/// Holds the path of the file to open at startup.
///
/// * On Windows / Linux the OS passes the associated file as `argv[1]`.
/// * On macOS the file path arrives via a `RunEvent::Opened` Apple Event,
///   which may fire before the webview is ready.  We park it here and let
///   the frontend pick it up via `get_startup_file` once it is ready.
pub struct StartupFile(pub std::sync::Mutex<Option<String>>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Windows / Linux: the associated file is passed as the first CLI argument.
    let startup_path = std::env::args()
        .nth(1)
        .filter(|a| !a.starts_with('-') && std::path::Path::new(a).exists());

    tauri::Builder::default()
        .manage(StartupFile(std::sync::Mutex::new(startup_path)))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // File I/O
            open_dbc,
            save_dbc,
            get_startup_file,
            // Model helpers
            default_message,
            default_signal,
            default_node,
            app_version,
            // Validation
            validate_dbc,
        ])
        .build(tauri::generate_context!())
        .expect("error while running dbc-studio")
        .run(|app_handle, event| {
            // macOS: the OS sends RunEvent::Opened when the user double-clicks
            // a .dbc file or uses "Open With".  This may arrive before the
            // webview has finished loading, so we store the path in StartupFile
            // state AND emit an event (for warm launches where the app is already
            // running and the webview is ready).
            #[cfg(target_os = "macos")]
            if let RunEvent::Opened { urls } = event {
                for url in urls {
                    if let Ok(path) = url.to_file_path() {
                        if let Some(path_str) = path.to_str() {
                            if let Some(state) = app_handle.try_state::<StartupFile>() {
                                if let Ok(mut guard) = state.0.lock() {
                                    *guard = Some(path_str.to_string());
                                }
                            }
                            let _ = app_handle.emit("open-file", path_str);
                        }
                    }
                }
            }
            #[cfg(not(target_os = "macos"))]
            let _ = (app_handle, event);
        });
}
