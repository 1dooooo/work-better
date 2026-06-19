---
title: 第一阶段 — 技术债清理
type: guide
domain: design
created: 2026-06-19
updated: 2026-06-19
status: active
---

# 第一阶段：技术债清理

统一技术基础，为后续视觉升级扫清障碍。本阶段不改变视觉外观，只清理内部实现。

## 前置条件

无。本阶段是所有后续工作的基础。

## 任务清单

### T1.1 合并重复 Token 系统

**问题**：`src/index.css` 和 `src/styles/global.css` 定义了两套冲突的设计 Token。

**方案**：
1. 将 `global.css` 中独有的 Token 合并到 `index.css` 的 `@theme` 块
2. 确保所有 Token 在 Tailwind v4 的 `@theme` 中统一定义
3. 删除 `global.css` 中的 Token 定义（保留其他非 Token 内容如有）

**验收标准**：
- [ ] `index.css` 包含所有设计 Token（颜色、间距、圆角、阴影、字体）
- [ ] `global.css` 不再定义重复 Token
- [ ] 所有组件渲染无变化（视觉回归测试）

**涉及文件**：
- `src/index.css` — 合并目标
- `src/styles/global.css` — 清理源

---

### T1.2 清理遗留 BEM CSS

**问题**：`main-window.css`、`menu-bar.css`、`capture-window.css` 定义了大量未使用的 BEM 类名。

**方案**：
1. 扫描每个 CSS 文件中的类名，确认是否被任何组件引用
2. 删除未使用的类名和整个文件（如完全未使用）
3. 对 `capture-window.css` 保留（CaptureWindow 仍在使用）

**验收标准**：
- [ ] `src/styles/main-window.css` 已删除或精简到仅包含实际使用的样式
- [ ] `src/styles/menu-bar.css` 已删除或精简
- [ ] `src/styles/global.css` 已精简（T1.1 后）
- [ ] `capture-window.css` 保留（CaptureWindow 使用）
- [ ] 应用运行无样式丢失

**涉及文件**：
- `src/styles/main-window.css`
- `src/styles/menu-bar.css`
- `src/styles/global.css`

---

### T1.3 启用 Geist 字体

**问题**：`@fontsource-variable/geist` 已安装但从未引入或使用。

**方案**：
1. 在 `main.tsx` 或 `index.css` 中引入 Geist 字体
2. 更新 `--font-sans` Token，将 Geist 放在系统字体之前
3. 验证中文回退（PingFang SC / Microsoft YaHei）正常工作

**验收标准**：
- [ ] Geist 字体实际渲染（检查 DevTools 计算样式）
- [ ] 中文字符正确回退到系统中文字体
- [ ] 字体加载无 FOUT（Flash of Unstyled Text）

**涉及文件**：
- `src/index.css` — 更新 `--font-sans`
- `src/main.tsx` — 可能需要 import

---

### T1.4 统一硬编码颜色为语义 Token

**问题**：多个组件绕过主题系统，直接使用硬编码颜色。

**已知违规位置**：

| 组件 | 硬编码内容 |
|------|-----------|
| EventsView | `bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400` |
| AuditView | `bg-blue-500/10 text-blue-600 dark:text-blue-400` |
| ModelSettings | `border-green-200 bg-green-50 dark:border-green-800 dark:bg-green-950/30` |
| MenuBar | `text-[#0A84FF]`, `bg-[#1C1C1C]/95`, `text-[#8E8E93]` |

**方案**：
1. 在 `index.css` 中定义语义化颜色 Token：
   - `--color-info` / `--color-success` / `--color-warning`
   - 对应的 `--color-info-foreground` 等
2. 将所有硬编码颜色替换为语义 Token 对应的 Tailwind 类
3. MenuBar 的内联样式转为 Tailwind 类 + 语义 Token

**验收标准**：
- [ ] 无硬编码 hex 颜色（`#[0-9A-Fa-f]`）
- [ ] 无绕过主题的 Tailwind 颜色类（如 `bg-blue-100` 直接使用）
- [ ] 所有颜色通过 `bg-info`、`text-success` 等语义类引用
- [ ] 深色模式下所有颜色正确切换

**涉及文件**：
- `src/index.css` — 新增语义 Token
- `src/components/views/EventsView.tsx`
- `src/components/views/AuditView.tsx`
- `src/components/settings/ModelSettings.tsx`
- `src/components/MenuBar.tsx`
- 其他包含硬编码颜色的文件（需扫描确认）

---

### T1.5 统一 Tab 实现

**问题**：SettingsView 使用自定义 Tab 实现，AuditView 使用 shadcn Tabs。

**方案**：
1. 将 SettingsView 的自定义 Tab 迁移到 shadcn `Tabs` / `TabsList` / `TabsTrigger`
2. 保持视觉一致（可通过 variant 调整样式）

**验收标准**：
- [ ] SettingsView 使用 shadcn Tabs 组件
- [ ] 视觉效果与当前一致或更好
- [ ] Tab 切换功能正常

**涉及文件**：
- `src/components/views/SettingsView.tsx`

## 执行顺序

```
T1.1 合并 Token ──→ T1.4 统一颜色（依赖 T1.1 的语义 Token）
    │
    ├──→ T1.2 清理 CSS
    │
    ├──→ T1.3 启用字体
    │
    └──→ T1.5 统一 Tab
```

建议按 `T1.1 → T1.4 → T1.2 → T1.3 → T1.5` 顺序执行。

## 风险与注意事项

1. **视觉回归**：合并 Token 时需确保色值完全一致，避免肉眼可见的变化
2. **MenuBar 特殊性**：MenuBar 是独立窗口（tray），其样式可能有特殊约束
3. **CaptureWindow**：本阶段不处理 CaptureWindow 的 BEM → Tailwind 迁移（留到第二阶段）

## 完成标志

- 所有 5 个任务的验收标准全部通过
- 应用运行无视觉回归
- `pnpm tsc --noEmit` 无错误
- 无硬编码颜色残留
