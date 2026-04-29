// Mirror types matching src-tauri/src/dbc_model.rs — keep in sync.

export interface DbcModel {
  version: string | null;
  nodes: NodeModel[];
  messages: MessageModel[];
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
}

export interface ValidationIssue {
  severity: 'error' | 'warning';
  message: string;
  message_id: number | null;
  signal_name: string | null;
}

// UI-level selection state
export interface Selection {
  messageId: number | null;
  signalName: string | null;
}

// Empty model factory
export function emptyModel(): DbcModel {
  return { version: '', nodes: [], messages: [] };
}

// Default message factory (client-side, matches Rust default_message)
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

// Default signal factory (client-side)
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
  };
}

/** Format a CAN ID as hex string, showing the raw ID (with extended bit mask). */
export function formatCanId(msg: MessageModel): string {
  const displayId = msg.is_extended ? msg.id & 0x1fffffff : msg.id;
  return `0x${displayId.toString(16).toUpperCase().padStart(msg.is_extended ? 8 : 3, '0')}`;
}
