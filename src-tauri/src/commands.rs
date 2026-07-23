// Tauri command layer: thin `#[tauri::command]` wrappers around plain
// `*_impl` functions. The impl split lets `on_menu_event` (in lib.rs) reuse the
// same logic without going through the command dispatch convention.
use crate::{
    autostart,
    db::{Db, Note, NoteRepository},
    geometry::{clamp_into_work_area, Rect},
    state::AppState,
    window_manager,
};
use chrono::Utc;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

#[tauri::command]
pub fn get_note(id: String, state: State<AppState>) -> Result<Option<Note>, String> {
    NoteRepository::get(&state.db, &id)
}

// --- create_note: impl + command wrapper -------------------------------------
pub fn create_note_impl(app: &AppHandle, state: &AppState) -> Result<Note, String> {
    let n = Note {
        id: Uuid::new_v4().to_string(),
        content: String::new(),
        color: "yellow".into(),
        x: 120.0,
        y: 40.0,
        w: 240.0,
        h: 170.0,
        snooze_minutes: default_snooze(state)? as i64,
        created_at: now_iso(),
        completed_at: None,
        is_hidden: false,
        hidden_until: None,
    };
    NoteRepository::create(&state.db, &n)?;
    window_manager::open_note(app, &n).map_err(|e| e.to_string())?;
    Ok(n)
}

#[tauri::command]
pub fn create_note(app: AppHandle, state: State<AppState>) -> Result<Note, String> {
    create_note_impl(&app, &state)
}

/// First-run welcome note: when `first_run_done` is unset, create a visible
/// sticky note that explains the app (so launching PinNotes is never a blank
/// screen), then mark the flag. Returns `Some(note)` on the first run (caller
/// opens its window) and `None` on every later run. Pure DB logic — testable
/// without Tauri.
pub(crate) fn maybe_welcome_note(db: &Db) -> Result<Option<Note>, String> {
    if get_setting(db, "first_run_done")?.is_some() {
        return Ok(None);
    }
    let note = Note {
        id: Uuid::new_v4().to_string(),
        content: "欢迎使用 PinNotes！\n\n这是一条置顶便签。\n右键托盘图标（屏幕右下角，可能在 ^ 隐藏区里）可：新建便签 / 显示全部 / 已完成 / 设置 / 退出。\n\n点「隐藏」会短暂收起、到点自动弹回；点「✓ 完成」才会让它消失。".into(),
        color: "yellow".into(),
        x: 160.0,
        y: 80.0,
        w: 240.0,
        h: 260.0,
        snooze_minutes: 2,
        created_at: now_iso(),
        completed_at: None,
        is_hidden: false,
        hidden_until: None,
    };
    NoteRepository::create(db, &note)?;
    set_setting(db, "first_run_done", "1")?;
    Ok(Some(note))
}

#[tauri::command]
pub fn hide_note(id: String, app: AppHandle, state: State<AppState>) -> Result<(), String> {
    // The note's own snooze_minutes is authoritative; fall back to the global
    // default only when the note has no (zero) value of its own.
    let note_mins = NoteRepository::get(&state.db, &id)?
        .map(|n| n.snooze_minutes)
        .unwrap_or(0);
    let mins = if note_mins > 0 {
        note_mins
    } else {
        default_snooze(&state)? as i64
    };
    let until = Utc::now() + chrono::Duration::minutes(mins);
    NoteRepository::snooze(&state.db, &id, &until.to_rfc3339())?;
    window_manager::hide_note(&app, &id).map_err(|e| e.to_string())?;
    let app2 = app.clone();
    state
        .scheduler
        .schedule(id.clone(), Duration::from_secs(mins as u64 * 60), move || {
            let _ = repop_note(&app2, &id);
        });
    Ok(())
}

