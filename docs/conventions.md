---
title: 文档规范
type: index
domain: conventions
created: 2026-06-06
updated: 2026-06-14
status: active
---

# 文档规范

## 导航规则

1. 找不到信息 → 先查 [docs/index.md](index.md)
2. 任何目录有 `_index.md` → 先读它再深入
3. 不读取 `deprecated/` 下的任何文件
4. 单文件超过 300 行 → 考虑拆分并更新索引

## 文档层级与优先级

Agent 在读取和应用规则时，遵循以下优先级顺序（从高到低）：

| 优先级 | 文档位置 | 说明 |
|-------|---------|------|
| 1（最高） | `agent.md`（项目根目录） | 项目核心指令，Agent 首先读取，不可被覆盖 |
| 2 | `.claude/agents/*.md` | Agent 角色定义，规定各 Agent 的职责和行为 |
| 3 | `docs/` | 项目业务与技术文档，包含架构、模块、流程等 |
| 4（最低） | `.claude/rules/common/` | 通用规则，提供默认值和最佳实践 |

### 冲突处理规则

当不同层级的文档对同一问题有不同规定时：

1. **高优先级文档覆盖低优先级文档** — 项目特定规则优先于通用规则
2. **同层级内，具体文档优先于概括文档** — 如 `docs/testing/architecture.md` 的具体规定优先于 `docs/conventions.md` 的概括描述
3. **冲突需显式声明** — 如果低优先级文档需要被覆盖，应在高优先级文档中明确说明

### 通用规则定位

`.claude/rules/common/` 中的规则是**默认值**，适用于所有项目。项目文档（`agent.md`、`docs/`）可以在以下情况下覆盖通用规则：

- 项目有特定的技术栈或架构约束
- 通用规则与项目实际实现不一致
- 项目需要更严格或更宽松的特定要求

### 示例

**示例 1：测试覆盖率**
- `.claude/rules/common/testing.md` 规定"最低测试覆盖率 80%"
- 如果项目实际目标不同，可在 `docs/conventions.md` 或 `agent.md` 中覆盖：
  ```
  项目测试覆盖率目标：70%（核心模块 80%）
  ```

**示例 2：文件大小限制**
- `.claude/rules/common/coding-style.md` 规定"文件最大 800 行"
- `docs/conventions.md` 规定"单文件 ≤300 行"
- 结果：遵循 `docs/conventions.md` 的 300 行限制（更高优先级）

**示例 3：Agent 行为**
- `.claude/agents/dev-agent.md` 定义 dev-agent 只负责编码
- `agent.md` 可能扩展其职责，加入特定的代码审查步骤
- 结果：遵循 `agent.md` 的扩展定义（更高优先级）

## 文档类型

| 类型 | 标识 | 存放位置 | 用途 |
|------|------|---------|------|
| structural | 结构性文档 | `docs/` 独立目录 | 产品定义、架构设计、模块总览 |
| codemap | 代码地图 | `docs/CODEMAPS/` | 概念层→代码层桥梁，开发时首选入口 |
| implementation | 实现文档 | `src/*/README.md` | 模块 API、组件说明、使用示例 |
| decision | 决策记录 | `docs/decisions/` | ADR 格式的架构决策 |
| guide | 操作指南 | `docs/guides/` | 开发流程、部署步骤、排错指南 |
| index | 索引文档 | 每个目录的 `_index.md` | 子目录导航 |

## 存放决策树

```
这个文档描述什么？
├── 产品/架构/模块总览 → docs/ 下对应目录（structural）
├── 某个模块的代码文件导航和职责映射 → docs/CODEMAPS/（codemap）
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
type: structural | codemap | implementation | decision | guide | index
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
| 新增/移除/重命名源文件 | 对应 CODEMAP（docs/CODEMAPS/） |
| 模块职责变更 | 对应 CODEMAP + 架构模块文档 |
| 产品方向调整 | 产品文档 + agent.md 核心思想（如涉及原则变化） |
| 数据模型变更 | 事件模型文档 + 对应模块文档 |
| 多 Agent 协作流程变更 | workflow spec + multi-agent-collaboration.md + agent 定义文件 |

## 健康检查

定期检查：
- [ ] 所有 `_index.md` 中的链接指向存在的文件
- [ ] 所有 active 文档的 `updated` 距今不超过 90 天
- [ ] 没有文件超过 300 行
- [ ] 没有文件缺少 frontmatter
- [ ] `deprecated/` 中的文件不出现在任何活跃 `_index.md` 中
