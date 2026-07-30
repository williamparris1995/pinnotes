# Design：便签 Markdown 格式化

> feature: markdown · release 0.6.0 · sprint-1
> 产出阶段：design(L 级 full) · 引用 [spec.md](spec.md),不重复 FR/NFR
> UI 原型：project/prototype/[CURRENT](../../../project/prototype/CURRENT) → v1

## 决策记录（ADR）

### ADR-1 编辑/预览切换,排除 WYSIWYG
- **决策**:启用 markdown 的便签,编辑态=源码 textarea,非编辑态=渲染 HTML;点内容进编辑,失焦回渲染。
- **否决**:WYSIWYG(撞轻量零依赖 + 240×170 放不下格式 toolbar)。
- **依据**:spec 决策摘要;产品目标"瞥一眼看清"→ 默认渲染态。

### ADR-2 per-note opt-in
- **决策**:每条便签单独 opt-in markdown;未启用(含所有旧数据)维持纯 textarea。
- **依据**:短文本便签不需要 markdown;保"打开就打字";向后兼容零成本。

### ADR-3 渲染管道 = marked + DOMPurify
- **决策**:`content → marked.parse() → DOMPurify.sanitize(html, ALLOWED_TAGS=子集) → Svelte {@html}`。
- **依赖**:marked(渲染,~7KB gz) + DOMPurify(sanitize,~7KB gz)。
- **白名单标签**:`b/strong/i/em/del/s/h1/h2/h3/ul/ol/li/code/blockquote/hr/p/br`(对应 spec 子集 8 类语法)。
- **否决**:自研 sanitizer(XSS 绕过风险,安全领域不自研);marked 历史内置 `sanitize` 选项(已废弃)。
- **关键 fact**:marked 默认**原样吐出原始 HTML**(`<script>` → `<script>`),故 sanitize 不可省;DOMPurify 是白名单清洗,只放行子集标签。

### ADR-4 已完成列表不渲染(调整 spec FR-7)
- **决策**:completedView 维持纯文本单行截断(`white-space:nowrap;ellipsis`),**不渲染** markdown。渲染只在便签本体生效。
- **依据**:[completedView.svelte:104-112](../../../../../src/lib/completedView.svelte#L104-L112) 是紧凑单行摘要列表,块级 markdown(标题/列表)撑破布局;已完成项用户看的是内容文字+时间,格式不重要。
- **spec 同步**:FR-7 改为"已完成列表不渲染,维持纯文本截断"。

### ADR-5 db migration = 幂等 ALTER TABLE
- **决策**:`init()` 里 `CREATE TABLE IF NOT EXISTS`(新库含 `markdown` 列)之后,加幂等 migration:查 `PRAGMA table_info(notes)`,无 `markdown` 列则 `ALTER TABLE notes ADD COLUMN markdown INTEGER NOT NULL DEFAULT 0`。
- **依据**:[db.rs:9](../../../../../src-tauri/src/db.rs#L9) `CREATE TABLE IF NOT EXISTS` 对已存在的表不加新列;SQLite `ADD COLUMN` 不幂等(重复报错),须先查列存在。项目无 schema_version 机制,用 PRAGMA 探测最简。
- **默认值 0**:既有便签 markdown 关,行为零变化(NFR-3)。

## HLD — 编辑/渲染状态机

便签显示由两个正交维度决定:**markdown 开关**(per-note, 搞 db) × **编辑/渲染态**(前端)。

```
markdown 关  ─→  纯 textarea(现状,完全不变)
markdown 开  ─→  ┌─ 渲染态(默认):显示 sanitize 后的 HTML
                  └─ 编辑态:源码 textarea
                     转换:点内容 → 编辑态;失焦 → 渲染态(+ 保存源码)
```

**数据流**:
- 持久化的始终是**源码** `note.content`(渲染 HTML 不入库,NFR-7)。
- 渲染态 HTML 是**派生**:`$derived(markdown 开 ? sanitize(marked(content)) : null)`。
- markdown 开关:`toolbar 按钮 → set_markdown command → db.update_markdown → note.markdown`。
- 编辑/渲染态:纯前端 `$state`,不持久化(每次开窗默认渲染态;空内容例外,见下)。

**空内容处理**:markdown 开 + content 空 → 直接进编辑态(延续 onMount focus 空便签逻辑,不显示空渲染)。

**多窗口**:每条便签独立 webview,状态机各自独立,互不干扰(A-7)。

## LLD — 实现要点

### db.rs
- `Note` struct 加 `pub markdown: bool`。
- `init()`:CREATE TABLE 加 `markdown INTEGER NOT NULL DEFAULT 0` 列 + 幂等 ALTER migration(PRAGMA 探测)。
- `row_to_note`:加 `markdown: row.get::<_, i64>(12)? != 0`(索引 12,新末位列)。
- `NOTES_COLS`:加 `markdown`。
- `create`:INSERT 加 `markdown`(`n.markdown as i64`)。
- 新增 `update_markdown(db, id, on: bool)`(类比 `update_color`)。

### commands.rs
- 新增 `set_markdown_impl(state, id, on)` + `#[tauri::command] set_markdown`(类比 `set_color`)。
- `create_note_impl` / welcome note:markdown 默认 false(FR-2)。

### noteView.svelte(主战场)
- 加 `markdown` 开关按钮(toolbar,紧凑图标,如 `M↓` / markdown 标志)。
- 渲染态:`<div class="note-md" onclick={进编辑}>{@html renderedHtml}</div>`,`renderedHtml = $derived(sanitize(marked(note.content)))`。
- 编辑态:复用现有 `<textarea>`(源码)。
- 状态:`let editing = $state(false)`(markdown 开时);markdown 关时恒为编辑态(等价现状)。
- 点击渲染态 div → `editing=true` + focus textarea;textarea `onfocusout` → `editing=false` + `commit()`(复用现有保存)。
- 空内容 + markdown 开:onMount 设 `editing=true`(空便签直接编辑)。
- 可发现性(NFR-5):渲染态 div `cursor:text` + hover 微提示("点击编辑")。

### completedView.svelte
- **不改**(ADR-4)。维持 `{it.content}` 纯文本单行截断。

### 渲染态 CSS(NFR-4 小窗排版)
- `.note-md` 填充 `.note-body`,继承便签色背景 + 字体。
- markdown 元素紧凑化:`h1-h3` 缩小字号(margin 减)、`ul/ol` 缩进收窄、`blockquote` 左边框、`code` 行内底色、`hr` 细线——全部适配 240×170,不横向溢出。
- 配色用 `rgba(15,15,25,0.86)` 系(继承现有 textarea 文字色),与便签色调和谐。

### 依赖
- `npm i marked dompurify` + `@types/dompurify`(若需)。
- 封装 `src/lib/markdown.ts`:`renderMd(src): string`(marked.parse + DOMPurify.sanitize 白名单),单点出口,便于测试。

## 留待 code/test 阶段
- marked/DOMPurify 具体版本 pin + 配置参数微调。
- toolbar markdown 开关按钮的最终图标 + 激活态视觉。
- 渲染态 CSS 的精确像素值(code 时迭代)。
- XSS 测试 payload 清单(test 阶段,验证 `<script>`/`on*`/伪协议/嵌套绕过)。
