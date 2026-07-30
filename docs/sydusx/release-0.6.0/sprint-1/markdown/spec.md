# Spec：便签 Markdown 格式化

> feature: markdown · release 0.6.0 · sprint-1
> 产出阶段：analysis · grill 收敛于 2026-07-30

## 决策摘要（grill 结论）

| 维度 | 决策 | 依据 |
|---|---|---|
| 编辑形态 | 编辑/预览切换（编辑=源码 / 非编辑=渲染），排除 WYSIWYG | WYSIWYG 撞轻量零依赖 + 小窗 |
| 默认态 | 渲染态（瞥一眼看清） | 产品目标：提醒清晰 |
| 范围 | per-note opt-in | 旧便签/短文本不动，保"打开就打字" |
| 触发 | 点内容进编辑，失焦回渲染 | 复用失焦保存，省 toolbar 空间 |
| 子集 | 精简排版：加粗/斜体/删除线/标题/列表/行内代码/引用/分隔线 | 小窗可排版，贴"提醒清晰" |
| 排除 | 表格/代码块/图片/链接 | 240×170 不实用 + XSS 面 |
| 渲染引擎 | marked（轻量库，配置子集） | 解析边界 case 成熟，自研得不偿失 |

## 功能需求（FR）

- **FR-1** per-note 开关：每条便签可单独启用/禁用 Markdown 渲染（toolbar 按钮）。
- **FR-2** 默认关闭：新建便签默认 Markdown 关（保持"打开就打字"）。
- **FR-3** 双态显示：启用 Markdown 的便签——非编辑态显示渲染 HTML；编辑态显示源码 textarea。
- **FR-4** 触发：点内容区进编辑态（源码），失焦回渲染态；失焦顺带触发既有保存逻辑（存源码 content）。
- **FR-5** 子集渲染：正确渲染 加粗 / 斜体 / 删除线 / 标题(#~###) / 无序+有序列表 / 行内代码(`) / 引用(>) / 分隔线(---)。
- **FR-6** 排除项降级：表格 / 代码块(```) / 图片 / 链接 语法不渲染——按原文安全显示，不报错。
- **FR-7** 已完成列表一致：已完成列表中启用 Markdown 的便签内容也渲染（只读），与便签渲染一致。
- **FR-8** 未启用便签不变：禁用 Markdown 的便签维持当前纯 textarea 行为/视觉，零变化。

## 非功能需求（NFR）

- **NFR-1 XSS 安全**：渲染的 HTML 必须 escape/sanitize——禁止原始 HTML、`<script>`、`javascript:` 等注入。排除链接/图片已缩小攻击面，仍须 escape。
- **NFR-2 轻量**：使用 marked（~20KB），配置仅启用子集规则；不引入双向 WYSIWYG 编辑器。
- **NFR-3 向后兼容**：未启用 Markdown 的便签（含所有既有数据）行为/视觉零变化。
- **NFR-4 小窗排版**：渲染元素在 240×170（普通）/ 360×260（大号）内良好排版，不横向溢出（子集选择已保证）。
- **NFR-5 可发现性**：渲染态有视觉提示表明"点击编辑"（hover 光标 text / 微提示）。
- **NFR-6 性能**：渲染开销仅作用于启用 Markdown 的便签；纯文本便签零渲染开销。
- **NFR-7 保存语义不变**：编辑态防抖 500ms + 失焦保存逻辑不变（持久化的是源码 content，非渲染 HTML）。

## 验收（Acceptance）

- **A-1** 新建便签 → 默认纯 textarea（Markdown 关），打开即可打字（延续 onMount focus）。
- **A-2** 启用 Markdown → 渲染态显示 HTML；点内容 → 进编辑态显示源码；失焦 → 回渲染态 + 保存源码。
- **A-3** 加粗/斜体/列表/标题等子集语法正确渲染；表格/代码块/图片/链接语法安全降级（不渲染、不报错）。
- **A-4** 便签内输入 `<script>alert(1)</script>` 或原始 HTML → 不执行、不渲染（XSS 验证）。
- **A-5** 已完成列表中启用 Markdown 的便签渲染与便签本体一致。
- **A-6** 禁用 Markdown 的便签行为/视觉与 0.5.x 完全一致（回归）。
- **A-7** 多便签：A 编辑失焦 → A 回渲染；切到 B 编辑互不干扰。

## 范围边界（Scope）

**范围内**：per-note 开关 + 编辑/渲染切换（点内容进编辑）+ 精简子集渲染 + 已完成列表渲染 + XSS 安全。

**范围外（本 release 不做 / YAGNI）**：WYSIWYG 编辑、表格/代码块/图片/链接、全局 Markdown、编辑态语法高亮、Markdown 导出、跨便签模板、Markdown 帮助/语法文档页。

**留 design 阶段定**：marked 的具体 sanitize 策略（配置 escape vs 叠加 DOMPurify）、notes 表 `markdown` 字段存储细节（i64，类比 is_hidden）、toolbar 开关按钮图标/位置、渲染态 CSS 适配。

## 技术约束（spec 级）

- **db**：notes 表新增 `markdown` 字段（布尔，i64 存储，类比 is_hidden）。
- **前端**：noteView.svelte 增加渲染态 + 编辑/渲染状态机；completedView.svelte 渲染保持一致。
- **依赖**：marked（渲染库）；sanitize 策略 design 定。

## S/M/L 评级

**L 级**。

- (A) 设计复杂度：中-高——编辑/渲染状态机 + per-note + 渲染集成 + XSS。
- (B) 风险/影响面：中——跨 noteView + completedView + db schema + 新依赖。
- (C) 需求不确定性：低——grill 已收敛，决策清晰。

→ L 级流程：analysis(full，本文档) → design(full，ADR/HLD/LLD) → code → review → test(含 NFR)。
