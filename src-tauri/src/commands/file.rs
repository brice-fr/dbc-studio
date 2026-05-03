//! Tauri commands for opening and saving DBC files.

use crate::dbc_model::{AttributeAssignmentModel, AttributeDefinitionModel, AttributeValueModel, DbcModel};
use crate::dbc_sanitize;
use serde::Serialize;
use std::fs;

// ─── open_dbc ────────────────────────────────────────────────────────────────

/// Result of opening a DBC file.
/// `warnings` carries non-fatal issues found during parsing (e.g. truncated strings).
#[derive(Debug, Serialize)]
pub struct OpenDbcResult {
    pub model:    DbcModel,
    pub warnings: Vec<String>,
}

/// Parse a DBC file at `path`.
///
/// The file is run through a tolerance pre-processor before handing it to
/// dbc-rs so that files with over-long strings (a common real-world deviation
/// from the spec) can still be opened.  Any strings that had to be truncated
/// are reported as warnings in the returned result.
#[tauri::command]
pub fn open_dbc(path: String) -> Result<OpenDbcResult, String> {
    let bytes = fs::read(&path)
        .map_err(|e| format!("Failed to read file: {e}"))?;

    // Decode raw bytes → UTF-8, substituting non-UTF-8 bytes as Latin-1.
    let (raw, had_non_utf8) = dbc_sanitize::decode_tolerant(&bytes);

    // Run tolerance pre-processor (truncates CM_ comments > 255 bytes,
    // tolerates multi-line CM_ strings up to 31 extra lines).
    let mut sanitized = dbc_sanitize::sanitize(&raw)
        .map_err(|e| format!("Malformed DBC file: {e}"))?;

    if had_non_utf8 {
        sanitized.warnings.insert(
            0,
            "File contained non-UTF-8 bytes; they were interpreted as Latin-1 (ISO 8859-1). \
             Re-saving will write the file as UTF-8."
                .to_owned(),
        );
    }

    // Parse with dbc-rs. On failure, include a hint about the sanitized content.
    let dbc = dbc_rs::Dbc::parse(&sanitized.content)
        .map_err(|e| format!("Failed to parse DBC file: {e}"))?;

    let warnings = sanitized.warnings;

    // If the raw file had a VERSION string longer than what dbc-rs now accepts
    // (1024, set via DBC_MAX_NAME_SIZE), the parse would already have failed
    // above. Any surviving warnings are content-level issues.
    let model = DbcModel::from(&dbc);
    Ok(OpenDbcResult { model, warnings })
}

/// Serialize `model` back to DBC format and write it to `path`.
#[tauri::command]
pub fn save_dbc(path: String, model: DbcModel) -> Result<(), String> {
    let dbc = dbc_rs::Dbc::try_from(&model)
        .map_err(|e| format!("Failed to build DBC model: {e}"))?;

    // dbc-rs 0.4.3 to_dbc_string() does not emit VAL_, BA_DEF_, BA_DEF_DEF_,
    // or BA_ lines, so we append them manually from the model.
    let mut content = dbc.to_dbc_string();
    append_val_lines(&mut content, &model);
    append_attribute_lines(&mut content, &model);

    fs::write(&path, content).map_err(|e| format!("Failed to write file: {e}"))
}

// ─── VAL_ ────────────────────────────────────────────────────────────────────

/// Append `VAL_ <msg_id> <signal> <n> "label" ... ;` lines.
fn append_val_lines(out: &mut String, model: &DbcModel) {
    for msg in &model.messages {
        for sig in &msg.signals {
            if sig.value_descriptions.is_empty() {
                continue;
            }
            out.push_str(&format!("VAL_ {} {}", msg.id, sig.name));
            for vd in &sig.value_descriptions {
                let label = vd.label.replace('"', "\\\"");
                out.push_str(&format!(" {} \"{}\"", vd.value, label));
            }
            out.push_str(" ;\n");
        }
    }
}

// ─── BA_DEF_ / BA_DEF_DEF_ / BA_ ─────────────────────────────────────────────

/// Append all attribute lines (BA_DEF_, BA_DEF_DEF_, BA_) to `out`.
fn append_attribute_lines(out: &mut String, model: &DbcModel) {
    if model.attribute_definitions.is_empty() && model.attribute_assignments.is_empty() {
        return;
    }

    out.push('\n');

    // BA_DEF_ lines
    for def in &model.attribute_definitions {
        append_ba_def(out, def);
    }

    // BA_DEF_DEF_ lines
    for def in &model.attribute_definitions {
        if let Some(ref dv) = def.default_value {
            out.push_str(&format!("BA_DEF_DEF_ \"{}\" {};\n", def.name, fmt_attr_value(dv)));
        }
    }

    // BA_ lines
    for assign in &model.attribute_assignments {
        append_ba_value(out, assign);
    }
}

fn append_ba_def(out: &mut String, def: &AttributeDefinitionModel) {
    let obj = match def.object_type.as_str() {
        "Network" => "".to_string(),
        "Node"    => "BU_ ".to_string(),
        "Message" => "BO_ ".to_string(),
        "Signal"  => "SG_ ".to_string(),
        other     => format!("{other} "),
    };
    let vt = match def.value_type.as_str() {
        "Int" => format!(
            "INT {} {}",
            def.min_int.unwrap_or(0),
            def.max_int.unwrap_or(0)
        ),
        "Hex" => format!(
            "HEX {} {}",
            def.min_int.unwrap_or(0),
            def.max_int.unwrap_or(0)
        ),
        "Float" => format!(
            "FLOAT {} {}",
            def.min_float.unwrap_or(0.0),
            def.max_float.unwrap_or(0.0)
        ),
        "String" => "STRING".to_string(),
        "Enum" => {
            let vals: Vec<String> = def
                .enum_values
                .iter()
                .map(|v| format!("\"{}\"", v.replace('"', "\\\"")))
                .collect();
            format!("ENUM {}", vals.join(","))
        }
        other => other.to_string(),
    };
    out.push_str(&format!("BA_DEF_ {}\"{}\" {};\n", obj, def.name, vt));
}

fn append_ba_value(out: &mut String, assign: &AttributeAssignmentModel) {
    let target = match assign.target_type.as_str() {
        "Network" => String::new(),
        "Node" => format!(
            " BU_ {}",
            assign.node_name.as_deref().unwrap_or("")
        ),
        "Message" => format!(
            " BO_ {}",
            assign.message_id.unwrap_or(0)
        ),
        "Signal" => format!(
            " SG_ {} {}",
            assign.message_id.unwrap_or(0),
            assign.signal_name.as_deref().unwrap_or("")
        ),
        other => format!(" {other}"),
    };
    out.push_str(&format!(
        "BA_ \"{}\"{}{};\n",
        assign.attr_name,
        target,
        if assign.target_type == "Network" {
            // network-level value comes right after the name
            format!(" {}", fmt_attr_value(&assign.value))
        } else {
            format!(" {}", fmt_attr_value(&assign.value))
        }
    ));
}

/// Format an attribute value as it appears in DBC text.
fn fmt_attr_value(v: &AttributeValueModel) -> String {
    if let Some(i) = v.int_val {
        return i.to_string();
    }
    if let Some(f) = v.float_val {
        return format!("{f}");
    }
    if let Some(ref s) = v.string_val {
        return format!("\"{}\"", s.replace('"', "\\\""));
    }
    "\"\"".to_string()
}
