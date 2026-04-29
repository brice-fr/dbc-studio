# DBC Studio — Specification

## Overview

DBC Studio is an open-source, cross-platform desktop application for creating, viewing, and editing
CAN DBC database files. Built with Tauri 2, SvelteKit (Svelte 5) and Rust.

**Target platforms:** macOS (arm64 + x86_64), Windows (x64), Linux (x64)
**License:** MIT

---

## Architecture

### Stack

| Layer | Technology | Purpose |
|---|---|---|
| UI Shell | Tauri 2 | Native window, IPC, file system, dialogs |
| Frontend | SvelteKit + Svelte 5 | Reactive UI, three-panel layout |
| Backend | Rust + `dbc-rs` 0.4.x | DBC parsing, serialization, validation |
| Styling | CSS variables (design tokens) | Light/dark theming |

### Directory Structure

```
dbc-studio/
├── src/                         Frontend (SvelteKit)
│   ├── routes/
│   │   └── +page.svelte         Main app shell (three-panel layout)
│   └── lib/
│       ├── components/
│       │   ├── Toolbar.svelte       File ops + undo/redo
│       │   ├── TreePanel.svelte     Message/signal tree
│       │   ├── SignalTable.svelte   Inline-editable signal grid
│       │   ├── PropertiesPanel.svelte  Detailed properties form
│       │   └── ToastContainer.svelte   Notifications
│       ├── stores/
│       │   ├── dbc.ts           DBC model store + undo/redo history
│       │   └── ui.ts            Selection state, toasts, panel widths
│       ├── api.ts               Typed Tauri invoke() wrappers
│       └── types.ts             TypeScript types mirroring Rust model
│
├── src-tauri/                   Rust backend
│   └── src/
│       ├── lib.rs               Tauri app setup + command registration
│       ├── dbc_model.rs         Serde-serializable mirror of dbc-rs types
│       └── commands/
│           ├── file.rs          open_dbc, save_dbc
│           ├── model.rs         default_message, default_signal, default_node
│           └── validate.rs      validate_dbc (duplicate IDs, bit ranges, etc.)
│
├── SPECIFICATION.md             This file
└── DEV_COMMANDS.md              Developer cheatsheet
```

### Data Flow

```
User opens .dbc
  → pickOpenFile() [dialog]
  → open_dbc(path) [Tauri cmd]
  → dbc_rs::Dbc::from_file() [Rust]
  → DbcModel::from(&dbc) [mirror conversion]
  → JSON over IPC
  → dbcStore.load(model) [Svelte store]
  → UI renders tree + table

User edits signal
  → dbcStore.updateSignal() [Svelte store, local]
  → UI re-renders immediately (reactive)
  → isDirty = true

User saves
  → pickSaveFile() [dialog]
  → save_dbc(path, model) [Tauri cmd]
  → dbc_rs::Dbc::try_from(&model) [Rust, via builders]
  → dbc.to_dbc_string() [Rust]
  → fs::write(path, content) [Rust]
```

---

## DBC Format Coverage

| DBC Element | Keyword | Phase |
|---|---|---|
| Messages | `BO_` | ✅ Phase 1 |
| Signals | `SG_` | ✅ Phase 1 |
| Nodes | `BU_` | ✅ Phase 1 |
| Value descriptions | `VAL_` | Phase 2 |
| Comments | `CM_` | Phase 2 |
| Attribute definitions | `BA_DEF_` | Phase 2 |
| Attribute values | `BA_` | Phase 2 |
| Value tables | `VAL_TABLE_` | Phase 2 |
| Message transmitters | `BO_TX_BU_` | Phase 2 |
| Basic multiplexing | `M` / `m<N>` | Phase 2 |
| Extended multiplexing | `SG_MUL_VAL_` | Phase 3 |
| Environment variables | `EV_` | Phase 3 |
| Signal groups | `SIG_GROUP_` | Phase 3 |
| J1939 mode (PGN/SPN) | via `BA_` | Phase 3 |

---

## Feature Specification

### Phase 1 — Core MVP (current)

**File operations**
- Open `.dbc` files via OS file dialog
- Save / Save As `.dbc` files via OS dialog
- New empty file
- Unsaved-changes indicator (● in title bar and toolbar)
- Confirmation dialog when discarding unsaved changes

**Tree panel (left)**
- Hierarchical view: Messages → Signals
- Expand/collapse messages
- Select message (highlights in table + properties)
- Select signal (highlights row + populates properties)
- Add signal to message (context + toolbar)
- Delete message (with confirmation)
- Delete signal (with confirmation)
- Node list (read-only display)

