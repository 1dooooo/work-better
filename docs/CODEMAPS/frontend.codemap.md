---
title: Frontend CODEMAP
type: codemap
domain: architecture
crate: frontend
created: 2026-06-12
updated: 2026-06-28
status: active
---

# Frontend CODEMAP

> **职责**：Tauri 桌面应用前端。React UI + Tauri 命令层（Rust ↔ TypeScript 桥接）。
> **对应文档**：[呈现层架构](../architecture/modules/presentation.md)

## 文件导航

### 应用入口 (`src/`)

| 文件 | 职责 |
|------|------|
| `main.tsx` | React 入口 |
| `App.tsx` | 路由分发：根据 `?view=` 参数渲染 MainWindow / CaptureWindow / MenuBar |
| `index.css` | 全局样式 |

### 布局组件 (`src/components/layout/`)

| 文件 | 职责 |
|------|------|
| `MainWindow.tsx` | 主窗口布局（侧边栏 + 内容区） |
| `Sidebar.tsx` | 侧边栏导航 |

### 顶层组件 (`src/components/`)

| 文件 | 职责 |
|------|------|
| `MenuBar.tsx` | 菜单栏组合根（组合 hooks 和子组件） |
| `MainWindow.tsx` | 主窗口（旧版，可能被 layout/ 版本替代） |
| `Sidebar.tsx` | 侧边栏（旧版） |

### 菜单栏子组件 (`src/components/menubar/`)

| 文件 | 职责 |
|------|------|
| `index.ts` | 模块导出 |
| `MenuBarHeader.tsx` | 头部组件（应用名称、状态指示器） |
| `MenuBarContent.tsx` | 内容区（事件列表、待办、通知） |
| `MenuBarActions.tsx` | 底部操作栏（快捷按钮） |
| `EventListItem.tsx` | 事件列表项 |
| `TaskListItem.tsx` | 待办列表项 |
| `NotificationGroup.tsx` | 通知分组 |
| `StatusIndicator.tsx` | 系统状态指示器 |

### 自定义 Hooks (`src/hooks/`)

| 文件 | 职责 |
|------|------|
| `useMenuBarData.ts` | MenuBar 数据获取 |
| `useAutoRefresh.ts` | 自动刷新 |
| `useWindowResize.ts` | 窗口尺寸调整 |
| `useCommandData.ts` | 命令面板数据 |
| `useKeyboardShortcuts.ts` | 键盘快捷键管理 |
| `useListNavigation.ts` | 列表导航（方向键 + Enter） |
| `useStatePersistence.ts` | 状态持久化 |

### 功能视图 (`src/components/views/`)

| 文件 | 职责 | 对应功能 |
|------|------|---------|
| `DashboardView.tsx` | 仪表盘视图 | 默认首页，数据概览 |
| `EventsView.tsx` | 事件列表视图 | F1 采集结果展示 |
| `ProcessingView.tsx` | 处理状态视图 | F2 处理状态 |
| `TasksView.tsx` | 任务管理视图 | F4 任务管理 |
| `ReportsView.tsx` | 报告视图 | F5 报告生成 |
| `SettingsView.tsx` | 设置视图 | F6.3 设置管理 |
| `AuditView.tsx` | 审计视图 | F2.5 全链路审计 |
| `TimelineView.tsx` | 时间线视图 | 时间线展示 |

### 设置子组件 (`src/components/settings/`)

| 文件 | 职责 |
|------|------|
| `CollectorSettings.tsx` | 采集器配置（分组+子采集器开关、飞书认证） |
| `ModelSettings.tsx` | 模型配置（大小模型 API、参数、预算） |
| `StorageSettings.tsx` | 存储配置（Obsidian 路径、向量DB） |
| `ShortcutSettings.tsx` | 快捷键配置 |
| `FreshnessSettings.tsx` | 保鲜规则配置 |
| `ReportSettings.tsx` | 报告配置（模板、定时任务） |
| `DeveloperSettings.tsx` | 开发者设置 |

### 命令面板 (`src/components/command-palette/`)

