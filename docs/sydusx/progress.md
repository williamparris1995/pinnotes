# Progress

> sydusx **当前位置指针**（resume 入口）。进度由嵌套目录 + checkbox 体现，本文件不聚合。

## 当前

- **Product Goal**：✅ [project/vision.md](project/vision.md)
- **项目基线**：✅ [project/](project/)
- **当前 Release**：0.5.1 质量基建 → [release-0.5.1/release.md](release-0.5.1/release.md)
- **当前 Sprint**：sprint-1 ✅ **全完成（4/4）** → [sprint-1/sprint.md](release-0.5.1/sprint-1/sprint.md)
- **下一步**：release 0.5.1 收尾——版本号 0.5.0→0.5.1 三处同步（package.json / tauri.conf.json / Cargo.toml）+ CHANGELOG 一节，然后发版（推 `v*` tag → CI）

## resume

sprint-1 全完成：README（S）、前端测试（M，20 用例）、Rust 测试（M，27 用例）、重构（L，包 A+B，行为不变）。release 0.5.1 仅剩版本 bump + CHANGELOG + 发版。

## 推荐执行顺序

README → 前端测试 → Rust 测试 → 重构（重构依赖测试网，放最后）。
