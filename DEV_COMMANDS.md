# DBC Studio — Developer Cheatsheet

All commands are written as single copy-paste lines that source the required toolchains
(`cargo`, `nvm`) inline, so they work from any shell session without a separate setup step.

---

## 0 · Shell setup (one-time per terminal, optional)

If you prefer to source toolchains once rather than inline, paste this block first:

```bash
source $HOME/.cargo/env && export NVM_DIR="$HOME/.nvm" && [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
```

---

## 1 · Development build (hot-reload)

Launches the full Tauri dev window with Vite hot-reload for the UI and automatic Rust
recompilation on file changes.

```bash
source $HOME/.cargo/env && export NVM_DIR="$HOME/.nvm" && [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh" && npm run tauri dev
```

---

## 2 · Type-check (frontend + Svelte)

Runs `svelte-check` against the full TypeScript config. Exits non-zero on any error.

```bash
export NVM_DIR="$HOME/.nvm" && [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh" && npm run check
```

---

## 3 · Type-check (Rust only)

Fast Rust analysis without linking — catches type errors and borrow issues in seconds.

```bash
source $HOME/.cargo/env && cargo check --manifest-path src-tauri/Cargo.toml
```

---

## 4 · Local release build (current platform)

Compiles an optimized binary and produces a native installer under
`src-tauri/target/release/bundle/`.

```bash
source $HOME/.cargo/env && export NVM_DIR="$HOME/.nvm" && [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh" && npm run tauri build
```

macOS universal binary (arm64 + x86_64):

```bash
source $HOME/.cargo/env && export NVM_DIR="$HOME/.nvm" && [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh" && npm run tauri build -- --target universal-apple-darwin
```

---

## 5 · Version bump

Replace `NEW_VER` with the new semver string (e.g. `0.2.0`). Updates all three version
fields that Tauri requires to agree: `package.json`, `src-tauri/Cargo.toml`, and
`src-tauri/tauri.conf.json`.

```bash
NEW_VER="0.2.0" && \
  node -e "const f='package.json',j=JSON.parse(require('fs').readFileSync(f));j.version='$NEW_VER';require('fs').writeFileSync(f,JSON.stringify(j,null,2)+'\n')" && \
  sed -i '' "s/^version = \".*\"/version = \"$NEW_VER\"/" src-tauri/Cargo.toml && \
  node -e "const f='src-tauri/tauri.conf.json',j=JSON.parse(require('fs').readFileSync(f));j.version='$NEW_VER';require('fs').writeFileSync(f,JSON.stringify(j,null,2)+'\n')"
```

Verify the three files updated correctly:

```bash
grep '"version"' package.json src-tauri/tauri.conf.json && grep '^version' src-tauri/Cargo.toml
```

---

## 6 · Stage and commit

Replace the commit message as needed. Uses the project's conventional-commit style.

```bash
git add -p && git commit -m "feat: describe your change here"
```

Stage everything (tracked files only) in one go:

```bash
git add -u && git commit -m "feat: describe your change here"
```

---

## 7 · Push to remote

```bash
git push origin main
```

---

## 8 · Tag, push tag, and draft GitHub release

Replace `v0.2.0` with the actual version. Creates an annotated tag locally, pushes it,
then opens a draft GitHub release pre-filled with that tag so you can write release notes
in the browser before publishing.

```bash
VER="v0.2.0" && \
  git tag -a "$VER" -m "Release $VER" && \
  git push origin "$VER" && \
  gh release create "$VER" --draft --title "DBC Studio $VER" --notes "## What's new\n\n- " --web
```

List existing tags and releases:

```bash
git tag --sort=-creatordate | head -10 && gh release list
```

---

## Useful Paths

| Path | Description |
|---|---|
| `src/routes/+page.svelte` | Main app shell |
| `src/lib/components/` | Svelte components |
| `src/lib/stores/dbc.ts` | DBC model store + undo/redo |
| `src/lib/stores/ui.ts` | UI state (selection, toasts, hexMode) |
| `src/lib/api.ts` | Tauri `invoke()` wrappers |
| `src/lib/types.ts` | TypeScript types (mirrors Rust) |
| `src-tauri/src/dbc_model.rs` | Rust serde types + dbc-rs conversions |
| `src-tauri/src/commands/` | Tauri command handlers |
| `src-tauri/tauri.conf.json` | App config, version, window size |
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

## Environment

- Rust: `1.94.0` stable (aarch64-apple-darwin) — toolchain at `~/.cargo/`
- Node: `v24.14.0` via nvm (`~/.nvm/`)
- npm: `11.9.0`
- Tauri: `2.x`