**Signal table (center)**
- Tabular view of all signals in the selected message
- Columns: Name, Start Bit, Length, Byte Order, Sign, Factor, Offset, Min, Max, Unit
- Inline cell editing: double-click any cell to edit in-place
- Dropdowns for Byte Order (LittleEndian / BigEndian) and Value Type (Signed / Unsigned)
- Add signal button
- Delete signal button per row

**Properties panel (right)**
- Message properties form: ID (hex), Name, DLC, Transmitter, Extended flag, Comment
- Signal properties form: full set of signal fields + physical value preview
- "Apply Changes" button commits edits to the store

**Undo / redo**
- Up to 100 history states (full model snapshots)
- Keyboard shortcuts: Ctrl/Cmd+Z (undo), Ctrl/Cmd+Y or Ctrl/Cmd+Shift+Z (redo)

**Validation (Phase 1 basic)**
- Duplicate CAN IDs
- Signal bit range overflow (start_bit + length > DLC × 8)
- Duplicate signal names in same message
- Empty names

**UX**
- Resizable panels (drag the dividers)
- Toast notifications (success / error / info)
- Keyboard shortcuts: Ctrl+O (open), Ctrl+S (save), Ctrl+Shift+S (save as)

### Phase 2 — Complete DBC

- Value descriptions editor (`VAL_` — enum-like signal mappings)
- Full comment editing (`CM_` on messages, signals, nodes)
- Attribute definitions (`BA_DEF_`) and values (`BA_`)
- Basic multiplexing: visual M/m tags, mux switch picker
- Node manager: add/rename/delete ECU nodes
- Global search/filter across messages and signals
- Validation panel: jump-to-error links
- Message transmitter lists (`BO_TX_BU_`)

### Phase 3 — Advanced

- Export to JSON / CSV
- Import signals from CSV (bulk add)
- Extended multiplexing (`SG_MUL_VAL_`)
- Environment variables (`EV_`)
- J1939 mode: PGN/SPN display, 29-bit ID helpers
- DBC diff / compare two files
- Dark / light theme toggle
- Drag-and-drop signal reordering
- Signal bit layout visualizer (grid view)

---

## Rust Backend

### Tauri Commands

| Command | Args | Returns | Description |
|---|---|---|---|
| `open_dbc` | `path: String` | `DbcModel` | Parse DBC file |
| `save_dbc` | `path: String, model: DbcModel` | `()` | Serialize and write DBC file |
| `validate_dbc` | `model: DbcModel` | `Vec<ValidationIssue>` | Run validation checks |
| `default_message` | `NewMessageArgs` | `MessageModel` | Template message |
| `default_signal` | `NewSignalArgs` | `SignalModel` | Template signal |
| `default_node` | `name: String` | `NodeModel` | Template node |
| `app_version` | — | `AppVersion` | Current app version |

### Key Rust Types

```rust
pub struct DbcModel {
    pub version: Option<String>,
    pub nodes: Vec<NodeModel>,
    pub messages: Vec<MessageModel>,
}
pub struct MessageModel {
    pub id: u32,           // raw ID (may include extended bit 31)
    pub is_extended: bool,
    pub name: String,
    pub dlc: u8,
    pub sender: String,
    pub signals: Vec<SignalModel>,
    pub comment: Option<String>,
}
pub struct SignalModel {
    pub name: String,
    pub start_bit: u16,
    pub length: u16,
    pub byte_order: String,           // "LittleEndian" | "BigEndian"
    pub is_unsigned: bool,
    pub factor: f64,
    pub offset: f64,
    pub min: f64, pub max: f64,
    pub unit: Option<String>,
    pub receivers: Vec<String>,
    pub is_multiplexer: bool,
    pub multiplexer_switch_value: Option<u64>,
    pub comment: Option<String>,
}
```

---

## Test DBC files

Compatible public DBC files for testing:
- [`opendbc`](https://github.com/commaai/opendbc) — comma.ai's large collection of real-world DBC files
- SavvyCAN sample: `examples/bms.dbc`
- CSS Electronics demo DBC files

---

## CI/CD (GitHub Actions)

Matrix build on push to `main`:
- macOS arm64 → `.dmg`
- macOS x86_64 → `.dmg`
- Windows x64 → `.msi` + `.exe`
- Linux x64 → `.AppImage` + `.deb`

Workflow file: `.github/workflows/build.yml`
