// HTML 托盘菜单窗(替代原生系统菜单,见 ADR-0001)。右键托盘在鼠标处弹出。
//
// 不透明 + 方角:Win10 上圆角需透明窗,而透明区经 DWM 合成会呈毛玻璃(圆角与
// 干净无边框在 Win10 不可兼得);不透明还顺带避开"透明+置顶"的冻结风险。
//
// 关闭三路:1) 点菜单项 → `tray_menu_action` 命令末尾关;2) Esc → 前端关;
// 3) 失焦 → WindowEvent::Focused(false) → 关(覆盖"点外部")。
use crate::geometry::{clamp_into_work_area, Rect};
use tauri::{AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};

const W: f64 = 250.0;
const H: f64 = 320.0;

/// 计算菜单窗的逻辑坐标:以点击点为锚,向屏幕内(通常向左上,托盘在右下)展开。
/// 用**点击所在显示器**的 scale 把逻辑尺寸换算到物理、夹进屏内、再转回逻辑
/// (builder.position 用逻辑坐标)。修"从任意窗口读 scale 导致 HiDPI 漂移"。
fn position_at(app: &AppHandle, pos: PhysicalPosition<f64>, w: f64, h: f64) -> (f64, f64) {
    let Some(win) = app.webview_windows().into_values().next() else {
        return (pos.x, pos.y);
    };
    let mons = match win.available_monitors() {
        Ok(m) if !m.is_empty() => m,
        _ => return (pos.x, pos.y),
    };
    let mons_phys: Vec<Rect> = mons
        .iter()
        .map(|m| Rect {
            left: m.position().x as f64,
            top: m.position().y as f64,
            width: m.size().width as f64,
            height: m.size().height as f64,
        })
        .collect();
    // 点击所在显示器的 scale(找不到则退第一个)。
    let scale = mons
        .iter()
        .zip(mons_phys.iter())
        .find(|(_, r)| pos.x >= r.left && pos.y >= r.top && pos.x < r.right() && pos.y < r.bottom())
        .map(|(m, _)| m.scale_factor())
        .unwrap_or_else(|| mons[0].scale_factor());
    let (w_phys, h_phys) = (w * scale, h * scale);
    // 向左上展开:菜单右下角 ≈ 点击点(像系统托盘菜单)。
    let menu_phys = Rect {
        left: pos.x - w_phys,
        top: pos.y - h_phys,
        width: w_phys,
        height: h_phys,
    };
    let clamped = clamp_into_work_area(menu_phys, &mons_phys, 8.0);
    (clamped.left / scale, clamped.top / scale)
}

/// 右键托盘 → 在 `pos`(鼠标物理坐标)处开 HTML 菜单窗。单例:已开则先关。
pub fn open(app: &AppHandle, pos: PhysicalPosition<f64>) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window("traymenu") {
        let _ = w.close();
        return Ok(());
    }
    let (x, y) = position_at(app, pos, W, H);

    let win = WebviewWindowBuilder::new(
        app,
        "traymenu",
        WebviewUrl::App("index.html#/traymenu".into()),
    )
    .title("PinNotes")
    .inner_size(W, H)
    .position(x, y)
    .decorations(false)
    .always_on_top(true) // 盖在置顶便签之上;不透明 → 不触碰冻结风险。
    .skip_taskbar(true)
    .resizable(false)
    .focused(true) // 抢焦点,失焦路径才能捕获"点外部"。
    .build()?;
    // 显式再抢一次焦点:.focused(true) 在程序化开窗时不一定拿到焦点,
    // 导致"首次点外部"不触发失焦(必须先点一下菜单才生效)。set_focus 修这个。
    let _ = win.set_focus();
    // 失焦即关。点项 / Esc 另有路径关;这里兜"点外部 / 切窗口"。
    let app2 = app.clone();
    win.on_window_event(move |e| {
        if let tauri::WindowEvent::Focused(false) = e {
            if let Some(w) = app2.get_webview_window("traymenu") {
                let _ = w.close();
            }
        }
    });
    Ok(())
}
