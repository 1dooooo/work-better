# Agent Guide

**Work Better**：Obsidian 为中心的 AI 工作观察者。被动采集、主动整理、数据归用户所有。

## 路由架构

```
用户请求
  ├── 代码变更 → workflow-runner → 判断复杂度
  │     ├── 简单 → workflow-runner 直接完成
  │     └── 复杂 → dev-agent → (test-agent + review-agent + product-reviewer) 并行
  ├── 文档/配置 → 主 Agent 直接处理
  └── 独立任务 → 对应 Agent（planner/architect/tdd-guide 等）
```

## 强制约束

### 约束 1：禁止主 Agent 直接派发子 Agent

主 Agent **不得**直接 spawn `dev-agent`、`test-agent`、`review-agent`、`product-reviewer`。
所有代码变更的子 Agent 调度**必须**通过 `workflow-runner` 进行。

```
✅ 主 Agent → workflow-runner → 子 Agent
❌ 主 Agent → dev-agent / test-agent / review-agent
```

### 约束 2：代码变更必须走 Workflow

触发条件（满足任一）：
- 修改了 `crates/` 下的 Rust 代码
- 修改了 `src/` 下的 TypeScript 代码
- 修改了 `src-tauri/` 下的 Tauri 命令代码
- 新增或删除了功能文件

不触发：仅修改文档 (`docs/`)、配置文件 (`.config/`)、脚本 (`scripts/`)

执行步骤：
1. `./scripts/create-dev-output.sh <task_id>` — 生成变更快照
2. `./scripts/run-workflow.sh <task_id>` — 运行测试+审查 workflow
3. 检查 `.workflow/artifacts/<task_id>/test-report.json`

### 约束 3：workflow-runner 决定协作模式

workflow-runner 收到任务后自行判断：
- 简单变更 → 直接完成
- 复杂变更 → 按 spec 派发子 Agent

主 Agent 不做这个判断。

## Agent 角色

| Agent | 职责 |
|-------|------|
| workflow-runner | 流程编排 + 重试管理 + 报告 |
| dev-agent | 功能开发 + L1-L2 测试 |
| test-agent | 测试执行 + L4-L5 测试生成 |
| review-agent | 代码审查 + H3-H5 安全测试 |
| product-reviewer | 产品审查，决策 bug/feature |

详见 [多 Agent 协作规范](docs/development/multi-agent-collaboration.md)。

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

**渐进式读取**：`agent.md` → CODEMAP → 目标源文件（最多 3 层定位）

## 文档导航

→ [文档索引](docs/index.md) | [文档规范](docs/conventions.md) | [ADR 决策记录](docs/decisions/)
→ [CODEMAP 索引](docs/CODEMAPS/_index.md) | [多 Agent 协作规范](docs/development/multi-agent-collaboration.md)
→ [自定义 Agent 注册](docs/development/custom-agents.md)
