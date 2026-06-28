---
title: Agent Guide
type: guide
domain: development
created: 2026-06-07
updated: 2026-06-23
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

## 编排规则（唯一强制规则）

**所有代码变更（crates/、src/、src-tauri/），主 Agent 必须：**

1. 调用 `workflow-runner`，传入任务描述
2. 等待 workflow-runner 返回结果
3. 将结果汇报给用户

**主 Agent 禁止：**

- 直接调用 dev-agent、test-agent、review-agent、product-reviewer
- 直接写代码（由 workflow-runner 委托 dev-agent 完成）
- 直接运行测试（由 workflow-runner 委托 test-agent 完成）

**workflow-runner 负责：**

- 按 v5 流程编排：Phase 1 并行启动（dev + test-plan + review-criteria）→ Phase 2 执行验证（test + review + product-review）→ Phase 3 验证 → Phase 4 监督
- 管理重试逻辑和熔断器
- 生成最终报告

**Hook 自动保障：**

- PreToolUse hook 在代码变更时自动创建 workflow artifact
- 无需手动创建 dev-output.json

→ [workflow-runner 定义](.claude/agents/workflow-runner.md) — 完整流程和路由表

## 关键引用

→ [多 Agent 协作规范](docs/development/multi-agent-collaboration.md) — 协作流程详情
→ [错误响应协议](docs/development/error-response-protocol.md) — 错误时必须先反思流程
→ [CODEMAP 索引](docs/CODEMAPS/_index.md) — 代码导航入口
