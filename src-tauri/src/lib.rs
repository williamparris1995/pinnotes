// PinNotes entry assembly. Task 1 (scaffold): registers the autostart plugin,
// keeps the main window hidden, and builds a small transparent always-on-top
// "smoke" window to verify the Tauri platform feasibility (transparent /
// borderless / topmost). Later tasks replace this with the real note windows.

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
