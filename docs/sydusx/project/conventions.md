# Conventions

> sydusx 工作流约定。编码 / 命令约定以 [CLAUDE.md](../../CLAUDE.md) 为权威，本文补充工作流层面。

## 文档约定（sydusx）

- **零代码**：docs/ 下只写目标 / 范围 / 边界 / 契约，不写实现代码（design contracts 允许）。
- **单源**：每个事实一个家；其它文档**引用**，不复制。版本号引用 package.json / Cargo.toml，模块细节引用 CLAUDE.md。
- **plain markdown only**：无 Docusaurus 等依赖（文档站点可选，经 `sydusx-docs`）。
- **四层结构**：Product → Release → Sprint → Feature，目录按 docs-layout 规则。

## 编码约定（要点，权威见 CLAUDE.md）

- 新后端逻辑放 `*_impl`，薄 `#[tauri::command]` 包装。
- `is_hidden` 在 SQLite 是 `i64`、在 struct 是 `bool`（[db.rs](../../src-tauri/src/db.rs) `row_to_note` 转换）。
- Rust 命令返回 `Result<T, String>`；前端捕获并提示；单窗口异常不影响其它窗口或后端。
- 前端用 Svelte runes（`$state`）做局部 UI 状态；无全局状态库。

## 提交 / 发布约定

- 版本号在 [package.json](../../package.json)、[tauri.conf.json](../../src-tauri/tauri.conf.json)、[Cargo.toml](../../src-tauri/Cargo.toml) **三处同步**。
- 每个 Release 在 [CHANGELOG.md](../../CHANGELOG.md) 记一节；发版正文由脚本从 CHANGELOG 提取。
- 发版：推 `v*` tag → CI 三平台构建发布。
