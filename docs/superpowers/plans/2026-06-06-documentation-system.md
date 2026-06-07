---
title: 文档体系规范实施计划
type: implementation
domain: conventions
created: 2026-06-06
updated: 2026-06-06
status: active
---

# 文档体系规范实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 建立索引驱动、渐进式披露的文档体系，约束防漂移，确保 agent 高效消费和维护文档。

**Architecture:** agent.md (≤10行入口) → docs/conventions.md (规范详情) → docs/index.md (领域索引) → 各领域 _index.md → 具体文档。结构性文档在 docs/，实现文档在 src/*/README.md，决策记录用 ADR 格式。

**Tech Stack:** Markdown, YAML frontmatter, Shell (CI checks)

---

## 文件结构总览

```
新建文件：
├── docs/conventions.md                ← 文档规范详情
├── docs/index.md                      ← 领域级总索引
├── docs/decisions/index.md            ← ADR 索引
├── docs/decisions/TEMPLATE.md         ← ADR 模板
├── docs/architecture/index.md         ← 架构子索引
├── docs/product/_index.md             ← 产品子索引
├── docs/features/_index.md            ← 功能子索引
├── docs/development/_index.md         ← 开发子索引
├── docs/testing/_index.md             ← 测试子索引
├── scripts/check-docs.sh              ← CI 文档检查脚本

修改文件：
├── agent.md                           ← 精简为 ≤10行实质内容
├── docs/architecture/overview.md      ← 添加 frontmatter
├── docs/architecture/modules/*.md     ← 添加 frontmatter（6个文件）
├── docs/product/overview.md           ← 添加 frontmatter
├── docs/features/index.md             ← 添加 frontmatter
├── docs/development/setup.md          ← 添加 frontmatter
├── docs/testing/*.md                  ← 添加 frontmatter（2个文件）
├── docs/testing/**/*.md               ← 添加 frontmatter（6个文件）
├── CONTRIBUTING.md                    ← 添加 frontmatter + 文档规范引用
├── README.md                          ← 添加 frontmatter
```

---

### Task 1: 创建 docs/conventions.md

**Files:**
- Create: `docs/conventions.md`

- [ ] **Step 1: 创建 docs/conventions.md**

```markdown
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
```

- [ ] **Step 2: 验证文件创建**

Run: `head -10 docs/conventions.md`
Expected: 显示 frontmatter 和标题

- [ ] **Step 3: 提交**

```bash
git add docs/conventions.md
git commit -m "docs: 创建文档规范 conventions.md"
```

---

### Task 2: 创建 docs/index.md（领域级总索引）

**Files:**
- Create: `docs/index.md`

- [ ] **Step 1: 创建 docs/index.md**

```markdown
---
title: 文档索引
type: index
domain: conventions
created: 2026-06-06
updated: 2026-06-06
status: active
---

# 文档索引

| 领域 | 入口 | 说明 |
|------|------|------|
| 产品 | [product/overview.md](product/overview.md) | 产品定义、用户场景、路线图 |
| 架构 | [architecture/index.md](architecture/index.md) | 四层架构、模块详情 |
| 决策 | [decisions/index.md](decisions/index.md) | ADR 决策记录 |
| 功能 | [features/index.md](features/index.md) | 功能清单与状态 |
| 开发 | [development/setup.md](development/setup.md) | 环境搭建与工作流 |
| 测试 | [testing/strategy.md](testing/strategy.md) | 测试策略与规范 |
| 规范 | [conventions.md](conventions.md) | 文档规范（本文档） |
```

- [ ] **Step 2: 验证**

Run: `cat docs/index.md | grep -c "|"`
Expected: 9（表头 + 分隔线 + 7 行数据）

- [ ] **Step 3: 提交**

```bash
git add docs/index.md
git commit -m "docs: 创建领域级总索引 index.md"
```

---

### Task 3: 创建 ADR 系统（docs/decisions/）

**Files:**
- Create: `docs/decisions/index.md`
- Create: `docs/decisions/TEMPLATE.md`

- [ ] **Step 1: 创建 docs/decisions/ 目录和 index.md**

```bash
mkdir -p docs/decisions
```

```markdown
---
title: 架构决策记录
type: index
domain: architecture
created: 2026-06-06
updated: 2026-06-06
status: active
---

