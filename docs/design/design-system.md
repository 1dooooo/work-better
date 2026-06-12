---
title: Work Better 设计体系
created: 2026-06-08
updated: 2026-06-12
status: active
---

# 设计体系

Work Better 的 UI 设计规范，基于 shadcn/ui + Tailwind CSS v4 构建。

## 技术栈

| 层 | 选型 | 版本 |
|---|---|---|
| 组件库 | shadcn/ui (Base UI primitives) | latest |
| 样式引擎 | Tailwind CSS v4 | 4.3+ |
| 图标库 | Lucide React | latest |
| 通知 | Sonner | latest |
| 命令面板 | cmdk | latest |
| 色彩空间 | oklch | — |

## 色彩系统

### 设计原则

- 使用 oklch 色彩空间，感知均匀
- 浅色/深色双主题，通过 `data-theme` 属性切换
- 语义化命名，不直接引用色值

### 色板

#### 浅色主题

| Token | 用途 | oklch 值 |
|---|---|---|
| `--color-background` | 应用背景 | `oklch(98% 0 0)` |
| `--color-foreground` | 主文字 | `oklch(14% 0 0)` |
| `--color-card` | 卡片背景 | `oklch(100% 0 0)` |
| `--color-primary` | 主色调（蓝） | `oklch(45% 0.19 260)` |
| `--color-secondary` | 次要背景 | `oklch(92% 0 0)` |
| `--color-muted` | 静音背景 | `oklch(95% 0 0)` |
| `--color-muted-foreground` | 次要文字 | `oklch(45% 0 0)` |
| `--color-destructive` | 危险/错误 | `oklch(55% 0.22 25)` |
| `--color-border` | 边框 | `oklch(90% 0 0)` |
| `--color-success` | 成功 | `oklch(65% 0.2 145)` |
| `--color-warning` | 警告 | `oklch(70% 0.18 80)` |
| `--color-info` | 信息 | `oklch(65% 0.18 240)` |

#### 深色主题

深色主题通过 `data-theme="dark"` 自动切换，所有 token 值在 `@theme dark` 块中重新定义。

### 使用方式

```tsx
// Tailwind 类名
<div className="bg-background text-foreground">
<div className="bg-primary text-primary-foreground">
<div className="text-muted-foreground">

// 条件样式
<div className="bg-card dark:bg-card/80">
```

## 排版

### 字体栈

```css
--font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", "PingFang SC",
  "Hiragino Sans GB", "Microsoft YaHei", sans-serif;
--font-mono: "SF Mono", "Fira Code", "JetBrains Mono", monospace;
```

### 字号

| Token | 大小 | 用途 |
|---|---|---|
| `--text-xs` | 0.6875rem (11px) | 标签、徽章 |
| `--text-sm` | 0.8125rem (13px) | 次要文字 |
| `--text-base` | 0.875rem (14px) | 正文（桌面应用基准） |
| `--text-lg` | 1rem (16px) | 标题 |
| `--text-xl` | 1.125rem (18px) | 大标题 |
| `--text-2xl` | 1.25rem (20px) | 页面标题 |

## 间距

| Token | 值 | 用途 |
|---|---|---|
| `--spacing-1` | 0.25rem | 紧凑间距 |
| `--spacing-2` | 0.5rem | 内边距、小间隙 |
| `--spacing-3` | 0.75rem | 组件内边距 |
| `--spacing-4` | 1rem | 区块间距 |
| `--spacing-6` | 1.5rem | 大区块间距 |
| `--spacing-8` | 2rem | 页面边距 |

## 圆角

| Token | 值 | 用途 |
|---|---|---|
| `--radius-sm` | 0.25rem | 小元素（徽章） |
| `--radius-md` | 0.375rem | 按钮、输入框 |
| `--radius-lg` | 0.5rem | 卡片 |
| `--radius-xl` | 0.75rem | 大卡片、弹窗 |

## 阴影

| Token | 值 | 用途 |
|---|---|---|
| `--shadow-xs` | `0 1px 2px oklch(0% 0 0 / 0.04)` | 微阴影 |
| `--shadow-sm` | `0 1px 3px oklch(0% 0 0 / 0.06)` | 卡片默认 |
| `--shadow-md` | `0 2px 8px oklch(0% 0 0 / 0.08)` | 悬浮状态 |
| `--shadow-lg` | `0 4px 16px oklch(0% 0 0 / 0.12)` | 弹窗、下拉 |

## 组件清单

### 已安装的 shadcn/ui 组件

