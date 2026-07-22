#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect { pub left: f64, pub top: f64, pub width: f64, pub height: f64 }
impl Rect {
    pub fn right(&self) -> f64 { self.left + self.width }
    pub fn bottom(&self) -> f64 { self.top + self.height }
}

/// 把窗口夹进"覆盖它最多的显示器"；无重叠则用第一个。
pub fn clamp_into_work_area(window: Rect, monitors: &[Rect], margin: f64) -> Rect {
    if monitors.is_empty() { return window; }
    let target = best_monitor(window, monitors);
    let lower_x = target.left + margin;
    let upper_x = ((target.right() - window.width).max(target.left)) - margin;
    let lower_y = target.top + margin;
    let upper_y = ((target.bottom() - window.height).max(target.top)) - margin;
    let new_left = if upper_x > lower_x { window.left.clamp(lower_x, upper_x) } else { lower_x };
    let new_top = if upper_y > lower_y { window.top.clamp(lower_y, upper_y) } else { lower_y };
    let new_left = new_left.max(target.left + margin);
    let new_top = new_top.max(target.top + margin);
    Rect { left: new_left, top: new_top, width: window.width, height: window.height }
}

fn best_monitor(window: Rect, monitors: &[Rect]) -> Rect {
    let mut best = monitors[0];
    let mut best_area = -1.0_f64;
    for m in monitors {
        let w = (window.right().min(m.right()) - window.left.max(m.left)).max(0.0);
        let h = (window.bottom().min(m.bottom()) - window.top.max(m.top)).max(0.0);
        let area = w * h;
        if area > best_area { best_area = area; best = *m; }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    fn mon() -> Rect { Rect { left: 0.0, top: 0.0, width: 1920.0, height: 1080.0 } }
    #[test]
    fn in_bounds_unchanged() {
        let w = Rect { left: 100.0, top: 100.0, width: 240.0, height: 170.0 };
        let c = clamp_into_work_area(w, &[mon()], 8.0);
        assert_eq!((c.left, c.top), (100.0, 100.0));
    }
    #[test]
    fn off_left_pulled_in() {
        let w = Rect { left: -300.0, top: 100.0, width: 240.0, height: 170.0 };
        assert_eq!(clamp_into_work_area(w, &[mon()], 8.0).left, 8.0);
    }
    #[test]
    fn off_right_pulled_in() {
        let w = Rect { left: 1900.0, top: 100.0, width: 240.0, height: 170.0 };
        let c = clamp_into_work_area(w, &[mon()], 8.0);
        assert!(c.right() <= 1920.0 - 8.0);
    }
    #[test]
    fn picks_most_overlap() {
        let m1 = mon();
        let m2 = Rect { left: 1920.0, top: 0.0, width: 1920.0, height: 1080.0 };
        let w = Rect { left: 1880.0, top: 100.0, width: 240.0, height: 170.0 };
        assert!(clamp_into_work_area(w, &[m1, m2], 8.0).left >= 1920.0);
    }
}
