# Agent Guide

**Work Better**：以 Obsidian 为中心的 AI 工作观察者。

## 准则

1. 观察者姿态——被动采集、主动整理
2. Obsidian 为中心——数据归用户所有
3. 自主但可干预——私有数据自主，共享数据需确认

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

## 关键引用

→ [多 Agent 协作规范](docs/development/multi-agent-collaboration.md) — 强制约束、路由架构、协作流程
→ [错误响应协议](docs/development/error-response-protocol.md) — 错误时必须先反思流程
→ [自定义 Agent 注册](docs/development/custom-agents.md) — Agent 定义和注册方式
→ [文档规范](docs/conventions.md) — frontmatter、索引更新、底线规则
→ [CODEMAP 索引](docs/CODEMAPS/_index.md) — 代码导航入口
→ [Workflow Spec](.workflow/specs/dev-test-review.yaml) — 测试+审查流程定义
