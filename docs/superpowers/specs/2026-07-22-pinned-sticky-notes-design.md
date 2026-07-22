# PinNotes（置顶便签提醒）设计文档

- **日期**：2026-07-22
- **状态**：v2 — 技术栈改为 Tauri + Svelte（产品需求与 v1 一致，未变）
- **平台**：Windows 优先（Tauri 天然跨平台，预留 macOS/Linux）
- **技术栈**：Tauri v2（Rust 后端）+ Svelte 5（Vite 前端）
- **工作名**：PinNotes（可改）

> v1 变更摘要：原为 Flutter 桌面应用，因 Flutter 桌面多窗口无稳定官方方案、`desktop_multi_window` 属 0.x 社区库，改为 **Tauri**——其多窗口/置顶/透明/无边框/托盘/自启/SQLite 均为官方稳定能力，使"每条便签一个真实置顶窗口"（方案 A）完全稳定。产品行为（无法永久隐藏、多便签可拖动、已完成列表、开机自启等）保持不变。

## 1. 概述

一个常驻系统托盘的桌面应用，把"重要提醒"以**便签样式**显示在屏幕上。每条便签是一个独立、置顶、无边框、透明的窗口，可拖到任意位置并记住位置。点击"隐藏"只是短暂收起——只要任务没被标记"完成"，便签会**在设定的隐藏时长后重新弹出到它自己的位置**（"无法永久隐藏"）。只有标记完成才让它从桌面消失（进入"已完成"列表，可重新激活/复制/编辑/永久删除）。

## 2. 目标与非目标

**目标**
- 多条便签，各自独立置顶窗口，可拖动定位、记住位置。
- "无法永久隐藏"：隐藏 = 短暂 snooze，到点未完成必弹回原位。
- 托盘常驻 + 右键菜单管理；已完成列表独立窗口，支持重新激活/复制/编辑/永久删除。
- 数据本地持久化（SQLite），开机自启。

**非目标（v1 不做，YAGNI）**
- 不做云同步、多设备、账号体系。
- 不做便签的优先级 / 截止日期 / 定时时间。
- 不做富文本 / Markdown，仅纯文本。
- 不做全局热键（仅托盘菜单）。

## 3. 核心决策

1. **技术栈**：Tauri v2（Rust）+ Svelte 5（Vite）。多窗口用 Tauri 原生 `WebviewWindow`（官方稳定）。
2. **架构**：方案 A——每条便签一个真实 OS 窗口。**Rust 后端为唯一数据真相源与管理者**（SQLite + snooze 计时 + 窗口生命周期 + 托盘）；Svelte 前端窗口是"薄视图"——加载时 `invoke` 取数据、用户操作 `invoke` 回调命令、`listen` Rust 事件做响应式更新。
3. **重新弹出模型**：无法永久隐藏。隐藏 = snooze（默认 2 分钟，每条可配 1/2/5/10/30 分钟），到点未完成弹回原位，持续到完成。
4. **管理入口**：系统托盘右键菜单（新建 / 显示全部 / 已完成 / 设置 / 退出）；"已完成"为独立窗口。
5. **持久化**：SQLite，后端用 `rusqlite`（成熟、类型化、可单测），带 `CREATE TABLE IF NOT EXISTS` 式迁移。
6. **完成语义**：软删除（置 `completed_at`）+ 5 秒撤销；不硬删。
7. **开机自启**：默认开（`tauri-plugin-autostart`），设置可关。
8. **便签内容**：纯文本 + 4 色；无优先级/日期。
9. **平台**：Windows 优先，预留跨平台。

## 4. 架构

单 Tauri 进程，分两层：

- **Rust 后端（`src-tauri/`，"管理者"）**：唯一数据真相源。持有 SQLite（`rusqlite`）、托盘（`TrayIconBuilder`）、开机自启（`tauri-plugin-autostart`）、snooze 计时（`tokio` 任务）、所有便签窗口的生命周期（`WebviewWindow`）。任何状态变更先落库再驱动窗口（显示/隐藏/关闭/移动）或发事件。
- **Svelte 前端（`src/`，"薄视图"）**：同一 SPA 按 URL hash 路由成三种窗口：
  - `index.html#/note?id=<uuid>`：`NoteView`，渲染单条便签（双击编辑、隐藏、✓ 完成、拖动上报坐标）。
  - `index.html#/completed`：`CompletedView`，已完成列表 + 操作。
  - `index.html#/settings`：`SettingsView`，默认隐藏时长 / 开机自启。
  每个窗口加载时 `invoke` 命令取数据；用户操作 `invoke` 回调；`listen` Rust 事件以响应式刷新。

