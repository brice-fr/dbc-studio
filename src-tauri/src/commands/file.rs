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

    // dbc-rs 0.4.3 to_dbc_string() does not emit VAL_ lines at all,
    // so we append them manually from the model after the base content.
    let mut content = dbc.to_dbc_string();
    append_val_lines(&mut content, &model);

    fs::write(&path, content).map_err(|e| format!("Failed to write file: {e}"))
}

/// Append `VAL_ <msg_id> <signal> <n> "label" ... ;` lines for every signal
/// that has value descriptions. dbc-rs 0.4.3 never writes these in
/// `to_dbc_string()`, so we handle them here directly from the model.
fn append_val_lines(out: &mut String, model: &DbcModel) {
    for msg in &model.messages {
        for sig in &msg.signals {
            if sig.value_descriptions.is_empty() {
                continue;
            }
            // VAL_ <raw_msg_id_with_extended_flag> <signal_name>
            out.push_str(&format!("VAL_ {} {}", msg.id, sig.name));
            for vd in &sig.value_descriptions {
                // Escape any double-quotes inside the label
                let label = vd.label.replace('"', "\\\"");
                out.push_str(&format!(" {} \"{}\"", vd.value, label));
            }
            out.push_str(" ;\n");
        }
    }
}
