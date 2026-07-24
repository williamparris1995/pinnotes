// Shared application state managed by Tauri. Injected once in `setup()` via
// `app.manage(AppState { ... })` and accessed in commands via `State<AppState>`.
use crate::{db::Db, snooze::SnoozeScheduler};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::async_runtime::JoinHandle;

pub struct AppState {
    pub db: Db,
    pub scheduler: SnoozeScheduler,
    /// Debounce handles for per-note drag-position DB writes: coalesce the
    /// rapid `WindowEvent::Moved` stream into a single write ~250ms after the
    /// drag stops, off the main thread.
    pub drag_writes: Mutex<HashMap<String, JoinHandle<()>>>,
    /// 启动后台查到的最新版本号(Some = 有新版本;None = 无/未查到)。
    /// tray 菜单读它决定是否显示"新版本"项(见 ADR-0002)。
    pub update_status: Mutex<Option<String>>,
}
