---
title: wb-collector CODEMAP
type: codemap
domain: architecture
crate: wb-collector
created: 2026-06-12
updated: 2026-06-12
status: active
---

# wb-collector CODEMAP

> **职责**：数据采集层。从各信息源采集原始数据，转换为标准 Event，写入 EventLog。
> **对应文档**：[采集层架构](../architecture/modules/collection.md)

## 文件导航

### 框架层

| 文件 | 职责 | 关键类型 |
|------|------|---------|
| `lib.rs` | 模块导出 | — |
| `traits.rs` | 采集器统一 trait | `Collector` trait, `HealthStatus`, `HealthLevel` |
| `manager.rs` | 采集器热插拔管理器 | `CollectorManager` — register/unregister/enable/disable/collect_all |
| `config.rs` | 采集器配置 | 采集器开关、飞书认证等配置 |
| `runner.rs` | 采集器运行器 | 定时触发采集、写入 EventLog |

### 飞书采集器 (`feishu/`)

| 文件 | 职责 | 采集内容 |
|------|------|---------|
| `mod.rs` | 飞书子模块导出 | — |
| `collector.rs` | 飞书采集器主入口 | 实现 `Collector` trait，管理子采集器 |
| `messages.rs` | 消息采集 | @提及、回复、私聊、关键词 |
| `docs.rs` | 文档采集 | 创建、编辑、评论 |
| `projects.rs` | 项目采集 | 任务创建、状态变更、评论 |
| `calendar.rs` | 日历采集 | 日程事件、会议安排 |
| `meetings.rs` | 会议采集 | 会议纪要、逐字稿、待办 |
| `emails.rs` | 邮箱采集 | 发送、接收、回复 |
| `approvals.rs` | 审批采集 | 审批实例 |
| `okr.rs` | OKR 采集 | 目标、关键结果、进度 |
| `bitable.rs` | 多维表格采集 | 记录、字段、视图变更 |
| `spreadsheets.rs` | 电子表格采集 | 单元格变更 |
| `wiki.rs` | 知识库采集 | 节点变更 |
| `minutes.rs` | 妙记采集 | 录音总结、待办、章节 |

### 系统行为采集器 (`system/`)

| 文件 | 职责 | 采集内容 |
|------|------|---------|
| `mod.rs` | 系统子模块导出 | — |
| `app_switch.rs` | 应用切换监听 | 前台应用变化、停留时长 |
| `browser.rs` | 浏览器监听 | URL + 页面标题 |

### 测试文件 (`tests/`)

| 文件 | 职责 |
|------|------|
| `contract.rs` | 采集器 trait 契约测试 |
| `feishu_collectors_l2.rs` | 飞书采集器 L2 集成测试 |
| `real_backend_collector_toggle.rs` | 真实后端采集器开关测试 |

## 关键设计

- **Collector trait**：所有采集器的统一抽象，支持热插拔
- **CollectorManager**：运行时注册/注销、开关控制、批量采集
- **Event 输出**：采集器只产出 `Event`，不直接写入 EventLog（由 runner 负责）

## 修改指引

| 你想改什么 | 先读 | 再改 |
|-----------|------|------|
| 新增飞书子采集器 | `feishu/mod.rs` + `traits.rs` | 新建文件 + 注册到 `feishu/collector.rs` |
| 修改消息过滤规则 | `feishu/messages.rs` | 直接修改 |
| 修改采集器管理逻辑 | `manager.rs` | 修改 `CollectorManager` |
| 新增非飞书采集器 | `traits.rs` | 新建目录 + 实现 `Collector` trait |
| 修改健康检查逻辑 | `traits.rs` + 对应采集器的 `health_check()` | — |
