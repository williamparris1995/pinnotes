# 更新日志 (Changelog)

PinNotes 各版本变更。安装包与自动更新清单(`latest.json`)发布在 [GitHub Releases](https://github.com/williamparris1995/pinnotes/releases)。

## 0.6.0

- **便签 Markdown 格式化**:每条便签可单独开启 Markdown(per-note opt-in,默认关闭;旧便签行为/视觉零变化)。
- 编辑/渲染切换:开启后默认渲染态——加粗 / 斜体 / 删除线 / 标题(#~###) / 有序·无序列表 / 行内代码 / 引用 / 分隔线 精简子集;点内容进源码编辑、失焦回渲染并保存源码。
- 渲染管道 marked + DOMPurify 白名单,排除表格 / 图片 / 链接 / 脚本(XSS 安全)。
- 已完成列表维持纯文本单行截断(不渲染 Markdown),保紧凑布局。
- 向后兼容:幂等 ALTER TABLE migration,0.5.x 老库平滑升级、既有便签 markdown 默认关。

## 0.5.1

- **质量基建**:无用户可见行为变化,加固测试覆盖与代码质量。
- **根 README**:补全 i18n 中英文 + 自动更新(OTA)两节(此前漏提)。
- **测试覆盖**:前端 Vitest 5→20 用例(新增 i18n.svelte.ts 全测 + 3 视图边界交互);Rust cargo 测试 14→27(db row_to_note 往返保真、snooze/geometry 边界、commands `lang` 分支)。
- **重构(行为不变)**:db 错误转换样板收敛(`run`/`run_exec`/`to_str`)+ `SELECT *` 改显式列名;4 色统一到 `theme.css` `var()`(顺带修黄色在便签窗与已完成列表不一致的视觉 bug)。
- **CI/工具链**:新增 PR/测试 CI(.github/workflows/ci.yml)+ pre-commit hook。

## 0.5.0

- **国际化(i18n)**:中英文切换。**默认英文**;老用户升级保留中文。语言选择器在设置页。
- 轻量自研 i18n(无第三方依赖,`src/lib/i18n.svelte.ts` + en/zh 字典):便签 / 已完成 / 设置 / 托盘菜单随语言即时翻转。
- Rust:欢迎便签内容 + aux 窗口标题(Completed / Settings)按语言。
- 设置页顺序调整:**开机自启 / Launch on startup** 移到第一项。
- 发版说明改为自动从 CHANGELOG.md 提取本版本内容写入 Release 正文(此前是指向 CHANGELOG 的链接)。

## 0.4.3

- **更新体验**:点击"更新"后实时显示**下载进度**(更新中… X%),并**防止重复点击**(按钮即时禁用 + 后端标志兜底——连点或重开菜单都不会触发第二次下载/安装)。
- 托盘菜单标题下 slogan 更新为「让重要的事,一直留在眼前」。
- 新增本更新日志。

## 0.4.2

- **品牌应用图标**:便签 + 图钉(Open Design 设计),用于菜单 Logo、窗口图标与安装包图标(替换默认 Tauri 图标)。
- 菜单版本号移到「PinNotes」标题旁边。

## 0.4.1

- 菜单标题区显示**当前版本号**。
- 菜单底部新增**「检查更新」**按钮(手动触发,启动自动检查之外的入口)。

## 0.4.0

- **自动更新(OTA)**:接入 Tauri updater,**无需服务器**——检测 GitHub Release 上的新版本 → 下载 → 校验签名 → 安装 → 重启。
- 三平台(Windows / macOS / Linux)签名发布;启动时后台自动检查,有新版会在托盘菜单顶部提示。
- 新增 `tauri-plugin-updater`;签名密钥配置详见 `docs/adr/0002`。
- 注:macOS 未购买 Apple 开发者证书,首次运行需在 Gatekeeper 处「右键 → 打开」放行一次。

## 0.3.0

- **自定义 HTML 托盘菜单**:替代原生系统菜单,样式/图标完全可控。左/右键单击托盘均弹出菜单。
- 全局快捷键 **Ctrl+N** 新建便签。
