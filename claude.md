---
title: Agent Guide
type: guide
domain: development
created: 2026-06-07
updated: 2026-06-21
status: active
---

# Agent Guide

**Work Better**：以 Obsidian 为中心的 AI 工作观察者。

## 准则

1. 观察者姿态——被动采集、主动整理
2. Obsidian 为中心——数据归用户所有
3. 自主但可干预——私有数据自主，共享数据需确认
4. 专业分工——每个 Agent 只做一件事，不越界
5. 自我进化——Agent 完成任务后记录优化点，持续改进

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

**渐进式读取**：`CLAUDE.md` → CODEMAP → 目标源文件（最多 3 层定位）

## 多 Agent 协作

本项目采用多 Agent 协作开发模式。**主 Agent 是指挥官**，使用 Workflow tool 直接编排 subagent，始终持有控制权。

### Agent 职责矩阵

| Agent | 职责 | 写代码 | 写测试 | 审查代码 | 编排调度 | 监督优化 |
|-------|------|--------|--------|---------|---------|---------|
| **主 Agent** | 指挥官，编排 subagent | ❌ | ❌ | ❌ | ✅ | ❌ |
| dev-agent | 功能开发 + L1-L2 测试 | ✅ | ✅ | ❌ | ❌ | ❌ |
| test-agent | 测试执行 + L4-L5 测试生成 | ❌ | ✅ | ❌ | ❌ | ❌ |
| review-agent | 代码审查 + H3-H5 安全测试 | ❌ | ✅ | ✅ | ❌ | ❌ |
| product-reviewer | 产品定义符合性审查 | ❌ | ❌ | ✅ | ❌ | ❌ |
| guardian Agent | 守护者，监督整个系统 | ❌ | ❌ | ❌ | ❌ | ✅ |
| optimizer Agent | 优化者，执行具体优化任务 | ✅ | ❌ | ❌ | ❌ | ✅ |
| orchestrator-agent | 监督者，监督所有 Agent | ❌ | ❌ | ❌ | ✅ | ✅ |
| validator-agent | 验证者，管道交叉点验证 | ❌ | ❌ | ✅ | ❌ | ❌ |
| cost-tracker-agent | 成本追踪者，追踪 token 使用 | ❌ | ❌ | ❌ | ❌ | ✅ |
| chaos-tester-agent | 混沌测试者，随机故障注入 | ❌ | ✅ | ❌ | ❌ | ❌ |
| checkpoint-manager-agent | 检查点管理者，管理恢复点 | ❌ | ❌ | ❌ | ❌ | ✅ |
| workflow-runner | 可选的编排辅助工具 | ❌ | ❌ | ❌ | ✅ | ❌ |

### 编排方式

- **主 Agent 禁止直接派发子 Agent**（dev-agent、test-agent、review-agent、product-reviewer）
- **所有代码变更的子 Agent 调度必须通过 `workflow-runner` 进行**
- workflow-runner 收到任务后自行判断：
  - 简单变更 → workflow-runner 直接完成
  - 复杂变更 → 按 spec 派发子 Agent
- **主 Agent 自己也不能直接写代码**（Edit/Write/Bash 修改 crates/、src/、src-tauri/ 下的文件会被 hook 阻止）

### 通信机制

- **文件契约通信**：通过 `.workflow/artifacts/` 下的 JSON 文件传递信息
- **Schema 验证**：所有 artifact 必须符合对应的 schema
- **Handoff skill**：用于会话传递

### 自迭代机制

- 每个 agent 完成任务后记录 improvements
- guardian Agent 审查并生成优化计划
- 用户审批后，optimizer Agent 执行优化
- optimizer Agent 自己验证优化效果

### 约束机制

约束分为两个层次：

**软约束（强烈推荐，违反需给出理由）**：
- **决策级约束**：CLAUDE.md — 定义项目准则和协作规则
- **流程级约束**：.claude/rules/ — 定义编码规范和测试要求
- **编排级约束**：.workflow/specs/ — 定义 workflow 流程
- **角色级约束**：.claude/agents/*.md — 定义每个 Agent 的职责边界

**硬约束（代码级强制，不可绕过）**：
- **执行级约束**：.claude/hooks/hooks.json — 工具级拦截和验证

## 关键引用

→ [多 Agent 协作规范](docs/development/multi-agent-collaboration.md) — 强制约束、路由架构、协作流程
→ [错误响应协议](docs/development/error-response-protocol.md) — 错误时必须先反思流程
→ [自定义 Agent 注册](docs/development/custom-agents.md) — Agent 定义和注册方式
→ [文档规范](docs/conventions.md) — frontmatter、索引更新、底线规则
→ [CODEMAP 索引](docs/CODEMAPS/_index.md) — 代码导航入口
→ [Workflow Spec](.workflow/specs/dev-test-review.yaml) — 测试+审查流程定义
