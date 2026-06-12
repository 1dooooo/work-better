---
title: wb-core CODEMAP
type: codemap
domain: architecture
crate: wb-core
created: 2026-06-12
updated: 2026-06-12
status: active
---

# wb-core CODEMAP

> **职责**：核心领域类型定义。所有其他 crate 依赖 wb-core，wb-core 不依赖任何业务 crate。
> **对应文档**：[架构总览](../architecture/overview.md) · [事件模型](../architecture/modules/event-model.md)

## 文件导航

| 文件 | 职责 | 关键类型 |
|------|------|---------|
| `lib.rs` | 模块导出 | — |
| `event.rs` | 事件定义（系统原子单位，不可变） | `Event`, `Source`, `EventType`, `Confidence`, `EventFilter`, `EventLog` trait |
| `task.rs` | 任务定义与状态机 | `Task`, `TaskStatus`, `Priority`, `can_transition_to()`, `transition()`, `archive()` |
| `record.rs` | 工作记录（处理层输出） | `WorkRecord`, `Category` |
| `audit.rs` | 审计记录（处理链路追踪） | `ProcessingAudit`, `AuditStep`, `ReviewResult`, `ReviewVerdict`, `Issue` |
| `error.rs` | 统一错误类型 | `WbError`, `Result<T>` |
| `test_helpers.rs` | 测试辅助工具 | 仅 `#[cfg(test)]` 编译 |

## 核心数据流

```
Event (event.rs)  →  处理层消费  →  WorkRecord (record.rs)
                                   →  ProcessingAudit (audit.rs)
                                   →  Task (task.rs)
```

## 关键设计

- **不可变**：Event 只追加不修改；Task 状态转换返回新实例
- **ts-rs 导出**：所有核心类型用 `#[ts(export)]` 标记，自动生成 TypeScript 类型
- **状态机**：TaskStatus 的合法转换定义在 `can_transition_to()` 中
- **EventLog trait**：异步 trait，由 wb-storage 的 SqliteEventLog 实现

## 修改指引

| 你想改什么 | 先读 | 再改 |
|-----------|------|------|
| 新增事件来源 | `event.rs` 的 `Source` 枚举 | 添加枚举变体 + 更新采集器 |
| 修改任务状态流转 | `task.rs` 的 `can_transition_to()` | 修改匹配规则 |
| 新增审计步骤 | `audit.rs` 的 `AuditStep` 枚举 | 添加枚举变体 |
| 修改错误类型 | `error.rs` | 添加 `WbError` 变体 |
