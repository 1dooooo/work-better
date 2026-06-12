---
title: wb-scheduler CODEMAP
type: codemap
domain: architecture
crate: wb-scheduler
created: 2026-06-12
updated: 2026-06-12
status: active
---

# wb-scheduler CODEMAP

> **职责**：定时任务调度框架。管理任务注册、Cron 调度、依赖关系、重试、资源感知。
> **对应文档**：[定时任务架构](../architecture/modules/scheduler.md)

## 文件导航

| 文件 | 职责 | 关键类型 |
|------|------|---------|
| `lib.rs` | 模块导出 | — |
| `scheduler.rs` | 调度器核心 | `Scheduler`, `TaskInfo` — 注册、启停、暂停、执行 |
| `task.rs` | 定时任务 trait | `ScheduledTask` trait, `TaskResult`, `TaskStatus`, `TaskLayer` |
| `cron.rs` | Cron 表达式解析 | `is_due()` — 判断任务是否到期 |
| `dependency.rs` | 任务依赖管理 | `DependencyGraph` — 前置依赖检查 |
| `resource.rs` | 资源感知 | `should_defer()` — 低预算时推迟低优先级任务 |
| `log.rs` | 执行日志 | 任务执行状态和结果记录 |

### 测试文件 (`tests/`)

| 文件 | 职责 |
|------|------|
| `scheduler_tests.rs` | 调度器核心测试 |
| `real_backend_scheduler_pause.rs` | 真实后端暂停/恢复测试 |

## 数据流

```
Scheduler::start()  →  后台 tokio 循环（每秒 tick）
  → 检查 paused 状态
  → 遍历已注册任务，判断是否到期（interval + cron）
  → DependencyGraph::can_run() 检查依赖
  → resource::should_defer() 检查资源
  → execute_with_retry() 执行（带超时 + 重试）
  → 更新 TaskState（last_run, last_status, last_result）
```

## 关键设计

- **ScheduledTask trait**：定时任务的统一抽象，每个任务声明 id、name、layer、cron、sla_ms、retry_limit
- **Scheduler**：中心调度器，支持 register_with_deps() 设置依赖关系
- **execute_with_retry()**：指数退避重试 + SLA 超时控制
- **DependencyGraph**：DAG 依赖图，can_run() 检查所有前置是否完成
- **resource::should_defer()**：token 预算不足时推迟低优先级任务

## 修改指引

| 你想改什么 | 先读 | 再改 |
|-----------|------|------|
| 新增定时任务 | `task.rs` 的 `ScheduledTask` trait | 新建文件实现 trait + 注册到 Scheduler |
| 修改调度逻辑 | `scheduler.rs` 的后台循环 | 修改 tick 逻辑 |
| 修改重试策略 | `scheduler.rs` 的 `execute_with_retry()` | 修改退避参数 |
| 修改依赖管理 | `dependency.rs` | 修改 `DependencyGraph` |
| 修改资源策略 | `resource.rs` | 修改 `should_defer()` 逻辑 |
