---
title: Phase 2 验收标准
type: structural
domain: testing
created: 2026-06-14
status: active
---

# Phase 2 验收标准 — F2.1.1 / F2.4.2 / F2.4.3

> **维护说明**：当验收标准变更时更新本文档。
> 本文档定义 AI Pipeline Phase 2 三个核心功能的验收标准。

## F2.1.1 分类器（Classifier + AI 辅助）

### 功能定义

对事件进行分类，返回处理路由（Instant/Aggregate/Pattern/Archive）。
规则引擎为基础，AI 辅助二次确认非 Archive 路由。

### 验收标准

| # | 标准 | 验证方法 | 状态 |
|---|------|----------|------|
| AC-1 | 低置信度事件直接归档，不调用 AI | L1: `test_low_confidence_archives` | PASS |
| AC-2 | 高价值事件（TaskUpdate/Approval/ManualNote）路由到 Instant | L1: `test_task_update_is_instant` 等 | PASS |
| AC-3 | 消息类事件根据 @mention 判断路由 | L1: `test_message_with_mention_is_instant` | PASS |
| AC-4 | AI 二次分类仅对非 Archive 路由执行 | L2: `test_classify_ai_passes_to_extraction` | PASS |
| AC-5 | AI 与规则不一致时取 AI 结果并记录审计 | L1: `test_classify_ai_overrides_rule` | PASS |
| AC-6 | AI 失败时降级到规则结果 | L1: `test_classify_ai_fallback_on_error` | PASS |
| AC-7 | 分类结果正确映射到 ProcessingRoute | L1: `test_pipeline_process_aggregate_route` | PASS |

### 已知限制

- AI 分类调用在 pipeline 层，classifier.rs 本身保持纯规则设计
- 这是合理的架构选择：分类器保持简单，AI 增强在流水线层

---

## F2.4.2 小模型审核（SmallModelReview）

### 功能定义

对 Summary 和 Extraction 类型的输出进行一致性检查和关键信息覆盖度验证。

### 验收标准

| # | 标准 | 验证方法 | 状态 |
|---|------|----------|------|
| AC-1 | 正常输出通过审核 | L1: `test_small_model_clean_output_approved` | PASS |
| AC-2 | 矛盾表述检测（如"已批准"+"已拒绝"） | L1: `test_small_model_contradiction_detected` | PASS |
| AC-3 | 关键信息覆盖度低于阈值时报告 | L1: `test_small_model_coverage_below_threshold` | PASS |
| AC-4 | 覆盖度高于阈值时通过 | L1: `test_small_model_coverage_above_threshold` | PASS |
| AC-5 | 自定义覆盖度阈值生效 | L1: `test_small_model_custom_coverage_threshold` | PASS |
| AC-6 | 无实体时跳过覆盖度检查 | L1: `test_small_model_no_entities_skips_coverage` | PASS |
| AC-7 | reviewer 字段标识为 "small_model" | L1: 验证 result.reviewer | PASS |
| AC-8 | 仅对 Document/Review 类别启用 | L2: `test_review_report_uses_small_model` | PASS |

### 已知限制

- 当前实现是规则+关键词匹配，非真正调用小模型 API
- 矛盾检测仅基于预定义关键词对，覆盖度检查基于字符串包含

---

## F2.4.3 大模型审核（LargeModelReview）

### 功能定义

对 Analysis 和 Report 类型的输出进行语义深度检查和分析维度覆盖验证。

### 验收标准

| # | 标准 | 验证方法 | 状态 |
|---|------|----------|------|
| AC-1 | 深度分析内容通过审核 | L1: `test_large_model_deep_analysis_approved` | PASS |
| AC-2 | 过短内容被标记为浅层分析 | L1: `test_large_model_shallow_analysis` | PASS |
| AC-3 | 缺少分析维度时报告 | L1: `test_large_model_missing_dimensions` | PASS |
| AC-4 | 至少覆盖一个维度（原因/影响/建议/结论/背景） | L1: 维度检查逻辑 | PASS |
| AC-5 | reviewer 字段标识为 "large_model" | L1: 验证 result.reviewer | PASS |
| AC-6 | 仅对 Analysis/Report 类型启用 | L2: TieredReview 路由逻辑 | PASS |

### 已知限制

