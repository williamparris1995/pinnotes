# Spec: Rust 测试覆盖加固（light · M 级）

## 范围（已确认：推荐范围）

### db.rs（最高价值）
- `row_to_note` 往返保真：create(`is_hidden=true`, `completed_at=Some`, `hidden_until=Some`) → get 全字段相等（验证 i64↔bool + Option 转换）
- `update_position` / `update_content` 往返
- `delete` 后 get → None
- get 不存在 → None

### snooze.rs
- `should_repop(None)` → false
- `should_repop(t == now)` → true（边界 `<=`）

### geometry.rs
- 空 monitors → 原样返回
- 垂直越界（off top）
- 窗口大于工作区 → 贴 margin

### commands.rs
- `lang(db)` 4 分支（explicit en/zh / first_run→zh / 新→en）
- `get_setting`/`set_setting` 往返 + upsert 覆盖

## Scope 边界（不测，需 runtime / mock）
- AppHandle 相关命令（create / hide / repop / move / clamp_note / show_all / hide_all / copy / 自动更新）
- `SnoozeScheduler` schedule/cancel（需 tokio runtime + 时间）
- `clamp_note` adapter（核心 `clamp_into_work_area` 已测）

## Acceptance
- `cargo test` 全绿；用例数 14 → ~26+。
