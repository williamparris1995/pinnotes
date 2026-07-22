# PinNotes 实现计划（Tauri + Svelte）

> **For agentic workers:** REQUIRED SUB-SKILL: 用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 来逐任务实现本计划。步骤用 `- [ ]` 复选框语法跟踪。

**Goal:** 构建一个常驻托盘的 Windows 桌面应用，把重要提醒做成可拖动的置顶半透明便签；点击"隐藏"只是短暂收起，到点未完成会弹回原位，标记"完成"才进入"已完成"列表。

**Architecture:** Tauri v2 单进程。Rust 后端为唯一数据真相源与管理者（rusqlite + tokio snooze + 窗口生命周期 + 托盘 + 自启）；Svelte 5 前端窗口是"薄视图"——`invoke` 取数/回调、`listen` 事件刷新。每条便签是一个原生置顶/透明/无边框 `WebviewWindow`（方案 A，官方稳定）。

**Tech Stack:** Tauri v2（Rust）· Svelte 5 + Vite · rusqlite（bundled）· tokio（time）· serde/uuid/chrono · tauri-plugin-autostart · Vitest + @testing-library/svelte。

参考：设计文档 `docs/superpowers/specs/2026-07-22-pinned-sticky-notes-design.md`；界面原型 Open Design 项目 `pinnotes-8c76`（HTML/CSS 可直接移植到 Svelte 组件）。

## Global Constraints

- 平台：Windows 优先；Tauri 跨平台，预留 macOS/Linux。
- Rust 测试：`cargo test`（在 `src-tauri/` 下）；前端测试：`npm test`（Vitest）。
- rusqlite 用 `bundled` feature（自带 sqlite，零系统依赖）。
- 异步用 Tauri 内置 `tauri::async_runtime`（底层 tokio），不自建 runtime；`tokio` 加 `time` feature 供 `sleep`。
- Tauri v2 权限：所有用到的插件/窗口/事件能力必须在 `src-tauri/capabilities/default.json` 声明。
- 便签内容仅纯文本；4 色 `yellow|pink|blue|green`。
- 默认隐藏时长 2 分钟，可选 1/2/5/10/30；开机自启默认开。
- 完成语义为软删除（置 `completed_at`），不做硬删。
- 每个任务结束提交一次（`feat:`/`test:`/`chore:` 前缀）。
- 便签窗口属性：`transparent + decorations(false) + always_on_top + skip_taskbar + resizable(false)`。

---

## File Structure

```
src-tauri/
  Cargo.toml
  tauri.conf.json
  capabilities/default.json     # v2 权限声明
  src/
    main.rs                     # 入口（调用 lib::run）
    lib.rs                      # 装配：插件/状态/托盘/命令/启动加载
    state.rs                    # AppState（Db + SnoozeScheduler + settings 缓存）
    db.rs                       # rusqlite 连接 + 迁移 + NoteRepository
    snooze.rs                   # SnoozeScheduler（tokio）+ should_repop 纯函数
    geometry.rs                 # clamp_into_work_area（纯函数）
    window_manager.rs           # WebviewWindow 创建/显示/隐藏/关闭/移动
    tray.rs                     # TrayIconBuilder + 菜单
    autostart.rs                # tauri-plugin-autostart 封装
    commands.rs                 # #[tauri::command] 全部命令
src/                            # Svelte 前端
  main.ts                       # mount(App)
  App.svelte                    # hash 路由分发
  lib/
    tauri.ts                    # invoke/listen 薄封装 + 类型
    theme.css                   # 令牌（移植自 OD 原型）
    noteView.svelte
    completedView.svelte
    settingsView.svelte
src-tauri/tests/ 或 #[cfg(test)] # Rust 单测
src/**/*.test.ts                # Vitest 组件测试
```

边界原则：`db`/`snooze`/`geometry` 为纯 Rust 逻辑，全部单测；`window_manager`/`tray`/`autostart` 封装 Tauri 平台能力，对外暴露稳定函数；前端组件只通过 `lib/tauri.ts` 与后端通信，测试时 mock。

---

## Phase 0 — 脚手架

### Task 1: 脚手架 Tauri v2 + Svelte 5 项目、依赖、能力声明、冒烟

**Files:** 整个 `src-tauri/` 与 `src/`（由 `create-tauri-app` 生成后改）

**Interfaces:** 产出一个能 `npm run tauri dev` 启动、打开一个透明置顶小窗的应用骨架。

- [ ] **Step 1: 生成项目**

在仓库根执行（选 Svelte + TypeScript + Vite 模板；包管理器用 npm）：

```bash
npm create tauri-app@latest .
```

交互选择：`Project name: pinnotes`、`Identifier: com.buhiyo.pinnotes`、`Frontend language: TypeScript`、`UI template: Svelte`、`UI flavor: TypeScript`、`Package manager: npm`。

> 若当前目录非空（已有 `docs/` 等），`create-tauri-app` 可能拒绝；可先在一个临时目录生成再合并到根，或加 `--` 适配。生成后确认根有 `src/`、`src-tauri/`、`package.json`、`vite.config.ts`。

- [ ] **Step 2: 添加 Rust 依赖**

