# PinNotes · 置顶便签提醒

一个常驻系统托盘的桌面应用，把"重要提醒"做成**便签**：独立、置顶的小窗口浮在屏幕
最上方，**点「隐藏」只是短暂收起——只要没标记「完成」，到点会自动弹回原位**（不夺焦
点），逼你把事做完。

跨平台（Windows / macOS / Linux），基于 **Tauri 2（Rust）+ Svelte 5**。

## 功能

- **置顶便签**：多条独立窗口，置顶显示；抓顶部 grip 拖动到任意位置并记住，重启仍在。
- **无法永久隐藏**：点「隐藏」按设定时长（1/2/5/10/30/60 分钟）收起，到点自动弹回原
  位、且**不抢占当前焦点**；只有「✓ 完成」才会让它消失。
- **就地编辑**：便签正文即输即存；支持 4 色（黄/粉/蓝/绿）、普通/大号尺寸、**每条独
  立的隐藏时长**。
- **完成撤销**：点「完成」先弹 5 秒撤销提示，避免误删。
- **系统托盘**：新建便签 / 显示全部 / 隐藏全部 / 已完成 / 设置 / 退出。
- **已完成列表**：重新激活（便签弹回）/ 删除。
- **开机自启**（默认开，可在设置关闭）。
- **SQLite 本地持久化**，所有便签、位置、状态都存本地。

## 下载

到 [Releases](https://github.com/williamparris1995/pinnotes/releases) 下载对应平台的安装包：

- Windows：`pinnotes_<version>_x64-setup.exe`（NSIS）或 `.msi`
- macOS：`.dmg`
- Linux：`.AppImage` / `.deb`

## 从源码构建

前置：[Node.js 22+](https://nodejs.org/)、[Rust (stable)](https://rustup.rs/)，以及各平台
的 Tauri 依赖：

- Windows：WebView2 运行时（Win10/11 一般自带）
- macOS：Xcode Command Line Tools
- Linux：`libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`

```bash
npm install          # 安装前端依赖
npm run tauri dev    # 开发模式运行
npm run tauri build  # 打包当前平台的安装包
```

## 技术栈

| 层 | 技术 |
|---|---|
| 桌面框架 | Tauri 2 |
| 后端 | Rust（窗口/托盘/计时/SQLite，单一真相源） |
| 前端 | Svelte 5 + Vite（薄视图，通过 `invoke`/`listen` 通信） |
| 持久化 | rusqlite（bundled SQLite） |
| 计时 | tokio（snooze 到点重弹） |

## 项目结构

```
src-tauri/src/   Rust 后端：db / snooze / geometry / window_manager / tray / autostart / commands
src/             Svelte 前端：App（路由）+ noteView / completedView / settingsView
docs/superpowers/specs/   设计文档
docs/superpowers/plans/   实现计划
```

## 发布

打 tag 即触发 GitHub Actions 在 Windows/macOS/Linux 三平台构建并发布到 Release：

```bash
git tag v0.1.0
git push origin v0.1.0
```

## 许可证

[MIT](LICENSE)
