// Shared application state managed by Tauri. Injected once in `setup()` via
// `app.manage(AppState { ... })` and accessed in commands via `State<AppState>`.
use crate::{db::Db, snooze::SnoozeScheduler};

pub struct AppState {
    pub db: Db,
    pub scheduler: SnoozeScheduler,
}
