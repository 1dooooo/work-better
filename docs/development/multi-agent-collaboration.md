---
title: 多 Agent 协作开发规范
type: structural
domain: development
created: 2026-06-07
updated: 2026-06-28
status: active
---

# 多 Agent 协作开发规范

> **维护说明**：当新增 agent、修改协作流程、或调整 artifact 契约时更新本文档。

## 设计原则

本项目采用多 agent 协作开发模式。核心原则：

1. **职责单一** — 每个 agent 只做一件事，不越界
2. **文件契约通信** — agent 之间通过 `.workflow/artifacts/` 下的文件传递信息，不共享对话上下文
3. **Workflow 驱动** — 协作流程由 workflow spec 定义，不依赖 agent 自觉
4. **约束力梯度** — 不同工具提供不同强度的约束（hook > rule > prompt）

## Agent 职责矩阵

| Agent | 职责 | 写代码 | 写测试 | 审查代码 | 产品审查 | 规划建议 |
|-------|------|--------|--------|---------|---------|---------|
| **workflow-advisor** | 任务分析 + 执行计划 + 流程监督 | ❌ | ❌ | ❌ | ❌ | ✅ |
| **dev-agent** | 功能开发 + L1-L2 测试 | ✅ | ✅ (L1-L2) | ❌ | ❌ | ❌ |
| **test-agent** | 测试执行 + L4-L5 测试生成 | ❌ | ✅ (L4-L5) | ❌ | ❌ | ❌ |
| **review-agent** | 代码审查 + H3-H5 安全测试 | ❌ | ✅ (H3-H5) | ✅ | ❌ | ❌ |
| **product-reviewer** | 产品定义符合性审查 | ❌ | ❌ | ❌ | ✅ | ❌ |
| **validator** | Schema + 数据完整性验证 | ❌ | ❌ | ❌ | ❌ | ❌ |
| **system-inspector** | 系统健康 + 执行效率监督 | ❌ | ❌ | ❌ | ❌ | ❌ |
| **optimizer** | Agent prompt + workflow 优化 | ❌ | ❌ | ❌ | ❌ | ✅ |

### 职责边界规则

- dev-agent **不负责** L4-L5 测试和安全测试
- test-agent **不负责** 代码修改和审查
- review-agent **不负责** 代码修改和功能测试
- workflow-advisor **不负责** 任何代码/测试/审查工作，只提供规划建议
- 主 Agent 是最终的调用方和决策者

### 代码复用规范

**原则**：优先复用已有的核心函数，禁止重复实现。

当项目中已存在功能等价的函数、工具或模块时，dev-agent 必须复用，而非重新编写。重复实现会带来维护成本上升、行为不一致风险和测试覆盖分散等问题。

**验证流程**：

```
dev-agent 搜索已有实现
    │
    ├── 找到可复用代码 → 使用已有实现 + 补充测试
    │
    └── 确认无可用实现 → 正常开发
         │
         ▼
    review-agent 审查（检查复用合规性）
         │
         ▼
    test-agent 验证（确认复用后行为正确）
```

**review-agent 审查要点**：

review-agent 在代码审查中**必须**检查以下复用合规项：

| 检查项 | 说明 |
|--------|------|
| 核心函数复用 | 新代码是否与已有核心函数功能等价但未复用 |
| 执行路径一致性 | 复用同一核心函数的多处调用，其输入输出行为是否一致 |
| 重复实现检测 | 是否存在两个以上独立实现的相同逻辑（含不同命名） |

review-agent 若发现违规，应在 `review-report.json` 中以 `HIGH` 级别标记，并注明应复用的具体函数和文件路径。

## 协作流程（v2 并行版 + 自动重试）

```
用户下达开发任务
    │
    ▼
run-workflow.sh（CLI 入口）
    │
    │ 阶段 1：开发（顺序）
    ├── 触发 dev-agent
    │   └── 写代码 + 写 L1-L2 测试
    │   └── 写入 dev-output.json
    │
    │ 阶段 2：并行审查（循环）
    ├── 同时触发 ─┬─ test-agent（测试 + 安全扫描）
    │            ├─ review-agent（代码审查 + H3-H5）
    │            └─ product-reviewer（产品符合性审查）
    │
    │ 阶段 3：评估结果
    ├── 收集三个 agent 的输出
    ├── 检查结果
    │   ├── 全部通过 → 写入 final-report.json (done)
    │   └── 有失败 → 合并失败信息
    │       ├── 检查重试次数
    │       │   ├── 未超限 → 触发 dev-agent 修复 → 回到阶段 2
    │       │   └── 超限 → 写入 final-report.json (fail) + 上报
    │       └── product fail (gap/new_feature) → 记录，不阻塞
    │
    └── 重试时：将所有失败的 artifact 一起传给 dev-agent
        避免多次重试只修一个问题
```

### 自动重试机制

`run-workflow.sh` 实现了自动重试循环：

