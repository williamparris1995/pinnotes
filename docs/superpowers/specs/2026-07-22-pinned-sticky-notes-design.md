# PinNotes（置顶便签提醒）设计文档

- **日期**：2026-07-22
- **状态**：草案，待用户确认 → 进入实现计划
- **平台**：Windows 优先（依赖与代码选跨平台方案，预留 macOS/Linux）
- **技术栈**：Flutter Desktop
- **工作名**：PinNotes（可改）

## 1. 概述

一个常驻系统托盘的 Flutter 桌面应用，把"重要提醒"以**便签样式**显示在屏幕上。每条便签是一个独立、置顶、无边框、透明的窗口，可拖到任意位置并记住位置。点击"隐藏"只是短暂收起——只要任务没被标记"完成"，便签会**在设定的隐藏时长后重新弹出到它自己的位置**（"无法永久隐藏"）。只有标记完成才让它从桌面消失（进入"已完成"列表，可重新激活/复制/编辑/永久删除）。

## 2. 目标与非目标

**目标**
- 多条便签，各自独立置顶窗口，可拖动定位、记住位置。
- "无法永久隐藏"：隐藏 = 短暂 snooze，到点未完成必弹回原位。
- 托盘常驻 + 右键菜单管理；已完成列表独立窗口，支持重新激活/复制/编辑/永久删除。
- 数据本地持久化（SQLite），开机自启。

**非目标（v1 不做，YAGNI）**
- 不做云同步、多设备、账号体系。
- 不做便签的优先级 / 截止日期 / 定时时间（"重要"由它作为置顶便签存在来体现）。
- 不做富文本 / Markdown，仅纯文本。
- 不做全局热键（仅托盘菜单）。

## 3. 已确认的核心决策

1. **架构**：方案 A——多真实窗口，每条便签一个 OS 窗口（`desktop_multi_window`）。主控 isolate 为唯一数据真相源；便签窗口是无状态视图 + 事件上报器。
2. **重新弹出模型**：无法永久隐藏。隐藏 = snooze（默认 2 分钟，每条可配 1/2/5/10/30 分钟），到点未完成弹回原位，持续到完成。
3. **管理入口**：系统托盘右键菜单（新建 / 显示全部 / 已完成 / 设置 / 退出）+ 独立"已完成"窗口。
4. **持久化**：SQLite（`drift` + `sqlite3_flutter_libs`）。
5. **完成语义**：软删除（置 `completedAt`）+ 5 秒撤销；不硬删。
6. **开机自启**：默认开（`launch_at_startup`），设置可关。
7. **便签内容**：纯文本 + 4 色；无优先级/日期。
8. **平台**：Windows 优先，预留跨平台。

## 4. 架构

常驻托盘的单进程，分三层：

- **主控 Manager（主 isolate，无可视窗口）**：唯一数据真相源。负责 SQLite 存储、托盘菜单、snooze 计时调度、所有窗口的生命周期。任何状态变更先落库再驱动窗口。
- **便签窗口 NoteWindow（每条一个子窗口）**：`desktop_multi_window` 创建的置顶/无边框/透明子窗口。只渲染单条便签 UI、捕获交互，通过方法通道把事件（隐藏 / 完成 / 拖动结束 / 文本编辑）上报主控；自身不持有权威状态。
- **普通窗口（按需）**：`已完成窗口`、`设置窗口`——由主控从托盘触发创建，普通（非置顶）Flutter 窗口。
- **托盘 Tray**：`tray_manager` 常驻图标 + 右键菜单。

**设计原则**：所有状态与决策集中在主控；便签窗口是"无状态视图 + 事件上报器"。隐藏/弹回/位置记忆/软删除等逻辑因此可脱离 UI 单独单测。用依赖注入隔离"窗口/通道"平台层，使其可 mock。

## 5. 组件

| 组件 | 职责 |
|---|---|
| `NoteStore` | 通过 drift 读写 `notes` 表；提供 active/completed 查询、CRUD、原子事务。 |
| `WindowManager` | 创建/显示/隐藏/关闭/移动便签窗口；下发窗口属性（置顶/无边框/透明/跳过任务栏/位置/尺寸）。 |
| `SnoozeScheduler` | 维护"隐藏中便签"的定时器；到点若未完成则通知 `WindowManager` 在记录位置重新显示。 |
| `TrayController` | 注册托盘图标与菜单，把菜单动作转成对主控的调用。 |
| `CompletedController` | 驱动"已完成"窗口：列表查询 + 重新激活 / 复制 / 编辑 / 永久删除。 |
| `SettingsService` | 读写设置（snooze 默认时长、开机自启开关）。 |
| `NoteWindow`（Widget，每窗口一份） | 文本（双击编辑）、`隐藏`、`✓ 完成` 按钮；拖动标题区上报新坐标。 |
| `CompletedWindow` / `SettingsWindow`（Widget） | 对应普通窗口的 UI。 |

## 6. 数据模型（drift）

```dart
class Notes extends Table {
  TextColumn get id => text()();               // uuid
  TextColumn get text => text()();
  TextColumn get color => text()();            // 'yellow'|'pink'|'blue'|'green'
  RealColumn get x => real()();
  RealColumn get y => real()();
  RealColumn get w => real().withDefault(const Constant(220))();
  RealColumn get h => real().withDefault(const Constant(160))();
  IntegerColumn get snoozeMinutes => integer().withDefault(const Constant(2))();
  DateTimeColumn get createdAt => dateTime()();
  DateTimeColumn get completedAt => dateTime().nullable()();   // 非空 = 已完成
  BoolColumn get isHidden => boolean().withDefault(const Constant(false))();
  DateTimeColumn get hiddenUntil => dateTime().nullable()();
}
```

