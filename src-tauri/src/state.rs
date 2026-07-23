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
    /// rapid `WindowEvent::Moved` stream into a single write ~200ms after the
    /// drag stops, off the main thread. (Writing SQLite on the main thread per
    /// pixel was lagging the drag — the "drift" the user saw.)
    pub drag_writes: Mutex<HashMap<String, JoinHandle<()>>>,
}