| 组件 | 文件 | 用途 |
|---|---|---|
| Button | `src/components/ui/button.tsx` | 按钮 |
| Badge | `src/components/ui/badge.tsx` | 标签、计数 |
| Card | `src/components/ui/card.tsx` | 卡片容器 |
| Dialog | `src/components/ui/dialog.tsx` | 模态对话框 |
| Sheet | `src/components/ui/sheet.tsx` | 侧边抽屉 |
| Input | `src/components/ui/input.tsx` | 文本输入 |
| Label | `src/components/ui/label.tsx` | 表单标签 |
| Select | `src/components/ui/select.tsx` | 下拉选择 |
| Tabs | `src/components/ui/tabs.tsx` | 选项卡 |
| Textarea | `src/components/ui/textarea.tsx` | 多行文本 |
| Switch | `src/components/ui/switch.tsx` | 开关 |
| Separator | `src/components/ui/separator.tsx` | 分隔线 |
| ScrollArea | `src/components/ui/scroll-area.tsx` | 滚动区域 |
| Tooltip | `src/components/ui/tooltip.tsx` | 提示气泡 |
| DropdownMenu | `src/components/ui/dropdown-menu.tsx` | 下拉菜单 |
| Command | `src/components/ui/command.tsx` | 命令面板 |
| Sonner | `src/components/ui/sonner.tsx` | Toast 通知 |

### 自定义组件

| 组件 | 文件 | 用途 |
|---|---|---|
| Sidebar | `src/components/layout/Sidebar.tsx` | 侧边栏导航 |
| MainWindow | `src/components/layout/MainWindow.tsx` | 主窗口布局 |

## 布局规范

### 主窗口

```
┌─────────────────────────────────────┐
│ ┌──────────┬────────────────────────│
│ │ Sidebar  │ Content Area           │
│ │ 200px    │ flex-1                 │
│ │          │                        │
│ │ Brand    │ ┌────────────────────┐ │
│ │ ──────── │ │ Header (border-b)  │ │
│ │ Nav      │ ├────────────────────┤ │
│ │  · 事件  │ │                    │ │
│ │  · 任务  │ │ Scroll Content     │ │
│ │  · 时间线│ │                    │ │
│ │  · 报告  │ │                    │ │
│ │  · 设置  │ │                    │ │
│ │ ──────── │ └────────────────────┘ │
│ │ Footer   │                        │
│ └──────────┴────────────────────────│
└─────────────────────────────────────┘
```

### 内容页头

```tsx
<header className="flex items-center justify-between border-b border-border px-6 py-4">
  <div className="flex items-center gap-3">
    <h1 className="text-lg font-semibold">页面标题</h1>
    <Badge variant="secondary" className="text-xs">计数</Badge>
  </div>
  <div className="flex items-center gap-2">
    {/* 操作按钮 */}
  </div>
</header>
```

## 图标规范

使用 Lucide React 图标库：

```tsx
import { CalendarDays, CheckSquare, Clock, BarChart3, Settings } from "lucide-react";

// 标准尺寸
<Icon className="h-4 w-4" />      // 16px - 按钮、导航
<Icon className="h-3.5 w-3.5" />  // 14px - 小按钮、标签
<Icon className="h-5 w-5" />      // 20px - 标题前缀
<Icon className="h-8 w-8" />      // 32px - 空状态
```

## 暗色模式

### 切换方式

```tsx
// 通过 data-theme 属性
document.documentElement.dataset.theme = "dark";
document.documentElement.dataset.theme = "light";
```

### Tailwind 中使用

```tsx
<div className="bg-background dark:bg-background/90">
<span className="text-foreground dark:text-foreground/80">
```

## 动画规范

### 使用 Tailwind 内置动画

```tsx
// 旋转（加载）
<Loader2 className="h-4 w-4 animate-spin" />

// 过渡
<div className="transition-colors hover:bg-muted">
<div className="transition-shadow hover:shadow-sm">
```

### 持续时间

| 场景 | 类名 |
|---|---|
| 快速响应 | `duration-150` |
| 标准过渡 | `duration-200` |
| 慢过渡 | `duration-300` |

## 添加新组件

```bash
# 安装 shadcn/ui 组件
pnpm dlx shadcn@latest add <component-name>

# 组件会自动安装到 src/components/ui/
```

## 紧凑型列表设计规范

### 设计理念

参考 Linear、GitHub、Raycast 的设计理念：
- **Linear**: 极简、高效、对齐精准
- **GitHub**: 图标驱动、紧凑布局
- **Raycast**: 高密度、键盘友好

### 核心原则

1. **信息密度优先**：减少 40-60% 的间距，提升信息展示密度
2. **视觉层次清晰**：使用颜色、字体大小、间距建立明确的层次
3. **渐进式披露**：默认显示关键信息，悬停显示详细操作
4. **对齐精准**：所有元素严格垂直居中，固定宽度列对齐

### Header 规范

```tsx
<header className="flex items-center justify-between border-b border-border px-5 py-3 min-h-[48px]">
  <div className="flex items-center gap-3">
    <h1 className="text-sm font-semibold text-foreground">页面标题</h1>
    <span className="text-[11px] text-muted-foreground bg-muted px-2 py-0.5 rounded-full">
      {count}
    </span>
  </div>
  <div className="flex items-center gap-2">
    {/* 操作按钮 */}
  </div>
</header>
```