- **活跃** = `completedAt IS NULL`；**已完成** = `completedAt IS NOT NULL`。
- 设置单独存（`shared_preferences` 或一张 `kv` 表）：`defaultSnoozeMinutes`、`autoStart`。

## 7. 关键流程

- **新建**（托盘"新建"）：主控写一条记录，默认位置 = 主屏顶部居中、`snoozeMinutes` 取设置默认值 → `WindowManager` 创建对应置顶窗口。
- **隐藏 / snooze**：窗口点"隐藏"上报 → 主控置 `isHidden=true`、`hiddenUntil = now + snoozeMinutes`，隐藏该窗口 → `SnoozeScheduler` 排定定时器 → 到点若 `completedAt` 仍为空：在记录的 `(x,y)` 重新显示并清 `isHidden/hiddenUntil`；若已完成则不弹。
- **完成**：点 `✓` → 主控置 `completedAt`（软删除）、关闭窗口 → 以临时通知/浮层形式提供 ~5 秒"撤销"；撤销则清空 `completedAt` 并重显。
- **重新激活**（已完成窗口）：清 `completedAt`、`isHidden=false` → 回到活跃并立即创建窗口弹显。
- **复制**（已完成窗口）：以相同 `text/color` 新建一条**活跃**便签（新 id，默认位置）并弹窗；原已完成项保留。
- **编辑**（已完成窗口）：就地改 `text`，仍留在已完成列表。
- **永久删除**（已完成窗口）：从库中删除该行。
- **拖动**：便签窗口拖动结束上报新坐标 → 主控更新 `x,y`（重弹即回到新位置）。
- **显示全部**（托盘）：清空所有活跃便签的 `isHidden`，立即显示全部窗口。
- **启动**：主控加载所有活跃记录 → 对每条：若 `isHidden` 且 `hiddenUntil` 在未来 → 保持隐藏并排定重弹定时器；若 `hiddenUntil` 已过 → 立即显示（补弹）；否则正常显示。

> **前提**：应用需保持运行（托盘常驻）才能计时弹回；退出即停止提醒。开机自启默认开以保证持续提醒。

## 8. 窗口属性与依赖

**便签窗口属性**：`alwaysOnTop=true`、`titleBarHidden`、`backgroundColor=transparent`、`skipTaskbar=true`、`resizable=false`、尺寸按内容、位置 `(x,y)`。已完成/设置窗口为普通窗口。

**依赖（均跨平台）**：`desktop_multi_window`、`window_manager`、`tray_manager`、`drift` + `sqlite3_flutter_libs`、`path_provider`、`shared_preferences`、`launch_at_startup`、`provider`（或 `riverpod`）、`uuid`。

## 9. 设置与默认值

- `defaultSnoozeMinutes`：默认 2（新建便签继承；单条可改 1/2/5/10/30）。
- `autoStart`：默认 **开**，首次运行时通过 `launch_at_startup` 启用；可在设置关闭。

## 10. 错误处理

- **位置越界**：坐标以设备像素存储；换显示器 / 分辨率 / DPI 变化导致 `(x,y)` 落到不可见区域 → 启动与显示前按当前 DPI 换算并把坐标夹回最近可见显示器的工作区。
- **数据完整性**：drift 事务保证原子写；加载时校验，损坏/缺字段记录跳过并记日志，不阻断启动。
- **通道通信**：窗口↔主控方法通道失败 → 重试 + 日志；便签窗口逻辑保持极简，单窗口异常不影响其他窗口或主控。
- **计时**：snooze 用进程内 `Timer`；应用未运行时不弹（可接受，靠开机自启覆盖常态）。

## 11. 测试策略

- **重点单测（纯 Dart，脱离 UI/平台）**：`NoteStore` 的 active/completed 查询与 CRUD；`SnoozeScheduler` 的排程与到点判定；重弹判定（已完成不弹、未完成弹、补弹）；位置夹回算法。
- **Widget 测试**：`NoteWindow`（隐藏/完成/编辑按钮、状态）、`CompletedWindow`（列表 + 操作）。
- **手动测试清单**（平台相关，难自动化）：多窗口置顶/透明/拖动/隐藏重弹、托盘菜单、已完成窗口各操作、开机自启、重启后状态恢复、换显示器位置夹回。
- **可测性**：用依赖注入把 `WindowManager`/通道层抽象成接口，单测注入 mock。

## 12. 风险与前置 spike

- **主要风险**：Flutter 桌面多窗口生态（`desktop_multi_window`）成熟度——窗口生命周期、多窗口间通信、异常隔离。
- **前置 spike（写功能前必做）**：用 `desktop_multi_window` 开 **2 个**置顶、透明、无边框小窗，验证：各自可拖动、可隐藏、可在指定位置重新显示、互不影响。spike 通过再全面开发；若不稳，回退到"单一透明覆盖窗口"方案（方案 B）。

## 13. 范围之外 / 未来

云同步、多设备、优先级/截止日期/定时、富文本/Markdown、全局热键、提醒音、多显示器独立布局。这些在 v1 之后按需再议。
