---
title: 文档体系规范设计
type: decision
status: active
created: 2026-06-06
updated: 2026-06-06
---

# 文档体系规范设计

## 背景

随着项目迭代，过程和结果文档会极具膨胀。需要在项目初期建立文档规范，防止：

- **找不到东西**：文档太多太分散，不知道去哪里找信息
- **过时文档误导**：文档与代码不一致，不能信任
- **粒度失控**：不知道该写成一个文档还是拆成多个
- **体系偏离**：初期规范很好，后期逐渐偏离

### 关键约束

- **Agent 是主要消费者**：文档结构优化 agent 检索和读取效率
- **Agent 是主要维护者**：写入规范明确，agent 能自主维护
- **渐进式披露**：宁可多读几次，不要单个文件过大
- **文档先行**：先写文档再实现，最终状态必须有完备文档
- **保持新鲜**：优先更新而非废弃

## 设计

### 1. 入口体系

#### agent.md（≤10行，纯入口 + 最高准则）

```markdown
# Agent Guide

Work Better：以 Obsidian 为中心的 AI 工作观察者。

## 准则
- 观察者姿态：被动采集、主动整理
- Obsidian 为中心：数据归用户所有
- 自主但可干预：私有数据自主，共享数据需确认

## 文档底线规则（不可违反）
1. 新增/修改文档 → 必须有 frontmatter（见 docs/conventions.md）
2. 新增文档 → 必须更新所在目录的 _index.md
3. 不读取 deprecated/ 下的任何文件
4. 文档超过 300 行 → 拆分

## 文档体系
→ [文档规范与导航](docs/conventions.md)
→ [文档总索引](docs/index.md)
→ [ADR 决策记录](docs/decisions/)
```

#### docs/conventions.md（文档规范详情，≤80行）

包含：导航规则、维护约束、frontmatter 模板、文件规范、生命周期、健康检查。

#### docs/index.md（领域索引，≤20行）

```markdown
# 文档索引

| 领域 | 入口 | 说明 |
|------|------|------|
| 产品 | product/overview.md | 定义、场景、路线图 |
| 架构 | architecture/index.md | 四层架构、模块详情 |
| 决策 | decisions/index.md | ADR 决策记录 |
| 功能 | features/index.md | 功能清单与状态 |
| 开发 | development/setup.md | 环境搭建 |
| 测试 | testing/strategy.md | 测试策略 |
| 规范 | conventions.md | 文档规范 |
```

#### 渐进式披露路径

```
agent.md (10行)
  → docs/conventions.md (规范详情，80行)
  → docs/index.md (20行)
    → docs/architecture/index.md (15行)
      → docs/architecture/modules/collection.md (≤300行)
```

### 2. 文档分类与存放

#### 五种文档类型

| 类型 | 标识 | 存放位置 | 用途 |
|------|------|---------|------|
| structural | 结构性文档 | `docs/` 独立目录 | 产品定义、架构设计、模块总览 |
| implementation | 实现文档 | `src/*/README.md`（代码旁） | 模块 API、组件说明、使用示例 |
| decision | 决策记录 | `docs/decisions/` | ADR 格式的架构决策 |
| guide | 操作指南 | `docs/guides/` | 开发流程、部署步骤、排错指南 |
| index | 索引文档 | 每个目录的 `_index.md` | 子目录导航 |

#### 存放决策树

```
这个文档描述什么？
├── 产品/架构/模块总览 → docs/ 下对应目录（structural）
├── 某个模块的 API/接口/使用 → src/模块名/README.md（implementation）
├── 一个架构决策及其推理过程 → docs/decisions/（decision）
├── 怎么做某件事的步骤 → docs/guides/（guide）
└── 某个目录下文件的导航 → 该目录/_index.md（index）
```

#### 混合存放结构

```
项目根/
├── agent.md                          ← 入口（≤10行）
├── docs/
│   ├── index.md                      ← 总索引
│   ├── conventions.md                ← 文档规范
│   ├── product/                      ← structural
│   │   └── overview.md
│   ├── architecture/                 ← structural
│   │   ├── index.md                  ← 子索引
│   │   └── modules/
│   │       ├── collection.md
│   │       └── processing.md
│   ├── decisions/                    ← decision (ADR)
│   │   ├── index.md
│   │   └── 001-obsidian-as-primary-storage.md
│   ├── features/                     ← structural
│   │   └── index.md
│   ├── guides/                       ← guide
│   │   ├── adding-a-collector.md
│   │   └── model-router-upgrade.md
│   └── testing/                      ← structural + guide
│       ├── strategy.md
│       └── conventions.md
├── src/
│   ├── collectors/
│   │   ├── README.md                 ← implementation
│   │   └── ...
│   └── processing/
│       ├── README.md                 ← implementation
│       └── ...
└── 每个目录可有：
    _index.md                         ← 子目录索引
    deprecated/                       ← 过时文档
```

