# Tech Stack

> **版本号真相源 = [package.json](../../package.json) + [src-tauri/Cargo.toml](../../src-tauri/Cargo.toml)**。本文只记选型决策与关键约束，不复制版本号（避免漂移）。

## 选型

| 层 | 选型 | 为什么 |
|---|---|---|
| 桌面框架 | Tauri 2（Rust + WebView） | 多窗口 / 置顶 / 透明 / 无边框 / 托盘 / 自启 / SQLite 均为官方稳定能力，使"每条便签一个真实窗口"稳定可行 |
| 前端 | Svelte 5（runes）+ Vite + TS | 薄视图、窗口间隔离，无需全局状态库（Rust 为真相源） |
| 存储 | rusqlite（`bundled`，自带 sqlite） | 成熟、类型化、可单测 |
| 异步计时 | tokio（time / sync / rt-multi-thread） | snooze 排程 |
| 自启 / 快捷键 / 更新 | tauri-plugin-{autostart, global-shortcut, updater} | 官方插件 |
| Win32 | `windows` crate（仅 Windows） | 原生 `ShowWindow` 做不抢焦点的重弹 |

## 关键约束（不可回退）

- **Vitest `pool: "vmThreads"`**（[vite.config.ts](../../vite.config.ts)）。默认 `forks` 在 Win10 / Node22 下无法收集套件。**不要改回 forks。**
- **Vite 固定端口 1420**（`strictPort`）。端口被占会阻塞 `tauri dev`——先杀残留进程。
- **平台**：Windows 优先；macOS / Linux 仅在 CI 构建（无法从 Windows 交叉编译）。
- **Rust 单测**注入可控时钟，避免真睡眠。

## 命令

`npm run tauri dev` / `npm run tauri build` / `npm run check` / `npm test` / `cd src-tauri && cargo test|check`。完整表见 [CLAUDE.md](../../CLAUDE.md) Commands 节。
