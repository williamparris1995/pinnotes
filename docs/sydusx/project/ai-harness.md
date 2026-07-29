# AI Harness

> 给 agent 的工作流上下文。sydusx 各 stage skill 读这里了解"如何在本仓工作"。

## 指南文件（必读）

- **[CLAUDE.md](../../CLAUDE.md)** — 项目主指南：命令、架构、不可回退约束。最高优先级。
- **[docs/agents/issue-tracker.md](../../docs/agents/issue-tracker.md)** — issue / PRD 走 GitHub issues，用 `gh` CLI。
- **[docs/agents/triage-labels.md](../../docs/agents/triage-labels.md)** — 五个 canonical triage 标签（`needs-triage` 等，标签串 = 角色名）。
- **[docs/agents/domain.md](../../docs/agents/domain.md)** — 领域文档消费方式。

## 领域语言

- 本仓**尚无** `CONTEXT.md`（按 domain.md 约定，缺失时静默继续，不预先创建；待 `/domain-modeling` 懒创建）。
- 术语权威来源：[设计文档](../../docs/superpowers/specs/2026-07-22-pinned-sticky-notes-design.md) 第 3–7 节。

## 决策记录

[docs/adr/](../../docs/adr/)：0001（托盘菜单原生 HTML）、0002（自动更新）。与 ADR 冲突时**显式标注**，不静默覆盖。

## 已装的 agent 工具链

- **superpowers**（`.superpowers/`）— skills 体系（brainstorming、systematic-debugging、TDD 等）。
- **graphify**（`graphify-out/`）— 代码库知识图谱；`/graphify` 查询架构 / 文件关系。
- **claude-mem** — 跨会话记忆（本仓首次会话播种）。
- **sydusx**（本目录）— Product → Release → Sprint → Feature 流程。

## sydusx 入口

`/sydusx-run` → 读 [../progress.md](../progress.md) 定位 → 当前 feature 的 `state.md` → 当前 stage。
