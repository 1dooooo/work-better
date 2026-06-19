---
title: 第三阶段 — 体验打磨
type: guide
domain: design
created: 2026-06-19
updated: 2026-06-19
status: active
---

# 第三阶段：体验打磨

对齐 P1-P2 趋势，打磨交互细节，提升应用质感。

## 前置条件

- [x] 第一阶段完成（技术债清理）
- [x] 第二阶段完成（视觉升级）

## 任务清单

### T3.1 全局键盘快捷键

**目标**：实现全面的键盘快捷键系统，匹配 Linear/Raycast 级别的键盘体验。

**快捷键规划**：

| 快捷键 | 功能 | 范围 |
|--------|------|------|
| `⌘K` | 打开命令面板 | 全局 |
| `⌘1` - `⌘5` | 切换视图 | 全局 |
| `⌘,` | 打开设置 | 全局 |
| `⌘N` | 新建任务 | 全局 |
| `⌘⇧N` | 新建事件 | 全局 |
| `Esc` | 关闭弹窗/返回 | 上下文 |
| `J` / `K` | 列表上下导航 | 列表焦点 |
| `Enter` | 展开/确认 | 列表焦点 |
| `Space` | 标记已读/切换 | 列表焦点 |
| `⌘A` | 全选 | 列表焦点 |
| `⌘⇧A` | 标记全部已读 | 列表焦点 |

**方案**：
1. 创建 `useKeyboardShortcuts` hook，管理全局和上下文快捷键
2. 在各视图中注册上下文快捷键
3. 在 Tooltip 和菜单中显示快捷键提示
4. 支持快捷键冲突检测

**验收标准**：
- [ ] 所有全局快捷键可用
- [ ] 列表视图支持 vim 风格导航（J/K/Enter/Space）
- [ ] 快捷键在 Tooltip 中显示
- [ ] 快捷键不与系统/Tauri 快捷键冲突

**涉及文件**：
- 新建 `src/hooks/useKeyboardShortcuts.ts`
- `src/components/layout/MainWindow.tsx`
- 各视图组件
- 各 Tooltip 组件

---

### T3.2 vim 风格列表导航

**目标**：在事件列表、任务列表等列表视图中实现 vim 风格的键盘导航。

**交互设计**：

```
当前焦点项高亮（左 border 或背景色微变）
J ↓  移动到下一项
K ↑  移动到上一项
Enter  展开/折叠当前项
Space  标记/取消标记
gg   跳转到列表顶部
G    跳转到列表底部
/    聚焦搜索框
Esc  退出列表导航模式
```

**方案**：
1. 创建 `useListNavigation` hook，管理焦点索引
2. 使用 `scrollIntoView` 确保焦点项可见
3. 焦点项有明确的视觉指示（左边框或背景变化）
4. 与 T3.1 的快捷键系统集成

**验收标准**：
- [ ] J/K 可在列表中移动焦点
- [ ] Enter 可展开/折叠项
- [ ] Space 可标记项
- [ ] gg/G 可跳转到顶/底
- [ ] 焦点项有清晰的视觉指示
- [ ] 滚动时焦点项保持可见

**涉及文件**：
- 新建 `src/hooks/useListNavigation.ts`
- `src/components/views/EventsView.tsx`
- `src/components/views/TasksView.tsx`
- `src/components/views/TimelineView.tsx`

---

### T3.3 弹簧物理动画

**目标**：使用弹簧物理动画替代线性 CSS 过渡，提升应用质感。

**应用场景**：

| 场景 | 动画类型 | 参数建议 |
|------|---------|---------|
| 列表项进入 | 渐入 + 上滑 | `stiffness: 300, damping: 30` |
| 卡片 hover | 轻微上浮 | `stiffness: 400, damping: 25` |
| 面板打开 | 从边缘滑入 | `stiffness: 300, damping: 28` |
| 弹窗打开 | 缩放 + 渐入 | `stiffness: 350, damping: 30` |
| 标签切换 | 内容交叉淡入 | `duration: 200ms` |

**方案**：
1. 安装 Framer Motion（或使用 CSS `@property` + spring 曲线）
2. 创建通用动画组件：`AnimatePresence`、`MotionCard`、`MotionList`
3. 在关键交互点应用弹簧动画
4. 尊重 `prefers-reduced-motion` 设置

**验收标准**：
- [ ] 列表项有进入/退出动画
- [ ] 卡片 hover 有弹簧反馈
- [ ] 面板/弹窗有流畅的打开动画
- [ ] `prefers-reduced-motion` 下禁用动画
- [ ] 动画不影响性能（60fps）

