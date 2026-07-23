// Shared application state managed by Tauri. Injected once in `setup()` via
// `app.manage(AppState { ... })` and accessed in commands via `State<AppState>`.
use crate::{db::Db, snooze::SnoozeScheduler};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use tauri::async_runtime::JoinHandle;

pub struct AppState {
    pub db: Db,
    pub scheduler: SnoozeScheduler,
    /// Debounce handles for per-note drag-position DB writes: coalesce the
    /// rapid `WindowEvent::Moved` stream into a single write ~250ms after the
    /// drag stops, off the main thread. (Writing SQLite on the main thread per
    /// pixel was lagging the drag.)
    pub drag_writes: Mutex<HashMap<String, JoinHandle<()>>>,
    /// Note ids whose acrylic effect is currently disabled (because they are
    /// being dragged). Re-blurring an acrylic window every frame lags the drag,
    /// so acrylic is turned off at drag start and back on when it stops.
    pub acrylic_off: Mutex<HashSet<String>>,
}
