// PinNotes entry assembly: wires every module into the Tauri builder —
// SQLite init, shared AppState, tray icon (left=new note, right=HTML menu),
// Ctrl+N shortcut, default autostart, startup load (open active note windows,
// arm snooze for hidden-until-future notes), and the command surface.

mod autostart;
mod commands;
mod db;
mod geometry;
mod snooze;
mod state;
mod tray;
mod tray_menu;
mod window_manager;

use chrono::Utc;
use state::AppState;
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let path = app.path().app_data_dir()?.join("pinnotes.sqlite");
            std::fs::create_dir_all(path.parent().unwrap())?;
            let conn = rusqlite::Connection::open(&path)?;
            let db = db::init(conn)?;
            app.manage(AppState {
                db,
                scheduler: snooze::SnoozeScheduler::new(),
                drag_writes: std::sync::Mutex::new(std::collections::HashMap::new()),
                update_status: std::sync::Mutex::new(None),
                updating: std::sync::Mutex::new(false),
            });
            tray::build(app.handle())?;
            // 全局快捷键 Ctrl+N → 新建便签。运行时注册:若已被其他应用占用,
            // 仅打印告警、不致启动失败(构建期注册会把 OS 错误上抛,让整个 app 起不来)。
            if let Err(e) = app.global_shortcut().on_shortcut("ctrl+n", |ah, _s, ev| {
                if ev.state == ShortcutState::Pressed {
                    let state = ah.state::<AppState>();
                    let _ = commands::new_note(ah, &state);
                }
            }) {
                eprintln!("pinnotes: Ctrl+N not registered ({e:?}) — another app may own it");
            }
            let state = app.state::<AppState>();
            // 默认开机自启：仅首次运行启用，之后尊重用户在设置中的选择。
            if commands::get_setting(&state.db, "autostart_configured")?.is_none() {
                let _ = autostart::configure(app.handle(), true);
                commands::set_setting(&state.db, "autostart_configured", "1")?;
            }
            // 首次运行：创建一条欢迎便签并显示，避免启动后"什么都没有"。
            if let Some(note) = commands::maybe_welcome_note(&state.db)? {
                window_manager::open_note(app.handle(), &note)?;
            }
            // 启动加载：开活跃便签窗、为隐藏中且未到期的便签排程重弹。
            // should_repop == true 表示隐藏便签的 snooze 已到期（离开期间到期）→
            // 先清掉残留的 hidden 标志再 open_note 立即显示；否则尚未到期 → 排程到到点再弹。
            for n in db::NoteRepository::active(&state.db).map_err(|e| anyhow::anyhow!(e))? {
                if n.is_hidden {
                    if let Some(until_iso) = n.hidden_until.clone() {
                        let until = chrono::DateTime::parse_from_rfc3339(&until_iso)?
                            .with_timezone(&Utc);
                        let now = Utc::now();
                        if !snooze::should_repop(n.completed_at.is_some(), Some(until), now) {
                            let app2 = app.handle().clone();
                            let id = n.id.clone();
                            state.scheduler.schedule(
                                id.clone(),
                                (until - now).to_std()?,
                                move || {
                                    let _ = commands_show(app2, &id);
                                },
                            );
                            continue;
                        } else {
                            // 离开期间已到期：清掉残留 hidden 标志后再显示。
                            db::NoteRepository::clear_snooze(&state.db, &n.id)?;
                        }
                    }
                }
                window_manager::open_note(app.handle(), &n)?;
            }
            // 启动后台查一次自动更新(见 ADR-0002);有新版则缓存版本号,tray 菜单据此提示。
            let ah = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(Some(v)) = commands::fetch_update_version(&ah).await {
                    *ah.state::<AppState>().update_status.lock().unwrap() = Some(v);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_note,
            commands::create_note,
            commands::hide_note,
            commands::complete_note,
            commands::edit_note,
            commands::set_color,
            commands::set_size,
            commands::set_snooze,
            commands::move_note,
            commands::reactivate,
            commands::copy_note,
            commands::delete_note,
            commands::list_completed,
            commands::list_active,
            commands::tray_menu_action,
            commands::get_update_status,
            commands::apply_update,
            commands::get_version,
            commands::check_for_updates,
            commands::show_all,
            commands::hide_all,
            commands::get_settings,
            commands::set_settings,
            commands::get_autostart,
            commands::set_autostart,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Repop a snoozed note when its timer fires during startup load. Thin wrapper
/// over the shared `commands::repop_note` helper; takes an owned `AppHandle`
/// so it can move into the scheduler's `FnOnce` wake callback.
fn commands_show(app: tauri::AppHandle, id: &str) -> Result<(), String> {
    commands::repop_note(&app, id)
}
