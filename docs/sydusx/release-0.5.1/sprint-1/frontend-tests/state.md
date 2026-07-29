---
feature: 前端测试覆盖加固
grade: M
current_stage: done
status: done
---

# State

grade = M。新增 `i18n.test.ts` + 补 3 个组件测试。`npm test`：4 文件 / 20 用例全绿（5 → 20，+15）。

## Per-stage

| stage | status |
|---|---|
| analysis | ✅ done → [spec.md](spec.md) |
| design | skip（M） |
| code | ✅ done → [i18n.test.ts](../../../../../src/lib/i18n.test.ts) + noteView / completedView / settingsView 补测 |
| review | ✅ self-check（覆盖 spec、mock 边界卡在 `./tauri`、未改产品代码、符合现有风格） |
| test | ✅ `npm test` 4 files / 20 passed |

## Deferred

_(none)_