# 架构决策记录 (ADR)

| 编号 | 决策 | 状态 | 日期 |
|------|------|------|------|
| — | 尚无决策记录 | — | — |

> 新增 ADR 请使用 [TEMPLATE.md](TEMPLATE.md) 模板，文件命名格式：`NNN-决策标题.md`
```

- [ ] **Step 2: 创建 ADR 模板**

```markdown
---
title: ADR 标题
type: decision
status: accepted | superseded | deprecated
date: YYYY-MM-DD
deciders: []
related: []
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

- [ ] **Step 3: 验证**

Run: `ls docs/decisions/`
Expected: `TEMPLATE.md` 和 `index.md`

- [ ] **Step 4: 提交**

```bash
git add docs/decisions/
git commit -m "docs: 创建 ADR 决策记录系统"
```

---

### Task 4: 创建架构子索引（docs/architecture/index.md）

**Files:**
- Create: `docs/architecture/index.md`

- [ ] **Step 1: 创建 docs/architecture/index.md**

```markdown
---
title: 架构文档索引
type: index
domain: architecture
created: 2026-06-06
updated: 2026-06-06
status: active
---

# 架构文档索引

| 模块 | 入口 | 说明 |
|------|------|------|
| 总览 | [overview.md](overview.md) | 四层架构设计、原则、数据流 |
| 采集层 | [modules/collection.md](modules/collection.md) | EventLog 采集器体系 |
| 处理层 | [modules/processing.md](modules/processing.md) | 分类器、模型路由、审核 |
| 存储层 | [modules/storage.md](modules/storage.md) | 三层存储架构 |
| 呈现层 | [modules/presentation.md](modules/presentation.md) | 菜单栏、主窗口、快捷捕获 |
| 定时任务 | [modules/scheduler.md](modules/scheduler.md) | 26 个定时任务调度 |
| 事件模型 | [modules/event-model.md](modules/event-model.md) | 核心数据结构定义 |
```

- [ ] **Step 2: 提交**

```bash
git add docs/architecture/index.md
git commit -m "docs: 创建架构子索引"
```

---

### Task 5: 为各子目录创建 _index.md

**Files:**
- Create: `docs/product/_index.md`
- Create: `docs/features/_index.md`
- Create: `docs/development/_index.md`
- Create: `docs/testing/_index.md`

- [ ] **Step 1: 创建 docs/product/_index.md**

```markdown
---
title: 产品文档索引
type: index
domain: product
created: 2026-06-06
updated: 2026-06-06
status: active
---

# 产品文档索引

| 文档 | 说明 |
|------|------|
| [overview.md](overview.md) | 产品定义、用户场景、功能方向、路线图 |
```

- [ ] **Step 2: 创建 docs/features/_index.md**

```markdown
---
title: 功能索引
type: index
domain: features
created: 2026-06-06
updated: 2026-06-06
status: active
---

# 功能索引

| 文档 | 说明 |
|------|------|
| [index.md](index.md) | 全功能点分层索引（108 个功能，F1-F6） |
```

- [ ] **Step 3: 创建 docs/development/_index.md**

```markdown
---
title: 开发文档索引
type: index
domain: development
created: 2026-06-06
updated: 2026-06-06
status: active
---

# 开发文档索引

| 文档 | 说明 |
|------|------|
| [setup.md](setup.md) | 开发环境搭建与工作流指引 |
```

- [ ] **Step 4: 创建 docs/testing/_index.md**

