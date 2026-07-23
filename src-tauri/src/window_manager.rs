use crate::db::{Note, NoteRepository};
use crate::state::AppState;
use tauri::utils::config::WindowEffectsConfig;
use tauri::utils::{WindowEffect, WindowEffectState};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

fn label(id: &str) -> String { format!("note-{id}") }

pub fn open_note(app: &AppHandle, note: &Note) -> tauri::Result<()> {
    let l = label(&note.id);
    if let Some(w) = app.get_webview_window(&l) {
        w.show()?;
        w.set_focus()?;
        return Ok(());
    }
    let url = format!("index.html#/note?id={}", note.id);
    let window = WebviewWindowBuilder::new(app, &l, WebviewUrl::App(url.into()))
        .title("PinNote")
        .inner_size(note.w, note.h)
        .position(note.x, note.y)
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        // Real frosted glass: CSS backdrop-filter can't blur the desktop
        // through a transparent webview, so request an OS-level Acrylic
        // effect. Acrylic works on Windows 10 (Mica is Win11-only, which is
        // why we avoid it). The noteView's rgba tint still colors the glass.
        .effects(WindowEffectsConfig {
            effects: vec![WindowEffect::Acrylic],
            state: Some(WindowEffectState::Active),
            radius: None,
            color: None,
        })
        .build()?;

    // Persist the note's logical position when the OS moves the window — but
    // DEBOUNCED, and with the acrylic effect toggled for a smooth drag.
    //
    // Re-blurring the desktop behind an acrylic window every frame is what
    // lags the drag on Windows, so on the FIRST Moved of a drag we disable
    // acrylic (tracked in AppState.acrylic_off). ~250ms after the drag stops
    // (no more Moved events) we persist the final position AND re-enable
    // acrylic — the frosted look returns only while the note is still. Each
    // event carries a PhysicalPosition, so divide by the scale factor to match
    // the logical x/y used elsewhere.
    let app_handle = app.clone();
    let id = note.id.clone();
    let lbl = l.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Moved(pos) = event {
            let scale = app_handle
                .get_webview_window(&lbl)
                .and_then(|w| w.scale_factor().ok())
                .unwrap_or(1.0);
            let x = pos.x as f64 / scale;
            let y = pos.y as f64 / scale;
            let state = app_handle.state::<AppState>();
            // Disable acrylic once at drag start (guarded so we don't re-call
            // set_effects on every pixel).
            {
                let mut off = state.acrylic_off.lock().unwrap();
                if !off.contains(&id) {
                    if let Some(w) = app_handle.get_webview_window(&lbl) {
                        let _ = w.set_effects(None);
                    }
                    off.insert(id.clone());
                }
            }
            // Debounce: persist final position + restore acrylic after the drag
            // settles, on a background task (no main-thread SQLite writes).
            let mut drag = state.drag_writes.lock().unwrap();
            if let Some(prev) = drag.remove(&id) {
                prev.abort();
            }
            let app2 = app_handle.clone();
            let id2 = id.clone();
            let lbl2 = lbl.clone();
            let handle = tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                let st = app2.state::<AppState>();
                let _ = NoteRepository::update_position(&st.db, &id2, x, y);
                if let Some(w) = app2.get_webview_window(&lbl2) {
                    let _ = w.set_effects(Some(WindowEffectsConfig {
                        effects: vec![WindowEffect::Acrylic],
                        state: Some(WindowEffectState::Active),
                        radius: None,
                        color: None,
                    }));
                }
                st.acrylic_off.lock().unwrap().remove(&id2);
            });
            drag.insert(id.clone(), handle);
        }
    });

    Ok(())
}

pub fn show_note(app: &AppHandle, id: &str) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(&label(id)) { w.show()?; w.set_focus()?; }
    Ok(())
}

/// Show a note window WITHOUT stealing keyboard focus — used by the snooze
/// re-pop so a reminder coming back doesn't interrupt what the user is doing.
/// On Windows this uses `SW_SHOWNOACTIVATE`; the note still shows on top
/// (always_on_top) but the currently-active window keeps focus. `show_note`
/// (with set_focus) is still used for user-initiated actions like 显示全部.
pub fn show_note_no_focus(app: &AppHandle, id: &str) -> tauri::Result<()> {
    #[cfg(target_os = "windows")]
    {
        if let Some(w) = app.get_webview_window(&label(id)) {
            use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_SHOWNOACTIVATE};
            let hwnd = w.hwnd()?;
            unsafe {
                let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            }
        }
        return Ok(());
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Some(w) = app.get_webview_window(&label(id)) {
            w.show()?;
        }
        Ok(())
    }
}

pub fn hide_note(app: &AppHandle, id: &str) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(&label(id)) { w.hide()?; }
    Ok(())
}

pub fn close_note(app: &AppHandle, id: &str) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(&label(id)) { w.close()?; }
    Ok(())
}

pub fn move_note(app: &AppHandle, id: &str, x: f64, y: f64) -> tauri::Result<()> {
    use tauri::LogicalPosition;
    if let Some(w) = app.get_webview_window(&label(id)) {
        w.set_position(LogicalPosition::new(x, y))?;
    }
    Ok(())
}

pub fn resize_note(app: &AppHandle, id: &str, w: f64, h: f64) -> tauri::Result<()> {
    use tauri::LogicalSize;
    if let Some(win) = app.get_webview_window(&label(id)) {
        win.set_size(LogicalSize::new(w, h))?;
    }
    Ok(())
}