/// Repop a snoozed note: if it still exists and isn't completed, clear the
/// snooze and re-show its window. Shared by the in-process scheduler wake
/// (hide_note's timer) and the startup scheduler wake (`lib::commands_show`).
pub(crate) fn repop_note(app: &AppHandle, id: &str) -> Result<(), String> {
    let state = app.state::<AppState>();
    if let Some(n) = NoteRepository::get(&state.db, id)? {
        if n.completed_at.is_none() {
            NoteRepository::clear_snooze(&state.db, id)?;
            window_manager::show_note_no_focus(app, id).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn complete_note(id: String, app: AppHandle, state: State<AppState>) -> Result<(), String> {
    NoteRepository::complete(&state.db, &id, &now_iso())?;
    state.scheduler.cancel(&id);
    window_manager::close_note(&app, &id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn edit_note(id: String, content: String, state: State<AppState>) -> Result<(), String> {
    NoteRepository::update_content(&state.db, &id, &content)
}

#[tauri::command]
pub fn set_color(id: String, color: String, state: State<AppState>) -> Result<(), String> {
    NoteRepository::update_color(&state.db, &id, &color)
}

#[tauri::command]
pub fn set_snooze(id: String, minutes: i64, state: State<AppState>) -> Result<(), String> {
    NoteRepository::update_snooze_minutes(&state.db, &id, minutes)
}

#[tauri::command]
pub fn move_note(
    id: String,
    x: f64,
    y: f64,
    app: AppHandle,
    state: State<AppState>,
) -> Result<(), String> {
    // w/h come from the stored note row (drag is native; the frontend only
    // reports the new top-left), so we look them up to clamp the full rect.
    let (w, h) = NoteRepository::get(&state.db, &id)?
        .map(|n| (n.w, n.h))
        .unwrap_or((240.0, 170.0));
    let clamped = clamp_note(&app, x, y, w, h);
    NoteRepository::update_position(&state.db, &id, clamped.0, clamped.1)?;
    window_manager::move_note(&app, &id, clamped.0, clamped.1).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn set_size(
    id: String,
    w: f64,
    h: f64,
    app: AppHandle,
    state: State<AppState>,
) -> Result<(), String> {
    NoteRepository::update_size(&state.db, &id, w, h)?;
    window_manager::resize_note(&app, &id, w, h).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn reactivate(id: String, app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    NoteRepository::reactivate(&state.db, &id)?;
    if let Some(n) = NoteRepository::get(&state.db, &id)? {
        // Async command -> runs on the runtime, so the main thread is free for
        // WebviewWindowBuilder::build(). A sync command would run on the main
        // thread inside the IPC handler and deadlock build (it needs the
        // message loop); from the runtime it marshals onto a free main thread.
        window_manager::open_note(&app, &n).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn copy_note(id: String, app: AppHandle, state: State<AppState>) -> Result<Note, String> {
    let src = NoteRepository::get(&state.db, &id)?.ok_or("not found")?;
    let n = Note {
        id: Uuid::new_v4().to_string(),
        content: src.content,
        color: src.color,
        x: src.x + 24.0,
        y: src.y + 24.0,
        w: src.w,
        h: src.h,
        snooze_minutes: src.snooze_minutes,
        created_at: now_iso(),
        completed_at: None,
        is_hidden: false,
        hidden_until: None,
    };
    NoteRepository::create(&state.db, &n)?;
    window_manager::open_note(&app, &n).map_err(|e| e.to_string())?;
    Ok(n)
}

#[tauri::command]
pub fn delete_note(id: String, state: State<AppState>) -> Result<(), String> {
    NoteRepository::delete(&state.db, &id)
}

#[tauri::command]
pub fn list_completed(state: State<AppState>) -> Result<Vec<Note>, String> {
    NoteRepository::completed(&state.db)
}

// --- show_all: impl + command wrapper ----------------------------------------
pub fn show_all_impl(app: &AppHandle, state: &AppState) -> Result<(), String> {
    for n in NoteRepository::active(&state.db)? {
        NoteRepository::clear_snooze(&state.db, &n.id)?;
        state.scheduler.cancel(&n.id);
        window_manager::show_note(app, &n.id).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn show_all(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    show_all_impl(&app, &state)
}

// --- hide_all: impl + command wrapper ----------------------------------------
/// Hide every active note's window (no snooze — they stay hidden until
/// 显示全部 or a per-note re-pop). Symmetric counterpart to show_all.
pub fn hide_all_impl(app: &AppHandle, state: &AppState) -> Result<(), String> {
    for n in NoteRepository::active(&state.db)? {
        window_manager::hide_note(app, &n.id).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn hide_all(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    hide_all_impl(&app, &state)
}

#[tauri::command]
pub fn get_settings(
    state: State<AppState>,
) -> Result<std::collections::HashMap<String, String>, String> {
    settings_map(&state)
}

#[tauri::command]
pub fn set_settings(key: String, value: String, state: State<AppState>) -> Result<(), String> {
    set_setting(&state.db, &key, &value)
}

/// Read a single setting value, or `None` if the key is absent. Shared with
/// `lib::setup` for the autostart first-run guard.
pub(crate) fn get_setting(db: &Db, key: &str) -> Result<Option<String>, String> {
    use rusqlite::OptionalExtension;
    let lock = db.lock().map_err(|e| e.to_string())?;
    lock.query_row(
        "SELECT val FROM settings WHERE key=?1",
        rusqlite::params![key],
        |r| r.get::<_, String>(0),
    )
    .optional()
    .map_err(|e| e.to_string())
}

/// Upsert a single setting value (INSERT … ON CONFLICT UPDATE).
pub(crate) fn set_setting(db: &Db, key: &str, val: &str) -> Result<(), String> {
    let lock = db.lock().map_err(|e| e.to_string())?;
    lock.execute(
        "INSERT INTO settings(key,val) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET val=excluded.val",
        rusqlite::params![key, val],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_autostart(app: AppHandle) -> Result<bool, String> {
    autostart::is_enabled(&app)
}

#[tauri::command]
pub fn set_autostart(enabled: bool, app: AppHandle) -> Result<(), String> {
    autostart::configure(&app, enabled)
}

fn default_snooze(state: &AppState) -> Result<u64, String> {
    Ok(settings_map(state)?
        .get("default_snooze_minutes")
        .and_then(|v| v.parse().ok())
        .unwrap_or(2))
}

fn settings_map(state: &AppState) -> Result<std::collections::HashMap<String, String>, String> {
    let lock = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = lock
        .prepare("SELECT key, val FROM settings")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?;
    let mut m = std::collections::HashMap::new();
    for r in rows {
        let row = r.map_err(|e| e.to_string())?;
        m.insert(row.0, row.1);
    }
    Ok(m)
}

/// Clamp a note's top-left so the full `w × h` rect stays inside the monitor
/// that covers it the most (8px margin). Monitors come from any live webview
/// window — `available_monitors()` reports the whole system list, not just the
/// one the window sits on, so any window suffices. Falls back to the raw
/// coordinates when no window exists yet or the monitor list is unavailable.
fn clamp_note(app: &AppHandle, x: f64, y: f64, w: f64, h: f64) -> (f64, f64) {
    let Some(win) = app.webview_windows().into_values().next() else {
        return (x, y);
    };
    let monitors = match win.available_monitors() {
        Ok(ms) if !ms.is_empty() => ms,
        _ => return (x, y),
    };
    let rects: Vec<Rect> = monitors
        .iter()
        .map(|m| {
            // Physical → logical: divide by scale so coords match the note's
            // logical pixel space (exact on a 1.0 scale factor).
            let scale = m.scale_factor();
            Rect {
                left: m.position().x as f64 / scale,
                top: m.position().y as f64 / scale,
                width: m.size().width as f64 / scale,
                height: m.size().height as f64 / scale,
            }
        })
        .collect();
    let clamped = clamp_into_work_area(
        Rect {
            left: x,
            top: y,
            width: w,
            height: h,
        },
        &rects,
        8.0,
    );
    (clamped.left, clamped.top)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init;
    use rusqlite::Connection;

    fn mem() -> Db {
        init(Connection::open_in_memory().unwrap()).unwrap()
    }

    #[test]
    fn welcome_note_created_only_on_first_run() {
        let db = mem();
        // First run: flag absent -> create a welcome note with guidance content.
        let n1 = maybe_welcome_note(&db)
            .unwrap()
            .expect("first run yields a welcome note");
        assert!(!n1.content.is_empty());
        assert_eq!(NoteRepository::active(&db).unwrap().len(), 1);
        // Flag now set -> second call is a no-op (None, no extra note).
        assert!(maybe_welcome_note(&db).unwrap().is_none());
        assert_eq!(NoteRepository::active(&db).unwrap().len(), 1);
    }
}
