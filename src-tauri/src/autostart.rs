use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

use crate::db::to_str;

pub fn configure(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let mgr = app.autolaunch();
    if enabled { mgr.enable().map_err(to_str) }
    else { mgr.disable().map_err(to_str) }
}

pub fn is_enabled(app: &AppHandle) -> Result<bool, String> {
    app.autolaunch().is_enabled().map_err(to_str)
}
