---
feature: markdown
title: 便签 Markdown 格式化
grade: L
current_stage: done
status: done
---

# State：便签 Markdown 格式化

## 阶段状态

| stage | done | output |
|---|---|---|
| analysis | ✓ | [spec.md](spec.md) |
| design | ✓ | [design.md](design.md) |
| code | ✓ | commit `23a1d40` + 修复（工作树未提交）：noteView `onfocus` pin editing + 回归测试 + spec 同步 |
| review | ✓ pass（重审）| fix diff 双轴 clean：bug 已修、spec 已同步 ADR-4，无新增问题 |
| test | ✓ | pass：vitest 38/38 + cargo 29/29 + svelte-check clean；NFR-1/3/7 测、2/5/6 结构、NFR-4 手动核验过 |

## 备注

L 级。design 已定（详见 design.md ADR）：编辑/预览切换（排除 WYSIWYG）· per-note opt-in · marked+DOMPurify 渲染管道 · 已完成列表不渲染（调整 FR-7）· 幂等 ALTER TABLE migration。

code 阶段已实现并提交（`23a1d40`）：db 字段/migration、commands.set_markdown（含 invoke_handler 注册）、noteView 编辑/渲染状态机、src/lib/markdown.ts 渲染管道、渲染态 CSS、前端/Rust 测试。提交时测试绿（vitest 37 / cargo 29/29）。

review 关卡已跑（sydusx-code-review 双轴）。裁决 **REJECT**——Standards 轴 clean（仅 1 轻微 CSS 重复），Spec 轴 1 个真实行为 bug：

**bug（FR-4 偏差）**：空内容 markdown 便签打字暂停 ~500ms 后被踢回渲染态。`showTextarea` 派生（noteView.svelte:30）用 `note.content === ''` 作"空便签进编辑"入口，但**未 pin `editing=true`**；而 `onInput` 防抖保存（:82）会写回 `note.content = draft`，一旦内容非空 → 末项翻 false → `showTextarea` 翻 false → textarea 被渲染态替换，**焦点丢失、并未 focusout**。偏离 design.md:76 LLD「空+markdown → onMount 设 editing=true」。测试漏覆盖（mock invoke 不写回 note.content，故 37 绿仍漏）。

**fix list（已完成，工作树未提交）**：
1. ✅ 修翻转 bug——noteView 加 `handleFocus`：textarea 获焦即 pin `editing=true`（markdown 开时），覆盖所有打字路径，防抖保存不再踢出编辑态。
2. ✅ 回归测试——noteView.test.ts 新增"空 markdown 便签防抖保存后不翻转"（红→绿）。
3. ✅ 文档同步——spec.md FR-7 / A-5 / 范围内外 / 技术约束 4 处对齐 ADR-4。

重审（fix diff）：双轴 clean，**PASS**。验证：vitest 38/38、svelte-check 0 error。

**test 关卡（含手动 NFR 核验）→ PASS，feature done。**
- 自动化：vitest 38/38 · cargo 29/29 · svelte-check 0 error（覆盖率工具未装，已记）。
- NFR-1/3/7 自动测试；NFR-2/5/6 结构性；**NFR-4 小窗排版 + 渲染正确性 dev 窗口手动核验通过**。
- 手动核验发现并修复 1 个渲染 bug：渲染态基础字重 600 在 CJK 字体被向上取整到 Bold 面，致 `<strong>` 无法更粗、看不出加粗。修：`.note-md` 基础 600→400、strong 800→700（textarea 编辑态不变，FR-8 安全）。jsdom 单测盲区，靠手动目视捕获。
- FR-4 焦点修复经真机核验（空便签打字不跳渲染）。

feature 全阶段（analysis/design/code/review/test）✓，达 DoD。下一步：commit 工作树修复 → merge feature/markdown → main → release 0.6.0 release gate。

## deferred

- CSS 重复（`.note-body textarea` 与 `.note-md` 共享 font-size/padding/color 等排版值）——轻微 Duplicated Code，review 标 JUDGEMENT，暂不修。
- 图片语法 `![](url)` 经 DOMPurify 整体剥离降级为空（FR-6「按原文」对图片仅宽松满足）——暂不修，图片本就被排除。
