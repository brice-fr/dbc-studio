//! Serde-serializable mirror types for the DBC model.
//! These are the types passed over the Tauri IPC boundary as JSON.

use serde::{Deserialize, Serialize};

// ─── Public model types ──────────────────────────────────────────────────────

/// Top-level DBC database model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbcModel {
    pub version: Option<String>,
    pub nodes: Vec<NodeModel>,
    pub messages: Vec<MessageModel>,
}

/// An ECU node (BU_ entry).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeModel {
    pub name: String,
    pub comment: Option<String>,
}

/// A CAN message (BO_ entry).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageModel {
    /// Raw CAN ID as stored in DBC (may have extended flag bit 31 set).
    pub id: u32,
    /// True if this is a 29-bit extended CAN ID.
    pub is_extended: bool,
    pub name: String,
    /// Data Length Code (bytes, 0–8 for CAN, 0–64 for CAN FD).
    pub dlc: u8,
    /// Transmitting node name.
    pub sender: String,
    pub signals: Vec<SignalModel>,
    pub comment: Option<String>,
}

impl MessageModel {
    /// Returns the display CAN ID (without the extended flag bit).
    pub fn display_id(&self) -> u32 {
        if self.is_extended {
            self.id & 0x1FFF_FFFF
        } else {
            self.id
        }
    }
}

/// A CAN signal (SG_ entry).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalModel {
    pub name: String,
    pub start_bit: u16,
    pub length: u16,
    /// "LittleEndian" (Intel @1) or "BigEndian" (Motorola @0).
    pub byte_order: String,
    /// true = unsigned (+), false = signed (-).
    pub is_unsigned: bool,
    pub factor: f64,
    pub offset: f64,
    pub min: f64,
    pub max: f64,
    pub unit: Option<String>,
    pub receivers: Vec<String>,
    /// True if this signal is a multiplexer switch (M).
    pub is_multiplexer: bool,
    /// Switch value for a multiplexed signal (m<N>), None for normal signals.
    pub multiplexer_switch_value: Option<u64>,
    pub comment: Option<String>,
}

/// A validation issue found in the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// "error" or "warning"
    pub severity: String,
    pub message: String,
    pub message_id: Option<u32>,
    pub signal_name: Option<String>,
}

// ─── dbc_rs → DbcModel ──────────────────────────────────────────────────────

impl From<&dbc_rs::Dbc> for DbcModel {
    fn from(dbc: &dbc_rs::Dbc) -> Self {
        let version = dbc.version().map(|v| v.to_string());

        let nodes = dbc
            .nodes()
            .iter()
            .map(|name| NodeModel {
                name: name.to_string(),
                comment: dbc.node_comment(name).map(str::to_string),
            })
            .collect();

        let messages = dbc.messages().iter().map(MessageModel::from).collect();

        DbcModel { version, nodes, messages }
    }
}

impl From<&dbc_rs::Message> for MessageModel {
    fn from(msg: &dbc_rs::Message) -> Self {
        MessageModel {
            id: msg.id(),
            is_extended: msg.is_extended(),
            name: msg.name().to_string(),
            dlc: msg.dlc(),
            sender: msg.sender().to_string(),
            signals: msg.signals().iter().map(SignalModel::from).collect(),
            comment: msg.comment().map(str::to_string),
        }
    }
}

impl From<&dbc_rs::Signal> for SignalModel {
    fn from(sig: &dbc_rs::Signal) -> Self {
        let byte_order = match sig.byte_order() {
            dbc_rs::ByteOrder::LittleEndian => "LittleEndian",
            dbc_rs::ByteOrder::BigEndian => "BigEndian",
        }
        .to_string();

        SignalModel {
            name: sig.name().to_string(),
            start_bit: sig.start_bit(),
            length: sig.length(),
            byte_order,
            is_unsigned: sig.is_unsigned(),
            factor: sig.factor(),
            offset: sig.offset(),
            min: sig.min(),
            max: sig.max(),
            unit: sig.unit().map(str::to_string),
            receivers: sig.receivers().iter().map(str::to_string).collect(),
            is_multiplexer: sig.is_multiplexer_switch(),
            multiplexer_switch_value: sig.multiplexer_switch_value(),
            comment: sig.comment().map(str::to_string),
        }
    }
}

// ─── DbcModel → dbc_rs::Dbc (for saving) ────────────────────────────────────

impl TryFrom<&DbcModel> for dbc_rs::Dbc {
    type Error = dbc_rs::Error;

    fn try_from(model: &DbcModel) -> Result<Self, Self::Error> {
        // Version
        let mut version_builder = dbc_rs::VersionBuilder::new();
        if let Some(ref v) = model.version {
            version_builder = version_builder.version(v.as_str());
        }

        // Nodes
        let mut nodes_builder = dbc_rs::NodesBuilder::new();
        for node in &model.nodes {
            nodes_builder = nodes_builder.add_node(node.name.as_str());
        }

        let mut dbc_builder =
            dbc_rs::DbcBuilder::new().version(version_builder).nodes(nodes_builder);

        // Messages & signals
        for msg in &model.messages {
            let mut msg_builder = dbc_rs::MessageBuilder::new()
                .id(msg.id)
                .name(msg.name.as_str())
                .dlc(msg.dlc)
                .sender(msg.sender.as_str());

            if let Some(ref c) = msg.comment {
                msg_builder = msg_builder.comment(c.as_str());
            }

            for sig in &msg.signals {
                let bo = if sig.byte_order == "LittleEndian" {
                    dbc_rs::ByteOrder::LittleEndian
                } else {
                    dbc_rs::ByteOrder::BigEndian
                };

                let mut recv_builder = dbc_rs::ReceiversBuilder::new();
                if sig.receivers.is_empty() {
                    recv_builder = recv_builder.none();
                } else {
                    for r in &sig.receivers {
                        recv_builder = recv_builder.add_node(r.as_str());
                    }
                }

                let mut sig_builder = dbc_rs::SignalBuilder::new()
                    .name(sig.name.as_str())
                    .start_bit(sig.start_bit)
                    .length(sig.length)
                    .byte_order(bo)
                    .unsigned(sig.is_unsigned)
                    .factor(sig.factor)
                    .offset(sig.offset)
                    .min(sig.min)
                    .max(sig.max)
                    .receivers(recv_builder);

                if let Some(ref unit) = sig.unit {
                    sig_builder = sig_builder.unit(unit.as_str());
                }
                if let Some(ref c) = sig.comment {
                    sig_builder = sig_builder.comment(c.as_str());
                }

                msg_builder = msg_builder.add_signal(sig_builder);
            }

            dbc_builder = dbc_builder.add_message(msg_builder);
        }

        dbc_builder.build()
    }
}
