# dev-agent 任务：修复 Task Discovery 上下文问题

## 问题描述

当用户连续录入两条语义相关的消息时：
1. "我今天要发邮件给lily" → 创建任务
2. "给Lily的邮件已经发送了" → 应该识别为已有任务的状态更新，但系统创建了新任务

根本原因：Task Discovery AI（discovery_ai.rs）是无状态的，每条消息独立调用 adapter.extract()，模型看不到已有的任务上下文。

## 修复方向

在 `discover_with_ai()` 调用时，把当前 Pending/Open 任务列表作为上下文传给 AI 模型，让模型能判断新消息是"新任务"还是"已有任务的状态更新"。

## 涉及文件

1. `crates/wb-processor/src/task/discovery_ai.rs` — AI 驱动的任务发现，需要接收已有任务列表参数
2. `crates/wb-processor/src/task/discovery.rs` — TaskDiscovery 结构体，需要维护或查询已有任务
3. `crates/wb-processor/src/pipeline.rs` — pipeline 中调用 discover_with_ai 的地方，需要传入上下文
4. `crates/wb-ai/src/adapter.rs` — 可能需要修改 Extraction 结构体，添加 is_status_update 和 related_task_id 字段

## 期望行为

当 AI 收到 "给Lily的邮件已经发送了" 时，如果已有一个 Pending/Open 的 "发邮件给lily" 任务，AI 应该返回类似 `{ title: "", is_status_update: true, related_task_id: "xxx" }` 这样的结果，表示这不是新任务而是状态更新。

## 实现要求

1. 修改 `discover_with_ai()` 函数签名，添加已有任务列表参数
2. 修改 `try_ai_extraction()` 函数，传递已有任务列表
3. 修改 `Extraction` 结构体，添加 `is_status_update` 和 `related_task_id` 字段
4. 修改 `TaskDiscovery` 结构体，维护已有任务列表
5. 修改 `pipeline.rs` 中调用 `discover_with_ai()` 的地方，传入已有任务列表
6. 添加单元测试验证新功能

## 验收条件

1. 当用户连续录入两条语义相关的消息时，系统应识别为已有任务的状态更新，而不是创建新任务
2. AI 模型应能接收已有任务列表作为上下文，判断新消息是新任务还是状态更新
3. 当 AI 判断为状态更新时，应返回 is_status_update=true 和 related_task_id
4. 所有现有测试应继续通过
5. 新增功能应有对应的单元测试
