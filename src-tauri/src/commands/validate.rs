//! Validation engine for DBC models.
//! Returns a list of issues (errors and warnings) found in the model.

use crate::dbc_model::{DbcModel, ValidationIssue};
use std::collections::HashMap;

/// Validate a DBC model and return all detected issues.
#[tauri::command]
pub fn validate_dbc(model: DbcModel) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    check_duplicate_message_ids(&model, &mut issues);
    check_signal_bit_ranges(&model, &mut issues);
    check_duplicate_signal_names(&model, &mut issues);
    check_empty_names(&model, &mut issues);

    issues
}

// ─── Checks ─────────────────────────────────────────────────────────────────

fn check_duplicate_message_ids(model: &DbcModel, issues: &mut Vec<ValidationIssue>) {
    let mut seen: HashMap<u32, &str> = HashMap::new();
    for msg in &model.messages {
        if let Some(existing) = seen.get(&msg.id) {
            issues.push(ValidationIssue {
                severity: "error".into(),
                message: format!(
                    "Duplicate CAN ID 0x{:X}: used by '{}' and '{}'",
                    msg.display_id(),
                    existing,
                    msg.name
                ),
                message_id: Some(msg.id),
                signal_name: None,
            });
        } else {
            seen.insert(msg.id, &msg.name);
        }
    }
}

fn check_signal_bit_ranges(model: &DbcModel, issues: &mut Vec<ValidationIssue>) {
    for msg in &model.messages {
        let max_bits = (msg.dlc as u32) * 8;
        for sig in &msg.signals {
            let end_bit = sig.start_bit as u32 + sig.length as u32;
            if end_bit > max_bits {
                issues.push(ValidationIssue {
                    severity: "error".into(),
                    message: format!(
                        "Signal '{}' in message '{}': bit range {}..{} exceeds DLC ({} bytes = {} bits)",
                        sig.name, msg.name, sig.start_bit, end_bit - 1, msg.dlc, max_bits
                    ),
                    message_id: Some(msg.id),
                    signal_name: Some(sig.name.clone()),
                });
            }
            if sig.length == 0 {
                issues.push(ValidationIssue {
                    severity: "error".into(),
                    message: format!(
                        "Signal '{}' in message '{}' has zero length",
                        sig.name, msg.name
                    ),
                    message_id: Some(msg.id),
                    signal_name: Some(sig.name.clone()),
                });
            }
        }
    }
}

fn check_duplicate_signal_names(model: &DbcModel, issues: &mut Vec<ValidationIssue>) {
    for msg in &model.messages {
        let mut seen: HashMap<&str, bool> = HashMap::new();
        for sig in &msg.signals {
            if seen.insert(sig.name.as_str(), true).is_some() {
                issues.push(ValidationIssue {
                    severity: "error".into(),
                    message: format!(
                        "Duplicate signal name '{}' in message '{}'",
                        sig.name, msg.name
                    ),
                    message_id: Some(msg.id),
                    signal_name: Some(sig.name.clone()),
                });
            }
        }
    }
}

fn check_empty_names(model: &DbcModel, issues: &mut Vec<ValidationIssue>) {
    for msg in &model.messages {
        if msg.name.trim().is_empty() {
            issues.push(ValidationIssue {
                severity: "error".into(),
                message: format!("Message with ID 0x{:X} has an empty name", msg.display_id()),
                message_id: Some(msg.id),
                signal_name: None,
            });
        }
        for sig in &msg.signals {
            if sig.name.trim().is_empty() {
                issues.push(ValidationIssue {
                    severity: "error".into(),
                    message: format!(
                        "Signal in message '{}' has an empty name",
                        msg.name
                    ),
                    message_id: Some(msg.id),
                    signal_name: Some(sig.name.clone()),
                });
            }
        }
    }
    for node in &model.nodes {
        if node.name.trim().is_empty() {
            issues.push(ValidationIssue {
                severity: "warning".into(),
                message: "A node has an empty name".into(),
                message_id: None,
                signal_name: None,
            });
        }
    }
}