| 文件 | 职责 |
|------|------|
| `CommandPalette.tsx` | 命令面板（cmdk 驱动，快捷操作入口） |

### 速记窗口 (`src/capture/`)

| 文件 | 职责 |
|------|------|
| `CaptureWindow.tsx` | 速记窗口（文本输入 + 图片粘贴） |

### UI 组件库 (`src/components/ui/`)

通用 UI 组件：badge, button, card, command, dialog, dropdown-menu, empty-state, glass-card, glass-panel, input, input-group, label, motion, scroll-area, select, separator, sheet, sonner, switch, tabs, textarea, tooltip。

### 工具函数 (`src/lib/`)

| 文件 | 职责 |
|------|------|
| `tauri.ts` | Tauri API 调用封装 |
| `utils.ts` | 通用工具函数（cn 等） |

### 测试文件

| 文件 | 职责 |
|------|------|
| `src/components/views/EventsView.test.tsx` | 事件视图测试 |
| `src/components/views/TasksView.test.tsx` | 任务视图测试 |
| `src/components/views/ReportsView.test.tsx` | 报告视图测试 |
| `src/components/views/SettingsView.test.tsx` | 设置视图测试 |
| `src/components/views/TimelineView.test.tsx` | 时间线视图测试 |
| `src/components/MenuBar.test.tsx` | 菜单栏测试 |
| `src/components/Sidebar.test.tsx` | 侧边栏测试 |
| `src/components/settings/*.test.tsx` | 设置子组件测试 |
| `src/capture/CaptureWindow.test.tsx` | 速记窗口测试 |
| `src/lib/store.test.tsx` | 状态管理测试 |
| `src/lib/utils.test.ts` | 工具函数测试 |

### Tauri 命令层 (`src-tauri/src/`)

| 文件 | 职责 | 对应前端调用 |
|------|------|------------|
| `main.rs` | Tauri 应用入口 | — |
| `lib.rs` | Tauri 插件注册 | — |
| `commands/mod.rs` | 命令模块导出 | — |
| `commands/collect.rs` | 采集命令 | 触发采集、获取采集状态 |
| `commands/collectors.rs` | 采集器管理命令 | 采集器列表、开关、健康检查 |
| `commands/events.rs` | 事件查询命令 | 事件列表、详情、过滤 |
| `commands/capture.rs` | 速记捕获命令 | 文本/图片/截图输入 |
| `commands/tasks.rs` | 任务管理命令 | 任务 CRUD、状态流转 |
| `commands/audit.rs` | 审计查询命令 | 审计记录查询 |
| `commands/settings.rs` | 设置命令 | 配置读写 |
| `commands/scheduler.rs` | 调度器命令 | 任务列表、启停、暂停 |
| `commands/notify.rs` | 通知命令 | 系统通知推送 |
| `commands/db.rs` | 数据库命令 | 数据库查询与管理 |
| `commands/window.rs` | 窗口命令 | 窗口创建、切换、管理 |

## 关键设计

- **App.tsx**：通过 URL 参数 `?view=` 分发到不同窗口（主窗口 / 速记 / 菜单栏）
- **Tauri 命令**：前端通过 `src/lib/tauri.ts` 调用 Rust 命令，实现前后端通信
- **组件测试**：使用 Vitest + Testing Library
- **CommandPalette**：基于 cmdk 的命令面板，快捷操作入口
- **Generated 类型**：`src/generated/` 由 ts-rs 自动生成，勿手动编辑

## 修改指引

| 你想改什么 | 先读 | 再改 |
|-----------|------|------|
| 修改主窗口布局 | `components/layout/MainWindow.tsx` | 修改布局组件 |
| 新增功能视图 | `components/views/` | 新建视图 + 更新 Sidebar 导航 |
| 修改设置页面 | `components/settings/` | 对应设置组件 |
| 新增 Tauri 命令 | `src-tauri/src/commands/mod.rs` | 新建命令文件 + 注册 |
| 修改速记窗口 | `capture/CaptureWindow.tsx` | 直接修改 |
| 修改前端调用后端的方式 | `lib/tauri.ts` | 修改封装函数 |
