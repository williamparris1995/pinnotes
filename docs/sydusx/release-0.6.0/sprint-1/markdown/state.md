---
feature: markdown
title: 便签 Markdown 格式化
grade: L
current_stage: code
status: in-progress
---

# State：便签 Markdown 格式化

## 阶段状态

| stage | done | output |
|---|---|---|
| analysis | ✓ | [spec.md](spec.md) |
| design | ✓ | [design.md](design.md) |
| code | - | - |
| review | - | - |
| test | - | - |

## 备注

L 级。design 已定（详见 design.md ADR）：编辑/预览切换（排除 WYSIWYG）· per-note opt-in · marked+DOMPurify 渲染管道 · 已完成列表不渲染（调整 FR-7）· 幂等 ALTER TABLE migration。

下一步：code 阶段——按 design.md LLD 实现（db 字段/migration、commands.set_markdown、noteView 编辑/渲染状态机、src/lib/markdown.ts 渲染管道、渲染态 CSS、测试）。

## deferred

（暂无）
