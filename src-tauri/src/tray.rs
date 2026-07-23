use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::AppHandle;

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let new = MenuItemBuilder::new("新建便签").id("new").build(app)?;
    let show_all = MenuItemBuilder::new("显示全部").id("showAll").build(app)?;
    let hide_all = MenuItemBuilder::new("隐藏全部").id("hideAll").build(app)?;
    let completed = MenuItemBuilder::new("已完成…").id("completed").build(app)?;
    let settings = MenuItemBuilder::new("设置…").id("settings").build(app)?;
    let quit = MenuItemBuilder::new("退出").id("quit").build(app)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let menu = MenuBuilder::new(app)
        .item(&new).item(&show_all).item(&hide_all).item(&completed)
        .item(&sep).item(&settings).item(&quit).build()?;
    let icon = app.default_window_icon().cloned()
        .ok_or_else(|| tauri::Error::Anyhow(anyhow::anyhow!("no default icon")))?;
    TrayIconBuilder::with_id("main")
        .icon(icon)
        .menu(&menu)
        .tooltip("PinNotes")
        .build(app)?;
    Ok(())
}
