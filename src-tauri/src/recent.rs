//! Persistent recent-files list stored in the app config directory.
//!
//! Used by both `commands::file` (Tauri commands) and `menu` (native menu
//! builder) so the logic lives in one place.

use std::path::PathBuf;
use tauri::Manager;

pub const MAX_RECENT: usize = 10;

// ─── Storage helpers ──────────────────────────────────────────────────────────

fn storage_path(app: &tauri::AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|d| d.join("recent_files.json"))
}

/// Load the stored list (no existence check).
pub fn load(app: &tauri::AppHandle) -> Vec<String> {
    let Some(path) = storage_path(app) else { return vec![] };
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

/// Load the stored list, filtering out paths that no longer exist on disk.
pub fn load_existing(app: &tauri::AppHandle) -> Vec<String> {
    load(app)
        .into_iter()
        .filter(|p| std::path::Path::new(p).exists())
        .collect()
}

fn persist(app: &tauri::AppHandle, files: &[String]) {
    let Some(path) = storage_path(app) else { return };
    if let Some(parent) = path.parent() { let _ = std::fs::create_dir_all(parent); }
    if let Ok(json) = serde_json::to_string(files) { let _ = std::fs::write(path, json); }
}

// ─── Public mutations ─────────────────────────────────────────────────────────

/// Prepend `file_path`, deduplicate, cap at `MAX_RECENT`, persist.
pub fn add(app: &tauri::AppHandle, file_path: &str) {
    let mut recent = load(app);
    recent.retain(|p| p != file_path);
    recent.insert(0, file_path.to_owned());
    recent.truncate(MAX_RECENT);
    persist(app, &recent);
}

/// Wipe the entire list.
pub fn clear(app: &tauri::AppHandle) {
    persist(app, &[]);
}