- 当前实现是规则+关键词匹配，非真正调用大模型 API
- 分析维度检查基于字符串包含，可能误判
- **关键缺陷**：reviewer.rs 中未集成 LargeModelReview，仅集成了 SmallModelReview

---

## 逻辑缺陷清单

### 缺陷 1：ReviewAgent 未集成大模型审核

**位置**：`crates/wb-processor/src/reviewer.rs`

**问题**：
- `review_with_small_model` 函数仅调用 `SmallModelReview`
- `TieredReview` 定义了 Analysis/Report 应使用 `LargeModelReview`
- 但 ReviewAgent 的 `review` 方法中，`work_record_to_processor_output` 将非 Document/Review 类别映射为 `OutputType::Analysis`
- 实际上 Analysis 类型的输出从未经过 `LargeModelReview`

**影响**：
- F2.4.3 大模型审核在流水线中未生效
- Analysis/Report 类型的输出仅经过规则审核，未经过语义深度检查

**建议修复**：
```rust
// reviewer.rs 中需要集成 TieredReview，而非仅 SmallModelReview
fn review_with_small_model(&self, record: &WorkRecord, rule_issues: Vec<Issue>) -> ReviewResult {
    let tiered = self.tiered_review.as_ref().expect("tiered_review should be set");
    let output = work_record_to_processor_output(record);
    let model_result = tiered.review(&output);  // TieredReview 会根据 OutputType 选择审核器
    // ...
}
```

### 缺陷 2：TaskDiscovery 未使用 AI 版本

**位置**：`crates/wb-processor/src/pipeline.rs`

**问题**：
- dev-output 中定义了 `discovery_ai.rs` 使用 LLM 分析消息内容
- 但 pipeline.rs 中调用的是 `self.task_discovery.discover_from_message(&event_text)`
- 这是 `TaskDiscovery::discover_from_message`，即关键词匹配版本
- `discovery_ai::discover_with_ai` 异步函数从未在流水线中调用

**影响**：
- F2.1.1 的 AI 任务发现功能未在流水线中生效
- 任务发现仍依赖关键词匹配，AI 提取能力被浪费

**建议修复**：
```rust
// pipeline.rs 中需要调用 AI 版本
let event_text = Self::extract_text_from_event(event);
let discovery_tasks = discovery_ai::discover_with_ai(&event_text, &*self.task_runner.adapter(ModelSize::Small)).await;
```

### 缺陷 3："涉及他人"审核逻辑缺失

**位置**：`crates/wb-processor/src/reviewer.rs`

**问题**：
- dev-output 定义："涉及他人→规则+小模型+推送"
- 但 `review` 方法中未检测"涉及他人"场景
- `UserConfirmPush` 已实现，但未集成到 ReviewAgent

**影响**：
- 涉及共享数据的输出未触发用户确认推送
- 违背产品理念"涉及共享数据时推送用户确认"

**建议修复**：
```rust
// reviewer.rs 中需要检测"涉及他人"场景
fn review(&self, record: &WorkRecord) -> ReviewResult {
    // ...
    let involves_others = !record.people.is_empty() || record.category == Category::Communication;
    if involves_others && self.tiered_review.is_some() {
        // 规则 + 小模型 + 推送
        let result = self.review_with_small_model(record, rule_issues);
        // 触发 UserConfirmPush
        self.trigger_user_confirm(record);
        result
    } else if use_small_model {
        self.review_with_small_model(record, rule_issues)
    } else {
        self.review_with_rules_only(rule_issues)
    }
}
```

---

## 测试覆盖总结

| 功能 | L1 单元测试 | L2 集成测试 | L4 验收测试 | 总计 |
|------|------------|------------|------------|------|
| F2.1.1 分类器 | 11 | 3 | 6 | 20 |
| F2.4.2 小模型审核 | 8 | 2 | - | 10 |
| F2.4.3 大模型审核 | 4 | 1 | - | 5 |
| **合计** | **23** | **6** | **6** | **35** |

---

## 相关文档

| 文档 | 关系 |
|------|------|
| [功能索引](../features/index.md) | 功能状态定义 |
| [功能完成标准](../features/completion-criteria.md) | 完成等级定义 |
| [产品概述](../product/overview.md) | 产品方向 |
| [Review Report](../../.workflow/artifacts/fix-ai-pipeline-phase2/review-report.json) | 代码审查结果 |
