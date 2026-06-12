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
| test-agent | 测试执行 + L4-L5 测试生成 |
| review-agent | 代码审查 + H3-H5 安全测试 |
| workflow-runner | 流程编排 + 重试管理 + 报告 |

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

**渐进式读取路径**：`agent.md` → CODEMAP → 目标源文件（最多读 3 层即可定位）

## 文档体系

→ [文档规范](docs/conventions.md) | [文档索引](docs/index.md) | [ADR 决策记录](docs/decisions/)
→ [CODEMAP 索引](docs/CODEMAPS/_index.md) | [多 Agent 协作规范](docs/development/multi-agent-collaboration.md)
→ [Workflow Spec](.workflow/specs/dev-test-review.yaml)
