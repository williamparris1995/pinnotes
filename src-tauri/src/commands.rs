// Tauri command layer: thin `#[tauri::command]` wrappers around plain
// `*_impl` functions. The impl split lets the tray menu / shortcuts reuse the
// same logic without going through the command dispatch convention.
use crate::{
    autostart,
    db::{to_str, Db, Note, NoteRepository},
    geometry::{clamp_into_work_area, Rect},
    state::AppState,
    window_manager,
};
use chrono::Utc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_updater::UpdaterExt;
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
        markdown: false,
    };
    NoteRepository::create(&state.db, &n)?;
    window_manager::open_note(app, &n).map_err(to_str)?;
    Ok(n)
}

#[tauri::command]
pub fn create_note(app: AppHandle, state: State<AppState>) -> Result<Note, String> {
    create_note_impl(&app, &state)
}

/// 新建便签。供托盘左键、全局 Ctrl+N、HTML 菜单"新建便签"三条入口共用。
pub(crate) fn new_note(app: &AppHandle, state: &AppState) -> Result<Note, String> {
    create_note_impl(app, state)
}

/// First-run welcome note: when `first_run_done` is unset, create a visible
/// sticky note that explains the app (so launching PinNotes is never a blank
/// screen), then mark the flag. Returns `Some(note)` on the first run (caller
/// opens its window) and `None` on every later run. Pure DB logic — testable
/// without Tauri.
const WELCOME_ZH: &str = "欢迎使用 PinNotes！\n\n这是一条置顶便签。\n右键托盘图标（屏幕右下角，可能在 ^ 隐藏区里）可：新建便签 / 显示全部 / 已完成 / 设置 / 退出。\n\n点「隐藏」会短暂收起、到点自动弹回；点「✓ 完成」才会让它消失。";
const WELCOME_EN: &str = "Welcome to PinNotes!\n\nThis is a pinned sticky note.\nRight-click the tray icon (bottom-right of the screen, maybe in the ^ hidden area) to: New note / Show all / Completed / Settings / Quit.\n\nClick Hide to snooze it (it pops back in place when due); click ✓ Done to remove it.";

