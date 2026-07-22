use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use tauri::async_runtime::{self, JoinHandle};

/// 到点判定：未完成、有 hidden_until、且该时刻已到 → 应重弹。
pub fn should_repop(is_completed: bool, hidden_until: Option<DateTime<Utc>>, now: DateTime<Utc>) -> bool {
    !is_completed && hidden_until.map(|t| t <= now).unwrap_or(false)
}

pub struct SnoozeScheduler {
    handles: Mutex<HashMap<String, JoinHandle<()>>>,
}
impl SnoozeScheduler {
    pub fn new() -> Self { Self { handles: Mutex::new(HashMap::new()) } }
    pub fn schedule<F>(&self, id: String, dur: Duration, on_wake: F)
    where F: FnOnce() + Send + 'static {
        self.cancel(&id);
        let h = async_runtime::spawn(async move {
            tokio::time::sleep(dur).await;
            on_wake();
        });
        self.handles.lock().unwrap().insert(id, h);
    }
    pub fn cancel(&self, id: &str) {
        if let Some(h) = self.handles.lock().unwrap().remove(id) { h.abort(); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    #[test]
    fn repops_when_due_and_not_completed() {
        let now = Utc.with_ymd_and_hms(2026, 7, 22, 10, 5, 0).unwrap();
        let until = Utc.with_ymd_and_hms(2026, 7, 22, 10, 4, 0).unwrap();
        assert!(should_repop(false, Some(until), now));
    }
    #[test]
    fn no_repop_when_completed() {
        let now = Utc.with_ymd_and_hms(2026, 7, 22, 10, 5, 0).unwrap();
        let until = Utc.with_ymd_and_hms(2026, 7, 22, 10, 4, 0).unwrap();
        assert!(!should_repop(true, Some(until), now));
    }
    #[test]
    fn no_repop_before_due() {
        let now = Utc.with_ymd_and_hms(2026, 7, 22, 10, 0, 0).unwrap();
        let until = Utc.with_ymd_and_hms(2026, 7, 22, 10, 4, 0).unwrap();
        assert!(!should_repop(false, Some(until), now));
    }
}
