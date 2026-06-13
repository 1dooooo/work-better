# Agent Guide

**Work Better**：以 Obsidian 为中心的 AI 工作观察者。本项目采用多 Agent 协作开发模式。

## 准则

1. 观察者姿态——被动采集、主动整理
2. Obsidian 为中心——数据归用户所有
3. 自主但可干预——私有数据自主，共享数据需确认

## 多 Agent 协作

本项目在必要时使用多 Agent 协作开发。各 Agent 职责单一，通过文件契约通信，不共享对话上下文。

| Agent | 职责 |
|-------|------|
| dev-agent | 功能开发 + L1-L2 测试 |
| product-reviewer | 产品审查，判断功能是否符合预期，决策 bug/feature |
| test-agent | 测试执行 + L4-L5 测试生成 |
| review-agent | 代码审查 + H3-H5 安全测试 |
| workflow-runner | 流程编排 + 重试管理 + 报告 |

### ⚠️ 强制约束（必须遵守）

**约束 1：禁止主 Agent 直接派发子 Agent**

主 Agent **不得**直接 spawn `dev-agent`、`test-agent`、`review-agent`、`product-reviewer`。
所有代码变更的子 Agent 调度**必须**通过 `workflow-runner` 进行。

```
✅ 正确：主 Agent → workflow-runner → workflow-runner 派发子 Agent
❌ 禁止：主 Agent → dev-agent / test-agent / review-agent（直接调用）
```

**约束 2：代码变更必须走 Workflow**

触发条件（满足任一即触发）：
- 修改了 `crates/` 下的 Rust 代码
- 修改了 `src/` 下的 TypeScript 代码
- 修改了 `src-tauri/` 下的 Tauri 命令代码
- 新增或删除了功能文件

不触发：仅修改文档 (`docs/`)、配置文件 (`.config/`)、脚本 (`scripts/`)

触发后必须执行：
1. 告知用户"检测到代码变更，启动 workflow-runner"
2. 运行 `./scripts/create-dev-output.sh <task_id>` 和 `./scripts/run-workflow.sh <task_id>`
3. 或启动 `workflow-runner` agent，由其判断是否需要多 Agent 协作

**约束 3：workflow-runner 决定是否需要多 Agent**

workflow-runner 收到任务后自行判断：
- 简单变更 → workflow-runner 直接完成
- 复杂变更 → 按 spec 派发 dev-agent → 并行 test-agent + review-agent + product-reviewer

主 Agent 不做这个判断，交给 workflow-runner。

完整规范见 [多 Agent 协作开发规范](docs/development/multi-agent-collaboration.md)。
Workflow 定义见 [.workflow/specs/dev-test-review.yaml](.workflow/specs/dev-test-review.yaml)。

## 文档底线规则

1. 新增/修改文档 → 必须有 frontmatter（见 [docs/conventions.md](docs/conventions.md)）
2. 新增文档 → 必须更新所在目录的 `_index.md`
3. 不读取 `deprecated/` 下的任何文件
4. 文档超过 300 行 → 拆分
5. 多 Agent 协作流程变更 → 更新 workflow spec + multi-agent-collaboration.md

## 代码导航

> **开发时先读 CODEMAP，再读源码。不要全量扫描代码。**

| 模块 | 代码位置 | CODEMAP | 职责 |
|------|---------|---------|------|
| 核心类型 | `crates/wb-core/` | [wb-core.codemap.md](docs/CODEMAPS/wb-core.codemap.md) | Event、Task、WorkRecord |
| 采集层 | `crates/wb-collector/` | [wb-collector.codemap.md](docs/CODEMAPS/wb-collector.codemap.md) | 飞书/系统/手动采集器 |
| 处理层 | `crates/wb-processor/` | [wb-processor.codemap.md](docs/CODEMAPS/wb-processor.codemap.md) | 分类、提取、审核、报告 |
| 存储层 | `crates/wb-storage/` | [wb-storage.codemap.md](docs/CODEMAPS/wb-storage.codemap.md) | Obsidian/SQLite/向量DB |
| AI 适配 | `crates/wb-ai/` | [wb-ai.codemap.md](docs/CODEMAPS/wb-ai.codemap.md) | 模型路由、预算、适配器 |
| 定时任务 | `crates/wb-scheduler/` | [wb-scheduler.codemap.md](docs/CODEMAPS/wb-scheduler.codemap.md) | 调度、依赖、重试 |
| 前端 | `src/` + `src-tauri/` | [frontend.codemap.md](docs/CODEMAPS/frontend.codemap.md) | React UI + Tauri 命令 |

**渐进式读取路径**：`claude.md` → CODEMAP → 目标源文件（最多读 3 层即可定位）

## 文档体系

→ [文档规范](docs/conventions.md) | [文档索引](docs/index.md) | [ADR 决策记录](docs/decisions/)
→ [CODEMAP 索引](docs/CODEMAPS/_index.md) | [多 Agent 协作规范](docs/development/multi-agent-collaboration.md)
→ [Workflow Spec](.workflow/specs/dev-test-review.yaml)

## 自定义 Agent 注册

本项目使用自定义 agent 来执行多 Agent 协作 workflow。由于 Claude Code 的限制，自定义 agent 需要通过以下方式注册：

### 方式 1：启动时传入（推荐）

```bash
# 使用启动脚本
./scripts/start-claude-with-agents.sh

# 或手动传入
claude --agents "$(cat ~/.claude/agents.json)"
```

### 方式 2：使用通用 agent + 角色 prompt

当自定义 agent 不可用时，使用 `general-purpose` agent 并在 prompt 中指定角色：

```
Agent type: general-purpose
prompt: "你是 [agent 角色]。职责：[具体职责]..."
```

### Agent 定义文件

- `~/.claude/agents/dev-agent.md` — 开发者 agent
- `~/.claude/agents/product-reviewer.md` — 产品审查者
- `~/.claude/agents/test-agent.md` — 测试执行者
- `~/.claude/agents/review-agent.md` — 代码审查者
- `~/.claude/agents/workflow-runner.md` — 流程编排者
