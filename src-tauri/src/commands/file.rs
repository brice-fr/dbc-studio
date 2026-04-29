//! Tauri commands for opening and saving DBC files.

use crate::dbc_model::DbcModel;
use std::fs;

/// Parse a DBC file at `path` and return a serializable model.
#[tauri::command]
pub fn open_dbc(path: String) -> Result<DbcModel, String> {
    let dbc = dbc_rs::Dbc::from_file(&path)
        .map_err(|e| format!("Failed to parse DBC file: {e}"))?;
    Ok(DbcModel::from(&dbc))
}

/// Serialize `model` back to DBC format and write it to `path`.
#[tauri::command]
pub fn save_dbc(path: String, model: DbcModel) -> Result<(), String> {
    let dbc = dbc_rs::Dbc::try_from(&model)
        .map_err(|e| format!("Failed to build DBC model: {e}"))?;
    let content = dbc.to_dbc_string();
    fs::write(&path, content).map_err(|e| format!("Failed to write file: {e}"))
}
