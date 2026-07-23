use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::{App, Emitter, Wry};

pub const MENU_EVENT_OPEN: &str = "menu:open";
pub const MENU_EVENT_SAVE: &str = "menu:save";
pub const MENU_EVENT_SAVE_AS: &str = "menu:save-as";
pub const MENU_EVENT_SETTINGS: &str = "menu:settings";

/// Builds the native macOS menu bar: an App menu with Settings/Quit, a File menu whose items
/// are forwarded to the frontend as events (file I/O itself stays in the frontend/Rust command
/// layer, the menu is just another trigger for it), and a standard Edit menu — the predefined
/// Undo/Redo/Cut/Copy/Paste/SelectAll items are what make those shortcuts work inside the
/// CodeMirror editor and text inputs at all on macOS.
pub fn setup(app: &App<Wry>) -> tauri::Result<()> {
    let settings_item = MenuItemBuilder::with_id(MENU_EVENT_SETTINGS, "Settings…")
        .accelerator("CmdOrCtrl+,")
        .build(app)?;
    let open_item = MenuItemBuilder::with_id(MENU_EVENT_OPEN, "Open…")
        .accelerator("CmdOrCtrl+O")
        .build(app)?;
    let save_item = MenuItemBuilder::with_id(MENU_EVENT_SAVE, "Save")
        .accelerator("CmdOrCtrl+S")
        .build(app)?;
    let save_as_item = MenuItemBuilder::with_id(MENU_EVENT_SAVE_AS, "Save As…")
        .accelerator("CmdOrCtrl+Shift+S")
        .build(app)?;

    let app_menu = SubmenuBuilder::new(app, "Markllama")
        .about(None)
        .separator()
        .item(&settings_item)
        .separator()
        .services()
        .separator()
        .hide()
        .hide_others()
        .show_all()
        .separator()
        .quit()
        .build()?;

    let file_menu = SubmenuBuilder::new(app, "File")
        .item(&open_item)
        .item(&save_item)
        .item(&save_as_item)
        .build()?;

    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;

    let menu = MenuBuilder::new(app)
        .item(&app_menu)
        .item(&file_menu)
        .item(&edit_menu)
        .build()?;

    app.set_menu(menu)?;

    app.on_menu_event(|app_handle, event| {
        let id = event.id().0.as_str();
        if matches!(
            id,
            MENU_EVENT_OPEN | MENU_EVENT_SAVE | MENU_EVENT_SAVE_AS | MENU_EVENT_SETTINGS
        ) {
            let _ = app_handle.emit(id, ());
        }
    });

    Ok(())
}