1. **并行审查**：同时触发 test-agent、review-agent、product-reviewer
2. **评估结果**：检查三个 agent 的输出，判断是否需要修复
3. **触发修复**：如果需要修复且未超重试次数，触发 dev-agent 修复
4. **重新审查**：修复完成后，重新运行并行审查
5. **终止条件**：
   - 全部通过 → 生成成功报告
   - 超过最大重试次数 → 生成失败报告并上报

### 重试触发条件

| 条件 | 触发重试 | 优先级 |
|------|---------|--------|
| product-review.verdict == "fail" && category == "bug" | ✅ | 最高 |
| test-report.result == "fail" | ✅ | 高 |
| review-report.verdict == "request_changes" | ✅ | 中 |
| review-report.verdict == "block" | ✅ | 中 |
| product-review.verdict == "fail" && category != "bug" | ❌ | 记录为非阻塞问题 |

## A2A 通信机制

### 契约文件

所有 agent 间通信通过 `.workflow/artifacts/{task_id}/` 下的文件：

| 文件 | 写入方 | 读取方 | Schema |
|------|--------|--------|--------|
| `dev-output.json` | dev-agent | workflow, test, review, product | [dev-output.schema.json](../../.workflow/templates/dev-output.schema.json) |
| `test-report.json` | test-agent | workflow, dev | [test-report.schema.json](../../.workflow/templates/test-report.schema.json) |
| `test-plan.json` | test-agent | workflow | [test-plan.schema.json](../../.workflow/templates/test-plan.schema.json) |
| `review-report.json` | review-agent | workflow | [review-report.schema.json](../../.workflow/templates/review-report.schema.json) |
| `review-criteria.json` | product-reviewer | workflow | [review-criteria.schema.json](../../.workflow/templates/review-criteria.schema.json) |
| `product-review.json` | product-reviewer | workflow, dev | [product-review.schema.json](../../.workflow/templates/product-review.schema.json) |
| `validation-report.json` | validator | workflow | [validation-report.schema.json](../../.workflow/templates/validation-report.schema.json) |
| `system-inspector-report.json` | system-inspector | workflow | [system-inspector-report.schema.json](../../.workflow/templates/system-inspector-report.schema.json) |
| `optimization-plan.json` | optimizer | workflow, 用户 | [optimization-plan.schema.json](../../.workflow/templates/optimization-plan.schema.json) |
| `error-response.json` | workflow | dev | [error-response.schema.json](../../.workflow/templates/error-response.schema.json) |
| `final-report.json` | 主 Agent | 用户 | [final-report.schema.json](../../.workflow/templates/final-report.schema.json) |

### 通信规则

1. **只写自己的输出文件** — 每个 agent 只写入职责内的 artifact
2. **只读需要的输入文件** — 不读取无关的 artifact
3. **不修改他人的 artifact** — 只读，不写
4. **Schema 验证** — 写入前必须符合对应 schema

## 触发机制（三级）

### 优先级 1：LLM 主动识别

LLM 识别到当前任务需要多 agent 协作时，主 Agent 调用 workflow-advisor 获取执行计划。
这是最自然的触发方式，适用于所有开发工具。

### 优先级 2：Hook 自动触发

工具层 hook 在代码变更时自动创建 artifact，主 Agent 调用 workflow-advisor 获取执行计划。

| 工具 | Hook 机制 | 强度 |
|------|----------|------|
| Claude Code | `PostToolUse` hook in settings.json | 强 |
| Codex | Hook mechanism | 强 |
| Cursor | 无原生 hook | 降级到优先级 3 |

### 优先级 3：用户手动触发

用户手动触发主 Agent 启动 workflow（兜底方案）。

## 重试策略

| 层级 | 最大重试 | 趋势停止条件 | 失败处理 |
|------|---------|------------|---------|
| L1 | 3 次 | 同一 source_location 连续失败 2 次 | dev-agent 判定 failure_type 后修复 |
| L2 | 2 次 | 同一测试用例连续失败 2 次 | 同上 |
| L4 | 1 次 | 同一测试连续失败 1 次 | 同上 |
| L5 | 0 次 | 首次失败即上报 | 回到产品设计层面讨论 |

### failure_type 判定

失败的测试由 **dev-agent** 读代码后判定 failure_type：

| failure_type | 含义 | 处理 |
|-------------|------|------|
| `code_bug` | 代码有 bug | dev-agent 修复代码 |
| `test_bug` | 测试写错了 | dev-agent 修复测试 |
| `env_issue` | 环境问题 | 记录 + 隔离，不阻塞 |
| `unknown` | 无法判定 | 标记为 unknown，人工介入 |

## Workflow Spec

协作流程的完整定义见：
- **主 workflow**: `.workflow/specs/dev-test-review.yaml`
- **Artifact schemas**: `.workflow/templates/`
- **测试架构**: [testing/architecture.md](../testing/architecture.md)

## 工具适配

