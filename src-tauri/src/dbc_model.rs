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
    /// Attribute definitions (`BA_DEF_` entries).
    pub attribute_definitions: Vec<AttributeDefinitionModel>,
    /// Attribute value assignments (`BA_` entries).
    pub attribute_assignments: Vec<AttributeAssignmentModel>,
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

/// A single VAL_ entry: maps a numeric value to a symbolic label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueDescriptionModel {
    pub value: u64,
    pub label: String,
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
    /// Symbolic value descriptions (VAL_ entries).
    pub value_descriptions: Vec<ValueDescriptionModel>,
}

// ─── Attribute model types ───────────────────────────────────────────────────

/// A concrete attribute value (from `BA_` or `BA_DEF_DEF_`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeValueModel {
    pub int_val: Option<i64>,
    pub float_val: Option<f64>,
    pub string_val: Option<String>,
}

impl AttributeValueModel {
    fn from_dbc(v: &dbc_rs::AttributeValue) -> Self {
        AttributeValueModel {
            int_val: v.as_int(),
            float_val: v.as_float(),
            string_val: v.as_string().map(str::to_string),
        }
    }
}

/// An attribute definition (`BA_DEF_` entry).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeDefinitionModel {
    pub name: String,
    /// "Network", "Node", "Message", or "Signal"
    pub object_type: String,
    /// "Int", "Hex", "Float", "String", or "Enum"
    pub value_type: String,
    pub min_int: Option<i64>,
    pub max_int: Option<i64>,
    pub min_float: Option<f64>,
    pub max_float: Option<f64>,
    pub enum_values: Vec<String>,
    /// Default value from `BA_DEF_DEF_` (if any).
    pub default_value: Option<AttributeValueModel>,
}

/// An attribute value assignment (`BA_` entry).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeAssignmentModel {
    pub attr_name: String,
    /// "Network", "Node", "Message", or "Signal"
    pub target_type: String,
    pub node_name: Option<String>,
    pub message_id: Option<u32>,
    pub signal_name: Option<String>,
    pub value: AttributeValueModel,
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

        // Build messages, passing dbc for value-description lookup
        let messages = dbc
            .messages()
            .iter()
            .map(|msg| message_model_from(msg, dbc))
            .collect();

        let attribute_definitions = build_attribute_definitions(dbc);
        let attribute_assignments = build_attribute_assignments(dbc);

        DbcModel { version, nodes, messages, attribute_definitions, attribute_assignments }
    }
}

fn message_model_from(msg: &dbc_rs::Message, dbc: &dbc_rs::Dbc) -> MessageModel {
    MessageModel {
        id: msg.id(),
        is_extended: msg.is_extended(),
        name: msg.name().to_string(),
        dlc: msg.dlc(),
        sender: msg.sender().to_string(),
        signals: msg.signals().iter().map(|s| signal_model_from(s, msg.id(), dbc)).collect(),
        comment: msg.comment().map(str::to_string),
    }
}

fn signal_model_from(sig: &dbc_rs::Signal, msg_id: u32, dbc: &dbc_rs::Dbc) -> SignalModel {
    let byte_order = match sig.byte_order() {
        dbc_rs::ByteOrder::LittleEndian => "LittleEndian",
        dbc_rs::ByteOrder::BigEndian => "BigEndian",
    }
    .to_string();

    let value_descriptions = dbc
        .value_descriptions_for_signal(msg_id, sig.name())
        .map(|vd| {
            vd.iter()
                .map(|(v, l)| ValueDescriptionModel { value: v, label: l.to_string() })
                .collect()
        })
        .unwrap_or_default();

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
        value_descriptions,
    }
}

fn build_attribute_definitions(dbc: &dbc_rs::Dbc) -> Vec<AttributeDefinitionModel> {
    dbc.attribute_definitions()
        .iter()
        .map(|def| {
            let object_type = match def.object_type() {
                dbc_rs::AttributeObjectType::Network => "Network",
                dbc_rs::AttributeObjectType::Node    => "Node",
                dbc_rs::AttributeObjectType::Message => "Message",
                dbc_rs::AttributeObjectType::Signal  => "Signal",
            }
            .to_string();

            let (value_type, min_int, max_int, min_float, max_float, enum_values) =
                match def.value_type() {
                    dbc_rs::AttributeValueType::Int { min, max } => {
                        ("Int".into(), Some(*min), Some(*max), None, None, vec![])
                    }
                    dbc_rs::AttributeValueType::Hex { min, max } => {
                        ("Hex".into(), Some(*min), Some(*max), None, None, vec![])
                    }
                    dbc_rs::AttributeValueType::Float { min, max } => {
                        ("Float".into(), None, None, Some(*min), Some(*max), vec![])
                    }
                    dbc_rs::AttributeValueType::String => {
                        ("String".into(), None, None, None, None, vec![])
                    }
                    dbc_rs::AttributeValueType::Enum { values } => {
                        let ev = values.iter().map(|s| s.as_str().to_string()).collect();
                        ("Enum".into(), None, None, None, None, ev)
                    }
                };

            let default_value = dbc
                .attribute_default(def.name())
                .map(AttributeValueModel::from_dbc);

            AttributeDefinitionModel {
                name: def.name().to_string(),
                object_type,
                value_type,
                min_int,
                max_int,
                min_float,
                max_float,
                enum_values,
                default_value,
            }
        })
        .collect()
}

fn build_attribute_assignments(dbc: &dbc_rs::Dbc) -> Vec<AttributeAssignmentModel> {
    dbc.attribute_values()
        .iter()
        .map(|((attr_name, target), value)| {
            let (target_type, node_name, message_id, signal_name) = match target {
                dbc_rs::AttributeTarget::Network => {
                    ("Network".into(), None, None, None)
                }
                dbc_rs::AttributeTarget::Node(n) => {
                    ("Node".into(), Some(n.as_str().to_string()), None, None)
                }
                dbc_rs::AttributeTarget::Message(id) => {
                    ("Message".into(), None, Some(*id), None)
                }
                dbc_rs::AttributeTarget::Signal(id, sig) => {
                    ("Signal".into(), None, Some(*id), Some(sig.as_str().to_string()))
                }
            };
            AttributeAssignmentModel {
                attr_name: attr_name.to_string(),
                target_type,
                node_name,
                message_id,
                signal_name,
                value: AttributeValueModel::from_dbc(value),
            }
        })
        .collect()
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

        // Pass 1 – add every message and its signals.
        // VAL_ entries must be added AFTER the message exists in the builder,
        // so we do a separate second pass below.
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

        // Pass 2 – add VAL_ entries now that all messages are registered.
        for msg in &model.messages {
            for sig in &msg.signals {
                if !sig.value_descriptions.is_empty() {
                    let mut vd_builder = dbc_rs::ValueDescriptionsBuilder::new();
                    for vd in &sig.value_descriptions {
                        vd_builder = vd_builder.add_entry(vd.value, vd.label.as_str());
                    }
                    dbc_builder =
                        dbc_builder.add_value_description(msg.id, sig.name.as_str(), vd_builder);
                }
            }
        }

        dbc_builder.build()
    }
}
