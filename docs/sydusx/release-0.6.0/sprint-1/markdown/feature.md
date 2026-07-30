# Feature：便签 Markdown 格式化

> 状态：analysis（形态/子集待定稿）

## 描述

为便签内容（`note.content`，当前为纯文本 textarea）引入 Markdown 格式化能力，让用户能用轻量格式让提醒更清晰。

**约束（来自现状）**：

- 便签窗口小（默认 240×170，大号 360×260）—— 完整 Markdown（表格/代码块/图片）不实用，倾向精简子集。
- 当前是单一 `<textarea>` 始终可编辑（noteView.svelte:158），输入防抖 500ms + 失焦自动保存；所见即所输，无编辑/预览之分。
- 项目偏轻量、零依赖（i18n 自研）。
- 已完成列表也展示 content，渲染需一致。
- 无 Acrylic / 窗口效果约束（透明纯色卡片，见 CLAUDE.md 第 6 条）。

**待 analysis 定稿**：编辑/渲染形态（编辑/预览切换 / 失焦渲染 / WYSIWYG）+ Markdown 语法子集 + 是否 per-note 开关。

## 关键词（用于 sydusx-run 路由）

markdown, 标签编辑, 格式化, 富文本, 渲染, 加粗, 列表

## 故事列表

- [ ] 1. per-note Markdown 开关（db `markdown` 字段 + toolbar 按钮 + 新建默认关）
- [ ] 2. 编辑/渲染状态机（点内容进编辑显源码，失焦回渲染 + 复用保存）
- [ ] 3. marked 渲染集成 + 精简子集配置（启用 8 类，禁用表格/代码块/图片/链接）
- [ ] 4. XSS 安全（escape/sanitize，验证 `<script>`/原始 HTML 不执行）
- [ ] 5. 已完成列表渲染一致（completedView 只读渲染）
- [ ] 6. 可发现性视觉提示 + 小窗排版 CSS 适配
- [ ] 7. 测试覆盖（子集渲染 + 状态机 + XSS + 向后兼容回归）