### 3. Frontmatter 规范

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

字段说明：
- `type` — agent 决定如何消费（索引型快速扫描，实现型详细阅读）
- `domain` — agent 在 docs/index.md 中定位目标领域
- `status` — agent 决定是否跳过（deprecated 直接跳过）
- `updated` — 判断文档新鲜度

### 4. ADR 决策记录

#### 格式模板

文件命名：`docs/decisions/NNN-决策标题.md`

```markdown
---
title: 使用 Obsidian 作为主要存储
type: decision
status: accepted | superseded | deprecated
date: 2026-06-06
deciders: [ido]
related: [docs/architecture/modules/storage.md]
---

## 背景
为什么需要做这个决策？面临什么约束？

## 决策
我们决定采用什么方案。

## 理由
为什么选这个而不是其他方案。

## 替代方案
考虑过但放弃的方案，以及放弃原因。

## 后果
这个决策带来的正面和负面影响。
```

#### ADR 触发条件

- 选择了技术栈（如数据库、AI 模型）
- 改变了架构分层或模块边界
- 引入了新的设计模式或约束
- 放弃了之前 ADR 中的决策（supersede）

### 5. 生命周期管理

#### 三阶段生命周期

```
draft → active → deprecated
  ↑        ↓
  └── 修改 ─┘
```

| 阶段 | 含义 | agent 行为 |
|------|------|-----------|
| draft | 草稿，正在编写 | 可读取，但标注为未完成 |
| active | 活跃文档，内容有效 | 正常读取 |
| deprecated | 过时文档 | 跳过，不读取 |

#### deprecated 处理方式

**策略 1（主）：目录内 deprecated/ 子目录**

```
docs/architecture/modules/
├── collection.md           ← active
└── deprecated/
    └── old-processor.md    ← deprecated
```

**策略 2（辅）：frontmatter 标记 + 索引排除**

文件留在原位，`_index.md` 中不列出，agent 通过 status 字段跳过。

### 6. 约束机制（四层防漂移）

| 层级 | 机制 | 触发时机 | 做什么 |
|------|------|---------|--------|
| L1 写入时 | frontmatter 校验 | agent 写入/修改文档时 | 检查必填字段 |
| L2 索引一致性 | 新增文档更新 _index.md | agent 新增文档后 | 同步更新索引 |
| L3 周期性 | 健康检查报告 | 定时任务 | 检查过时文档、缺失索引、超长文件 |
| L4 提交前 | CI 文档检查 | git commit / PR | 强制检查 |

#### CI 检查项

1. 所有 docs/ 和 src/ 下的 .md 文件必须有 frontmatter
2. 所有 _index.md 中的链接必须指向存在的文件
3. 没有文件超过 300 行（或有拆分标记）
4. deprecated/ 中的文件不出现在任何活跃 _index.md 中

#### 健康检查项

1. 所有 _index.md 中的链接是否指向存在的文件
2. 所有 active 文档的 updated 距今是否超过 90 天
3. 是否有文件超过 300 行
4. 是否有文件缺少 frontmatter
5. deprecated/ 中的文件是否仍在某个 _index.md 中被引用

### 7. 与现有规范的整合

当前 agent.md 中的自维护规范（触发表、维护流程、健康检查清单）整合方式：

1. **触发表** → 移入 `docs/conventions.md`，细化为 frontmatter 中 `domain` 字段驱动
2. **维护流程** → 移入 `docs/conventions.md`，增加"更新 _index.md"步骤
3. **健康检查清单** → 移入 `docs/conventions.md`，与 CI 检查脚本对齐

agent.md 只保留底线规则（4条）和文档索引链接。

## 核心原则总结

1. **渐进式披露**：agent.md → conventions → index → 领域索引 → 具体文档
2. **索引驱动**：所有导航通过 _index.md，不硬编码路径
3. **约束防漂移**：4层约束从自律到强制
4. **保持新鲜**：优先更新，deprecated 为最后手段
5. **文档先行**：先写文档，再实现，再更新文档