```markdown
---
title: 测试文档索引
type: index
domain: testing
created: 2026-06-06
updated: 2026-06-06
status: active
---

# 测试文档索引

| 文档 | 说明 |
|------|------|
| [strategy.md](strategy.md) | 测试策略总览、分层定义、技术栈 |
| [conventions.md](conventions.md) | 命名、组织、编写规范 |
| [infrastructure/harness.md](infrastructure/harness.md) | 测试夹具系统架构 |
| [infrastructure/mocking.md](infrastructure/mocking.md) | AI/飞书 Mock 策略 |
| [infrastructure/fixtures.md](infrastructure/fixtures.md) | 工厂模式、种子数据 |
| [layers/unit.md](layers/unit.md) | 单元测试编写指南 |
| [layers/integration.md](layers/integration.md) | 集成测试编写指南 |
| [layers/e2e.md](layers/e2e.md) | E2E 测试编写指南 |
| [ci.md](ci.md) | 流水线中的测试阶段 |
```

- [ ] **Step 5: 提交**

```bash
git add docs/product/_index.md docs/features/_index.md docs/development/_index.md docs/testing/_index.md
git commit -m "docs: 为各子目录创建 _index.md 索引"
```

---

### Task 6: 为所有现有文档添加 frontmatter

**Files:**
- Modify: `docs/architecture/overview.md`
- Modify: `docs/architecture/modules/collection.md`
- Modify: `docs/architecture/modules/processing.md`
- Modify: `docs/architecture/modules/storage.md`
- Modify: `docs/architecture/modules/presentation.md`
- Modify: `docs/architecture/modules/scheduler.md`
- Modify: `docs/architecture/modules/event-model.md`
- Modify: `docs/product/overview.md`
- Modify: `docs/features/index.md`
- Modify: `docs/development/setup.md`
- Modify: `docs/testing/strategy.md`
- Modify: `docs/testing/conventions.md`
- Modify: `docs/testing/infrastructure/harness.md`
- Modify: `docs/testing/infrastructure/mocking.md`
- Modify: `docs/testing/infrastructure/fixtures.md`
- Modify: `docs/testing/layers/unit.md`
- Modify: `docs/testing/layers/integration.md`
- Modify: `docs/testing/layers/e2e.md`
- Modify: `docs/testing/ci.md`
- Modify: `README.md`
- Modify: `CONTRIBUTING.md`

- [ ] **Step 1: 为 docs/architecture/overview.md 添加 frontmatter**

在文件最前面插入：

```yaml
---
title: 架构总览
type: structural
domain: architecture
created: 2026-06-06
updated: 2026-06-06
status: active
---
```

- [ ] **Step 2: 为 docs/architecture/modules/ 下的 6 个文件添加 frontmatter**

每个文件在最前面插入对应的 frontmatter。字段值：

| 文件 | title | type | domain |
|------|-------|------|--------|
| collection.md | 采集层 | structural | architecture |
| processing.md | 处理层 | structural | architecture |
| storage.md | 存储层 | structural | architecture |
| presentation.md | 呈现层 | structural | architecture |
| scheduler.md | 定时任务 | structural | architecture |
| event-model.md | 事件模型 | structural | architecture |

所有文件：`created: 2026-06-06`, `updated: 2026-06-06`, `status: active`

- [ ] **Step 3: 为 docs/product/overview.md 添加 frontmatter**

```yaml
---
title: 产品文档
type: structural
domain: product
created: 2026-06-06
updated: 2026-06-06
status: active
---
```

- [ ] **Step 4: 为 docs/features/index.md 添加 frontmatter**

```yaml
---
title: 功能索引
type: structural
domain: features
created: 2026-06-06
updated: 2026-06-06
status: active
---
```

- [ ] **Step 5: 为 docs/development/setup.md 添加 frontmatter**

```yaml
---
title: 开发环境搭建
type: guide
domain: development
created: 2026-06-06
updated: 2026-06-06
status: active
---
```

- [ ] **Step 6: 为 docs/testing/ 下的 9 个文件添加 frontmatter**

