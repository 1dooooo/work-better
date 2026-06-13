---
title: 多 Agent 协作开发规范
type: structural
domain: development
created: 2026-06-07
updated: 2026-06-07
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

| Agent | 职责 | 写代码 | 写测试 | 审查代码 | 产品审查 | 编排调度 |
|-------|------|--------|--------|---------|---------|---------|
| **workflow-runner** | 流程编排 + 重试管理 + 报告 | ❌ | ❌ | ❌ | ❌ | ✅ |
| **dev-agent** | 功能开发 + L1-L2 测试 | ✅ | ✅ (L1-L2) | ❌ | ❌ | ❌ |
| **test-agent** | 测试执行 + L4-L5 测试生成 | ❌ | ✅ (L4-L5) | ❌ | ❌ | ❌ |
| **review-agent** | 代码审查 + H3-H5 安全测试 | ❌ | ✅ (H3-H5) | ✅ | ❌ | ❌ |
| **product-reviewer** | 产品定义符合性审查 | ❌ | ❌ | ❌ | ✅ | ❌ |

### 职责边界规则

- dev-agent **不负责** L4-L5 测试和安全测试
- test-agent **不负责** 代码修改和审查
- review-agent **不负责** 代码修改和功能测试
- workflow-runner **不负责** 任何代码/测试/审查工作

## 协作流程（v2 并行版）

```
用户下达开发任务
    │
    ▼
workflow-runner（编排入口）
    │
    │ 阶段 1：开发（顺序）
    ├── 触发 dev-agent
    │   └── 写代码 + 写 L1-L2 测试
    │   └── 写入 dev-output.json
    │
    │ 阶段 2：并行审查
    ├── 同时触发 ─┬─ test-agent（测试 + 安全扫描）
    │            ├─ review-agent（代码审查 + H3-H5）
    │            └─ product-reviewer（产品符合性审查）
    │
    │ 阶段 3：汇总
    ├── 收集三个 agent 的输出
    ├── 检查结果
    │   ├── 全部通过 → 写入 final-report.json (done)
    │   └── 有失败 → 触发 dev-agent 修复 → 回到阶段 2
    │
    └── 重试超限 → escalate（上报用户）
```

## A2A 通信机制

### 契约文件

所有 agent 间通信通过 `.workflow/artifacts/{task_id}/` 下的文件：

| 文件 | 写入方 | 读取方 | Schema |
|------|--------|--------|--------|
| `dev-output.json` | dev-agent | workflow, test, review, product | [dev-output.schema.json](../../.workflow/templates/dev-output.schema.json) |
| `test-report.json` | test-agent | workflow, dev | [test-report.schema.json](../../.workflow/templates/test-report.schema.json) |
| `review-report.json` | review-agent | workflow | [review-report.schema.json](../../.workflow/templates/review-report.schema.json) |
| `product-review.json` | product-reviewer | workflow, dev | [product-review.schema.json](../../.workflow/templates/product-review.schema.json) |
| `final-report.json` | workflow-runner | 用户 | [final-report.schema.json](../../.workflow/templates/final-report.schema.json) |

### 通信规则

1. **只写自己的输出文件** — 每个 agent 只写入职责内的 artifact
2. **只读需要的输入文件** — 不读取无关的 artifact
3. **不修改他人的 artifact** — 只读，不写
4. **Schema 验证** — 写入前必须符合对应 schema

## 触发机制（三级）

### 优先级 1：LLM 主动识别

LLM 识别到当前任务需要多 agent 协作时，主动启动 workflow-runner。
这是最自然的触发方式，适用于所有开发工具。

### 优先级 2：Hook 自动触发

工具层 hook 在 dev-output.json 写入后自动启动 workflow-runner。

| 工具 | Hook 机制 | 强度 |
|------|----------|------|
| Claude Code | `PostToolUse` hook in settings.json | 强 |
| Codex | Hook mechanism | 强 |
| Cursor | 无原生 hook | 降级到优先级 3 |

### 优先级 3：用户手动触发

用户手动启动 workflow-runner（兜底方案）。

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

- Agents: `.claude/agents/workflow-runner.md`
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
        └── final-report.json

.claude/agents/
└── workflow-runner.md             # workflow-agent 定义
```
