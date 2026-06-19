---
title: UI 优化调研报告
type: structural
domain: design
created: 2026-06-19
updated: 2026-06-19
status: active
---

# UI 优化调研报告

基于社区趋势分析与项目现状审计，形成的 UI 优化方向与实施路线。

## 项目现状审计

### 技术栈

| 层 | 选型 | 状态 |
|---|---|---|
| 框架 | React 19 + TypeScript | ✅ |
| 构建 | Vite 6 | ✅ |
| 桌面 | Tauri v2 | ✅ |
| 组件库 | shadcn/ui v4 + @base-ui/react | ✅ |
| 样式 | Tailwind CSS v4 (oklch) | ✅ |
| 图标 | lucide-react | ✅ |
| 命令面板 | cmdk | ✅ |
| 通知 | sonner | ✅ |
| 字体 | Geist variable（已安装未启用） | ⚠️ |

### 发现的 10 个问题

| # | 问题 | 严重度 | 类型 |
|---|------|--------|------|
| 1 | 重复设计 Token 系统（`index.css` vs `global.css`） | HIGH | 技术债 |
| 2 | 混合样式方案（Tailwind vs BEM CSS，迁移未完成） | HIGH | 技术债 |
| 3 | 硬编码颜色值绕过主题系统 | HIGH | 一致性 |
| 4 | Geist 字体安装但未使用 | MEDIUM | 技术债 |
| 5 | ProcessingView 存在但未注册到导航 | MEDIUM | 功能缺失 |
| 6 | 遗留 CSS 文件可能是死代码 | LOW | 技术债 |
| 7 | Tab 实现不一致（自定义 vs shadcn Tabs） | MEDIUM | 一致性 |
| 8 | 视图状态不持久（useState vs URL 参数） | MEDIUM | 体验 |
| 9 | CaptureWindow 使用独立 BEM CSS | MEDIUM | 一致性 |
| 10 | 空状态/加载状态不一致 | MEDIUM | 体验 |

### 问题分布

```
技术债 ████████ 4 个（#1, #2, #4, #6）
一致性 ████████ 4 个（#3, #7, #9, #10）
功能缺失 ████ 1 个（#5）
体验     ████ 1 个（#8）
```

## 社区趋势分析（2025-2026）

### 参考应用

| 应用 | 关键特征 |
|------|---------|
| Linear | 深色优先、命令面板、键盘密集、AI 集成 |
| Raycast | 毛玻璃浮窗、高密度列表、快捷键驱动 |
| Arc Browser | Bento 布局、空间层级、创新导航 |
| Notion | AI 原生界面、模块化内容、自适应布局 |
| Figma | 协作 UI、命令面板、实时反馈 |
| Things 3 | 空间分层、精致动画、macOS 原生感 |

### Top 10 趋势与关联度

| 优先级 | 趋势 | 关联度 | 当前状态 |
|--------|------|--------|---------|
| P0 | 命令面板进化 (Cmd+K + AI) | ⭐⭐⭐ | 已有 cmdk，需扩展 |
| P0 | 深色模式优先 + oklch | ⭐⭐⭐ | 已有基础，需翻转默认 |
| P0 | Bento Grid 模块化卡片布局 | ⭐⭐⭐ | 信息密集型仪表盘适配 |
| P1 | 键盘优先 UX | ⭐⭐⭐ | 桌面工具必备 |
| P1 | AI 原生界面模式 | ⭐⭐⭐ | 核心价值对齐 |
| P2 | Glassmorphism 2.0 毛玻璃 | ⭐⭐ | 菜单栏/弹窗适用 |
| P2 | 微交互 + 弹簧物理动画 | ⭐⭐ | 提升质感 |
| P2 | 空间层级 UI | ⭐⭐ | 浮动面板适用 |
| ✅ | shadcn/ui + Radix | — | 已采用 |
| ✅ | 可变字体 (Geist) | — | 已安装未启用 |

