# Release 0.5.1 — 质量基建

> **status**: 代码完成（release-0.5.1 分支,95fa3c7）,待手动测试 + 发版
> **version**: 0.5.1 (patch) · 无用户可见行为变化

## Release Goal

在 0.5.0（i18n）功能成熟的基础上，补齐**质量地基**：项目门面（根 README）、前后端测试覆盖加固、关键模块降债重构。为后续功能演进铺路。

## Sprints

- [x] [sprint-1：质量加固](sprint-1/sprint.md)

## Done-criteria

- 根 README 完成（定位、截图、快速上手、构建/发版说明、许可证）。
- 前端测试覆盖加固：noteView / completedView / settingsView 边界用例 + i18n / tauri.ts 测试。
- Rust 测试覆盖加固：commands `*_impl` / snooze / geometry / db.NoteRepository 逻辑单测。
- 关键模块重构：识别 smell → 重构（行为不变），全测试绿。
- CI（ci.yml）+ pre-commit hook 生效（首次运行已建立）。
- 版本号三处同步至 0.5.1（[package.json](../../../package.json) / [tauri.conf.json](../../../src-tauri/tauri.conf.json) / [Cargo.toml](../../../src-tauri/Cargo.toml)）+ [CHANGELOG](../../../CHANGELOG.md) 记一节。
