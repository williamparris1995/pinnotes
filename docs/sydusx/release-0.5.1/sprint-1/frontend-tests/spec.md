# Spec: 前端测试覆盖加固（light · M 级）

## 范围（已确认：推荐范围）

### 新增 `src/lib/i18n.test.ts`
- `t()`：当前语言命中 / en 回退 / 键回退 / `{var}` 占位符替换
- `initLocale()`：language 命中(zh/en) / 无效+first_run_done=1→zh / 无效+新用户→en / 异常→保持
- `setLocalePersist()`：切 locale + `invoke('set_settings', {key:'language'})`

### 补 `noteView.test.ts`
- `edit_note`（focusout，内容变才存）
- `set_color`（点色点）
- `set_snooze`（cycle 1/2/5/10/30/60）
- `loadError`（`get_note` reject）

### 补 `completedView.test.ts`
- `delete_note` + 列表刷新

### 补 `settingsView.test.ts`
- 语言切换 → `set_settings` language

## Scope 边界（不测）
- `tauri.ts`（纯 re-export，无逻辑）
- `fmtTime`（私有纯函数）
- CSS / 视觉、拖动（已 mock，无逻辑）

## Acceptance
- 全绿；`npm test` 用例数从 5 显著提升；mock 边界卡在 `./tauri`。
