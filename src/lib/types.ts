// Mirror types matching src-tauri/src/dbc_model.rs — keep in sync.

export interface DbcModel {
  version: string | null;
  nodes: NodeModel[];
  messages: MessageModel[];
  attribute_definitions: AttributeDefinitionModel[];
  attribute_assignments: AttributeAssignmentModel[];
}

export interface NodeModel {
  name: string;
  comment: string | null;
}

export interface MessageModel {
  id: number;
  is_extended: boolean;
  name: string;
  dlc: number;
  sender: string;
  signals: SignalModel[];
  comment: string | null;
}

/** A single VAL_ entry: numeric value → symbolic label. */
export interface ValueDescriptionModel {
  value: number;
  label: string;
}

export interface SignalModel {
  name: string;
  start_bit: number;
  length: number;
  /** "LittleEndian" | "BigEndian" */
  byte_order: string;
  is_unsigned: boolean;
  factor: number;
  offset: number;
  min: number;
  max: number;
  unit: string | null;
  receivers: string[];
  is_multiplexer: boolean;
  multiplexer_switch_value: number | null;
  comment: string | null;
  /** Symbolic value descriptions (VAL_ entries). */
  value_descriptions: ValueDescriptionModel[];
}

// ─── Attribute types ──────────────────────────────────────────────────────────

export interface AttributeValueModel {
  int_val: number | null;
  float_val: number | null;
  string_val: string | null;
}

export interface AttributeDefinitionModel {
  name: string;
  /** "Network" | "Node" | "Message" | "Signal" */
  object_type: string;
  /** "Int" | "Hex" | "Float" | "String" | "Enum" */
  value_type: string;
  min_int: number | null;
  max_int: number | null;
  min_float: number | null;
  max_float: number | null;
  enum_values: string[];
  default_value: AttributeValueModel | null;
}

export interface AttributeAssignmentModel {
  attr_name: string;
  /** "Network" | "Node" | "Message" | "Signal" */
  target_type: string;
  node_name: string | null;
  message_id: number | null;
  signal_name: string | null;
  value: AttributeValueModel;
}

export interface ValidationIssue {
  severity: 'error' | 'warning';
  message: string;
  message_id: number | null;
  signal_name: string | null;
}

// ─── Factories ────────────────────────────────────────────────────────────────

export function emptyModel(): DbcModel {
  return { version: '', nodes: [], messages: [], attribute_definitions: [], attribute_assignments: [] };
}

export function newMessage(id: number): MessageModel {
  return {
    id,
    is_extended: false,
    name: `Message_${id.toString(16).toUpperCase()}`,
    dlc: 8,
    sender: '',
    signals: [],
    comment: null,
  };
}

export function newSignal(): SignalModel {
  return {
    name: 'NewSignal',
    start_bit: 0,
    length: 8,
    byte_order: 'LittleEndian',
    is_unsigned: true,
    factor: 1,
    offset: 0,
    min: 0,
    max: 255,
    unit: null,
    receivers: [],
    is_multiplexer: false,
    multiplexer_switch_value: null,
    comment: null,
    value_descriptions: [],
  };
}

// ─── ID formatting ────────────────────────────────────────────────────────────

/** Raw display ID (strips extended flag bit). */
export function rawDisplayId(msg: MessageModel): number {
  return msg.is_extended ? msg.id & 0x1fffffff : msg.id;
}

/** Format a CAN ID respecting the global hex/dec mode. */
export function formatCanId(msg: MessageModel, hex: boolean): string {
  const id = rawDisplayId(msg);
  if (hex) {
    return `0x${id.toString(16).toUpperCase().padStart(msg.is_extended ? 8 : 3, '0')}`;
  }
  return id.toString(10);
}

/** Format a plain numeric ID (no MessageModel needed). */
export function formatId(id: number, isExtended: boolean, hex: boolean): string {
  const display = isExtended ? id & 0x1fffffff : id;
  if (hex) {
    return `0x${display.toString(16).toUpperCase().padStart(isExtended ? 8 : 3, '0')}`;
  }
  return display.toString(10);
}
