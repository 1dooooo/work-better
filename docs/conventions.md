---
title: 文档规范
type: index
domain: conventions
created: 2026-06-06
updated: 2026-06-06
status: active
---

# 文档规范

## 导航规则

1. 找不到信息 → 先查 [docs/index.md](index.md)
2. 任何目录有 `_index.md` → 先读它再深入
3. 不读取 `deprecated/` 下的任何文件
4. 单文件超过 300 行 → 考虑拆分并更新索引

## 文档类型

| 类型 | 标识 | 存放位置 | 用途 |
|------|------|---------|------|
| structural | 结构性文档 | `docs/` 独立目录 | 产品定义、架构设计、模块总览 |
| implementation | 实现文档 | `src/*/README.md` | 模块 API、组件说明、使用示例 |
| decision | 决策记录 | `docs/decisions/` | ADR 格式的架构决策 |
| guide | 操作指南 | `docs/guides/` | 开发流程、部署步骤、排错指南 |
| index | 索引文档 | 每个目录的 `_index.md` | 子目录导航 |

## 存放决策树

```
这个文档描述什么？
├── 产品/架构/模块总览 → docs/ 下对应目录（structural）
├── 某个模块的 API/接口/使用 → src/模块名/README.md（implementation）
├── 一个架构决策及其推理过程 → docs/decisions/（decision）
├── 怎么做某件事的步骤 → docs/guides/（guide）
└── 某个目录下文件的导航 → 该目录/_index.md（index）
```

## Frontmatter 模板

所有文档必须有 frontmatter：

```yaml
---
title: 文档标题
type: structural | implementation | decision | guide | index
domain: product | architecture | features | development | testing | conventions
created: YYYY-MM-DD
updated: YYYY-MM-DD
status: draft | active | deprecated
---
```

## 文件规范

- 单文件 ≤300 行，超出则拆分
- 索引文件 ≤50 行
- 文件命名：kebab-case（如 `event-model.md`）
- 索引文件命名：`_index.md` 或 `index.md`

## 生命周期

```
draft → active → deprecated
  ↑        ↓
  └── 修改 ─┘
```

- **draft**：草稿，agent 可读取但标注为未完成
- **active**：活跃文档，agent 正常读取
- **deprecated**：过时文档，agent 跳过不读取

### deprecated 处理

1. 移入所在目录的 `deprecated/` 子目录（主策略）
2. 或标记 frontmatter `status: deprecated` 并从 `_index.md` 中移除（辅策略）

## 维护约束

1. 新增文档 → 更新对应 `_index.md`
2. 文档与代码变更同 PR
3. 文档中的架构图、数据模型必须与代码实现一致
4. 模块文档中的接口定义必须与实际代码同步

## 触发条件

| 变更类型 | 需要更新的文档 |
|---------|--------------|
| 新增/移除功能 | 功能索引 + 对应模块文档 + 产品文档（如涉及方向变化） |
| 架构变更 | 架构总览 + 对应模块文档 |
| 新增/移除模块 | 架构总览 + docs/index.md |
| 产品方向调整 | 产品文档 + agent.md 核心思想（如涉及原则变化） |
| 数据模型变更 | 事件模型文档 + 对应模块文档 |

## 健康检查

定期检查：
- [ ] 所有 `_index.md` 中的链接指向存在的文件
- [ ] 所有 active 文档的 `updated` 距今不超过 90 天
- [ ] 没有文件超过 300 行
- [ ] 没有文件缺少 frontmatter
- [ ] `deprecated/` 中的文件不出现在任何活跃 `_index.md` 中
