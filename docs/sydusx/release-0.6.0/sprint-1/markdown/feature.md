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

- [ ] （analysis 定稿形态/子集后填充具体故事）
