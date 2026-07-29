# Design: 关键模块重构（包 A 后端收敛 + 包 B 颜色单一来源）

> L 级。**所有改动保证行为不变**（包 B 顺带修"黄色三值不一致"的视觉 bug）。
> review 报告：Agent 通读 17 候选（无架构级债务），本轮经 grill 选 A + B。

## 包 A：后端收敛

### ADR-0003a · 错误转换样板收敛（#1，~50 处）
- **决策**：db.rs 抽 `run<F,R>(db, f)` helper（取锁 → 执行 rusqlite 闭包 → 统一转 String），每个 repository 方法从「取锁 map_err + execute map_err」压成一行；新增 `pub(crate) fn to_str<E: ToString>(e) -> String`，commands.rs / autostart.rs 的 `.map_err(|e| e.to_string())` 改 `.map_err(to_str)`。
- **行为不变**：错误仍是 `String`（前端契约不变）；run 内部仍是 lock + execute + to_string，语义完全相同。不引入 thiserror（过度）。
- **风险**：低。

### ADR-0003b · `SELECT *` 改显式列名（#2）
- **决策**：db.rs 三处 `SELECT *` 改为显式 12 列，顺序对齐 `row_to_note` 的 0–11。
- **行为不变**：列集与顺序与原 `SELECT *`（schema 列序）完全一致；`row_to_note` 不变。消除「加列导致静默错位」的脆弱点。
- **风险**：低。

## 包 B：颜色单一来源 + 修 bug（#3）

### ADR-0003c · 颜色收敛到 theme.css `var()`
- **决策**：theme.css 的 `--c-{yellow,pink,blue,green}` 定为权威值，采用**便签窗当前值** `#ffe678 / #ffc0c8 / #a0cdff / #aae6a8`；noteView 的 `.note-*` 与 `.color-dot-*`、completedView 的 `.swatch-*` 全部改 `background: var(--c-*)`，删散落 hex。
- **修 bug**：黄色从三值（`#ffe678` / `#ffe078` / `#fff59d`）统一为 `#ffe678`；已完成列表黄色色块 `#ffe078 → #ffe678`。
- **行为不变**：除「已完成列表黄色色块」从 `#ffe078` 微调到 `#ffe678`（修不一致）外，其余颜色值不变（仅来源改 var()）。权威值取便签窗值（主展示面 + 色点已一致）。
- **风险**：低。

## 验证（NFR：行为不变）
- `cargo test`：db.rs 重构后 **27 用例须仍绿**
- `npm test`：**20 用例须仍绿**
- 手动：便签 4 色 + 已完成列表 4 色视觉一致（尤其黄色）
