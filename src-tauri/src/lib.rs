// PinNotes entry assembly. Task 1 (scaffold): registers the autostart plugin,
// keeps the main window hidden, and builds a small transparent always-on-top
// "smoke" window to verify the Tauri platform feasibility (transparent /
// borderless / topmost). Later tasks replace this with the real note windows.

// rusqlite data layer + NoteRepository. Not yet wired into setup(); consumed in
// a later task, so silence the unused-module warning in non-test builds.
#[allow(dead_code)]
mod db;

// Work-area geometry clamp (keep note windows inside the monitor that covers
// them the most) + snooze repop decision/scheduler. Pure domain logic; not yet
// wired into setup(), so silence the unused-module warning until Task 10.
#[allow(dead_code)]
mod geometry;
#[allow(dead_code)]
mod snooze;

// Per-note WebviewWindow lifecycle (open/show/hide/close/move) via the Tauri v2
// WebviewWindow API. Not yet wired into setup(); silence the unused-module
// warning until Task 10.
#[allow(dead_code)]
mod window_manager;

// System-tray menu (new / show-all / completed / settings / quit + separator)
// built with Tauri v2's TrayIconBuilder. Not yet wired into setup(); menu
// dispatch lands in `on_menu_event` in Task 10, so silence the unused-module
// warning until then.
#[allow(dead_code)]
mod tray;

// OS-level autostart enable/disable wrapper around tauri-plugin-autostart
// (already registered in the builder below). Not yet wired into setup();
// silence the unused-module warning until Task 10.
#[allow(dead_code)]
mod autostart;

use tauri::{WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::MacosLauncher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            // Smoke window: transparent, no decorations, always-on-top, no taskbar.
            // Visual confirmation of the sticky-note window primitives; removed in a later task.
            WebviewWindowBuilder::new(app, "smoke", WebviewUrl::App("index.html".into()))
                .title("PinNotes smoke")
                .inner_size(240.0, 170.0)
                .transparent(true)
                .decorations(false)
                .always_on_top(true)
                .skip_taskbar(true)
                .resizable(false)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
