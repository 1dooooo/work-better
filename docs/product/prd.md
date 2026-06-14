---
title: 产品需求文档 (PRD)
type: structural
domain: product
created: 2026-06-14
status: active
---

# Work Better - 产品需求文档 (PRD)

> **维护说明**：当产品需求变更时更新本文档。
> 本文档描述产品功能需求和技术约束，指导开发和测试。

## 产品概述

Work Better 是一个以 Obsidian 为中心的 AI 工作观察者，通过被动采集工作信息，经由分级智能处理，自动维护任务与报告。

详细产品定义见 [产品概述](overview.md)。

---

## 功能需求

### F2：智能处理

#### F2.1.1 分类器

**需求描述**：
- 规则引擎判断事件类型和处理路由
- AI 辅助二次确认非 Archive 路由
- AI 失败时降级到规则结果

**验收标准**：
- 低置信度事件直接归档
- 高价值事件路由到 Instant
- AI 与规则不一致时取 AI 结果并记录审计

**当前状态**：🟡 开发中（规则已实现，AI 辅助在 pipeline 层）

---

#### F2.4.2 小模型审核

**需求描述**：
- 一致性检查：检测内容自相矛盾
- 关键信息覆盖度：验证实体被提及的比例
- 适用于 Summary 和 Extraction 类型的输出

**验收标准**：
- 矛盾表述检测生效
- 覆盖度低于阈值时报告
- reviewer 字段标识为 "small_model"

**当前状态**：🟡 开发中（基础实现完成，集成到 ReviewAgent）

---

#### F2.4.3 大模型审核

**需求描述**：
- 语义深度检查：分析内容是否足够深入
- 分析维度覆盖：验证是否覆盖原因/影响/建议/结论/背景
- 适用于 Analysis 和 Report 类型的输出

**验收标准**：
- 过短内容被标记为浅层分析
- 缺少分析维度时报告
- reviewer 字段标识为 "large_model"

**当前状态**：🟡 开发中（基础实现完成，未集成到 ReviewAgent）

---

## 逻辑缺陷记录

### 缺陷 1：ReviewAgent 未集成大模型审核

**发现日期**：2026-06-14

**问题描述**：
`TieredReview` 定义了 Analysis/Report 应使用 `LargeModelReview`，但 `ReviewAgent` 的 `review` 方法中，`work_record_to_processor_output` 将非 Document/Review 类别映射为 `OutputType::Analysis`，实际上 Analysis 类型的输出从未经过 `LargeModelReview`。

**影响**：
- F2.4.3 大模型审核在流水线中未生效
- Analysis/Report 类型的输出仅经过规则审核

**修复建议**：
在 `reviewer.rs` 中集成 `TieredReview`，根据 `OutputType` 自动选择审核器。

**状态**：⬜ 待修复

---

### 缺陷 2：TaskDiscovery 未使用 AI 版本

**发现日期**：2026-06-14

**问题描述**：
`discovery_ai.rs` 实现了 AI 驱动的任务发现，但 `pipeline.rs` 中调用的是 `TaskDiscovery::discover_from_message`（关键词匹配版本），`discovery_ai::discover_with_ai` 从未在流水线中调用。

**影响**：
- 任务发现仍依赖关键词匹配
- AI 提取能力被浪费

**修复建议**：
在 `pipeline.rs` 中调用 `discovery_ai::discover_with_ai` 异步函数。

**状态**：⬜ 待修复

---

### 缺陷 3："涉及他人"审核逻辑缺失

**发现日期**：2026-06-14

**问题描述**：
产品定义中"涉及他人→规则+小模型+推送"，但 `ReviewAgent` 中未检测"涉及他人"场景，`UserConfirmPush` 已实现但未集成。

**影响**：
- 涉及共享数据的输出未触发用户确认推送
- 违背产品理念"涉及共享数据时推送用户确认"

**修复建议**：
在 `reviewer.rs` 中检测 `!record.people.is_empty() || record.category == Category::Communication`，触发 `UserConfirmPush`。

**状态**：⬜ 待修复

---

## 技术约束

### 约束 1：AI 调用降级策略

所有 AI 调用必须有降级策略：
- AI 失败时降级到规则结果
- AI 超时时降级到规则结果
- AI 置信度低于阈值时降级

### 约束 2：审核分层策略

审核按输出类型分层：
- Summary / Extraction → SmallModelReview
- Analysis / Report → LargeModelReview
- 涉及他人 → SmallModelReview + UserConfirmPush

### 约束 3：处理路由

事件处理路由由分类器决定：
- Instant：高优先级，立即调用 AI 提取
- Aggregate：批量收集后统一处理
- Pattern：长周期数据分析
- Archive：低价值事件，直接归档

---

## 验收流程

### Phase 2 验收

1. 开发完成 L1/L2 测试
2. 测试完成 L3 集成测试、L4 验收测试
3. 代码审查无 CRITICAL 问题
4. 产品审查确认功能符合预期
5. 签字确认进入 Phase 3

### 相关文档

| 文档 | 关系 |
|------|------|
| [产品概述](overview.md) | 产品方向和用户场景 |
| [Phase 2 验收标准](../testing/acceptance-criteria-phase2.md) | 详细验收标准 |
| [功能索引](../features/index.md) | 功能状态 |
| [功能完成标准](../features/completion-criteria.md) | 完成等级定义 |