| 文件 | title | type | domain |
|------|-------|------|--------|
| strategy.md | 测试策略 | structural | testing |
| conventions.md | 测试规范 | structural | testing |
| infrastructure/harness.md | 测试夹具 | structural | testing |
| infrastructure/mocking.md | Mock 系统 | structural | testing |
| infrastructure/fixtures.md | 测试数据 | structural | testing |
| layers/unit.md | 单元测试指南 | guide | testing |
| layers/integration.md | 集成测试指南 | guide | testing |
| layers/e2e.md | E2E 测试指南 | guide | testing |
| ci.md | CI 集成 | guide | testing |

所有文件：`created: 2026-06-06`, `updated: 2026-06-06`, `status: active`

- [ ] **Step 7: 为 README.md 添加 frontmatter**

```yaml
---
title: Work Better
type: structural
domain: product
created: 2026-06-06
updated: 2026-06-06
status: active
---
```

- [ ] **Step 8: 为 CONTRIBUTING.md 添加 frontmatter**

```yaml
---
title: 贡献指南
type: guide
domain: development
created: 2026-06-06
updated: 2026-06-06
status: active
---
```

- [ ] **Step 9: 验证所有文档都有 frontmatter**

Run: `for f in $(find docs/ README.md CONTRIBUTING.md -name "*.md" -not -path "*/superpowers/*"); do head -1 "$f" | grep -q "^---" || echo "MISSING: $f"; done`
Expected: 无输出（所有文件都有 frontmatter）

- [ ] **Step 10: 提交**

```bash
git add docs/ README.md CONTRIBUTING.md
git commit -m "docs: 为所有现有文档添加 frontmatter"
```

---

### Task 7: 精简 agent.md

**Files:**
- Modify: `agent.md`

- [ ] **Step 1: 重写 agent.md**

将当前 65 行的 agent.md 替换为 ≤10 行实质内容的轻量版本：

```markdown
# Agent Guide

**Work Better**：以 Obsidian 为中心的 AI 工作观察者。

## 准则

1. 观察者姿态——被动采集、主动整理
2. Obsidian 为中心——数据归用户所有
3. 自主但可干预——私有数据自主，共享数据需确认

## 文档底线规则

1. 新增/修改文档 → 必须有 frontmatter（见 [docs/conventions.md](docs/conventions.md)）
2. 新增文档 → 必须更新所在目录的 `_index.md`
3. 不读取 `deprecated/` 下的任何文件
4. 文档超过 300 行 → 拆分

## 文档体系

→ [文档规范](docs/conventions.md) | [文档索引](docs/index.md) | [ADR 决策记录](docs/decisions/)
```

- [ ] **Step 2: 验证行数**

Run: `wc -l agent.md`
Expected: ≤ 25 行（含空行）

- [ ] **Step 3: 提交**

```bash
git add agent.md
git commit -m "refactor: 精简 agent.md 为轻量入口（≤10行实质内容）"
```

---

### Task 8: 创建 CI 文档检查脚本

**Files:**
- Create: `scripts/check-docs.sh`

- [ ] **Step 1: 创建 scripts/check-docs.sh**