**主窗口**：`tauri.conf.json` 里的主窗口设为不可见（`visible:false`），仅作进程入口；实际可见窗口全部由 Rust 按需创建。

**通信**：前端 `invoke('command_name', { ... })` 调 Rust 命令；Rust `window.emit("event", payload)` 推送，前端 `listen("event", cb)`。命令/事件即接口边界。

**设计原则**：Rust 后端集中状态与决策，前端窗口无权威状态、只反映后端。领域逻辑（重弹判定、位置夹回、CRUD）在 Rust 侧单测覆盖。前端用 Svelte runes（`$state`）做局部 UI 状态，无需额外全局状态库（窗口彼此隔离，Rust 为真相源）。

## 5. 组件

**Rust（`src-tauri/src/`）**
| 模块 | 职责 |
|---|---|
| `db.rs` | `rusqlite` 连接（`Mutex<Connection>` 存于 `State`）+ 迁移 + `NoteRepository`（CRUD/查询）。 |
| `snooze.rs` | `SnoozeScheduler`：tokio 任务，到点未完成则通知窗口层重显。 |
| `geometry.rs` | `clamp_into_work_area`：坐标夹回最近可见显示器工作区（纯函数，单测）。 |
| `window_manager.rs` | 创建/显示/隐藏/关闭/移动便签窗口；下发窗口属性（置顶/无边框/透明/跳过任务栏/位置/尺寸）。 |
| `tray.rs` | `TrayIconBuilder` + 菜单 + 事件分发。 |
| `autostart.rs` | `tauri-plugin-autostart` 封装。 |
| `commands.rs` | `#[tauri::command]`：`create_note/hide_note/complete_note/move_note/edit_note/reactivate/copy_note/delete_note/get_note/list_completed/get_settings/set_settings`。 |
| `lib.rs`/`main.rs` | 装配：插件、状态、托盘、命令注册、启动加载。 |

**Svelte（`src/`）**
| 组件 | 职责 |
|---|---|
| `App.svelte` + `main.ts` | hash 路由分发到三种视图。 |
| `lib/noteView.svelte` | 单条便签 UI（复用 OD 原型样式）：grip、文本、隐藏/完成按钮、拖动。 |
| `lib/completedView.svelte` | 已完成列表 + 行内操作。 |
| `lib/settingsView.svelte` | 默认隐藏时长分段 + 开机自启开关。 |
| `lib/tauri.ts` | 对 `invoke`/`listen` 的薄封装与类型。 |

## 6. 数据模型（SQLite）

```sql
CREATE TABLE IF NOT EXISTS notes (
  id            TEXT PRIMARY KEY,
  content       TEXT NOT NULL,
  color         TEXT NOT NULL DEFAULT 'yellow',   -- yellow|pink|blue|green
  x             REAL NOT NULL DEFAULT 120,
  y             REAL NOT NULL DEFAULT 40,
  w             REAL NOT NULL DEFAULT 240,
  h             REAL NOT NULL DEFAULT 170,
  snooze_minutes INTEGER NOT NULL DEFAULT 2,
  created_at    TEXT NOT NULL,                     -- RFC3339
  completed_at  TEXT,                              -- 非空 = 已完成
  is_hidden     INTEGER NOT NULL DEFAULT 0,
  hidden_until  TEXT
);
CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  val TEXT NOT NULL
); -- 默认 default_snooze_minutes=2, autostart=1
```

- **活跃** = `completed_at IS NULL`；**已完成** = `completed_at IS NOT NULL`。

## 7. 关键流程

