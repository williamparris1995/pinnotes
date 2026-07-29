# Feature: Rust 测试覆盖加固

- **title**: Rust 测试覆盖加固
- **keywords**: test, cargo, rust, coverage, unit

## Description

加固 Rust 后端单测。注入可控时钟 / in-memory SQLite，覆盖纯逻辑分支。AppHandle 相关命令（需 runtime）与 SnoozeScheduler 排程（需 tokio+时间）划入 scope 边界。

## Stories

1. [x] snooze.rs：`should_repop` 谓词（None 分支 + `==` 边界）。〔排程/cancel → scope 边界〕
2. [x] geometry.rs：空 monitors + 垂直越界 + 窗口大于工作区。
3. [x] db.rs：`row_to_note` 往返保真（`is_hidden` i64↔bool 转换）+ update_position/content + delete + get-missing。
4. [x] commands.rs：`lang` 4 分支 + get/set_setting 往返 upsert。〔AppHandle 命令 → scope 边界〕
