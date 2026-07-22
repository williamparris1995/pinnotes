use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

pub fn configure(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let mgr = app.autolaunch();
    if enabled { mgr.enable().map_err(|e| e.to_string()) }
    else { mgr.disable().map_err(|e| e.to_string()) }
}

pub fn is_enabled(app: &AppHandle) -> Result<bool, String> {
    app.autolaunch().is_enabled().map_err(|e| e.to_string())
}
