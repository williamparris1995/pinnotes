# Release 0.6.0 — 便签 Markdown 格式化

> **status**: done (v0.6.0 tagged；CI 三平台构建发布中)
> **version**: 0.6.0 (minor) · 新增便签内容 Markdown 格式化能力

## Release Goal

在 0.5.x 质量地基上，为便签内容引入 Markdown 格式化能力，让提醒更清晰（加粗重点 / 列表 / 标题等轻量格式）。编辑/渲染形态与 Markdown 语法子集由首个 feature 的 analysis 阶段细化定稿。

## Sprints

- [x] [sprint-1：Markdown 核心](sprint-1/sprint.md)

## Done-criteria

- 便签内容支持 Markdown 格式化（形态 + 子集由 analysis 定稿）。
- **向后兼容**：无 Markdown 语法的便签仍按纯文本正常显示。
- 编辑 + 渲染路径有自动测试覆盖 + 手动验证。
- 已完成列表维持纯文本不渲染(design ADR-4 决策),保紧凑布局。
- 版本号三处同步至 0.6.0（package.json / tauri.conf.json / Cargo.toml）+ CHANGELOG 一节。
- CI 通过 + GitHub Release 发布。
