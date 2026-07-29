---
feature: 关键模块轻重构
grade: L
current_stage: done
status: done
---

# State

grade = L。包 A（后端收敛）+ 包 B（颜色单一来源 + 修 bug）完成。**行为不变**：cargo 27 + npm 20 全绿。

## Per-stage

| stage | status |
|---|---|
| analysis | ✅ done → review 报告（17 候选，选 A+B） |
| design | ✅ done → [design.md](design.md)（ADR-0003a/b/c） |
| code | ✅ done → db `run`/`run_exec` + 显式列；commands/autostart `to_str`；theme.css + noteView/completedView `var()` |
| review | ✅ self-check（符合 ADR + 现有约定，未碰有意设计） |
| test | ✅ cargo 27 + npm 20 全绿（行为不变） |

## Deferred

- 包 C（前端 DRY #8/#9）、包 D（结构/可测性 #4/#5）——本轮未选，留后续。
- 颜色视觉一致性（尤其黄色统一为 `#ffe678`）建议手动确认。
