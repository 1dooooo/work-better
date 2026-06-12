---
title: 采集器激活与可观测性修复
date: 2026-06-12
status: completed
type: fix
---

# 采集器激活与可观测性修复

## 问题诊断

### 问题 1: 采集器不工作

**根因分析**:

1. **Scheduler 未启动** - `lib.rs` 中没有调用 `scheduler.start()`
2. **系统采集器未注册** - `register_builtin_collectors()` 只注册了飞书采集器
3. **没有定时采集任务** - 即使 Scheduler 启动，也没有注册任何 ScheduledTask
4. **飞书采集器默认禁用** - `feishu_enabled` 默认为 `false`

### 问题 2: 无法观测行为

1. **审计流已实现但没有数据** - 因为采集器不工作，没有事件被处理
2. **采集器执行结果未记录** - 缺少执行日志
3. **缺少实时状态反馈** - 无法看到采集器正在工作

## 修复方案

### 1. 创建采集器定时任务

**文件**: `crates/wb-collector/src/collector_task.rs`

创建了 `CollectorTask` 结构体，实现 `ScheduledTask` trait，将 CollectorManager 中的采集器包装为可调度的任务。

```rust
pub struct CollectorTask {
    manager: Arc<CollectorManager>,
    collector_id: String,
    task_name: String,
    interval_secs: u64,
}
```

同时添加了 `ExecutionLogger` 用于记录执行日志。

### 2. 注册系统采集器

**文件**: `src-tauri/src/commands/collectors.rs`

修改 `register_builtin_collectors()` 函数，注册所有采集器：

- `feishu` - 飞书消息采集器（默认禁用）
- `system.app_switch` - 前台应用切换采集器（默认启用）
- `system.browser_history` - Chrome 浏览历史采集器（默认启用）

### 3. 启动 Scheduler 并注册任务

**文件**: `src-tauri/src/lib.rs`

在 Tauri setup 阶段启动 Scheduler 并注册采集任务。

**按照产品设计文档对齐的频率**:

| 任务 ID | 采集器 | 间隔 | 设计来源 |
|---------|--------|------|----------|
| C-02 | feishu | 30 分钟 | scheduler.md: C-02 飞书任务同步 |
| C-04 | system.browser_history | 15 分钟 | scheduler.md: C-04 浏览器历史采样 |
| - | system.app_switch | 5 分钟 | collection.md: 采样，停留 > 30 秒记录 |

### 4. 添加采集器状态观测

**文件**: `src-tauri/src/commands/collectors.rs`

新增 `get_collector_statuses` 命令，返回详细的采集器状态：

```rust
pub struct CollectorDetailedStatus {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub health_level: String,
    pub health_message: Option<String>,
    pub last_run: Option<String>,
    pub last_status: Option<String>,
    pub last_summary: Option<String>,
}
```

## 修改的文件

| 文件 | 修改内容 |
|------|----------|
| `crates/wb-collector/src/collector_task.rs` | 新增：采集器定时任务实现 + 执行日志 |
| `crates/wb-collector/src/lib.rs` | 修改：导出 collector_task 模块 |
| `crates/wb-collector/Cargo.toml` | 修改：添加 wb-scheduler 依赖 |
| `src-tauri/src/commands/collectors.rs` | 修改：注册系统采集器，添加状态查询命令 |
| `src-tauri/src/commands/settings.rs` | 修改：移除重复的 get_collector_statuses 命令 |
| `src-tauri/src/lib.rs` | 修改：启动 Scheduler，注册采集任务 |
| `docs/fixes/collector-activation-fix.md` | 修复文档 |
| `docs/index.md` | 更新索引 |

## 与产品设计的一致性

### ✅ 符合设计

- CollectorManager 架构（热插拔、开关控制、健康监控）
- Scheduler 架构（cron 调度、超时控制、重试策略）
- 采集频率对齐产品设计（C-02: 30 分钟，C-04: 15 分钟）
- 执行日志记录

### ⚠️ 已知差异

1. **飞书子采集器**: 产品设计有 12 个独立子采集器，当前实现合并为 1 个
2. **UserCaptureCollector**: 未注册（text_input, image_paste, screenshot）
3. **错峰执行**: 未实现"采集类在整点后 0-5 分钟"策略
4. **执行日志持久化**: 当前输出到 stderr，未写入 execution_logs 表

### 📋 后续待实现

参考产品设计文档，以下功能需要后续实现：

1. **F1.1 飞书子采集器** - 拆分为独立的子采集器
2. **F1.3 用户手动采集器** - 注册 UserCaptureCollector
3. **F6.2.1 错峰执行** - 实现错峰调度策略
4. **F6.2.7 执行日志** - 写入 execution_logs 表

## 验证方法

1. **编译验证**: `cargo check` 通过
2. **测试验证**: `cargo test --lib` 全部通过（146 个测试）

## 使用方式

### 启动应用后

应用启动后会自动：
1. 注册 3 个采集器（飞书、应用切换、浏览器历史）
2. 启动 Scheduler 调度器
3. 按设定间隔自动执行采集任务
4. 在控制台输出执行日志

### 观测采集器状态

在前端调用 `get_collector_statuses` 命令，获取：
- 采集器启用状态
- 健康检查结果
- 最近执行时间
- 最近执行状态
- 最近执行摘要

### 开发者模式

开启开发者模式后，可以在"审计"页面查看：
- 处理审计日志（Processing Audit）
- 执行日志（Execution Log）