pub(crate) fn maybe_welcome_note(db: &Db) -> Result<Option<Note>, String> {
    if get_setting(db, "first_run_done")?.is_some() {
        return Ok(None);
    }
    let note = Note {
        id: Uuid::new_v4().to_string(),
        content: (if lang(db) == "zh" { WELCOME_ZH } else { WELCOME_EN }).into(),
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
        markdown: false,
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
    window_manager::hide_note(&app, &id).map_err(to_str)?;
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
            window_manager::show_note_no_focus(app, id).map_err(to_str)?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn complete_note(id: String, app: AppHandle, state: State<AppState>) -> Result<(), String> {
    NoteRepository::complete(&state.db, &id, &now_iso())?;
    state.scheduler.cancel(&id);
    window_manager::close_note(&app, &id).map_err(to_str)?;
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
pub fn set_markdown(id: String, on: bool, state: State<AppState>) -> Result<(), String> {
    NoteRepository::update_markdown(&state.db, &id, on)
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
    window_manager::move_note(&app, &id, clamped.0, clamped.1).map_err(to_str)?;
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
    window_manager::resize_note(&app, &id, w, h).map_err(to_str)?;
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
        window_manager::open_note(&app, &n).map_err(to_str)?;
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
        markdown: src.markdown,
    };
    NoteRepository::create(&state.db, &n)?;
    window_manager::open_note(&app, &n).map_err(to_str)?;
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

#[tauri::command]
pub fn list_active(state: State<AppState>) -> Result<Vec<Note>, String> {
    NoteRepository::active(&state.db)
}

/// HTML 托盘菜单点项后的统一入口:执行动作,再关掉菜单窗。
/// 必须 async:同步命令在主线程跑,而 new/open_aux 会 build 窗口,
/// 主线程 IPC handler 里 build 会死锁(同 reactivate 的坑)。
#[tauri::command]
pub async fn tray_menu_action(
    action: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    match action.as_str() {
        "new" => {
            new_note(&app, &state)?;
        }
        "showAll" => show_all_impl(&app, &state)?,
        "hideAll" => hide_all_impl(&app, &state)?,
        "completed" => window_manager::open_aux(&app, "completed").map_err(to_str)?,
        "settings" => window_manager::open_aux(&app, "settings").map_err(to_str)?,
        "quit" => app.exit(0),
        _ => {}
    }
    if let Some(w) = app.get_webview_window("traymenu") {
        let _ = w.close();
    }
    Ok(())
}

// --- 自动更新(见 ADR-0002;Rust 中心化,tray 菜单只调下面命令)---

/// 查更新,有则返回最新版本号。startup 后台调一次,缓存进 AppState.update_status。
pub(crate) async fn fetch_update_version(app: &AppHandle) -> Result<Option<String>, String> {
    match app
        .updater()
        .map_err(to_str)?
        .check()
        .await
        .map_err(to_str)?
    {
        Some(u) => Ok(Some(u.version)),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn get_update_status(state: State<AppState>) -> Option<String> {
    state.update_status.lock().unwrap().clone()
}

#[derive(serde::Serialize, Clone)]
struct UpdateProgress {
    downloaded: u64,
    total: Option<u64>,
}

/// 执行更新:再查一次 → 下载安装 → 重启。
/// - 防重复:`state.updating` 标志,已在更新中则直接返回(连点/菜单重开都不会再起一次)。
/// - 进度:下载期间 throttled(~150ms)发 `update-progress` 事件给前端显示百分比。
/// Windows: download_and_install 拉起 installer 后 exit(),request_restart 走不到;
/// macOS/Linux: 原地替换后返回 → request_restart 重启。
#[tauri::command]
pub async fn apply_update(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    {
        let mut g = state.updating.lock().unwrap();
        if *g {
            return Ok(());
        }
        *g = true;
    }
    *state.update_status.lock().unwrap() = None;

    let update = app
        .updater()
        .map_err(to_str)?
        .check()
        .await
        .map_err(to_str)?
        .ok_or("no update available")?;

    let app_cb = app.clone();
    let mut downloaded: u64 = 0;
    let mut last_emit: Option<Instant> = None;
    let res = update
        .download_and_install(
            move |chunk_len, content_len| {
                downloaded += chunk_len as u64;
                let now = Instant::now();
                let fire =
                    last_emit.map_or(true, |t| now.duration_since(t) > Duration::from_millis(150));
                if fire {
                    last_emit = Some(now);
                    let _ = app_cb.emit(
                        "update-progress",
                        UpdateProgress {
                            downloaded,
                            total: content_len,
                        },
                    );
                }
            },
            || {},
        )
        .await;
    match res {
        Ok(()) => {
            app.request_restart();
            Ok(())
        }
        Err(e) => {
            *state.updating.lock().unwrap() = false; // 失败:放开,允许重试
            Err(format!("{e:?}"))
        }
    }
}

/// 当前版本号(编译期取 Cargo.toml version)。
#[tauri::command]
pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// 手动检查更新:重查一次、刷新缓存、返回最新版本号(Some = 有新版本)。
#[tauri::command]
pub async fn check_for_updates(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let v = fetch_update_version(&app).await?;
    *state.update_status.lock().unwrap() = v.clone();
    Ok(v)
}

// --- show_all: impl + command wrapper ----------------------------------------
pub fn show_all_impl(app: &AppHandle, state: &AppState) -> Result<(), String> {
    for n in NoteRepository::active(&state.db)? {
        NoteRepository::clear_snooze(&state.db, &n.id)?;
        state.scheduler.cancel(&n.id);
        // 用 open_note 而非 show_note:隐藏便签可能根本没有窗口(启动加载会跳过
        // 未到期的隐藏便签开窗)。open_note 对无窗的会新建、对已存在隐藏窗的会重显。
        window_manager::open_note(app, &n).map_err(to_str)?;
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
        window_manager::hide_note(app, &n.id).map_err(to_str)?;
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
    let lock = db.lock().map_err(to_str)?;
    lock.query_row(
        "SELECT val FROM settings WHERE key=?1",
        rusqlite::params![key],
        |r| r.get::<_, String>(0),
    )
    .optional()
    .map_err(to_str)
}

/// Upsert a single setting value (INSERT … ON CONFLICT UPDATE).
pub(crate) fn set_setting(db: &Db, key: &str, val: &str) -> Result<(), String> {
    let lock = db.lock().map_err(to_str)?;
    lock.execute(
        "INSERT INTO settings(key,val) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET val=excluded.val",
        rusqlite::params![key, val],
    )
    .map_err(to_str)?;
    Ok(())
}

/// 当前语言("en"/"zh"):读 settings.language;缺省按 first_run_done 判定
/// (老用户→zh,新用户→en)。供欢迎便签 + aux 窗口标题用。
pub(crate) fn lang(db: &Db) -> String {
    if let Ok(Some(l)) = get_setting(db, "language") {
        if l == "en" || l == "zh" {
            return l;
        }
    }
    let first_run = matches!(get_setting(db, "first_run_done"), Ok(Some(ref v)) if v == "1");
    if first_run { "zh" } else { "en" }.to_string()
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
    let lock = state.db.lock().map_err(to_str)?;
    let mut stmt = lock
        .prepare("SELECT key, val FROM settings")
        .map_err(to_str)?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(to_str)?;
    let mut m = std::collections::HashMap::new();
    for r in rows {
        let row = r.map_err(to_str)?;
        m.insert(row.0, row.1);
    }
    Ok(m)
}

/// Clamp a note's top-left so the full `w × h` rect stays inside the monitor
/// that covers it the most (8px margin). Monitors come from any live webview
/// window — `available_monitors()` reports the whole system list, not just the
/// one the window sits on, so any window suffices. Falls back to the raw
/// coordinates when no window exists yet or the monitor list is unavailable.
pub(crate) fn clamp_note(app: &AppHandle, x: f64, y: f64, w: f64, h: f64) -> (f64, f64) {
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

    #[test]
    fn lang_uses_explicit_setting_when_valid() {
        let db = mem();
        set_setting(&db, "language", "en").unwrap();
        assert_eq!(lang(&db), "en");
        set_setting(&db, "language", "zh").unwrap();
        assert_eq!(lang(&db), "zh");
    }

    #[test]
    fn lang_defaults_to_zh_for_existing_users() {
        let db = mem();
        set_setting(&db, "first_run_done", "1").unwrap();
        // language 未设 → 老用户 zh
        assert_eq!(lang(&db), "zh");
    }

    #[test]
    fn lang_defaults_to_en_for_new_users() {
        let db = mem();
        // 无 language,无 first_run_done → 新用户 en
        assert_eq!(lang(&db), "en");
    }

    #[test]
    fn settings_roundtrip_and_upsert() {
        let db = mem();
        assert_eq!(get_setting(&db, "k").unwrap(), None);
        set_setting(&db, "k", "v1").unwrap();
        assert_eq!(get_setting(&db, "k").unwrap().as_deref(), Some("v1"));
        set_setting(&db, "k", "v2").unwrap(); // upsert 覆盖
        assert_eq!(get_setting(&db, "k").unwrap().as_deref(), Some("v2"));
    }
}