**涉及文件**：
- 新建 `src/components/ui/motion.tsx`（动画组件）
- 各视图和交互组件
- `package.json`（如需安装 Framer Motion）

---

### T3.4 毛玻璃效果

**目标**：在浮窗、弹窗、侧边栏应用 Glassmorphism 2.0 效果。

**应用场景**：

| 组件 | 效果 |
|------|------|
| MenuBar 浮窗 | `backdrop-filter: blur(20px)` + 半透明背景 |
| Dialog 弹窗 | `backdrop-filter: blur(12px)` + 半透明背景 |
| Sheet 侧边栏 | `backdrop-filter: blur(16px)` + 半透明背景 |
| DropdownMenu | `backdrop-filter: blur(10px)` + 半透明背景 |

**方案**：
1. 定义毛玻璃 Token：
   - `--glass-blur`: 模糊强度
   - `--glass-bg`: 半透明背景色
   - `--glass-border`: 半透明边框
2. 创建 `.glass` 工具类或 `glass` Tailwind 插件
3. 在目标组件上应用

**验收标准**：
- [ ] MenuBar 浮窗有毛玻璃效果
- [ ] Dialog/Sheet 有毛玻璃背景
- [ ] DropdownMenu 有毛玻璃效果
- [ ] 深色/浅色模式下效果都自然
- [ ] 性能无明显影响

**涉及文件**：
- `src/index.css` — 新增 glass Token
- `src/components/MenuBar.tsx`
- `src/components/ui/dialog.tsx`
- `src/components/ui/sheet.tsx`
- `src/components/ui/dropdown-menu.tsx`

---

### T3.5 空状态/加载状态统一

**目标**：为所有视图提供一致的空状态和加载状态设计。

**空状态设计模板**：

```
┌─────────────────────────────────────┐
│                                     │
│         [图标 - 48px, muted]        │
│                                     │
│      暂无事件                        │
│      当有新事件时会自动显示在这里     │
│                                     │
│      [开始采集]  (可选)              │
│                                     │
└─────────────────────────────────────┘
```

**加载状态**：骨架屏（Skeleton），匹配实际内容布局。

**方案**：
1. 创建 `EmptyState` 通用组件（图标 + 标题 + 描述 + 可选操作）
2. 创建 `Skeleton` 通用组件（匹配列表/卡片布局）
3. 在所有视图中集成

**验收标准**：
- [ ] 所有视图有空状态显示
- [ ] 所有视图有加载骨架屏
- [ ] 空状态设计风格统一
- [ ] 骨架屏匹配实际布局

**涉及文件**：
- 新建 `src/components/ui/empty-state.tsx`
- 新建 `src/components/ui/skeleton.tsx`
- 各视图组件

---

### T3.6 视图状态 URL 持久化

**目标**：将当前视图状态持久化到 URL，支持刷新保持和深度链接。

**方案**：
1. 使用 URL search params 存储当前视图：`?view=events`
2. 同步 URL 与视图状态
3. 支持浏览器前进/后退

**验收标准**：
- [ ] 刷新页面后保持当前视图
- [ ] URL 可直接跳转到指定视图
- [ ] 浏览器前进/后退正常工作

**涉及文件**：
- `src/components/layout/MainWindow.tsx`
- 可能需要 React Router 或自定义 URL 管理

## 执行顺序

```
T3.1 全局快捷键 ──→ T3.2 vim 列表导航（依赖 T3.1）
    │
    ├──→ T3.3 弹簧动画（可并行）
    │
    ├──→ T3.4 毛玻璃（可并行）
    │
    └──→ T3.5 空状态（可并行）
              │
              └──→ T3.6 URL 持久化
```

建议按 `T3.1 → T3.2 → T3.3 / T3.4 / T3.5 → T3.6` 顺序执行。

## 风险与注意事项

1. **快捷键冲突**：需检查与 Tauri 全局快捷键、系统快捷键的冲突
2. **动画性能**：弹簧动画需在低端设备上测试，确保 60fps
3. **毛玻璃性能**：`backdrop-filter` 在某些设备上可能有性能问题
4. **vim 导航**：需确保不影响文本输入场景

## 完成标志

- 所有 6 个任务的验收标准全部通过
- 键盘体验匹配 Linear/Raycast 水平
- 动画流畅且尊重用户偏好
- 所有视图有完整的空/加载状态
- URL 状态持久化正常工作

## 相关文档

- [UI 优化调研报告](ui-optimization-research.md)
- [第一阶段实现计划](ui-phase-1-technical-debt.md)
- [第二阶段实现计划](ui-phase-2-visual-upgrade.md)
- [设计体系](design-system.md)
