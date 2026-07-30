# Progress

> sydusx **当前位置指针**（resume 入口）。进度由嵌套目录 + checkbox 体现，本文件不聚合。

## 当前

- **Product Goal**：✅ [project/vision.md](project/vision.md)
- **项目基线**：✅ [project/](project/)
- **当前 Release**：0.5.1 质量基建 → [release-0.5.1/release.md](release-0.5.1/release.md)
- **当前 Sprint**：sprint-1 ✅ **全完成（4/4）** → [sprint-1/sprint.md](release-0.5.1/sprint-1/sprint.md)
- **下一步**：手动测试应用（颜色 / 功能）→ 确认后发版（merge main + push `v0.5.1` tag → CI 三平台构建）

## resume

sprint-1 全完成 + release 0.5.1 收尾（version bump 0.5.1 + CHANGELOG + commit 到 `release-0.5.1` 分支 95fa3c7，pre-commit hook 验证通过）。仅剩手动测试应用 + 发版。

## 推荐执行顺序

README → 前端测试 → Rust 测试 → 重构（重构依赖测试网，放最后）。
