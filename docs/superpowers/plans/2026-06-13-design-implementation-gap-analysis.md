---
title: 设计与实现断裂分析
date: 2026-06-13
status: active
goal: 记录设计文档与实际实现之间的断裂点，作为修复计划的依据
---

# 设计与实现断裂分析

> 设计文档从上到下逻辑自洽，问题出在实现层走了捷径。

## 设计文档逻辑链

```
产品定义(PRD)     ✅ 逻辑自洽
    ↓
Pipeline 架构设计  ✅ 逻辑自洽，与 PRD 对齐
    ↓
Phase 计划         ✅ 逻辑自洽，与架构对齐
    ↓
实际实现           ❌ 多处断裂，AI 能力被架空
    ↓
Feature Index      ❌ 标记完成但代码未实现
```

## 断裂点清单

### 断裂 1：Task Discovery 用零 AI（严重）

**设计要求**（Phase 4 Task 3）：
> 使用 AI 模型提取任务候选项（标题、优先级、截止日期）

**实际实现**：
- `discovery_message.rs`：硬编码 45 个中文字关键词做 `string.find()`
- `discovery_email.rs`：硬编码 15 个关键词
- `discovery_meeting.rs`：硬编码 9 个关键词

**影响**：没有关键词的消息（如「登录超时的问题需要排查一下」）不会被发现为任务。

### 断裂 2：Task Discovery 未接入 pipeline（严重）

**设计要求**（Phase 4 Task 3）：
> Event 进入处理流水线 -> TaskDiscovery 检查是否包含任务信号

**实际实现**：
- `task/` 模块是独立模块
- `pipeline.rs` 的 `process()` 方法不调用 TaskDiscovery
- 事件流经 pipeline 时不会触发任务发现

**影响**：即使修复了 AI 提取，pipeline 也不会调用它。

### 断裂 3：Classifier 跳过 AI（中等）

**设计要求**（架构文档 + F2.1.1）：
> 规则引擎 + 小模型，判断事件类型

**实际实现**：
- `classifier.rs`：纯规则分类
- `TaskRunner::run_classify()`：存在但从未被调用
- `has_mention()`：简单 `content.contains('@')` 检查

**影响**：Classifier 无法检测隐含任务的消息（如「登录超时的问题需要排查一下」没有 @ 符号）。

### 断裂 4：Aggregate/Pattern 路由不走 AI（中等）

**设计要求**（Phase 3 Task 3）：
> Event -> Classifier(路由) -> TaskRunner(模型处理) -> Extraction(结构化) -> ReviewAgent(审核) -> PersistStep(持久化) -> WorkRecord

**实际实现**：
- Aggregate 路由：直接存原始数据，跳过 AI
- Pattern 路由：直接存原始数据，跳过 AI
- Archive 路由：直接丢弃，完全不处理

**影响**：大量普通消息不经过 AI 处理，无法提取结构化信息。

### 断裂 5：ReviewAgent 只有规则层（中等）

**设计要求**（Phase 4 Task 4 + 架构文档）：
> 规则层 + 小模型层 + 大模型层

**实际实现**：
- 只有 4 条规则（RequiredFields、ConfidenceThreshold、ContentLength、CategoryConsistency）
- reviewer 字段永远是 "rule"
- 没有调用小模型或大模型的代码路径

**影响**：无法进行语义质量检查，只能做格式校验。

### 断裂 6：Feature Index 标记不准确（次要）

**设计要求**：Feature Index 标记 done 表示功能可用。

**实际实现**：
- F2.1.1（Classifier: 规则+小模型）标记 done，但 AI 分类未接入
- F2.4.2（小模型审核）标记 done，但小模型审核未实现
- F2.4.3（大模型审核）标记 done，但大模型审核未实现

**影响**：给人「功能已完成」的错觉，掩盖了实现断裂。

## 设计文档本身的逻辑缺陷

### 缺陷 1：Task Discovery 的触发时机不明确

Phase 4 说「候选任务进入 pending 状态」，但没说 Task Discovery 在 pipeline 的哪个环节触发。是在 Classifier 之后？还是与 Extraction 并行？还是独立运行？这导致实现时 Task Discovery 变成了独立模块，没有接入 pipeline。

### 缺陷 2：Classifier 路由与 Task Discovery 的关系不清

Classifier 把消息路由到 Aggregate（普通消息），但 Task Discovery 需要分析所有消息来发现任务。如果消息走了 Aggregate 路由（跳过 AI），Task Discovery 就永远看不到它。设计没有说明这个冲突如何解决。

### 缺陷 3：ReviewAgent 的成本控制策略缺失

设计说「报告/摘要 -> 规则层 + 小模型层」，但没有说明：
- 小模型审核的 token 预算从哪来？
- 是否与提取阶段共享 TokenBudget？
- 审核失败后的重试策略是什么？

## 测试为什么没有发现

### 根因 1：Mock 测试造成了「皇帝的新衣」

469 个测试全部通过，但全部使用 MockAdapter：
- MockAdapter 永远返回 `{ title: "Mock Title", confidence: 0.95 }`
- Pipeline 测试断言 title 不为空 -> 通过
- 但从未验证真实输入会产出什么

### 根因 2：G2 验收测试是空壳

`g2_processing.rs` 的 37 个步骤全部是自证循环：
```rust
// Given: 设置 priority = "P0_P1"
// When:  设置 processing_result = "classified"（不做任何真实调用）
// Then:  断言 priority 包含 "P0" <- 验证的是 Given 设置的值
```

### 根因 3：Feature Index 标记无验收标准

标记 done 的依据是「代码存在」而不是「功能可用」。`run_classify()` 存在于代码中，但从未被调用。

### 根因 4：测试金字塔上层为空

```
单元测试：验证组件内部逻辑 ✅（做得好）
    ↓
集成测试：验证组件间数据传递 ❌（用 Mock 跳过）
    ↓
验收测试：验证产品行为 ❌（空壳状态机）
    ↓
E2E 测试：验证真实用户场景 ❌（不存在）
```

## 关联文档

- [修复计划](./2026-06-13-ai-pipeline-repair-plan.md)
- [处理层架构](../architecture/modules/processing.md)
- [Phase 4 计划](../superpowers/plans/2026-06-06-phase4-task-intelligence.md)
- [特性索引](../features/index.md)
