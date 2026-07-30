# Design system — v1

> 便签 Markdown 渲染态原型。**复用** [theme.css](../../../../../src/lib/theme.css) + noteView 现有样式；**新增** markdown 排版 token（小窗适配）。

## 输入来源（synthesized，非 invented）

- **Project docs**：theme.css（token）、noteView.svelte（便签样式/字号/文字色/toolbar 布局）、design.md（约束：per-note / 编辑-渲染切换 / 子集 / 已完成列表不渲染）。
- **Reference**：CommonMark 标准渲染视觉惯例 + 小窗便签约束（无外部产品直接对标——Bear/Obsidian 是大窗编辑器，不适用 240×170）。
- **User input**：design.md grill 结论 + theme.css 体现的品味（亮色卡片、Segoe UI、紧凑）。

## Tokens

### 复用（现有，不变）

| token | 值 | 来源 |
|---|---|---|
| `--c-yellow/pink/blue/green` | #ffe678 / #ffc0c8 / #a0cdff / #aae6a8 | theme.css |
| `--font` | "Segoe UI", system-ui, sans-serif | theme.css |
| `--radius` | 12px | theme.css |
| 正文文字色 | rgba(15,15,25,0.86) | noteView textarea |
| 正文字号 | 15px / weight 600 | noteView textarea |

### 新增（markdown 排版，240×170 小窗适配）

| 元素 | 规格 | 理由 |
|---|---|---|
| h1 | 16px bold, margin 2px 0 | 小窗标题不能撑满，略大于正文 |
| h2 | 15px bold, margin 2px 0 | 与正文同级但加粗 |
| h3 | 14px bold, margin 1px 0 | 次级 |
| ul/ol | padding-left 18px;li margin 1px 0 | 列表缩进，圆点/数字 |
| blockquote | 左 2px solid rgba(0,0,0,0.16);padding-left 8px;色 rgba(15,15,25,0.6) | 引用弱化 |
| code(行内) | 底色 rgba(0,0,0,0.07);radius 3px;padding 0 4px;font ui-monospace | 行内代码 |
| del | text-decoration line-through;色 rgba(15,15,25,0.6) | 删除线弱化 |
| hr | 1px solid rgba(0,0,0,0.16);margin 6px 0 | 分隔线 |
| strong/em | 继承正文色 + weight/style | 加粗/斜体 |

## Palette

4 色便签背景（`--c-*`）+ 正文 rgba(15,15,25,0.86) + 次要 rgba(15,15,25,0.6) + code 底色 rgba(0,0,0,0.07)。无 dark 变体（便签是亮色不透明卡片）。

## Motion

- 编辑 ↔ 渲染切换：opacity 淡入 120ms ease（避免硬切闪烁）。
- toolbar 按钮 hover：background 0.12s（沿用现有 `.size-btn`/`.color-dot` 节奏）。

## Interaction

- **渲染态**：`cursor:text`；hover 显示"点击编辑"微提示（小 bubble，11px，rgba 底色）。
- **toolbar markdown 开关**：默认（描边图标）/ 激活（实心 + accent 描边）两态。
- **编辑态**：源码 textarea（现状样式）；失焦 → 回渲染 + 保存。

## Accessibility

- 对比度：rgba(15,15,25,0.86) on 4 色——yellow(#ffe678) 最弱但仍 > 4.5:1（正文 AA 达标）。
- 触控目标：toolbar 控件 ~14px（便签小窗取舍；非触屏优先）。
- focus：编辑态 textarea 需可见 focus 指示（现有 `outline:none` 需补）。
- reduced-motion：切换淡入 respect `prefers-reduced-motion`。
