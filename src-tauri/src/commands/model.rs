//! Tauri commands for in-model CRUD operations.
//! The frontend stores the canonical model; these commands provide
//! helpers for creating new default objects to add to it.

use crate::dbc_model::{MessageModel, NodeModel, SignalModel};
use serde::{Deserialize, Serialize};

/// Arguments for adding a new message.
#[derive(Debug, Deserialize)]
pub struct NewMessageArgs {
    pub id: u32,
    pub name: String,
    pub dlc: u8,
    pub sender: String,
}

/// Arguments for adding a new signal.
#[derive(Debug, Deserialize)]
pub struct NewSignalArgs {
    pub name: String,
}

/// A default message template (returned to the frontend for insertion).
#[tauri::command]
pub fn default_message(args: NewMessageArgs) -> MessageModel {
    MessageModel {
        id: args.id,
        is_extended: false,
        name: args.name,
        dlc: args.dlc,
        sender: args.sender,
        signals: vec![],
        comment: None,
    }
}

/// A default signal template (returned to the frontend for insertion).
#[tauri::command]
pub fn default_signal(args: NewSignalArgs) -> SignalModel {
    SignalModel {
        name: args.name,
        start_bit: 0,
        length: 8,
        byte_order: "LittleEndian".into(),
        is_unsigned: true,
        factor: 1.0,
        offset: 0.0,
        min: 0.0,
        max: 255.0,
        unit: None,
        receivers: vec![],
        is_multiplexer: false,
        multiplexer_switch_value: None,
        comment: None,
    }
}

/// A default node template.
#[tauri::command]
pub fn default_node(name: String) -> NodeModel {
    NodeModel { name, comment: None }
}

/// Payload returned when describing the app's current status.
#[derive(Debug, Serialize)]
pub struct AppVersion {
    pub version: String,
}

#[tauri::command]
pub fn app_version() -> AppVersion {
    AppVersion { version: env!("CARGO_PKG_VERSION").to_string() }
}
