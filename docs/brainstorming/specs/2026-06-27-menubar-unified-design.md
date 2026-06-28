# MenuBar 统一设计系统规范

**日期**: 2026-06-27
**状态**: 已批准
**方案**: C - 完全统一

## 背景

MenuBar（菜单栏弹窗）和 MainWindow（主窗口）的 UI 风格存在严重割裂：
- MenuBar 使用硬编码的深色玻璃态样式，完全绕过主题系统
- MainWindow 使用标准的主题 token 和共享 UI 组件
- 两个窗口看起来不属于同一个应用

## 设计目标

1. MenuBar 作为 MainWindow 的视图模式，完全统一设计系统
2. 保留玻璃态特色，但使用共享的设计 token
3. 支持明暗主题切换
4. 使用共享 UI 组件

## 核心决策

### 1. 主题系统统一

- 移除 `forcedTheme="dark"`，MenuBar 支持明暗主题切换
- 添加明暗两套玻璃态 token 到 `index.css`
- 所有颜色改用主题 token

### 2. 玻璃态 Token 设计

**深色模式 (Dark)**:
- `--color-glass-bg`: `oklch(18% 0.012 250 / 0.85)` — 带静谧蓝色调的半透明深色
- `--color-glass-border`: `oklch(70% 0.08 250 / 0.15)` — 蓝色调边框
- `--color-glass-accent`: `oklch(65% 0.15 250)` — 强调色
- `--glass-blur`: `16px`

**浅色模式 (Light)**:
- `--color-glass-bg`: `oklch(98% 0.005 250 / 0.8)` — 带极淡蓝调的半透明白色
- `--color-glass-border`: `oklch(30% 0.05 250 / 0.12)` — 浅色边框
- `--color-glass-accent`: `oklch(50% 0.18 250)` — 强调色
- `--glass-blur`: `12px`

### 3. 组件库扩展

**新建组件**:
- `GlassPanel` — 可复用的玻璃态容器，支持 `tray`、`sidebar`、`card`、`popover` 变体
- `GlassCard` — 事件/任务卡片，支持明暗主题

**复用现有组件**:
- Badge — 状态标签、计数器
- Button — 操作按钮
- ScrollArea — 滚动容器
- Tooltip — 提示信息
- Separator — 分隔线
- Skeleton — 加载状态

### 4. 代码结构重构

**目录结构**:
```
src/components/
├── menubar/
│   ├── index.ts              # 导出入口
│   ├── MenuBarHeader.tsx     # 头部：应用名 + 状态指示器 (~50 行)
│   ├── MenuBarContent.tsx    # 内容区：事件/任务/通知列表 (~120 行)
│   ├── MenuBarActions.tsx    # 底部：快捷操作按钮 (~60 行)
│   ├── EventListItem.tsx     # 事件列表项组件
│   ├── TaskListItem.tsx      # 任务列表项组件
│   ├── NotificationGroup.tsx # 通知分组组件
│   └── StatusIndicator.tsx   # 状态指示器组件
├── ui/
│   ├── glass-panel.tsx       # 新增
│   ├── glass-card.tsx        # 新增
│   └── ... (已有组件)
└── MenuBar.tsx               # 简化为 ~100 行的组合组件
```

**提取共享 Hooks**:
```
src/hooks/
├── useMenuBarData.ts    # MenuBar 数据获取
├── useAutoRefresh.ts    # 自动刷新逻辑
├── useWindowResize.ts   # 窗口尺寸调整
└── useTrayPosition.ts   # 托盘图标位置计算
```

### 5. 视觉统一规范

**圆角系统**:
- 8px — 按钮、标签
- 10px — 卡片
- 12px — 面板、弹窗

**字体栈**: `-apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Helvetica Neue', sans-serif`

**间距 Token**: 4px 倍数系统 (`--space-1` 到 `--space-12`)

**阴影系统**:
- `--shadow-sm` — 卡片/按钮
- `--shadow-md` — 弹窗/下拉
- `--shadow-lg` — 模态框

## 文件变更清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/index.css` | 修改 | 添加明暗玻璃态 token |
| `src/components/ui/glass-panel.tsx` | 新建 | 可复用的玻璃态面板组件 |
| `src/components/ui/glass-card.tsx` | 新建 | 可复用的玻璃态卡片组件 |
| `src/components/MenuBar.tsx` | 重构 | 使用主题系统 + 共享组件 |
| `src/components/menubar/*` | 新建 | MenuBar 子组件目录 |
| `src/hooks/useMenuBarData.ts` | 新建 | 数据获取 hook |
| `src/hooks/useAutoRefresh.ts` | 新建 | 自动刷新 hook |
| `src/hooks/useWindowResize.ts` | 新建 | 窗口尺寸调整 hook |
| `src/App.tsx` | 修改 | 移除 forcedTheme，统一 ThemeProvider |

## 参考资源

- [Raycast](https://raycast.com) — 玻璃态参考
- [Linear](https://linear.app) — 深色设计语言
- [shadcn/ui](https://ui.shadcn.com) — 组件库
- [Tauri v2 Tray Icons](https://v2.tauri.app/reference/config/) — 窗口配置

## 验收标准

1. MenuBar 和 MainWindow 使用相同的设计 token
2. 两个窗口都支持明暗主题切换
3. MenuBar 使用共享 UI 组件（Badge、Button 等）
4. MenuBar.tsx 主文件不超过 150 行
5. 所有新组件有 TypeScript 类型定义
6. 玻璃态效果在明暗模式下都正常显示