编辑 `src-tauri/Cargo.toml` 的 `[dependencies]`：

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-autostart = "2"
rusqlite = { version = "0.32", features = ["bundled"] }
tokio = { version = "1", features = ["time", "sync", "rt-multi-thread"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 3: 添加前端依赖**

```bash
npm i @tauri-apps/plugin-autostart
npm i -D vitest @testing-library/svelte jsdom @testing-library/jest-dom
```

- [ ] **Step 4: 配置主窗口隐藏 + 一个验证用透明置顶窗**

编辑 `src-tauri/tauri.conf.json`：把默认窗口 `"visible": true` 改为 `false`（主窗口仅作进程入口，不显示）。先不改其他。

在 `src-tauri/src/lib.rs` 的 `setup` 里临时加一段，创建一个透明置顶小窗验证（后续任务覆盖）：

```rust
use tauri::{WebviewUrl, WebviewWindowBuilder};
// 在 setup(|app| { ... }) 内：
WebviewWindowBuilder::new(app, "smoke", WebviewUrl::App("index.html".into()))
    .title("PinNotes smoke")
    .inner_size(240.0, 170.0)
    .transparent(true)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .build()?;
```

- [ ] **Step 5: 声明能力（v2 权限）**

编辑 `src-tauri/capabilities/default.json`，确保 `permissions` 含窗口与事件能力：

```json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:default",
    "core:window:allow-create",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-close",
    "core:window:allow-set-position",
    "core:window:allow-set-size",
    "core:event:default",
    "autostart:default"
  ]
}
```

> 动态创建的便签窗口标签为 `note-<id>`；v2 能力按 `windows` 名匹配——需把这些标签也加入，或改用通配 `"windows": ["*"]`。本计划统一用 `"windows": ["*"]` 让所有窗口共享 default 能力，简化配置。

- [ ] **Step 6: 冒烟运行**

```bash
npm run tauri dev
```

预期：弹出一个小、无标题栏、置顶、半透明（前端默认背景）的窗口，关闭即退出开发态。确认透明/置顶/无边框生效（这是 Tauri 平台可行性的实测，替代原 Flutter 的 spike）。

- [ ] **Step 7: 提交**

```bash
git add -A
git commit -m "chore: scaffold tauri v2 + svelte 5 project"
```

---

## Phase 1 — Rust 数据与领域（纯逻辑，重点单测）

### Task 2: rusqlite 连接 + 迁移 + NoteRepository

**Files:**
- Create: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/lib.rs`（`mod db;`）

**Interfaces:** Produces `db::Db = Mutex<Connection>`、`db::init(path)`、`db::Note`、`db::NoteRepository::{active, completed, get, create, update_position, update_content, snooze, clear_snooze, complete, reactivate, copy, delete}`，均返回 `Result<_, String>`。

- [ ] **Step 1: 写 db.rs（含行内单测）**

`src-tauri/src/db.rs`：

```rust
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

pub type Db = Mutex<Connection>;

pub fn init(conn: Connection) -> rusqlite::Result<Db> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY, content TEXT NOT NULL,
            color TEXT NOT NULL DEFAULT 'yellow',
            x REAL NOT NULL DEFAULT 120, y REAL NOT NULL DEFAULT 40,
            w REAL NOT NULL DEFAULT 240, h REAL NOT NULL DEFAULT 170,
            snooze_minutes INTEGER NOT NULL DEFAULT 2,
            created_at TEXT NOT NULL,
            completed_at TEXT, is_hidden INTEGER NOT NULL DEFAULT 0, hidden_until TEXT
        );
        CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, val TEXT NOT NULL);",
    )?;
    Ok(Mutex::new(conn))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    pub id: String,
    pub content: String,
    pub color: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub snooze_minutes: i64,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub is_hidden: bool,
    pub hidden_until: Option<String>,
}

fn row_to_note(row: &rusqlite::Row) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get(0)?,
        content: row.get(1)?,
        color: row.get(2)?,
        x: row.get(3)?,
        y: row.get(4)?,
        w: row.get(5)?,
        h: row.get(6)?,
        snooze_minutes: row.get(7)?,
        created_at: row.get(8)?,
        completed_at: row.get(9)?,
        is_hidden: row.get::<_, i64>(10)? != 0,
        hidden_until: row.get(11)?,
    })
}

pub struct NoteRepository;

