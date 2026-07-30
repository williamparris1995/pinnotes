# Progress

> sydusx **当前位置指针**（resume 入口）。进度由嵌套目录 + checkbox 体现，本文件不聚合。

## 当前

- **Product Goal**：✅ [project/vision.md](project/vision.md)
- **项目基线**：✅ [project/](project/)
- **当前 Release**：0.5.1 质量基建 ✅ **已发版**（`v0.5.1` tag pushed, CI 构建发布中）→ [release-0.5.1/release.md](release-0.5.1/release.md)
- **当前 Sprint**：sprint-1 ✅ **全完成（4/4）** → [sprint-1/sprint.md](release-0.5.1/sprint-1/sprint.md)
- **下一步**：① 等 CI 构建完成 → 验证 GitHub Release（三平台安装包）→ ② 规划下一个 release：Markdown 支持（从 vision「范围外/YAGNI」移入范围）

## resume

release 0.5.1 已发版：手动测试通过 → commit `a6a3ae2`（文档状态同步）→ fast-forward merge 进 main → push `v0.5.1` annotated tag（触发 release.yml 三平台 CI）。等 CI 构建完成并生成 GitHub Release 后全流程收尾。下一个 release 候选：Markdown 支持（用户主动提议，当前在 vision「范围外/YAGNI」，需做范围决策 + 分解）。

## 推荐执行顺序

README → 前端测试 → Rust 测试 → 重构（重构依赖测试网，放最后）。