### 趋势详解

#### 1. 命令面板进化

主流生产力应用（Raycast、Linear、Arc、Notion、Figma）已将 Cmd+K 作为主导航。趋势方向：
- AI 驱动的自然语言命令
- 上下文感知的分组操作
- 内联预览与多步工作流

#### 2. 深色模式优先

开发者/生产力工具的标准。关键设计原则：
- 使用丰富深灰而非纯黑（Linear: `#0E0E10`，Raycast: 深色背景）
- 通过表面色差分层（非阴影）
- 1px 边框分隔（非粗线）

#### 3. Bento Grid 布局

受 macOS Widget 系统启发，使用不对称、变尺寸卡片组织信息密集型仪表盘。参考：Arc Browser、macOS System Settings。

#### 4. 键盘优先 UX

- 全局快捷键
- vim 风格列表导航
- 工具栏内联快捷键提示
- 渐进式快捷键教学

#### 5. AI 原生界面

- 内联 AI 建议（文本字段中）
- AI 生成摘要（仪表盘卡片中）
- 对话式命令面板
- 自适应布局

#### 6. Glassmorphism 2.0

`backdrop-filter: blur()` + 半透明表面 + 微妙渐变。适用于浮窗、侧边栏、弹窗。

#### 7. 微交互

弹簧物理动画（Framer Motion）用于状态变化：卡片悬浮、列表重排、骨架加载。

#### 8. 空间层级

Vision Pro 影响：浮动面板、视差滚动、z 轴分离（阴影 + 模糊 + 透明度叠加）。

## 优化路线

### 第一阶段：技术债清理

**目标**：统一技术基础，为后续视觉升级扫清障碍。

| 任务 | 优先级 | 预估工时 |
|------|--------|---------|
| 合并重复 Token 系统 | HIGH | 2h |
| 清理遗留 BEM CSS | HIGH | 2h |
| 启用 Geist 字体 | MEDIUM | 1h |
| 统一硬编码颜色为语义 Token | HIGH | 3h |
| 统一 Tab 实现（→ shadcn Tabs） | MEDIUM | 2h |

### 第二阶段：视觉升级

**目标**：对齐 P0 趋势，提升视觉品质。

| 任务 | 优先级 | 预估工时 |
|------|--------|---------|
| 翻转深色模式为默认 | HIGH | 1h |
| 精炼深色表面层级 | HIGH | 4h |
| Bento Grid 仪表盘设计 | HIGH | 8h |
| 命令面板 AI 操作扩展 | HIGH | 6h |
| CaptureWindow 统一到 Tailwind | MEDIUM | 3h |

### 第三阶段：体验打磨

**目标**：对齐 P1-P2 趋势，打磨交互细节。

| 任务 | 优先级 | 预估工时 |
|------|--------|---------|
| 全局键盘快捷键 | HIGH | 4h |
| vim 风格列表导航 | MEDIUM | 3h |
| 弹簧物理动画 | MEDIUM | 4h |
| 毛玻璃效果（浮窗/弹窗） | MEDIUM | 2h |
| 空状态/加载状态统一 | MEDIUM | 3h |
| 视图状态 URL 持久化 | MEDIUM | 2h |

## 参考资源

- [shadcn/ui 组件库](https://ui.shadcn.com)
- [Tailwind CSS v4 文档](https://tailwindcss.com)
- [Linear 设计语言](https://linear.app)
- [Raycast 扩展商店](https://raycast.com)
- [Mobbin 设计参考](https://mobbin.com)
- [Dribbble 趋势](https://dribbble.com)

## 相关文档

- [设计体系](design-system.md) — 当前 UI 设计规范
- [第一阶段实现计划](ui-phase-1-technical-debt.md)
- [第二阶段实现计划](ui-phase-2-visual-upgrade.md)
- [第三阶段实现计划](ui-phase-3-experience-polish.md)