**关键点**：
- 高度：`min-h-[48px]`
- 内边距：`px-5 py-3`
- 标题：`text-sm font-semibold`
- 计数：`text-[11px]` + `bg-muted` + `rounded-full`

### 列表项规范

```tsx
<div className="group flex items-center px-5 py-2 border-b border-border/50 hover:bg-muted/50 transition-colors cursor-pointer min-h-[40px]">
  {/* 状态指示器 */}
  <div className="w-1.5 h-1.5 rounded-full mr-3 flex-shrink-0 bg-primary" />

  {/* 类型标签 */}
  <span className="text-[10px] font-semibold px-1.5 py-0.5 rounded mr-2 flex-shrink-0 uppercase tracking-wider leading-none bg-blue-100 text-blue-700">
    MSG
  </span>

  {/* 来源 */}
  <span className="text-[11px] text-muted-foreground mr-3 flex-shrink-0 min-w-[80px]">
    来源名称
  </span>

  {/* 内容摘要 */}
  <span className="flex-1 text-xs text-foreground truncate mr-3">
    内容摘要...
  </span>

  {/* 标签 */}
  <div className="flex gap-1 mr-3 flex-shrink-0">
    <span className="text-[10px] px-1 py-0.5 bg-muted text-muted-foreground rounded">
      tag
    </span>
  </div>

  {/* 时间 */}
  <span className="text-[10px] text-muted-foreground flex-shrink-0 min-w-[60px] text-right mr-3">
    2分钟前
  </span>

  {/* 操作按钮（悬停显示） */}
  <button className="opacity-0 group-hover:opacity-100 transition-opacity ...">
    操作
  </button>
</div>
```

**关键点**：
- 行高：`min-h-[40px]`
- 内边距：`px-5 py-2`
- 状态指示器：`w-1.5 h-1.5 rounded-full`
- 类型标签：`text-[10px] uppercase tracking-wider`
- 来源：`text-[11px] min-w-[80px]`
- 内容：`text-xs truncate`
- 标签：`text-[10px]`
- 时间：`text-[10px] min-w-[60px] text-right`
- 操作按钮：`opacity-0 group-hover:opacity-100`

### 事件类型配色

```tsx
const EVENT_TYPE_CONFIG = {
  message: "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400",
  issue: "bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400",
  pr: "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400",
  document: "bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400",
};
```

### 时间显示规范

```tsx
const formatTime = (timestamp: string) => {
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return "刚刚";
  if (diffMins < 60) return `${diffMins}分钟前`;
  if (diffHours < 24) return `${diffHours}小时前`;
  if (diffDays < 7) return `${diffDays}天前`;
  return date.toLocaleDateString("zh-CN", { month: "short", day: "numeric" });
};
```

### Footer 规范

```tsx
<footer className="flex items-center justify-between px-5 py-2 border-t border-border bg-muted/30 text-[10px] text-muted-foreground">
  <span>共 {total} 条记录</span>
  <span>{unread} 条未读</span>
</footer>
```

### 按钮规范

#### Header 按钮

```tsx
// 主按钮
<Button size="sm" className="h-7 gap-1 text-[11px] px-3">
  <Icon className="h-3 w-3" />
  操作
</Button>

// 次按钮
<Button variant="outline" size="sm" className="h-7 gap-1 text-[11px] px-3">
  <Icon className="h-3 w-3" />
  操作
</Button>
```

#### 列表项操作按钮

```tsx
<button className="flex items-center gap-1 text-[10px] px-2 py-1 bg-background border border-border rounded text-muted-foreground opacity-0 group-hover:opacity-100 transition-all hover:bg-primary hover:text-primary-foreground hover:border-primary flex-shrink-0">
  <Icon className="h-3 w-3" />
  操作
</button>
```

### 间距系统

| 场景 | 间距 |
|---|---|
| Header 内边距 | `px-5 py-3` |
| 列表项内边距 | `px-5 py-2` |
| 元素间水平间距 | `mr-2` 或 `mr-3` |
| 标签间距 | `gap-1` |
| 垂直居中 | `flex items-center` |

### 字体系统

| 元素 | 字体大小 |
|---|---|
| 页面标题 | `text-sm` (14px) |
| 计数徽章 | `text-[11px]` |
| 列表项内容 | `text-xs` (13px) |
| 类型标签 | `text-[10px]` |
| 来源 | `text-[11px]` |
| 时间 | `text-[10px]` |
| 标签 | `text-[10px]` |
| Footer | `text-[10px]` |

## 相关文件

- `src/index.css` — Tailwind 主题配置
- `src/components/ui/` — shadcn/ui 组件
- `src/components/layout/` — 布局组件
- `src/lib/utils.ts` — 工具函数
- `ui-preview.html` — UI 设计预览（根目录）
