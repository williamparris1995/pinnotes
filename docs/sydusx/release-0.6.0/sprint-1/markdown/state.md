---
feature: markdown
title: 便签 Markdown 格式化
grade: L
current_stage: design
status: in-progress
---

# State：便签 Markdown 格式化

## 阶段状态

| stage | done | output |
|---|---|---|
| analysis | ✓ | [spec.md](spec.md) |
| design | - | - |
| code | - | - |
| review | - | - |
| test | - | - |

## 备注

L 级（设计复杂度中-高 + 跨 noteView/completedView/db/新依赖；需求经 grill 收敛，不确定性低）。

形态决策（grill 收敛，详见 spec.md 决策摘要）：编辑/预览切换（排除 WYSIWYG）· 默认渲染态 · per-note opt-in · 点内容进编辑/失焦回渲染 · 精简排版子集（排除表格/代码块/图片/链接）· marked 渲染引擎。

下一步：design 阶段（ADR/HLD/LLD）—— sanitize 策略、db 字段存储细节、toolbar 开关 UI、渲染态 CSS 适配。

## deferred

（暂无）
