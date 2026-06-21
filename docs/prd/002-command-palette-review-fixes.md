# PRD: 命令面板 Review 修复

## Problem Statement

命令面板功能 (#21-#26) 的代码审查发现了 6 个编码标准违规和 8 个 PRD 合规问题。这些问题包括：缺失错误处理、未验证的外部数据、调试日志遗留、测试覆盖率为零、多个 PRD 需求未实现或部分实现。

## Solution

修复所有编码标准违规，补齐 PRD 要求但未实现的功能，添加测试覆盖。

## User Stories

### 编码标准修复

1. 作为开发者，我希望 `useCommandData` 有完整的错误处理，以便 API 调用失败时用户能看到友好的错误提示
2. 作为开发者，我希望 `useStatePersistence` 对 localStorage 数据做 schema 验证，以便损坏数据不会导致应用崩溃
3. 作为开发者，我希望生产代码中没有 `console.log`，以便不泄露调试信息
4. 作为开发者，我希望 `useCommandData` 暴露 `error` 字段，以便 UI 层能区分加载中和加载失败
5. 作为开发者，我希望 `CommandPalette` 有视觉定制而非纯 shadcn 默认，以便符合设计质量标准
6. 作为开发者，我希望 `CommandPalette` 移到 `components/command-palette/` 目录，以便按 feature 组织文件

### PRD 合规修复

7. 作为用户，我希望按 ⌘, 能切换到设置视图，以便与 PRD User Story 20 一致
8. 作为用户，我希望命令面板中的"新建任务"操作能实际执行，而不是只打印日志
9. 作为用户，我希望命令面板中的"触发采集"操作能实际执行，而不是只打印日志
10. 作为用户，我希望命令面板中的"标记事件已处理"操作能实际执行，而不是只打印日志
11. 作为用户，我希望工作台视图显示今日待处理任务，以便快速了解工作优先级
12. 作为用户，我希望工作台视图显示最近事件，以便了解最新动态
13. 作为用户，我希望工作台视图显示待处理项数量，以便了解工作量
14. 作为用户，我希望工作台视图显示快速操作入口，以便快速执行常用操作
15. 作为用户，我希望工作台视图显示采集器状态，以便了解系统运行状况
16. 作为用户，我希望空状态根据我的使用进度动态调整，以便获得个性化的引导
17. 作为开发者，我希望 `PersistedState` 使用 `ViewId`、`FilterSource`、`SortField` 类型而非 `string`，以便获得类型安全
18. 作为用户，我希望应用记住侧边栏的折叠状态，以便保持一致的布局

### 测试覆盖

19. 作为开发者，我希望 `useCommandData` 有单元测试，以便验证数据加载和搜索功能
20. 作为开发者，我希望 `useStatePersistence` 有单元测试，以便验证状态保存和恢复
21. 作为开发者，我希望 `useKeyboardShortcuts` 有单元测试，以便验证快捷键匹配和触发
22. 作为开发者，我希望 `CommandPalette` 有集成测试，以便验证打开/关闭/搜索/执行流程

## Implementation Decisions

### 模块变更

1. **useCommandData** — 添加 `try/catch`、`error` 状态字段、搜索防抖
2. **useStatePersistence** — 添加 localStorage schema 验证、使用具体类型替代 `string`
3. **MainWindow** — 移除 `console.log`、实现 `handleCommandAction` 实际逻辑
4. **CommandPalette** — 移到 `components/command-palette/`、视觉定制
5. **Sidebar** — 集成 `sidebarCollapsed` 持久化
6. **DashboardView** — 实现工作台内容（今日任务、最近事件、快速操作等）
7. **EmptyState** — 实现基于使用进度的动态引导

### 测试模块

1. `src/hooks/useCommandData.test.ts` — 数据加载、搜索、错误处理
2. `src/hooks/useStatePersistence.test.ts` — 状态保存、恢复、默认值、schema 验证
3. `src/hooks/useKeyboardShortcuts.test.ts` — 快捷键匹配、触发、输入框行为
4. `src/components/command-palette/CommandPalette.test.tsx` — 打开/关闭、搜索、命令执行

### 测试原则

- 只测试外部行为，不测试实现细节
- 测试用户可见的功能，不测试内部状态
- 测试边界情况：空状态、错误状态、加载状态
- 测试覆盖率目标：≥ 80%

## Out of Scope

1. **AI 推荐功能** — 作为未来迭代方向
2. **插件化架构** — 作为未来扩展方向
3. **高级动画** — 弹簧物理动画作为体验提升

## Further Notes

### 参考资源

- **原始 PRD**: `docs/prd/001-command-palette-integration.md`
- **领域模型**: `CONTEXT.md`
- **编码标准**: `.claude/rules/common/coding-style.md`
- **设计质量标准**: `.claude/rules/web/design-quality.md`

### 验收标准

#### 编码标准
- [ ] useCommandData 有 try/catch + error 状态
- [ ] useStatePersistence 有 localStorage schema 验证
- [ ] 无 console.log 在生产代码中
- [ ] CommandPalette 有视觉定制
- [ ] 文件按 feature 组织

#### PRD 合规
- [ ] ⌘, 可切换设置视图
- [ ] 命令面板操作可实际执行
- [ ] 工作台显示今日任务、最近事件、快速操作
- [ ] 空状态动态引导
- [ ] PersistedState 使用具体类型
- [ ] 侧边栏折叠状态持久化

#### 测试覆盖
- [ ] 测试覆盖率 ≥ 80%
- [ ] 无 TypeScript 错误
