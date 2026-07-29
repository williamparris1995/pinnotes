# Architecture

> sydusx 视角的架构基线。**模块细节、命令清单、"踩过的坑"以 [CLAUDE.md](../../CLAUDE.md) 的 Architecture / Things that bit us 节为权威**——本文不复制。

## 一句话

方案 A：**每条便签 = 一个真实 OS 窗口**。单 Tauri 进程两层——**Rust 后端是唯一数据真相源与管理者**（SQLite + snooze 计时 + 窗口生命周期 + 托盘），**Svelte 前端是薄视图**（无权威状态，只反映后端）。

## 通信边界

- 前端 → Rust：`invoke('command', {...})`
- Rust → 前端：`window.emit("event", payload)` + 前端 `listen`
- 命令 / 事件即接口边界。新逻辑一律放 `*_impl`（[commands.rs](../../src-tauri/src/commands.rs)），薄 `#[tauri::command]` 包装以供托盘菜单 / 快捷键复用。

## 路由

hash 路由 SPA（[src/App.svelte](../../src/App.svelte)）：`#/note?id=<id>` / `#/completed` / `#/settings`。`tauri.conf.json` 主窗口 `visible:false` 仅作进程入口，可见窗口全部由 Rust 按需创建。

## 不可回退的约束（改前必读 CLAUDE.md）

hide = snooze；show/hide 走原生 Win32；`reactivate` 必须 async；拖动位置去抖；坐标是逻辑像素；便签窗口禁用 Acrylic。→ 完整说明见 [CLAUDE.md](../../CLAUDE.md)「Things that bit us」。

## 决策记录

ADR 在 [docs/adr/](../../docs/adr/)：0001 托盘菜单保持原生 HTML、0002 自动更新经 Tauri updater + GitHub。