```bash
#!/bin/bash
# 文档健康检查脚本
# 用法: bash scripts/check-docs.sh

set -e

ERRORS=0
WARNINGS=0

echo "=== 文档健康检查 ==="
echo ""

# 1. Frontmatter 检查
echo "▶ 检查 frontmatter..."
for f in $(find docs/ README.md CONTRIBUTING.md -name "*.md" -not -path "*/superpowers/*" -not -path "*/deprecated/*" 2>/dev/null); do
  if ! head -1 "$f" | grep -q "^---"; then
    echo "  ❌ 缺少 frontmatter: $f"
    ERRORS=$((ERRORS + 1))
  fi
done
echo ""

# 2. 索引链接一致性
echo "▶ 检查索引链接..."
for index in $(find docs/ -name "_index.md" -o -name "index.md" | grep -v superpowers | grep -v decisions/index.md); do
  dir=$(dirname "$index")
  grep -oE '\[.*?\]\(([^)]+)\)' "$index" | grep -oE '\(([^)]+)\)' | tr -d '()' | while read link; do
    target="$dir/$link"
    if [ ! -f "$target" ]; then
      echo "  ❌ 死链接: $index → $link"
      ERRORS=$((ERRORS + 1))
    fi
  done
done
echo ""

# 3. 文件长度检查
echo "▶ 检查文件长度..."
for f in $(find docs/ -name "*.md" -not -path "*/superpowers/*" -not -path "*/deprecated/*" 2>/dev/null); do
  lines=$(wc -l < "$f")
  if [ "$lines" -gt 300 ]; then
    echo "  ⚠️  文件过长 ($lines 行): $f"
    WARNINGS=$((WARNINGS + 1))
  fi
done
echo ""

# 4. deprecated 一致性
echo "▶ 检查 deprecated 一致性..."
for f in $(find docs/ -path "*/deprecated/*" -name "*.md" 2>/dev/null); do
  basename_f=$(basename "$f")
  for index in $(find docs/ -name "_index.md" -o -name "index.md" | grep -v superpowers | grep -v deprecated); do
    if grep -q "$basename_f" "$index"; then
      echo "  ⚠️  deprecated 文件仍在索引中: $f → $index"
      WARNINGS=$((WARNINGS + 1))
    fi
  done
done
echo ""

# 总结
echo "=== 检查完成 ==="
echo "错误: $ERRORS | 警告: $WARNINGS"

if [ "$ERRORS" -gt 0 ]; then
  exit 1
fi
```

- [ ] **Step 2: 设置执行权限**

Run: `chmod +x scripts/check-docs.sh`

- [ ] **Step 3: 运行验证**

Run: `bash scripts/check-docs.sh`
Expected: 无错误，可能有警告（索引链接在索引文件创建前会报死链接）

- [ ] **Step 4: 提交**

```bash
git add scripts/check-docs.sh
git commit -m "ci: 创建文档健康检查脚本 check-docs.sh"
```

---

### Task 9: 更新 README.md 和 CONTRIBUTING.md 引用新规范

**Files:**
- Modify: `README.md`
- Modify: `CONTRIBUTING.md`

- [ ] **Step 1: 在 README.md 中添加文档规范引用**

在 README.md 的文档表格中添加一行：

```markdown
| 文档规范 | [docs/conventions.md](docs/conventions.md) | 导航规则、维护约束、生命周期 |
```

- [ ] **Step 2: 在 CONTRIBUTING.md 中添加文档规范引用**

在 CONTRIBUTING.md 的适当位置添加：

```markdown
## 文档规范

所有文档遵循 [文档规范](docs/conventions.md)：
- 必须有 frontmatter
- 新增文档需更新对应 `_index.md`
- 单文件不超过 300 行
- 文档与代码变更同 PR
```

- [ ] **Step 3: 提交**

```bash
git add README.md CONTRIBUTING.md
git commit -m "docs: 更新 README 和 CONTRIBUTING 引用新文档规范"
```

---

### Task 10: 最终验证与收尾

**Files:**
- None (verification only)

- [ ] **Step 1: 运行完整健康检查**

Run: `bash scripts/check-docs.sh`
Expected: 0 错误，0 警告

- [ ] **Step 2: 验证渐进式披露路径**

依次读取以下文件，确认每个都存在且内容正确：

```bash
cat agent.md | grep "docs/conventions.md"
cat docs/conventions.md | head -5
cat docs/index.md | head -10
cat docs/architecture/index.md | head -10
```

- [ ] **Step 3: 验证 agent.md 行数**

Run: `wc -l agent.md`
Expected: ≤ 25 行

- [ ] **Step 4: 最终提交**

```bash
git add -A
git commit -m "docs: 文档体系规范实施完成"
```
