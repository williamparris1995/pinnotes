# CI/CD

## 现状

- **发布 CI**：[.github/workflows/release.yml](../../.github/workflows/release.yml) — 推 `v*` tag（或手动 `workflow_dispatch`）→ Windows / macOS / Linux 三平台用 `tauri-action` 构建 + 签名 + 发 GitHub Release + 生成 `latest.json` 自动更新清单；正文从 [CHANGELOG.md](../../CHANGELOG.md) 提取。
- **OTA**：经 Tauri updater 无服务器直连 GitHub Release 的 `latest.json`（[ADR-0002](../../docs/adr/0002-auto-update-via-tauri-updater-and-github.md)）。签名公钥内嵌 tauri.conf.json，私钥 + 密码在 GitHub Secrets。

## Guardrails 现状

> skills 是提示；以下三层是"代码墙"，AI 无法绕过。

1. ✅ **PR / 测试 CI** — [.github/workflows/ci.yml](../../.github/workflows/ci.yml)：push 到 main / 任意 PR 时，在 ubuntu-22.04 + windows-latest 跑 `npm run check` + `npm test` + `cargo test`。
2. ✅ **pre-commit hook** — `.git/hooks/pre-commit`：提交前本地跑 check + Vitest + cargo test。⚠️ **不进版本控制**（git 不跟踪 `.git/hooks/`），换 clone 或协作者需重装，或迁移到 Husky（`.husky/pre-commit`）。
3. ⏳ **分支保护（待用户在 GitHub 设置）** — main 未要求 CI 绿才允许合并。平台层、无法代设：GitHub → Settings → Branches → Branch protection rules → main → Require status checks to pass（选 `test` job）。
