use crate::db::{Note, NoteRepository};
use crate::state::AppState;
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
        .build()?;

    // Persist the note's logical position when the OS moves the window.
    // Dragging the grip fires WindowEvent::Moved repeatedly; the event carries
    // a PhysicalPosition, so divide by the scale factor to match the logical
    // x/y used everywhere else (open_note/move_note/clamp_note). Each
    // update_position locks and releases the Mutex within itself, so nothing
    // is held across an await (there is none here).
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
            let _ = NoteRepository::update_position(&state.db, &id, x, y);
        }
    });

    Ok(())
}

pub fn show_note(app: &AppHandle, id: &str) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(&label(id)) { w.show()?; w.set_focus()?; }
    Ok(())
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
