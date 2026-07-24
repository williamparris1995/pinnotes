use tauri::tray::{MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::AppHandle;

/// 构建系统托盘图标。不挂原生菜单(见 ADR-0001 + `tray_menu`):
/// 左键或右键单击 → 开 HTML 弹出菜单。新建便签走菜单第一项,或全局 Ctrl+N。
pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| tauri::Error::Anyhow(anyhow::anyhow!("no default icon")))?;
    TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("PinNotes")
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            // 左键或右键单击释放 → 在鼠标处开 HTML 菜单。
            if let TrayIconEvent::Click {
                button_state: MouseButtonState::Up,
                position,
                ..
            } = event
            {
                let _ = crate::tray_menu::open(tray.app_handle(), position);
            }
        })
        .build(app)?;
    Ok(())
}
