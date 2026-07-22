// PinNotes entry assembly (Task 10): wires every module into the Tauri
// builder — SQLite init, shared AppState, tray menu, default autostart,
// startup load (open active note windows, arm snooze for hidden-until-future
// notes), tray menu dispatch, and the full command surface for the frontend.

mod autostart;
mod commands;
mod db;
mod geometry;
mod snooze;
mod state;
mod tray;
mod window_manager;

use chrono::Utc;
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let path = app.path().app_data_dir()?.join("pinnotes.sqlite");
            std::fs::create_dir_all(path.parent().unwrap())?;
            let conn = rusqlite::Connection::open(&path)?;
            let db = db::init(conn)?;
            app.manage(AppState {
                db,
                scheduler: snooze::SnoozeScheduler::new(),
            });
            tray::build(app.handle())?;
            let state = app.state::<AppState>();
            // 默认开机自启：仅首次运行启用，之后尊重用户在设置中的选择。
            if commands::get_setting(&state.db, "autostart_configured")?.is_none() {
                let _ = autostart::configure(app.handle(), true);
                commands::set_setting(&state.db, "autostart_configured", "1")?;
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
            Ok(())
        })
        .on_menu_event(|app, e| {
            let id = e.id().as_ref();
            match id {
                "new" => {
                    let state = app.state::<AppState>();
                    let _ = commands::create_note_impl(app, &state);
                }
                "showAll" => {
                    let state = app.state::<AppState>();
                    let _ = commands::show_all_impl(app, &state);
                }
                "completed" => {
                    let _ = open_simple(app, "completed");
                }
                "settings" => {
                    let _ = open_simple(app, "settings");
                }
                "quit" => app.exit(0),
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_note,
            commands::create_note,
            commands::hide_note,
            commands::complete_note,
            commands::edit_note,
            commands::move_note,
            commands::reactivate,
            commands::copy_note,
            commands::delete_note,
            commands::list_completed,
            commands::show_all,
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

/// Open one of the auxiliary single-instance windows (completed / settings).
fn open_simple(app: &tauri::AppHandle, route: &str) -> tauri::Result<()> {
    let label = route;
    if app.get_webview_window(label).is_some() {
        return Ok(());
    }
    tauri::WebviewWindowBuilder::new(
        app,
        label,
        tauri::WebviewUrl::App(format!("index.html#/{route}").into()),
    )
    .title(if route == "completed" {
        "已完成"
    } else {
        "设置"
    })
    .inner_size(420.0, 520.0)
    .resizable(true)
    .build()?;
    Ok(())
}
