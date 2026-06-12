---
title: CODEMAP 索引
type: index
domain: codemaps
created: 2026-06-12
updated: 2026-06-12
status: active
---

# CODEMAP 索引

> **用途**：CODEMAP 是概念层（docs/）与代码层（crates/、src/）之间的桥梁。
> LLM 在开发时应先读 CODEMAP 定位文件，再读源码，避免全量扫描。

## 使用方式

```
收到任务 → 读 agent.md 代码导航表 → 读对应 CODEMAP → 只读需要修改的源文件
```

## CODEMAP 列表

| 模块 | CODEMAP | 对应代码 | 说明 |
|------|---------|---------|------|
| 核心类型 | [wb-core.codemap.md](wb-core.codemap.md) | `crates/wb-core/` | Event、Task、WorkRecord 等领域类型 |
| 采集层 | [wb-collector.codemap.md](wb-collector.codemap.md) | `crates/wb-collector/` | 飞书/系统/手动采集器 |
| 处理层 | [wb-processor.codemap.md](wb-processor.codemap.md) | `crates/wb-processor/` | 分类、提取、审核、报告 |
| 存储层 | [wb-storage.codemap.md](wb-storage.codemap.md) | `crates/wb-storage/` | Obsidian/SQLite/向量DB |
| AI 适配 | [wb-ai.codemap.md](wb-ai.codemap.md) | `crates/wb-ai/` | 模型路由、预算、适配器 |
| 定时任务 | [wb-scheduler.codemap.md](wb-scheduler.codemap.md) | `crates/wb-scheduler/` | 调度、依赖、重试 |
| 前端 | [frontend.codemap.md](frontend.codemap.md) | `src/` + `src-tauri/` | React UI + Tauri 命令 |
| 测试 | [testing.codemap.md](testing.codemap.md) | 测试文件 | 测试体系导航（原 testing/ 目录） |

## 维护规则

1. 新增/移除/重命名源文件 → 更新对应 CODEMAP
2. 模块职责变更 → 更新 CODEMAP 的职责说明
3. 新增 crate → 创建新 CODEMAP + 更新本索引
4. CODEMAP 中的文件路径必须与实际代码一致