impl NoteRepository {
    pub fn active(db: &Db) -> Result<Vec<Note>, String> {
        let lock = db.lock().map_err(|e| e.to_string())?;
        let mut stmt = lock
            .prepare("SELECT * FROM notes WHERE completed_at IS NULL ORDER BY created_at")
            .map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], row_to_note).map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    pub fn completed(db: &Db) -> Result<Vec<Note>, String> {
        let lock = db.lock().map_err(|e| e.to_string())?;
        let mut stmt = lock
            .prepare("SELECT * FROM notes WHERE completed_at IS NOT NULL ORDER BY completed_at DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], row_to_note).map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    pub fn get(db: &Db, id: &str) -> Result<Option<Note>, String> {
        let lock = db.lock().map_err(|e| e.to_string())?;
        lock.query_row("SELECT * FROM notes WHERE id = ?1", params![id], row_to_note)
            .optional()
            .map_err(|e| e.to_string())
    }

    pub fn create(db: &Db, n: &Note) -> Result<(), String> {
        let lock = db.lock().map_err(|e| e.to_string())?;
        lock.execute(
            "INSERT INTO notes (id, content, color, x, y, w, h, snooze_minutes, created_at, completed_at, is_hidden, hidden_until)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            params![n.id, n.content, n.color, n.x, n.y, n.w, n.h, n.snooze_minutes,
                    n.created_at, n.completed_at, n.is_hidden as i64, n.hidden_until],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_position(db: &Db, id: &str, x: f64, y: f64) -> Result<(), String> {
        let lock = db.lock().map_err(|e| e.to_string())?;
        lock.execute("UPDATE notes SET x=?1, y=?2 WHERE id=?3", params![x, y, id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_content(db: &Db, id: &str, content: &str) -> Result<(), String> {
        let lock = db.lock().map_err(|e| e.to_string())?;
        lock.execute("UPDATE notes SET content=?1 WHERE id=?2", params![content, id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn snooze(db: &Db, id: &str, until_iso: &str) -> Result<(), String> {
        let lock = db.lock().map_err(|e| e.to_string())?;
        lock.execute("UPDATE notes SET is_hidden=1, hidden_until=?1 WHERE id=?2", params![until_iso, id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn clear_snooze(db: &Db, id: &str) -> Result<(), String> {
        let lock = db.lock().map_err(|e| e.to_string())?;
        lock.execute("UPDATE notes SET is_hidden=0, hidden_until=NULL WHERE id=?1", params![id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn complete(db: &Db, id: &str, at_iso: &str) -> Result<(), String> {
        let lock = db.lock().map_err(|e| e.to_string())?;
        lock.execute("UPDATE notes SET completed_at=?1, is_hidden=0, hidden_until=NULL WHERE id=?2", params![at_iso, id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn reactivate(db: &Db, id: &str) -> Result<(), String> {
        let lock = db.lock().map_err(|e| e.to_string())?;
        lock.execute("UPDATE notes SET completed_at=NULL, is_hidden=0, hidden_until=NULL WHERE id=?1", params![id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete(db: &Db, id: &str) -> Result<(), String> {
        let lock = db.lock().map_err(|e| e.to_string())?;
        lock.execute("DELETE FROM notes WHERE id=?1", params![id]).map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn mem() -> Db { init(Connection::open_in_memory().unwrap()).unwrap() }
    fn sample(id: &str) -> Note {
        Note { id: id.into(), content: "c".into(), color: "yellow".into(), x: 0.0, y: 0.0,
               w: 240.0, h: 170.0, snooze_minutes: 2, created_at: "2026-07-22T10:00:00Z".into(),
               completed_at: None, is_hidden: false, hidden_until: None }
    }

    #[test]
    fn active_completed_partition() {
        let db = mem();
        NoteRepository::create(&db, &sample("a")).unwrap();
        NoteRepository::complete(&db, "a", "2026-07-22T11:00:00Z").unwrap();
        NoteRepository::create(&db, &sample("b")).unwrap();
        assert_eq!(NoteRepository::active(&db).unwrap().len(), 1);
        assert_eq!(NoteRepository::completed(&db).unwrap().len(), 1);
    }

    #[test]
    fn snooze_sets_then_clears() {
        let db = mem();
        NoteRepository::create(&db, &sample("a")).unwrap();
        NoteRepository::snooze(&db, "a", "2026-07-22T10:05:00Z").unwrap();
        assert!(NoteRepository::get(&db, "a").unwrap().unwrap().is_hidden);
        NoteRepository::clear_snooze(&db, "a").unwrap();
        assert!(!NoteRepository::get(&db, "a").unwrap().unwrap().is_hidden);
    }

    #[test]
    fn reactivate_brings_back() {
        let db = mem();
        NoteRepository::create(&db, &sample("a")).unwrap();
        NoteRepository::complete(&db, "a", "2026-07-22T11:00:00Z").unwrap();
        assert!(NoteRepository::active(&db).unwrap().is_empty());
        NoteRepository::reactivate(&db, "a").unwrap();
        assert_eq!(NoteRepository::active(&db).unwrap().len(), 1);
    }
}
```

- [ ] **Step 2: 在 lib.rs 声明模块**

`src-tauri/src/lib.rs` 顶部加 `mod db;`（若 setup 内引用会报未用，先保留）。

- [ ] **Step 3: 运行测试**

```bash
cd src-tauri && cargo test db::tests
```

预期：3 passed。

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/db.rs src-tauri/src/lib.rs
git commit -m "feat: rusqlite db + NoteRepository with tests"
```

---

### Task 3: 领域逻辑——重弹判定 + 坐标夹回 + SnoozeScheduler

**Files:**
- Create: `src-tauri/src/geometry.rs`
- Create: `src-tauri/src/snooze.rs`

**Interfaces:**
- Produces `geometry::{Rect, clamp_into_work_area(window, monitors, margin)}`；`snooze::should_repop(is_completed, hidden_until: Option<DateTime>, now: DateTime) -> bool`；`snooze::SnoozeScheduler::{new, schedule, cancel}`。

- [ ] **Step 1: 写 geometry.rs**

`src-tauri/src/geometry.rs`：

```rust
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
```

- [ ] **Step 2: 写 snooze.rs（纯判定 + tokio 调度器）**

`src-tauri/src/snooze.rs`：

```rust
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
```

- [ ] **Step 3: 运行测试**

```bash
cd src-tauri && cargo test geometry::tests && cargo test snooze::tests
```

预期：4 + 3 passed。

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/geometry.rs src-tauri/src/snooze.rs src-tauri/src/lib.rs
git commit -m "feat: geometry clamp + snooze decision + scheduler"
```

---

## Phase 2 — 窗口/托盘/自启（平台封装，手动验收）

### Task 4: WindowManager

**Files:**
- Create: `src-tauri/src/window_manager.rs`

**Interfaces:** Produces `window_manager::{open_note(app, &Note), show_note(app, id), hide_note(app, id), close_note(app, id), move_note(app, id, x, y)}`，均返回 `tauri::Result<()>`。无单测（平台），手动验收。

- [ ] **Step 1: 写 window_manager.rs**

`src-tauri/src/window_manager.rs`：

```rust
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
```

> 说明：`position/inner_size` 接受逻辑像素（f64）。坐标夹回在调用方（`commands.rs`）用 `geometry::clamp_into_work_area` 处理后传入。透明窗口在 Windows 上需确认渲染正常（Task 1 已验证）。

- [ ] **Step 2: 手动验收**

在 `commands.rs`/`lib.rs` 接线后（Task 10），`npm run tauri dev`，托盘"新建"弹出置顶透明便签，可显示/隐藏/移动/关闭。

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/window_manager.rs src-tauri/src/lib.rs
git commit -m "feat: window manager for note windows"
```

---

### Task 5: 托盘 + 开机自启

**Files:**
- Create: `src-tauri/src/tray.rs`
- Create: `src-tauri/src/autostart.rs`

**Interfaces:** Produces `tray::build(app)`、`autostart::configure(app, enabled)`、`autostart::is_enabled(app)`。

- [ ] **Step 1: 写 tray.rs**

`src-tauri/src/tray.rs`：

```rust
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let new = MenuItemBuilder::new("新建便签").id("new").build(app)?;
    let show_all = MenuItemBuilder::new("显示全部").id("showAll").build(app)?;
    let completed = MenuItemBuilder::new("已完成…").id("completed").build(app)?;
    let settings = MenuItemBuilder::new("设置…").id("settings").build(app)?;
    let quit = MenuItemBuilder::new("退出").id("quit").build(app)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let menu = MenuBuilder::new(app)
        .item(&new).item(&show_all).item(&completed)
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
```

> 需在 `Cargo.toml` 加 `anyhow = "1"`（或改用 `tauri::Error` 变体）。菜单点击在 `lib.rs` 的 `on_menu_event` 里按 id 分发（Task 10）。

- [ ] **Step 2: 写 autostart.rs**

`src-tauri/src/autostart.rs`：

```rust
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

pub fn configure(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let mgr = app.autolaunch();
    if enabled { mgr.enable().map_err(|e| e.to_string()) }
    else { mgr.disable().map_err(|e| e.to_string()) }
}

pub fn is_enabled(app: &AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}
```

- [ ] **Step 3: 手动验收**

接线后 `npm run tauri dev`：托盘出现图标，右键 5 项 + 分隔线；自启在打包后才完全生效（开发期跳过）。

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/tray.rs src-tauri/src/autostart.rs src-tauri/Cargo.toml src-tauri/src/lib.rs
git commit -m "feat: tray menu + autostart wrapper"
```

---

## Phase 3 — 前端（Svelte 5）

### Task 6: 前端壳、路由、tauri 封装、主题

**Files:**
- Create: `src/lib/tauri.ts`
- Create: `src/lib/theme.css`
- Modify: `src/main.ts`、`src/App.svelte`

**Interfaces:** Produces `lib/tauri.ts`（re-export `invoke`/`listen`）、`App.svelte`（hash 路由 → note/completed/settings）、主题 CSS 令牌。

- [ ] **Step 1: tauri 封装 + 类型**

`src/lib/tauri.ts`：

```ts
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
export { invoke, listen };

export interface Note {
  id: string; content: string; color: string;
  x: number; y: number; w: number; h: number;
  snooze_minutes: number; created_at: string;
  completed_at: string | null; is_hidden: boolean; hidden_until: string | null;
}
```

- [ ] **Step 2: 主题（移植自 OD 原型）**

`src/lib/theme.css`：

```css
:root {
  --accent: #4a6fa5;
  --radius: 12px;
  --font: "Segoe UI", system-ui, sans-serif;
  --c-yellow: #fff59d; --c-pink: #ffcdd2; --c-blue: #bbdefb; --c-green: #c8e6c9;
}
html, body, #app { margin: 0; height: 100%; font-family: var(--font); background: transparent; }
```

并在 `src/main.ts` 顶部 `import './lib/theme.css';`。

- [ ] **Step 3: App.svelte 路由**

`src/App.svelte`：

```svelte
<script lang="ts">
  import NoteView from './lib/noteView.svelte';
  import CompletedView from './lib/completedView.svelte';
  import SettingsView from './lib/settingsView.svelte';

  let hash = $state(window.location.hash);
  window.addEventListener('hashchange', () => (hash = window.location.hash));

  const route = $derived(parse(hash));
  function parse(h: string): { name: string; id?: string } {
    const m = h.match(/#\/(note|completed|settings)\??(.*)/);
    if (!m) return { name: 'blank' };
    const name = m[1];
    const id = new URLSearchParams(m[2]).get('id') ?? undefined;
    return { name, id };
  }
</script>

{#if route.name === 'note' && route.id}
  <NoteView id={route.id} />
{:else if route.name === 'completed'}
  <CompletedView />
{:else if route.name === 'settings'}
  <SettingsView />
{:else}
  <div />
{/if}
```

- [ ] **Step 4: 提交**

```bash
git add src/lib/tauri.ts src/lib/theme.css src/main.ts src/App.svelte
git commit -m "feat: frontend shell + hash routing + theme"
```

---

### Task 7: NoteView 组件

**Files:**
- Create: `src/lib/noteView.svelte`
- Create: `src/lib/noteView.test.ts`

**Interfaces:** Props `{ id: string }`；加载 `invoke('get_note',{id})`；隐藏→`invoke('hide_note',{id})`；完成→`invoke('complete_note',{id})`；编辑→`invoke('edit_note',{id,content})`。

- [ ] **Step 1: 写组件**

`src/lib/noteView.svelte`：

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke, type Note } from './tauri';

  let { id }: { id: string } = $props();
  let note = $state<Note | null>(null);
  let editing = $state(false);
  let draft = $state('');

  onMount(async () => (note = await invoke<Note>('get_note', { id })));

  function startEdit() { if (note) { draft = note.content; editing = true; } }
  async function commit() {
    if (note && draft !== note.content) await invoke('edit_note', { id, content: draft });
    if (note) note.content = draft;
    editing = false;
  }
  const colorVar = $derived(note ? `var(--c-${note.color})` : 'transparent');
</script>

{#if note}
  <div class="note" style="background:{colorVar}">
    <div class="grip"></div>
    {#if editing}
      <textarea bind:value={draft} onfocusout={commit}></textarea>
    {:else}
      <p ondblclick={startEdit}>{note.content || '（空）'}</p>
    {/if}
    <div class="actions">
      <button onclick={() => invoke('hide_note', { id })}>隐藏</button>
      <button onclick={() => invoke('complete_note', { id })}>✓ 完成</button>
    </div>
  </div>
{/if}

<style>
  .note { width: 240px; padding: 10px; border-radius: var(--radius);
          box-shadow: 0 6px 16px rgba(0,0,0,.2); }
  .grip { width: 40px; height: 4px; margin: 0 auto 8px; background: rgba(0,0,0,.25); border-radius: 2px; }
  textarea { width: 100%; min-height: 60px; }
  .actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 8px; }
</style>
```

> 原生窗口拖动：Tauri 用 `data-tauri-drag-region` 属性让元素可拖窗。给 `.grip` 加 `data-tauri-drag-region` 即可整窗拖动；拖动结束上报坐标需监听 Tauri 窗口事件（在 Task 10 的 Rust 侧用 `window.on_window_event` 捕获 `Move` 更新坐标，或前端 `getCurrent().onMoved`）。v1 可先用 `data-tauri-drag-region`，落库时机在 Task 10 收尾。

- [ ] **Step 2: 写测试（mock invoke）**

`src/lib/noteView.test.ts`：

```ts
import { render, screen, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import NoteView from './noteView.svelte';

vi.mock('./tauri', () => ({
  invoke: vi.fn(),
  listen: vi.fn(),
}));

describe('NoteView', () => {
  beforeEach(() => vi.clearAllMocks());
  it('renders note and fires hide/complete', async () => {
    const { invoke } = await import('./tauri');
    (invoke as any).mockResolvedValue({
      id: 'n1', content: '提交季度报告', color: 'yellow',
      x: 0, y: 0, w: 240, h: 170, snooze_minutes: 2, created_at: '',
      completed_at: null, is_hidden: false, hidden_until: null,
    });
    render(NoteView, { props: { id: 'n1' } });
    expect(await screen.findByText('提交季度报告')).toBeTruthy();
    await fireEvent.click(screen.getByText('隐藏'));
    await fireEvent.click(screen.getByText('✓ 完成'));
    expect((invoke as any).mock.calls.map((c: any) => c[0])).toContain('hide_note');
    expect((invoke as any).mock.calls.map((c: any) => c[0])).toContain('complete_note');
  });
});
```

> 配置 `vite.config.ts` 加 `test: { environment: 'jsdom', globals: true }` 并 `import '@testing-library/jest-dom'`（可选）。

- [ ] **Step 3: 运行测试**

```bash
npm test -- noteView
```

预期：1 passed。

- [ ] **Step 4: 提交**

```bash
git add src/lib/noteView.svelte src/lib/noteView.test.ts vite.config.ts
git commit -m "feat: NoteView sticky component"
```

---

### Task 8: CompletedView 组件

**Files:**
- Create: `src/lib/completedView.svelte`
- Create: `src/lib/completedView.test.ts`

**Interfaces:** 加载 `invoke('list_completed')`；行内操作 `reactivate`/`copy_note`/`edit_note`/`delete_note`。

- [ ] **Step 1: 写组件**

`src/lib/completedView.svelte`：

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke, type Note } from './tauri';
  let items = $state<Note[]>([]);
  onMount(async () => (items = await invoke<Note[]>('list_completed')));
</script>

<main>
  <h2>已完成</h2>
  {#if items.length === 0}
    <p>暂无已完成项</p>
  {:else}
    <ul>
      {#each items as it}
        <li>
          <span class="dot" style="background:var(--c-{it.color})"></span>
          <span class="txt">{it.content}</span>
          <button onclick={() => invoke('reactivate', { id: it.id })}>重新激活</button>
          <button onclick={() => invoke('copy_note', { id: it.id })}>复制</button>
          <button onclick={() => invoke('edit_note', { id: it.id, content: prompt('编辑', it.content) ?? it.content })}>编辑</button>
          <button onclick={() => invoke('delete_note', { id: it.id })}>删除</button>
        </li>
      {/each}
    </ul>
  {/if}
</main>

<style>
  main { font-family: var(--font); padding: 16px; }
  li { list-style: none; display: flex; align-items: center; gap: 8px; padding: 6px 0; }
  .dot { width: 12px; height: 12px; border-radius: 50%; display: inline-block; }
  .txt { flex: 1; }
</style>
```

> `prompt` 仅作最小可用编辑；若需更佳体验，后续可改成内联输入框（非 v1 必须）。

- [ ] **Step 2: 写测试**

`src/lib/completedView.test.ts`：

```ts
import { render, screen, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import CompletedView from './completedView.svelte';

vi.mock('./tauri', () => ({ invoke: vi.fn(), listen: vi.fn() }));

describe('CompletedView', () => {
  beforeEach(() => vi.clearAllMocks());
  it('shows empty hint', async () => {
    const { invoke } = await import('./tauri');
    (invoke as any).mockResolvedValue([]);
    render(CompletedView);
    expect(await screen.findByText('暂无已完成项')).toBeTruthy();
  });
  it('renders rows and fires reactivate', async () => {
    const { invoke } = await import('./tauri');
    (invoke as any).mockResolvedValue([
      { id: 'a', content: '旧任务', color: 'pink', x: 0, y: 0, w: 0, h: 0, snooze_minutes: 2, created_at: '', completed_at: 'x', is_hidden: false, hidden_until: null },
    ]);
    render(CompletedView);
    await screen.findByText('旧任务');
    await fireEvent.click(screen.getByText('重新激活'));
    expect((invoke as any).mock.calls.some((c: any) => c[0] === 'reactivate')).toBe(true);
  });
});
```

- [ ] **Step 3: 运行测试**：`npm test -- completedView` → 预期 2 passed。

- [ ] **Step 4: 提交**

```bash
git add src/lib/completedView.svelte src/lib/completedView.test.ts
git commit -m "feat: CompletedView list with row actions"
```

---

### Task 9: SettingsView 组件

**Files:**
- Create: `src/lib/settingsView.svelte`
- Create: `src/lib/settingsView.test.ts`

**Interfaces:** 加载 `invoke('get_settings')`；改 snooze → `invoke('set_settings',{key:'default_snooze_minutes',value})`；改自启 → `invoke('set_autostart',{enabled})`。

- [ ] **Step 1: 写组件**

`src/lib/settingsView.svelte`：

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from './tauri';
  const opts = [1, 2, 5, 10, 30];
  let snooze = $state(2);
  let auto = $state(true);
  onMount(async () => {
    const s = await invoke<Record<string, string>>('get_settings');
    snooze = Number(s.default_snooze_minutes ?? 2);
    auto = await invoke<boolean>('get_autostart');
  });
  function setSnooze(m: number) { snooze = m; invoke('set_settings', { key: 'default_snooze_minutes', value: String(m) }); }
  function setAuto(v: boolean) { auto = v; invoke('set_autostart', { enabled: v }); }
</script>

<main>
  <h2>设置</h2>
  <label>默认隐藏时长</label>
  <div>
    {#each opts as m}
      <button class:active={snooze === m} onclick={() => setSnooze(m)}>{m} 分钟</button>
    {/each}
  </div>
  <label><input type="checkbox" checked={auto} onchange={(e) => setAuto(e.currentTarget.checked)} /> 开机自启</label>
</main>

<style>
  main { font-family: var(--font); padding: 16px; }
  .active { background: var(--accent); color: #fff; }
</style>
```

- [ ] **Step 2: 写测试**（mock 返回 `{default_snooze_minutes:'2'}` 与 `true`，点 5 分钟断言 `set_settings` 调用，切自启断言 `set_autostart`）。参考 Task 7/8 的 mock 模式。

- [ ] **Step 3: 运行测试**：`npm test -- settingsView` → 预期 passed。

- [ ] **Step 4: 提交**

```bash
git add src/lib/settingsView.svelte src/lib/settingsView.test.ts
git commit -m "feat: SettingsView (snooze default + autostart)"
```

---

## Phase 4 — 命令装配与集成

### Task 10: commands.rs + lib.rs 装配 + 启动加载 + 默认自启 + 托盘分发

**Files:**
- Create: `src-tauri/src/commands.rs`、`src-tauri/src/state.rs`
- Modify: `src-tauri/src/lib.rs`、`src-tauri/tauri.conf.json`（DB 路径）、`src-tauri/Cargo.toml`（anyhow）

**Interfaces:** 注册全部 `#[tauri::command]`；`AppState { db, scheduler }`；启动加载活跃便签开窗并排程；默认开机自启；托盘菜单 → 命令。

- [ ] **Step 1: 写 state.rs**

`src-tauri/src/state.rs`：

```rust
use crate::{db::Db, snooze::SnoozeScheduler};
pub struct AppState {
    pub db: Db,
    pub scheduler: SnoozeScheduler,
}
```

- [ ] **Step 2: 写 commands.rs**

`src-tauri/src/commands.rs`：

```rust
use crate::{autostart, db::{Note, NoteRepository}, geometry::clamp_into_work_area, snooze::SnoozeScheduler, state::AppState, window_manager};
use chrono::Utc;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

fn now_iso() -> String { Utc::now().to_rfc3339() }

#[tauri::command]
pub fn get_note(id: String, state: State<AppState>) -> Result<Option<Note>, String> {
    NoteRepository::get(&state.db, &id)
}

#[tauri::command]
pub fn create_note(app: AppHandle, state: State<AppState>) -> Result<Note, String> {
    let n = Note {
        id: Uuid::new_v4().to_string(),
        content: String::new(),
        color: "yellow".into(),
        x: 120.0, y: 40.0, w: 240.0, h: 170.0,
        snooze_minutes: default_snooze(&state)? as i64,
        created_at: now_iso(),
        completed_at: None, is_hidden: false, hidden_until: None,
    };
    NoteRepository::create(&state.db, &n)?;
    window_manager::open_note(&app, &n).map_err(|e| e.to_string())?;
    Ok(n)
}

#[tauri::command]
pub fn hide_note(id: String, app: AppHandle, state: State<AppState>) -> Result<(), String> {
    let until = Utc::now() + chrono::Duration::minutes(default_snooze(&state)? as i64);
    NoteRepository::snooze(&state.db, &id, &until.to_rfc3339())?;
    window_manager::hide_note(&app, &id).map_err(|e| e.to_string())?;
    let app2 = app.clone();
    state.scheduler.schedule(id.clone(), Duration::from_secs(default_snooze(&state)? as u64 * 60), move || {
        let _ = repop(&app2, &id);
    });
    Ok(())
}

fn repop(app: &AppHandle, id: &str) -> Result<(), String> {
    let state = app.state::<AppState>();
    if let Some(n) = NoteRepository::get(&state.db, id)? {
        if n.completed_at.is_none() {
            NoteRepository::clear_snooze(&state.db, id)?;
            window_manager::show_note(app, id).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn complete_note(id: String, app: AppHandle, state: State<AppState>) -> Result<(), String> {
    NoteRepository::complete(&state.db, &id, &now_iso())?;
    state.scheduler.cancel(&id);
    window_manager::close_note(&app, &id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn edit_note(id: String, content: String, state: State<AppState>) -> Result<(), String> {
    NoteRepository::update_content(&state.db, &id, &content)
}

#[tauri::command]
pub fn move_note(id: String, x: f64, y: f64, app: AppHandle, state: State<AppState>) -> Result<(), String> {
    let clamped = clamp_note(&state, x, y);
    NoteRepository::update_position(&state.db, &id, clamped.0, clamped.1)?;
    window_manager::move_note(&app, &id, clamped.0, clamped.1).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn reactivate(id: String, app: AppHandle, state: State<AppState>) -> Result<(), String> {
    NoteRepository::reactivate(&state.db, &id)?;
    if let Some(n) = NoteRepository::get(&state.db, &id)? {
        window_manager::open_note(&app, &n).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn copy_note(id: String, app: AppHandle, state: State<AppState>) -> Result<Note, String> {
    let src = NoteRepository::get(&state.db, &id)?.ok_or("not found")?;
    let n = Note {
        id: Uuid::new_v4().to_string(),
        content: src.content, color: src.color,
        x: src.x + 24.0, y: src.y + 24.0, w: src.w, h: src.h,
        snooze_minutes: src.snooze_minutes,
        created_at: now_iso(),
        completed_at: None, is_hidden: false, hidden_until: None,
    };
    NoteRepository::create(&state.db, &n)?;
    window_manager::open_note(&app, &n).map_err(|e| e.to_string())?;
    Ok(n)
}

#[tauri::command]
pub fn delete_note(id: String, state: State<AppState>) -> Result<(), String> {
    NoteRepository::delete(&state.db, &id)
}

#[tauri::command]
pub fn list_completed(state: State<AppState>) -> Result<Vec<Note>, String> {
    NoteRepository::completed(&state.db)
}

#[tauri::command]
pub fn show_all(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    for n in NoteRepository::active(&state.db)? {
        NoteRepository::clear_snooze(&state.db, &n.id)?;
        state.scheduler.cancel(&n.id);
        window_manager::show_note(&app, &n.id).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<std::collections::HashMap<String, String>, String> {
    settings_map(&state)
}
#[tauri::command]
pub fn set_settings(key: String, value: String, state: State<AppState>) -> Result<(), String> {
    let lock = state.db.lock().map_err(|e| e.to_string())?;
    lock.execute("INSERT INTO settings(key,val) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET val=excluded.val",
        rusqlite::params![key, value]).map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
pub fn get_autostart(app: AppHandle) -> bool { autostart::is_enabled(&app) }
#[tauri::command]
pub fn set_autostart(enabled: bool, app: AppHandle) -> Result<(), String> { autostart::configure(&app, enabled) }

fn default_snooze(state: &State<AppState>) -> Result<u64, String> {
    Ok(settings_map(state)?.get("default_snooze_minutes").and_then(|v| v.parse().ok()).unwrap_or(2))
}
fn settings_map(state: &State<AppState>) -> Result<std::collections::HashMap<String, String>, String> {
    let lock = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = lock.prepare("SELECT key, val FROM settings").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))).map_err(|e| e.to_string())?;
    let mut m = std::collections::HashMap::new();
    for r in rows { m.insert(r.map_err(|e| e.to_string())?.0, r.map_err(|e| e.to_string())?.1); }
    Ok(m)
}
fn clamp_note(state: &State<AppState>, _x: f64, _y: f64) -> (f64, f64) {
    // TODO(落地)：用 tauri::window 当前屏幕工作区构造 monitors，调用 clamp_into_work_area。
    // v1 先直接返回原坐标，多屏/越界处理在收尾补。
    // 注意：完成后删除此 TODO，改为真实夹回，不留占位。
    (_x, _y)
}
```

> `clamp_note` 是唯一需要落地的占位：用 `app` 的可用显示器列表（Tauri v2 可用 `app.available_monitors()`）构造 `geometry::Rect`，调用 `clamp_into_work_area`。完成后删除 TODO，不留占位。

- [ ] **Step 3: 写 lib.rs 装配**

`src-tauri/src/lib.rs`：

```rust
mod autostart; mod commands; mod db; mod geometry; mod snooze; mod state; mod tray; mod window_manager;
use chrono::Utc;
use state::AppState;
use std::time::Duration;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .setup(|app| {
            let path = app.path().app_data_dir()?.join("pinnotes.sqlite");
            std::fs::create_dir_all(path.parent().unwrap())?;
            let conn = rusqlite::Connection::open(&path)?;
            let db = db::init(conn).map_err(|e| anyhow::anyhow!(e))?;
            app.manage(AppState { db, scheduler: snooze::SnoozeScheduler::new() });
            tray::build(app.handle())?;
            // 默认开机自启（首次）
            let _ = autostart::configure(app.handle(), true);
            // 启动加载：开活跃便签窗、为隐藏中的排程
            let state = app.state::<AppState>();
            for n in db::NoteRepository::active(&state.db)? {
                if let Some(until_iso) = n.hidden_until.clone() {
                    let until = chrono::DateTime::parse_from_rfc3339(&until_iso)
                        .map_err(|e| anyhow::anyhow!(e))?.with_timezone(&Utc);
                    let now = Utc::now();
                    if n.is_hidden && until > now {
                        let app2 = app.handle().clone();
                        let id = n.id.clone();
                        state.scheduler.schedule(id.clone(), (until - now).to_std().map_err(|e| anyhow::anyhow!(e))?, move || {
                            let _ = commands_show(app2, &id);
                        });
                        continue;
                    }
                }
                window_manager::open_note(app.handle(), &n)?;
            }
            Ok(())
        })
        .on_menu_event(|app, e| {
            let id = e.id().as_ref();
            match id {
                "new" => { let _ = commands::create_note(app.handle().clone(), app.state::<AppState>()); }
                "showAll" => { let _ = commands::show_all(app.handle().clone(), app.state::<AppState>()); }
                "completed" => { let _ = open_simple(app, "completed"); }
                "settings" => { let _ = open_simple(app, "settings"); }
                "quit" => { app.exit(0); }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_note, commands::create_note, commands::hide_note, commands::complete_note,
            commands::edit_note, commands::move_note, commands::reactivate, commands::copy_note,
            commands::delete_note, commands::list_completed, commands::show_all,
            commands::get_settings, commands::set_settings, commands::get_autostart, commands::set_autostart,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn commands_show(app: tauri::AppHandle, id: &str) -> Result<(), String> {
    let state = app.state::<AppState>();
    if let Some(n) = db::NoteRepository::get(&state.db, id)? {
        if n.completed_at.is_none() {
            db::NoteRepository::clear_snooze(&state.db, id)?;
            window_manager::show_note(&app, id).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn open_simple(app: &tauri::AppHandle, route: &str) -> tauri::Result<()> {
    let label = route;
    if app.get_webview_window(label).is_some() { return Ok(()); }
    tauri::WebviewWindowBuilder::new(app, label,
        tauri::WebviewUrl::App(format!("index.html#/{route}").into()))
        .title(if route == "completed" { "已完成" } else { "设置" })
        .inner_size(420.0, 520.0).resizable(true).build()?;
    Ok(())
}
```

> `create_note` 命令签名需与 `generate_handler!` 一致：它取 `AppHandle` + `State<AppState>`，但在 `on_menu_event` 里调用时传 `app.handle().clone()`——命令是 `#[tauri::command]`，不能像普通函数直接调。**落地时**：把 `on_menu_event` 里的 `create_note/show_all` 调用改为通过 `app.try_state` + 直接调用内部逻辑函数（即把命令体拆出一个不含 `State` 参数的内部函数 `create_note_impl(app, &state)` 供两处复用）。同理 `commands_show` 已是此模式。请据此重构 `create_note`/`show_all` 为 `impl + 命令包装` 两层，避免在事件回调里调用 `#[tauri::command]`。

- [ ] **Step 4: 手动验收**

```bash
npm run tauri dev
```

确认：托盘出现；"新建便签"弹出置顶透明便签；拖动后位置持久（重启仍在）；点"隐藏"窗口消失、约 2 分钟后弹回原位；点"✓ 完成"便签消失并进入"已完成"；托盘"已完成"打开列表，重新激活/复制/编辑/删除生效；"设置"可改默认隐藏时长与开机自启；重启应用状态恢复、开机自启（打包后）生效。

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/commands.rs src-tauri/src/state.rs src-tauri/src/lib.rs src-tauri/tauri.conf.json src-tauri/Cargo.toml
git commit -m "feat: commands + wiring + startup load + default autostart"
```

---

## Self-Review（计划自检）

**1. Spec coverage**（对照设计文档）：
- 多窗口置顶/透明/无边框/拖动/位置记忆 → Task 1、4、7、10 ✅
- "无法永久隐藏" snooze/重弹 → Task 3、10 ✅
- SQLite 持久化、软删除、active/completed → Task 2、10 ✅
- 已完成窗口（重新激活/复制/编辑/删除）→ Task 8、10 ✅
- 设置（默认隐藏时长/开机自启）→ Task 5、9、10 ✅
- 托盘菜单 5 项 → Task 5、10 ✅
- 便签 4 色、纯文本 → Task 6、7 ✅
- 错误处理（位置夹回）→ Task 3、10（`clamp_note` 待落地）✅
- 测试策略（Rust 单测 + Svelte 组件测试 + 平台手动）→ 各任务 ✅
- 风险（透明窗渲染）→ Task 1 实测 ✅

**2. Placeholder scan**：核心逻辑（db/geometry/snooze）有完整可编译 Rust + 测试；前端组件有完整 Svelte + Vitest。两处明确待落地且附补救说明：① `commands.rs` 的 `clamp_note`（用 `app.available_monitors()` 接 `clamp_into_work_area`）；② `lib.rs` 里 `on_menu_event` 调用命令需拆 `impl + 命令包装` 两层（`create_note`/`show_all`）。其余 Tauri API（`TrayIconBuilder`/`WebviewWindowBuilder`/`autolaunch`）为 v2 稳定 API。

**3. Type consistency**：`Note` 结构在 Rust `db.rs` 与前端 `tauri.ts` 字段一一对应（snake_case 两侧一致）；命令名在 `commands.rs` 定义、前端 `invoke` 字符串逐一对应（`get_note/create_note/hide_note/complete_note/edit_note/move_note/reactivate/copy_note/delete_note/list_completed/show_all/get_settings/set_settings/get_autostart/set_autostart`）；`SnoozeScheduler::{new,schedule,cancel}` Task 3 定义、Task 10 消费一致。

---

## Execution Handoff

**Plan complete and saved to `docs/superpowers/plans/2026-07-22-pinned-sticky-notes.md`. Two execution options:**

**1. Subagent-Driven（推荐）** — 每个任务派一个全新子代理实现，任务间我做两段式 review，迭代快、上下文干净。适合这种 10 任务的长计划。

**2. Inline Execution** — 在当前会话里用 executing-plans 顺序批量执行，带检查点 review。

**Which approach?**