- **新建**（托盘"新建"）：Rust 插一条记录（默认主屏顶部居中）→ 创建对应置顶透明窗口（`#/note?id=`）。
- **隐藏 / snooze**：`NoteView` 点"隐藏" → `invoke('hide_note',{id})` → Rust 置 `is_hidden=1, hidden_until=now+snooze`、隐藏窗口 → `SnoozeScheduler` 排程 → 到点若 `completed_at` 仍空：在记录的 `(x,y)` 重显并清隐藏态；若已完成则不弹。
- **完成**：点 `✓` → `invoke('complete_note',{id})` → Rust 置 `completed_at`（软删除）、关闭窗口 → 前端弹 ~5 秒"撤销"（`invoke('reactivate',{id})` 撤销）。
- **重新激活**（已完成窗口）：清 `completed_at, is_hidden, hidden_until` → 回到活跃并立即开窗。
- **复制**：以相同 `content/color` 新建一条**活跃**便签（新 id，默认位置）并开窗；原已完成项保留。
- **编辑**（已完成窗口）：就地改 `content`，仍留已完成。
- **永久删除**：从库中删除该行。
- **拖动**：`NoteView` 原生窗口拖动结束 → `invoke('move_note',{id,x,y})` → Rust 更新坐标（重弹回到新位置）。
- **显示全部**（托盘）：清空所有活跃便签隐藏态并立即显示全部窗口。
- **启动**：Rust 加载所有活跃记录 → 对每条：`hidden_until` 在未来则排程；已过期则补弹并显示；否则正常开窗。

> **前提**：应用需保持运行（托盘常驻）才能计时弹回；退出即停止提醒。开机自启默认开以保证持续提醒。

## 8. 窗口属性与依赖

**便签窗口属性**（`WebviewWindowBuilder`）：`transparent(true)`、`decorations(false)`、`always_on_top(true)`、`skip_taskbar(true)`、`resizable(false)`、`inner_size(w,h)`、`position(x,y)`。已完成/设置窗口为普通有边框窗口。

**依赖（Rust，均成熟）**：`tauri = 2`、`rusqlite`（含 `bundled` feature，自带 sqlite）、`tokio`（rt-multi-thread, time）、`serde`/`serde_json`、`uuid`、`tauri-plugin-autostart`、`chrono`。**前端**：`@tauri-apps/api`、`@tauri-apps/plugin-autostart`、Svelte 5 + Vite。

> 不再使用 `desktop_multi_window`（Tauri 多窗口为原生稳定）。

## 9. 设置与默认值

- `default_snooze_minutes`：默认 2（新建便签继承；单条可改 1/2/5/10/30）。
- `autostart`：默认 **开**，首次启动时通过 `tauri-plugin-autostart` 启用；可在设置关闭。

## 10. 错误处理

- **位置越界**：坐标以设备像素存储；换显示器/分辨率/DPI 变化导致 `(x,y)` 落到不可见区域 → 开窗前用当前屏幕信息按 DPI 换算并把坐标夹回最近可见显示器工作区。
- **数据完整性**：`rusqlite` 事务保证原子写；加载/迁移时校验，损坏记录跳过并记日志，不阻断启动。
- **窗口/命令失败**：Rust 命令返回 `Result<T, String>`，前端捕获并提示；单窗口异常不影响其他窗口或后端。
- **计时**：snooze 用 tokio 任务；应用未运行时不弹（可接受，靠开机自启覆盖常态）。

## 11. 测试策略

- **Rust 单测（重点）**：`NoteRepository` 的 CRUD/active/completed 分区；`SnoozeScheduler` 排程与到点判定；`clamp_into_work_area`（含多显示器选区）。注入可控时钟避免真睡眠。
- **Svelte 组件测试**：`noteView`/`completedView`/`settingsView` 用 Vitest + `@testing-library/svelte`，mock `invoke`/`listen`。
- **手动测试清单**：多窗口置顶/透明/拖动/隐藏重弹、托盘菜单、已完成窗口各操作、开机自启、重启后状态恢复、换显示器位置夹回。

## 12. 风险与缓解

- **主要风险**：Windows 上"透明 + 置顶 + 无边框"窗口的渲染 quirks（透明需开启相关 feature；个别显卡下透明可能发黑）。缓解：先用一个最小窗口验证透明+置顶；必要时退化为"近透明/纯色圆角"。
- 多窗口现已为 Tauri 原生稳定能力，不再视为风险（原 Flutter 下的 `desktop_multi_window` 风险已消除）。
- **tokio 与 Tauri 运行时**：snooze 任务用 Tauri 内置 async runtime（`async-runtime`），避免自建 runtime 冲突。

## 13. 范围之外 / 未来

云同步、多设备、优先级/截止日期/定时、富文本/Markdown、全局热键、提醒音、多显示器独立布局。这些在 v1 之后按需再议。
