# Feature: 前端测试覆盖加固

- **title**: 前端测试覆盖加固
- **keywords**: test, vitest, frontend, coverage, svelte

## Description

加固前端 Vitest 覆盖：现有 [noteView](../../../../src/lib/noteView.svelte) / [completedView](../../../../src/lib/completedView.svelte) / [settingsView](../../../../src/lib/settingsView.svelte) 补边界用例；为新模块补测试——[i18n.svelte.ts](../../../../src/lib/i18n.svelte.ts)（语言切换 / 字典查找 / 缺键回退）。mock 边界卡在 `invoke` / `listen`。

## Stories

1. [x] noteView / completedView / settingsView 补边界与交互（edit_note / set_color / set_snooze / loadError / delete_note / 语言切换）。
2. [x] i18n.svelte.ts：`t()` 三级回退 + `{var}` / `initLocale` 4 分支 + 异常 / `setLocalePersist`。
3. [~] tauri.ts：**跳过**（scope 边界——纯 re-export，无逻辑可测）。
