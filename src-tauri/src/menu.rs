//! Native application menu (File ▸ …, Edit ▸ …).
//!
//! Call [`build`] once at startup and whenever the recent-files list changes
//! (via [`rebuild`]).  Menu-item clicks are forwarded to the frontend as a
//! single Tauri event `"menu-action"` whose payload is a string action token:
//!
//! | Token                   | Meaning                                  |
//! |-------------------------|------------------------------------------|
//! | `file-open`             | File › Open…                             |
//! | `file-save`             | File › Save                              |
//! | `file-save-as`          | File › Save As…                          |
//! | `recent-clear`          | File › Open Recent › Clear List          |
//! | `open-recent:<path>`    | File › Open Recent › <filename>          |
//! | `edit-undo`             | Edit › Undo                              |
//! | `edit-redo`             | Edit › Redo                              |
//! | `edit-new-message`      | Edit › New Message                       |

use tauri::{AppHandle, Emitter};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};

// ─── Public API ───────────────────────────────────────────────────────────────

/// Build and return the full application menu.
pub fn build(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let menu = Menu::new(app)?;

    // macOS: the first entry is the traditional "[App name]" menu.
    #[cfg(target_os = "macos")]
    menu.append(&build_app_menu(app)?)?;

    menu.append(&build_file_menu(app)?)?;
    menu.append(&build_edit_menu(app)?)?;

    Ok(menu)
}

/// Rebuild the app menu and re-apply it (call after the recent-files list changes).
pub fn rebuild(app: &AppHandle) {
    if let Ok(m) = build(app) {
        let _ = app.set_menu(m);
    }
}

/// Handle a menu-item click: translate the item ID to a frontend action token
/// and emit it as a `"menu-action"` event.
pub fn handle_event(app: &AppHandle, id: &str) {
    let action: Option<String> = match id {
        "file-open"         => Some("file-open".into()),
        "file-save"         => Some("file-save".into()),
        "file-save-as"      => Some("file-save-as".into()),
        "edit-undo"         => Some("edit-undo".into()),
        "edit-redo"         => Some("edit-redo".into()),
        "edit-new-message"  => Some("edit-new-message".into()),
        "recent-clear"      => Some("recent-clear".into()),
        // recent-0, recent-1, … — resolve to the actual path.
        other if other.starts_with("recent-") => {
            other["recent-".len()..].parse::<usize>().ok().and_then(|idx| {
                crate::recent::load_existing(app).into_iter().nth(idx)
            }).map(|path| format!("open-recent:{path}"))
        }
        _ => None,
    };

    if let Some(token) = action {
        let _ = app.emit("menu-action", token);
    }
}

// ─── Menu builders ────────────────────────────────────────────────────────────

/// macOS "[App Name]" menu (About, Services, Hide, Quit, …).
#[cfg(target_os = "macos")]
fn build_app_menu(app: &AppHandle) -> tauri::Result<Submenu<tauri::Wry>> {
    Submenu::with_items(app, "DBC Studio", true, &[
        &PredefinedMenuItem::about(app, None, None)?,
        &PredefinedMenuItem::separator(app)?,
        &PredefinedMenuItem::services(app, None)?,
        &PredefinedMenuItem::separator(app)?,
        &PredefinedMenuItem::hide(app, None)?,
        &PredefinedMenuItem::hide_others(app, None)?,
        &PredefinedMenuItem::show_all(app, None)?,
        &PredefinedMenuItem::separator(app)?,
        &PredefinedMenuItem::quit(app, None)?,
    ])
}

fn build_file_menu(app: &AppHandle) -> tauri::Result<Submenu<tauri::Wry>> {
    Submenu::with_items(app, "File", true, &[
        &MenuItem::with_id(app, "file-open",    "Open…",    true, Some("CmdOrCtrl+O"))?,
        &build_recent_submenu(app)?,
        &PredefinedMenuItem::separator(app)?,
        &MenuItem::with_id(app, "file-save",    "Save",     true, Some("CmdOrCtrl+S"))?,
        &MenuItem::with_id(app, "file-save-as", "Save As…", true, Some("CmdOrCtrl+Shift+S"))?,
    ])
}

fn build_recent_submenu(app: &AppHandle) -> tauri::Result<Submenu<tauri::Wry>> {
    let sub   = Submenu::new(app, "Open Recent", true)?;
    let paths = crate::recent::load_existing(app);

    if paths.is_empty() {
        sub.append(
            &MenuItem::with_id(app, "recent-empty", "No Recent Files", false, None::<&str>)?
        )?;
    } else {
        for (i, path) in paths.iter().enumerate() {
            let name = std::path::Path::new(path)
                .file_name().and_then(|n| n.to_str()).unwrap_or(path);
            sub.append(&MenuItem::with_id(app, format!("recent-{i}"), name, true, None::<&str>)?)?;
        }
        sub.append(&PredefinedMenuItem::separator(app)?)?;
        sub.append(&MenuItem::with_id(app, "recent-clear", "Clear List", true, None::<&str>)?)?;
    }

    Ok(sub)
}

fn build_edit_menu(app: &AppHandle) -> tauri::Result<Submenu<tauri::Wry>> {
    Submenu::with_items(app, "Edit", true, &[
        &MenuItem::with_id(app, "edit-undo",        "Undo",        true, Some("CmdOrCtrl+Z"))?,
        &MenuItem::with_id(app, "edit-redo",        "Redo",        true, Some("CmdOrCtrl+Shift+Z"))?,
        &PredefinedMenuItem::separator(app)?,
        &MenuItem::with_id(app, "edit-new-message", "New Message", true, None::<&str>)?,
    ])
}
