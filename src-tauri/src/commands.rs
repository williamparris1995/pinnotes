// Tauri command layer: thin `#[tauri::command]` wrappers around plain
// `*_impl` functions. The impl split lets `on_menu_event` (in lib.rs) reuse the
// same logic without going through the command dispatch convention.
use crate::{
    autostart,
    db::{Note, NoteRepository},
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

#[tauri::command]
pub fn hide_note(id: String, app: AppHandle, state: State<AppState>) -> Result<(), String> {
    let mins = default_snooze(&state)? as i64;
    let until = Utc::now() + chrono::Duration::minutes(mins);
    NoteRepository::snooze(&state.db, &id, &until.to_rfc3339())?;
    window_manager::hide_note(&app, &id).map_err(|e| e.to_string())?;
    let app2 = app.clone();
    state
        .scheduler
        .schedule(id.clone(), Duration::from_secs(mins as u64 * 60), move || {
            let _ = repop(&app2, &id);
        });
    Ok(())
}

fn repop(app: &AppHandle, id: &str) -> Result<(), String> {
    let state = app.state::<AppState>();
    if let Some(n) = NoteRepository::get(&state.db, id)? {
        if n.completed_at.is_none() {
            NoteRepository::clear_snooze(&state.db, id)?;
            window_manager::show_note(app, id).map_err(|e| e.to_string())?;
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
pub fn reactivate(id: String, app: AppHandle, state: State<AppState>) -> Result<(), String> {
    NoteRepository::reactivate(&state.db, &id)?;
    if let Some(n) = NoteRepository::get(&state.db, &id)? {
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

#[tauri::command]
pub fn get_settings(
    state: State<AppState>,
) -> Result<std::collections::HashMap<String, String>, String> {
    settings_map(&state)
}

#[tauri::command]
pub fn set_settings(key: String, value: String, state: State<AppState>) -> Result<(), String> {
    let lock = state.db.lock().map_err(|e| e.to_string())?;
    lock.execute(
        "INSERT INTO settings(key,val) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET val=excluded.val",
        rusqlite::params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_autostart(app: AppHandle) -> bool {
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
