# DBC Studio — Developer Cheatsheet

## Prerequisites

```bash
# Load Rust toolchain
source $HOME/.cargo/env

# Load Node.js via nvm
export NVM_DIR="$HOME/.nvm" && [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
```

---

## Daily Dev Commands

```bash
# Start full dev environment (hot-reload UI + Rust rebuild on change)
npm run tauri dev

# Frontend only (browser, no Tauri)
npm run dev

# Type-check frontend
npm run check

# Watch type-check
npm run check:watch
```

---

## Rust Backend

```bash
# Check Rust code only (fast, no linking)
source $HOME/.cargo/env && cd src-tauri && cargo check

# Build release binary
source $HOME/.cargo/env && cd src-tauri && cargo build --release

# Run Rust tests
source $HOME/.cargo/env && cd src-tauri && cargo test

# Add a Rust dependency
source $HOME/.cargo/env && cd src-tauri && cargo add <crate-name>

# Update Rust dependencies
source $HOME/.cargo/env && cd src-tauri && cargo update

# Check for outdated Rust deps
source $HOME/.cargo/env && cd src-tauri && cargo outdated
```

---

## Frontend

```bash
# Install npm dependencies
npm install

# Add a frontend package
npm install <package-name>

# Add a dev-only package
npm install --save-dev <package-name>

# Update npm packages
npm update

# Format code (if prettier configured)
npx prettier --write src/
```

---

## Building for Distribution

```bash
# Build for current platform (produces installers in src-tauri/target/release/bundle/)
npm run tauri build

# Build without bundling (raw binary only)
npm run tauri build -- --no-bundle

# macOS universal binary (arm64 + x86_64)
npm run tauri build -- --target universal-apple-darwin
```

---

## Tauri Plugin Management

```bash
# Add a Tauri plugin (Rust side)
source $HOME/.cargo/env && cd src-tauri && cargo add tauri-plugin-<name>@2

# Add the JS bindings
npm install @tauri-apps/plugin-<name>

# Then register in src-tauri/src/lib.rs:
# .plugin(tauri_plugin_<name>::init())
# And add permissions in src-tauri/capabilities/default.json
```

---

## Useful Paths

| Path | Description |
|---|---|
| `src/routes/+page.svelte` | Main app shell |
| `src/lib/components/` | Svelte components |
| `src/lib/stores/dbc.ts` | DBC model store + undo/redo |
| `src/lib/stores/ui.ts` | UI state (selection, toasts) |
| `src/lib/api.ts` | Tauri invoke() wrappers |
| `src/lib/types.ts` | TypeScript types (mirrors Rust) |
| `src-tauri/src/dbc_model.rs` | Rust serde types |
| `src-tauri/src/commands/` | Tauri command handlers |
| `src-tauri/tauri.conf.json` | App config (window size, name, ID) |
| `src-tauri/capabilities/default.json` | Tauri permissions |
| `src-tauri/Cargo.toml` | Rust dependencies |

---

## Adding a New Tauri Command

1. Write the function in `src-tauri/src/commands/<module>.rs`:
   ```rust
   #[tauri::command]
   pub fn my_command(arg: String) -> Result<String, String> { ... }
   ```
2. Register it in `src-tauri/src/lib.rs`:
   ```rust
   .invoke_handler(tauri::generate_handler![..., my_command])
   ```
3. Add a typed wrapper in `src/lib/api.ts`:
   ```ts
   export async function myCommand(arg: string): Promise<string> {
     return invoke<string>('my_command', { arg });
   }
   ```

---

## GitHub

```bash
# Push to main
git push origin main

# Create a release tag
git tag v0.1.0 && git push origin v0.1.0

# View repo
gh repo view --web

# Create a PR
gh pr create --title "feat: ..." --body "..."

# Check CI status
gh run list
```

---

## dbc-rs Quick Reference

```rust
// Parse a DBC file
let dbc = dbc_rs::Dbc::from_file("path/to/file.dbc")?;

// Iterate messages
for msg in dbc.messages().iter() {
    println!("{} (ID: 0x{:X})", msg.name(), msg.id());
    for sig in msg.signals().iter() {
        println!("  {} start={} len={}", sig.name(), sig.start_bit(), sig.length());
    }
}

// Build a DBC programmatically
let dbc = dbc_rs::DbcBuilder::new()
    .version(dbc_rs::VersionBuilder::new().version("1.0"))
    .nodes(dbc_rs::NodesBuilder::new().add_node("ECM"))
    .add_message(
        dbc_rs::MessageBuilder::new()
            .id(0x100).name("EngineData").dlc(8).sender("ECM")
            .add_signal(
                dbc_rs::SignalBuilder::new()
                    .name("RPM").start_bit(0).length(16)
                    .byte_order(dbc_rs::ByteOrder::LittleEndian)
                    .unsigned(true).factor(0.25).offset(0.0)
                    .min(0.0).max(8000.0).unit("rpm")
                    .receivers(dbc_rs::ReceiversBuilder::new().none())
            )
    )
    .build()?;

// Serialize back to DBC string
let content = dbc.to_dbc_string();
```

---

## Environment (from memory)

- Rust: `1.94.0` stable (aarch64-apple-darwin)
- Node: `v24.14.0` via nvm
- npm: `11.9.0`
- Tauri: `2.x`
