use crate::db::Note;
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
    WebviewWindowBuilder::new(app, &l, WebviewUrl::App(url.into()))
        .title("PinNote")
        .inner_size(note.w, note.h)
        .position(note.x, note.y)
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .build()?;
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
