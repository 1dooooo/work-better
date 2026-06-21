# Command Palette Prototype

## Question

What should the command palette look like and how should it work?

## Variants

### Variant A: Compact List
- 紧凑列表布局，所有功能平铺展示
- 导航、操作、搜索结果分组显示
- 快捷键提示直接显示在命令旁边

### Variant B: Categorized with Icons
- 分类图标布局，功能按类别分组
- 每个命令有独立的图标容器
- 包含 AI 推荐功能（生成今日报告）
- 视觉更丰富，信息层级更清晰

### Variant C: Minimal with Sections
- 极简分段布局，突出搜索和常用操作
- "最近使用"分组优先显示
- 更紧凑的布局，减少视觉噪音

## How to Run

访问 `http://localhost:1420/?view=prototype-command-palette` 即可查看原型。

命令面板直接显示在页面上（非 Dialog 弹出），方便查看和比较。

使用底部的切换器或键盘左右箭头在三个变体之间切换。

## Keyboard Shortcuts

- `⌘1-5` — 切换视图（事件、任务、时间线、报告、设置）
- `⌘,` — 打开设置
- `⌘N` — 新建任务
- `←` `→` — 在变体之间切换（底部切换器）

## Design Decisions to Validate

1. **信息架构**：命令应该如何分组？（导航 vs 操作 vs 搜索）
2. **视觉层级**：是否需要为不同类型的命令使用不同的视觉样式？
3. **快捷键展示**：快捷键提示应该放在哪里？如何格式化？
4. **AI 集成**：AI 推荐功能应该如何呈现？
5. **搜索结果**：搜索结果应该显示哪些信息？

## Next Steps

1. 用户选择偏好的变体（或组合多个变体的优点）
2. 将验证后的设计集成到主应用
3. 删除原型代码
