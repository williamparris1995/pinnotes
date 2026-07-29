# Feature: 关键模块轻重构

- **title**: 关键模块轻重构
- **keywords**: refactor, smell, tech-debt, cleanup

## Description

经 review 通读（17 候选，无架构级债务），本轮做包 A（后端收敛）+ 包 B（颜色单一来源 + 修黄色 bug）。均行为不变。

## Stories

1. [x] review 后端 / 前端，列出 smell（17 候选）。
2. [x] 与用户确认重构点（选包 A + B）。
3. [x] 包 A：错误样板收敛（`run`/`run_exec`/`to_str`）+ `SELECT *` 显式列名。
4. [x] 包 B：颜色收敛到 `theme.css` `var()` + 修黄色三值不一致 bug。