### Claude Code

- Agents: `.claude/agents/workflow-advisor.md`
- Hooks: `.claude/settings.json` 中配置 `PostToolUse` hook
- Rules: `.claude/rules/` 中注入 workflow 提示

### Codex

- Instructions: `.codex/instructions.md`
- Hooks: Codex hook mechanism
- Skills: workflow 相关 skill

### Cursor

- Rules: `.cursor/rules/` 中注入 workflow 提示
- 无原生 hook，依赖 LLM 主动识别或用户手动触发

> **适配状态**：Claude Code、Codex、Cursor 适配均已实现。

## 日志系统（开发阶段）

**开发阶段，所有 workflow 执行必须写入日志文件，帮助调试任务流转问题。**

### 日志文件位置

每个任务的日志文件：`.workflow/artifacts/{task_id}/workflow.log`

### 日志格式

```
[ISO_TIMESTAMP] [LEVEL] [PHASE] message
```

- `ISO_TIMESTAMP`：`2026-06-14 19:30:45.123`
- `LEVEL`：`DEBUG` | `INFO` | `WARN` | `ERROR`
- `PHASE`：`INIT` | `DEV` | `PARALLEL_REVIEW` | `EVALUATE` | `RETRY` | `DONE`

### 必须记录的节点

| 节点 | 级别 | 内容 |
|------|------|------|
| workflow 启动 | INFO | task_id, 输入文件内容摘要 |
| 阶段切换 | INFO | 从哪个阶段到哪个阶段 |
| agent 调用前 | INFO | agent 名称, 输入文件, 期望输出文件 |
| agent 完成后 | INFO | agent 名称, 输出文件是否存在, 关键字段值 |
| agent 失败 | ERROR | agent 名称, 失败原因, 是否重试 |
| artifact 读取 | DEBUG | 文件路径, 关键字段值 |
| artifact 缺失 | WARN | 文件路径, 哪个 agent 应该生成 |
| 决策点 | INFO | 决策内容, 判断依据 |
| 重试 | WARN | 第几次重试, 重试原因, 目标文件 |
| 最终结果 | INFO | overall_result, 各 gate 结果, 总耗时 |

### 脚本层日志

Shell 脚本通过 `workflow-logger.sh` 提供统一的日志能力：

```bash
source "$(dirname "$0")/workflow-logger.sh"
workflow_log_init "<task_id>"

log_info "消息"
log_warn "警告"
log_error "错误"
log_phase "阶段名"
log_agent_call "agent名" "start|complete|fail" "详情"
log_artifact "read|write|missing" "文件路径" "详情"
log_decision "决策内容" "判断依据"
log_retry "当前次" "最大次" "原因"
log_gate_result "gate名" "pass|fail" "详情"
```

### Agent 层日志

workflow-advisor 通过 Bash 命令追加日志：

```bash
echo "[$(date '+%Y-%m-%d %H:%M:%S.%3N')] [INFO] [PHASE] message" >> "$LOG_FILE"
```

### 日志查看

```bash
# 查看完整日志
./scripts/workflow-log-view.sh <task_id>

# 查看最后 50 行
./scripts/workflow-log-view.sh <task_id> --tail 50

# 只看错误
./scripts/workflow-log-view.sh <task_id> --errors

# 只看特定阶段
./scripts/workflow-log-view.sh <task_id> --phase DEV

# 只看 agent 相关
./scripts/workflow-log-view.sh <task_id> --agent

# 只看决策点
./scripts/workflow-log-view.sh <task_id> --decision
```

### 调试流程

当任务流转出现问题时：

1. 查看日志文件：`.workflow/artifacts/{task_id}/workflow.log`
2. 搜索 `ERROR` 和 `WARN` 级别日志
3. 检查 `AGENT` 相关日志，确认 agent 调用顺序和结果
4. 检查 `DECISION` 日志，确认决策点判断是否正确
5. 检查 `ARTIFACT` 日志，确认文件读写是否正常

## 文件结构

```
.workflow/
├── specs/                         # workflow 定义（tool-agnostic）
│   └── dev-test-review.yaml       # 主 workflow spec
├── templates/                     # artifact schema 模板
│   ├── dev-output.schema.json
│   ├── test-report.schema.json
│   ├── review-report.schema.json
│   └── final-report.schema.json
└── artifacts/                     # 运行时 artifacts
    └── {task_id}/                 # 每任务一个子目录
        ├── dev-output.json
        ├── test-report.json
        ├── review-report.json
        ├── final-report.json
        └── workflow.log           # 执行日志（开发阶段保留）

scripts/
├── workflow-logger.sh             # 日志工具库（source 使用）
├── create-dev-output.sh           # 生成 dev-output.json
├── run-workflow.sh                # CLI 入口
└── workflow-log-view.sh           # 日志查看工具

.claude/agents/
└── workflow-advisor.md             # workflow-agent 定义（含日志指令）
```
