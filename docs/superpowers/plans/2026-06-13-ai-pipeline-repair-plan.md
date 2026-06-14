---
title: AI Pipeline 修复计划
date: 2026-06-13
status: done
goal: 修复设计与实现的断裂，让 AI 真正参与事件处理全流程
phases: 4
estimated_hours: 29h
actual_hours: ~32h（含 Phase 4 质量打磨）
---

# AI Pipeline 修复计划

> 修复设计与实现的断裂，让 AI 真正参与事件处理全流程。

## 背景

产品设计文档（PRD、Phase 计划、架构文档）从上到下逻辑自洽，但在实现层多处断裂：

| 设计要求 | 实际实现 | 断裂点 |
|---------|---------|--------|
| Classifier = 规则 + 小模型 | 纯规则，run_classify() 从未调用 | 有代码不用 |
| Task Discovery 用 AI 提取 | 45 个关键词做 string.find() | 完全未实现 |
| Task Discovery 接入 pipeline | task/ 模块独立存在 | 未集成 |
| ReviewAgent 三层审核 | 只有规则层 | 缺两层 |
| 测试验证行为 | 469 个测试全用 Mock | 只验证结构 |

详见 [设计与实现断裂分析](./2026-06-13-design-implementation-gap-analysis.md)。

## 全局约定

### 角色职责边界

| 角色 | 交付物 | 验收方式 | 禁止做 |
|------|--------|---------|--------|
| Dev | 代码 + L1(单元) + L2(集成) 测试 | `cargo test` 通过 | 不写 L3/L4/L5 测试，不做产品验收 |
| Test | L3(组件集成) + L4(场景验收) + L5(E2E) 测试 + 测试报告 | 测试用例可执行、断言明确 | 不改业务代码，不做产品决策 |
| Reviewer | 代码审查报告 + 安全审查报告 + 验收标准对照表 | 每条 CR 有明确的 pass/fail/warn | 不写代码，不做测试 |
| Product | 验收标准定义 + 场景确认 + 最终签字 | 逐条确认行为符合 PRD | 不写代码，不做技术审查 |

### 测试层级定义

| 层级 | 定义 | Mock 策略 | 谁写 |
|------|------|----------|------|
| L1 | 单个函数/方法的逻辑正确性 | 全 Mock | Dev |
| L2 | 两个组件之间的数据传递 | Mock 外部依赖（HTTP、文件系统） | Dev |
| L3 | 多个组件串联，验证完整链路 | Mock HTTP（wiremock），真实内部组件 | Test |
| L4 | 特定业务场景的端到端行为 | Mock HTTP，真实全链路 | Test |
| L5 | 真实环境下的端到端流程 | 无 Mock（.dev-data 隔离环境） | Test |

### 交付门禁规则

每个阶段必须通过以下门禁才能进入下一阶段：
1. Dev 的 `cargo test` 全绿
2. Test 的 L3/L4 测试可执行且全绿
3. Reviewer 的审查报告无 CRITICAL 项
4. Product 逐条签字确认

---

## 阶段一：修补测试基础设施

### 目标

让测试体系能检测出「设计要求的行为未实现」，而不仅仅是「代码不崩溃」。

### 1.1 Dev 任务

#### 任务 1.1.1：修复 G2 验收步骤

**当前问题**：`tests/acceptance/src/steps/g2_processing.rs` 的 37 个步骤全部是字符串赋值，不调用真实组件。

**修复内容**：

| 步骤 | 当前实现 | 修复为 |
|------|---------|--------|
| Given `一条来自飞书的即时消息` | `world.source = Some("feishu".into())` | 创建真实 Event 对象，设置 source=Source::Feishu，event_type=EventType::Message，content 为真实测试消息 |
| When `分类` | `world.processing_result = Some("classified".into())` | 调用 Classifier::classify(&event)，将结果存入 world |
| When `提取关键信息` | `world.processing_result = Some("extracted".into())` | 调用 TaskRunner.run_extract(&event) 用 MockAdapter，将 Extraction 存入 world |
| When `审核` | `world.processing_result = Some("reviewed".into())` | 调用 ReviewAgent::review(&output)，将 ReviewResult 存入 world |
| Then `应即时处理` | `assert!(prio.contains("P0"))` | 断言 world.route == Some(ProcessingRoute::Instant) |
| Then `生成 WorkRecord` | `assert!(!title.is_empty())` | 断言 world.work_record 存在，且 title、summary、category 字段非空 |

**L1/L2 测试**：

```
test_g2_classify_calls_real_classifier:
  Given: Event { source: Feishu, event_type: Message, content: "请帮忙处理一下" }
  When: classify()
  Then: 返回 ProcessingRoute::Instant

test_g2_extract_calls_real_task_runner:
  Given: Event { content: "明天下午5点完成代码发布" }
  When: run_extract() with MockAdapter
  Then: Extraction 结构体非空，title 非空
```

**验收标准**：
- [ ] `cargo test --test g2` 通过
- [ ] G2 步骤中至少 3 个调用了真实组件（Classifier、TaskRunner、ReviewAgent）
- [ ] G2 的 Then 步骤断言的是组件返回值，不是 Given 设置的字符串

#### 任务 1.1.2：修复 G4 验收步骤

**当前问题**：`tests/acceptance/src/steps/g4_task.rs` 的步骤同样是字符串赋值。

**修复内容**：

| 步骤 | 当前实现 | 修复为 |
|------|---------|--------|
| Given `一条包含任务信息的消息` | `world.content = Some("...")` | 创建 Event，content 为测试消息 |
| When `分析是否包含任务` | `world.has_task = Some(true)` | 调用 TaskDiscovery.discover_from_message(&event)，将结果存入 world |
| Then `应发现一个任务` | `assert!(world.has_task)` | 断言 world.discovery_result.unwrap().candidates.len() > 0 |
| Then `任务应有截止日期` | `assert!(world.has_due_date)` | 断言 candidates[0].due_date.is_some() |

**L1/L2 测试**：

```
test_g4_discover_from_message_with_deadline:
  Given: Event { content: "明天下午5点完成代码发布" }
  When: discover_from_message()
  Then: candidates.len() == 1, candidates[0].due_date.is_some()

test_g4_discover_from_message_without_task:
  Given: Event { content: "今天天气不错" }
  When: discover_from_message()
  Then: candidates.len() == 0
```

**验收标准**：
- [ ] `cargo test --test g4` 通过
- [ ] G4 步骤调用了真实 TaskDiscovery
- [ ] Then 步骤断言的是 TaskDiscovery 返回值

#### 任务 1.1.3：创建 wiremock 基础设施

**内容**：在 `tests/common/` 下创建 HTTP mock server 模块，供 L3/L4 测试复用。

**提供的能力**：
- `start_mock_server()` → 返回 mock server 地址
- `mock_classify_response(input, output)` → 注册分类 mock
- `mock_extract_response(input, output)` → 注册提取 mock
- `mock_discover_response(input, output)` → 注册任务发现 mock

**验收标准**：
- [ ] `tests/common/wiremock.rs` 存在
- [ ] 其他测试文件可以 `mod common` 引入
- [ ] mock server 可以启动并响应预设请求

### 1.2 Test 任务

#### 任务 1.2.1：编写回归测试清单

**交付物**：`docs/testing/regression-checklist.md`

**内容**：对当前 469 个测试逐一分类：

| 分类 | 标准 | 处理方式 |
|------|------|---------|
| 有效单元测试 | 断言真实逻辑，不依赖 Mock 返回值 | 保留 |
| 伪测试 | 只断言 result.is_ok() 或 Mock 返回值 | 标记为 `#[ignore]`，附原因 |
| 需升级测试 | 断言结构但不断言行为 | 标记待升级，阶段二处理 |

**验收标准**：
- [ ] 清单覆盖所有 469 个测试
- [ ] 每个测试有明确分类和处理方式
- [ ] 伪测试被标记为 `#[ignore]`

#### 任务 1.2.2：验证 G2/G4 修复

**验证方法**：
1. 在 Classifier::classify()、TaskRunner::run_extract()、TaskDiscovery::discover_from_message() 中临时添加 println! 日志
2. 运行 G2/G4 测试
3. 确认日志输出中包含真实组件调用
4. 移除临时日志

**验收标准**：
- [ ] G2 测试运行时，Classifier 和 TaskRunner 被真实调用
- [ ] G4 测试运行时，TaskDiscovery 被真实调用
- [ ] 测试通过（不只是「不崩溃」，而是断言正确）

### 1.3 Reviewer 任务

#### 任务 1.3.1：审查 Mock 边界

**审查清单**：

| 检查项 | 标准 | 结果 |
|--------|------|------|
| L1 测试中 MockAdapter 使用 | 仅用于隔离外部依赖，不用于跳过内部逻辑 | pass/fail |
| L2 测试中 Mock 范围 | 仅 Mock HTTP 和文件系统，不 Mock 内部组件 | pass/fail |
| G2/G4 验收步骤 | 不使用 MockAdapter，使用真实组件 | pass/fail |
| wiremock 使用范围 | 仅在 L3/L4 测试中使用 | pass/fail |

**验收标准**：
- [ ] 审查报告中无 CRITICAL 项
- [ ] 每个 FAIL 项有对应的修复建议

#### 任务 1.3.2：审查 G2/G4 修复

**审查清单**：

| 检查项 | 标准 |
|--------|------|
| G2 的 When 步骤 | 是否调用了真实组件而非字符串赋值 |
| G2 的 Then 步骤 | 是否断言了组件返回值而非 Given 字符串 |
| G4 的 When 步骤 | 是否调用了真实 TaskDiscovery |
| G4 的 Then 步骤 | 是否断言了 DiscoveryResult 的具体字段 |
| 测试隔离 | 每个测试是否独立，不依赖其他测试的状态 |

### 1.4 Product 任务

#### 任务 1.4.1：定义验收标准模板

**交付物**：`docs/testing/acceptance-criteria-template.md`

**模板内容**：

```markdown
Feature: [Feature ID + 名称]
PRD 引用: [prd.md 中的对应章节]

行为描述:
  - [行为1]: 当 [条件] 时，系统应该 [行为]
  - [行为2]: ...

验收场景:
  场景1:
    输入: [具体输入]
    预期输出: [具体输出]
    验证点: [验证什么字段/行为]

  场景2:
    ...

边界条件:
  - [边界1]: 当 [极端条件] 时，系统应该 [行为]
  - [边界2]: ...

E2E 测试路径:
  [从哪个入口触发] → [经过哪些组件] → [产出什么结果]
```

**验收标准**：
- [ ] 模板覆盖行为描述、验收场景、边界条件、E2E 路径
- [ ] 至少为 F2.1.1、F2.4.2、F2.4.3 用此模板填写了验收标准

#### 任务 1.4.2：回溯标记 Feature Index

**修改 `docs/features/index.md`**：

| Feature | 当前状态 | 修改为 | 原因 |
|---------|---------|--------|------|
| F2.1.1 | done | in-progress | Classifier 缺少 AI 层，run_classify() 未被调用 |
| F2.4.2 | done | in-progress | ReviewAgent 缺少小模型层 |
| F2.4.3 | done | in-progress | ReviewAgent 缺少大模型层 |

**验收标准**：
- [ ] Feature Index 状态已更新
- [ ] 每个 in-progress 项附带缺失说明

### 阶段一总验收门禁

```
□ Dev: G2 验收步骤调用真实 Classifier + TaskRunner + ReviewAgent
□ Dev: G4 验收步骤调用真实 TaskDiscovery
□ Dev: wiremock 基础设施可用
□ Dev: cargo test 全绿
□ Test: 回归测试清单覆盖 469 个测试
□ Test: 伪测试标记为 #[ignore]
□ Test: G2/G4 修复验证通过
□ Reviewer: Mock 边界审查无 CRITICAL
□ Reviewer: G2/G4 修复审查无 CRITICAL
□ Product: 验收标准模板定义完成
□ Product: Feature Index 回溯标记完成
□ Product: 签字确认进入阶段二
```

---

## 阶段二：修复核心链路

### 目标

让 AI 真正参与事件处理全流程：Classifier 用 AI 辅助分类，Task Discovery 用 AI 提取任务，ReviewAgent 用小模型做质量检查。

### 2.1 Dev 任务

#### 任务 2.1.1：Classifier 接入 AI

**当前问题**：`classifier.rs` 纯规则分类，`TaskRunner::run_classify()` 存在但从未被调用。

**修复内容**：

```rust
// pipeline.rs 中的分类流程改为:
let rule_result = Classifier::classify(&event);

// 对非 Archive 路由，调用 AI 做二次确认
if rule_result.route != ProcessingRoute::Archive {
    match task_runner.run_classify(&event).await {
        Ok(ai_result) => {
            // AI 分类与规则分类不一致时，取 AI 结果并记录审计
            if ai_result.route != rule_result.route {
                audit.record("classify_ai_override", ...);
            }
            final_result = ai_result;
        }
        Err(_) => {
            // AI 调用失败，降级到规则结果
            audit.record("classify_ai_fallback", ...);
            final_result = rule_result;
        }
    }
} else {
    final_result = rule_result;
}
```

**L1 测试**：

```
test_classify_ai_agrees_with_rule:
  Given: Event { event_type: Message, content: "请帮忙处理一下" }
  And: MockAdapter 返回 route = Instant
  When: classify() -> run_classify()
  Then: final_result.route == Instant

test_classify_ai_overrides_rule:
  Given: Event { event_type: Message, content: "今天天气不错" }
  And: 规则分类返回 Aggregate（普通消息）
  And: MockAdapter 返回 route = Instant（AI 认为有任务）
  When: classify() -> run_classify()
  Then: final_result.route == Instant
  And: 审计记录包含 "classify_ai_override"

test_classify_ai_fallback_on_error:
  Given: Event { event_type: Message }
  And: MockAdapter 的 classify 返回 Err
  When: classify() -> run_classify()
  Then: final_result.route == 规则分类结果
  And: 审计记录包含 "classify_ai_fallback"
```

**L2 测试**：

```
test_classify_ai_passes_to_extraction:
  Given: Event 经过 AI 分类后 route = Instant
  When: pipeline 继续执行
  Then: TaskRunner::run_extract() 被调用
```

**验收标准**：
- [ ] run_classify() 在 pipeline 中被调用（非 Archive 路由）
- [ ] AI 失败时降级到规则结果，不阻塞流程
- [ ] AI 覆盖规则结果时记录审计
- [ ] L1 测试 3 个 + L2 测试 1 个全绿

#### 任务 2.1.2：Task Discovery 接入 pipeline

**当前问题**：`task/` 模块独立存在，pipeline 不调用。

**修复内容**：

```rust
// pipeline.rs 中，在 Extraction 之后、ReviewAgent 之前插入:
let work_record = extractor.extract(&extraction);

// Task Discovery：检查是否为任务
let discovery_result = task_discovery.discover_from_message(&event).await;
if let Some(candidate) = discovery_result.candidates.first() {
    work_record.category = Category::Task;
    work_record.task_due = candidate.due_date;
    work_record.task_priority = candidate.priority;
    audit.record("task_discovered", ...);
}

// ReviewAgent
let review_result = reviewer.review(&output);
```

**L1 测试**：

```
test_discovery_sets_task_category:
  Given: Event { content: "明天下午5点完成代码发布" }
  And: TaskDiscovery 返回 candidate { due_date: Some(...), priority: Some(High) }
  When: pipeline 处理
  Then: work_record.category == Task
  Then: work_record.task_due.is_some()
  Then: work_record.task_priority == Some(High)

test_discovery_no_candidate_keeps_original_category:
  Given: Event { content: "今天天气不错" }
  And: TaskDiscovery 返回 candidates 为空
  When: pipeline 处理
  Then: work_record.category == Info（保持原分类）
```

**L2 测试**：

```
test_discovery_result_flows_to_review:
  Given: TaskDiscovery 发现了任务
  When: pipeline 继续执行到 ReviewAgent
  Then: ReviewAgent 收到的 output 中 category == Task
```

**验收标准**：
- [ ] TaskDiscovery 在 pipeline 中被调用
- [ ] Discovery 结果正确传递到 WorkRecord
- [ ] Discovery 失败时不阻塞流程
- [ ] L1 测试 2 个 + L2 测试 1 个全绿

#### 任务 2.1.3：Task Discovery AI 化

**当前问题**：`discovery_message.rs` 用 45 个关键词做 string.find()。

**修复内容**：新增 `discovery_ai.rs`，调用 LLM 分析消息内容。

```
输入: Event { content, source, timestamp }
Prompt: "分析以下消息，判断是否包含任务。如包含，提取：标题、截止日期、优先级、责任人"
输出: DiscoveryResult { candidates: Vec<TaskCandidate> }

保留关键词匹配作为 fallback：
1. 先调 AI 分析
2. AI 返回 candidates 非空 -> 使用 AI 结果
3. AI 返回空或失败 -> 降级到关键词匹配
4. 两者都为空 -> 确认无任务
```

**L1 测试**：

```
test_ai_discovers_task_with_deadline:
  Given: Event { content: "明天下午5点完成代码发布" }
  And: MockAdapter 返回 candidate { title: "完成代码发布", due_date: "明天下午5点" }
  When: discover_from_message()
  Then: candidates.len() == 1
  Then: candidates[0].title == "完成代码发布"

test_ai_discovers_implicit_task:
  Given: Event { content: "登录超时的问题需要排查一下" }
  And: MockAdapter 返回 candidate { title: "排查登录超时问题" }
  When: discover_from_message()
  Then: candidates.len() == 1

test_ai_returns_empty_for_non_task:
  Given: Event { content: "今天天气不错" }
  And: MockAdapter 返回 candidates 为空
  When: discover_from_message()
  Then: candidates.len() == 0

test_ai_fallback_to_keywords:
  Given: Event { content: "请你帮忙处理一下这个bug" }
  And: MockAdapter 的 discover 返回 Err
  When: discover_from_message()
  Then: 降级到关键词匹配，candidates.len() >= 1（因为"请你帮忙"是关键词）
```

**验收标准**：
- [ ] AI 提取优先于关键词匹配
- [ ] AI 失败时降级到关键词匹配
- [ ] L1 测试 4 个全绿

#### 任务 2.1.4：ReviewAgent 补充小模型层

**当前问题**：ReviewAgent 只有 4 条规则，reviewer 永远是 "rule"。

**修复内容**：

```rust
// reviewer.rs 中，根据输出类型选择审核策略:
match output.category {
    Category::Report | Category::Summary => {
        // 规则层 + 小模型层
        let rule_result = rule_review(output);
        let model_result = task_runner.run_review(output).await;
        merge_results(rule_result, model_result)
    }
    _ if output.involves_others => {
        // 规则层 + 小模型层 + 推送用户
        let rule_result = rule_review(output);
        let model_result = task_runner.run_review(output).await;
        push_to_user(merge_results(rule_result, model_result))
    }
    _ => {
        // 仅规则层
        rule_review(output)
    }
}
```

**L1 测试**：

```
test_review_report_uses_small_model:
  Given: ProcessorOutput { category: Report, ... }
  And: MockAdapter 返回 review result
  When: review()
  Then: result.reviewer == "small_model"
  Then: result.verdict == Approved 或 NeedsFix

test_review_task_uses_rule_only:
  Given: ProcessorOutput { category: Task, involves_others: false, ... }
  When: review()
  Then: result.reviewer == "rule"

test_review_involving_others_uses_small_model:
  Given: ProcessorOutput { involves_others: true, ... }
  And: MockAdapter 返回 review result
  When: review()
  Then: result.reviewer == "small_model"
```

**验收标准**：
- [ ] ReviewAgent 的 reviewer 字段根据输出类型变化
- [ ] 报告/摘要和涉及他人信息的输出使用小模型审核
- [ ] L1 测试 3 个全绿

### 2.2 Test 任务

#### 任务 2.2.1：L3 集成测试——完整链路

**测试文件**：`tests/integration/pipeline_full_chain.rs`

**测试用例**：

```
test_full_chain_message_with_task:
  输入: Event { source: Feishu, event_type: Message, content: "明天下午5点完成代码发布" }
  Mock: wiremock 返回 AI 分类=Instant, AI 提取=任务, AI 发现=有任务
  验证:
    - Classifier 调用 AI（审计记录有 "classify" step）
    - Extraction 包含 title, summary, detail
    - TaskDiscovery 发现任务（审计记录有 "task_discovered" step）
    - WorkRecord.category == Task
    - WorkRecord.task_due 有值
    - ReviewAgent 执行审核（审计记录有 "review" step）
    - PersistStep 写入文件
    - 审计链路 trace_id 贯穿所有步骤

test_full_chain普通消息:
  输入: Event { content: "今天天气不错" }
  Mock: wiremock 返回 AI 分类=Aggregate, AI 提取=Info, AI 发现=无任务
  验证:
    - Classifier 调用 AI
    - WorkRecord.category == Info
    - WorkRecord.task_due 为空
    - 不调用 TaskDiscovery（Aggregate 路由跳过）

test_full_chain低置信度归档:
  输入: Event { source: System, confidence: Low }
  验证:
    - Classifier 直接归档，不调 AI
    - 不调用 Extraction
    - 不调用 ReviewAgent
    - 审计记录有 "classify" step，route = Archive
```

**验收标准**：
- [ ] 3 个 L3 测试全绿
- [ ] 每个测试验证完整链路（从 Event 到 WorkRecord）
- [ ] 审计链路 trace_id 贯穿所有步骤

#### 任务 2.2.2：L4 验收测试——6 个场景

**测试文件**：`tests/acceptance/pipeline_scenarios.rs`

**场景 1：含截止日期的消息**

```
输入: Event {
  source: Feishu,
  event_type: Message,
  content: "明天下午5点完成代码发布",
  timestamp: "2026-06-13T10:00:00Z"
}
Mock: wiremock 返回:
  - classify: route=Instant, confidence=0.9
  - extract: title="完成代码发布", summary="代码发布任务", category=Task
  - discover: candidates=[{title:"完成代码发布", due_date:"2026-06-14T17:00:00+08:00", priority:High}]

断言:
  - work_record.title == "完成代码发布"
  - work_record.category == Category::Task
  - work_record.task_due == Some("2026-06-14T17:00:00+08:00")
  - work_record.task_priority == Some(Priority::High)
  - 审计链路有 4 个步骤: classify -> extract -> discover -> review
  - PersistStep 写入了 Obsidian 文件
```

**场景 2：普通飞书消息**

```
输入: Event { content: "今天天气不错" }
Mock: wiremock 返回:
  - classify: route=Aggregate, confidence=0.3
  - extract: title="天气讨论", category=Info

断言:
  - work_record.category == Category::Info
  - work_record.task_due == None
  - work_record.task_priority == None
  - 审计链路有 3 个步骤: classify -> extract -> review（无 discover）
```

**场景 3：含隐式任务的消息**

```
输入: Event { content: "登录超时的问题需要排查一下" }
Mock: wiremock 返回:
  - classify: route=Instant, confidence=0.8
  - extract: title="排查登录超时问题", category=Task
  - discover: candidates=[{title:"排查登录超时", priority:Medium}]

断言:
  - work_record.title 包含 "登录超时"
  - work_record.category == Category::Task
  - work_record.task_priority == Some(Priority::Medium)
```

**场景 4：审批消息**

```
输入: Event { event_type: Approval, content: "审批：Q2 OKR 评分" }
Mock: wiremock 返回:
  - classify: route=Instant, confidence=0.95
  - extract: title="Q2 OKR 评分审批", category=Task

断言:
  - work_record.category == Category::Task
  - route == ProcessingRoute::Instant
```

**场景 5：会议待办**

```
输入: Event { event_type: Meeting, content: "会议结束，待办：张三负责修复首页bug" }
Mock: wiremock 返回:
  - classify: route=Instant, confidence=0.9
  - extract: title="修复首页bug", category=Meeting, people=["张三"]
  - discover: candidates=[{title:"修复首页bug", assignee:"张三"}]

断言:
  - work_record.category == Category::Meeting
  - work_record.people 包含 "张三"
  - work_record.task_due 可能有值
```

**场景 6：低置信度事件**

```
输入: Event { source: System, confidence: Low, content: "系统日志: ..." }
无 Mock（不调 AI）

断言:
  - route == ProcessingRoute::Archive
  - work_record.confidence < 0.4
  - 审计链路只有 1 个步骤: classify
  - 不调用 Extraction、TaskDiscovery、ReviewAgent
```

**验收标准**：
- [ ] 6 个 L4 测试全绿
- [ ] 每个测试断言具体字段值，不只是非空
- [ ] 每个测试验证审计链路步骤数

#### 任务 2.2.3：回归测试

**验收标准**：
- [ ] `cargo test` 全绿
- [ ] 阶段一标记为 `#[ignore]` 的伪测试仍然被跳过
- [ ] 新增测试数量 >= 10（L1 7 个 + L2 3 个 + L3 3 个 + L4 6 个）

### 2.3 Reviewer 任务

#### 任务 2.3.1：代码审查——Classifier AI 接入

| 检查项 | 标准 | 结果 |
|--------|------|------|
| AI 分类的调用时机 | 仅对非 Archive 路由调用 | pass/fail |
| AI 失败的降级策略 | 降级到规则结果，不阻塞流程 | pass/fail |
| AI 覆盖规则的审计 | 覆盖时记录审计，包含原结果和新结果 | pass/fail |
| 性能影响 | AI 调用是异步的，不阻塞其他事件处理 | pass/fail |
| 成本控制 | AI 分类使用小模型，不使用大模型 | pass/fail |

#### 任务 2.3.2：代码审查——Task Discovery 接入

| 检查项 | 标准 | 结果 |
|--------|------|------|
| 集成点位置 | 在 Extraction 之后、ReviewAgent 之前 | pass/fail |
| Discovery 结果传递 | due_date、priority 正确写入 WorkRecord | pass/fail |
| 失败处理 | Discovery 失败不阻塞 pipeline | pass/fail |
| 与关键词匹配的关系 | AI 优先，关键词为 fallback | pass/fail |
| 重复发现 | 同一事件不重复创建任务 | pass/fail |

#### 任务 2.3.3：代码审查——ReviewAgent 小模型层

| 检查项 | 标准 | 结果 |
|--------|------|------|
| 审核策略与 processing.md 对齐 | 报告/摘要 -> 规则+小模型；涉及他人 -> 规则+小模型+推送 | pass/fail |
| reviewer 字段标识 | 小模型审核时为 "small_model"，规则审核时为 "rule" | pass/fail |
| 审核结果合并 | 规则层和小模型层的结果正确合并 | pass/fail |
| 成本控制 | 小模型审核使用小模型，token 预算合理 | pass/fail |

#### 任务 2.3.4：安全审查

| 检查项 | 标准 | 结果 |
|--------|------|------|
| Prompt 注入风险 | 用户输入是否被转义后传入 prompt | pass/fail |
| 数据泄漏风险 | 敏感信息（密码、token）是否出现在 prompt 中 | pass/fail |
| LLM 响应验证 | AI 返回的内容是否经过验证再写入 WorkRecord | pass/fail |
| 审计日志安全性 | 审计日志是否包含敏感信息 | pass/fail |

**验收标准**：
- [ ] 审查报告无 CRITICAL 项
- [ ] 每个 FAIL 项有对应的修复建议
- [ ] 修复后重新审查通过

### 2.4 Product 任务

#### 任务 2.4.1：填写 F2.1.1/F2.4.2/F2.4.3 验收标准

**F2.1.1（分类器：规则 + 小模型）**：

```
行为描述:
  - 当事件进入 pipeline 时，Classifier 先用规则分类
  - 对非 Archive 路由，调用 AI 做二次确认
  - AI 与规则不一致时，取 AI 结果并记录审计
  - AI 失败时，降级到规则结果

验收场景:
  场景1: 普通消息 + AI 确认为 Aggregate -> 最终 route = Aggregate
  场景2: 普通消息 + AI 判定为 Instant -> 最终 route = Instant，审计记录 override
  场景3: AI 调用失败 -> 降级到规则结果，审计记录 fallback

边界条件:
  - AI 返回异常 JSON -> 降级到规则结果
  - AI 返回超时 -> 降级到规则结果
  - Archive 路由不调 AI -> 直接归档
```

**F2.4.2（小模型审核）**：

```
行为描述:
  - 报告/摘要类输出：规则层 + 小模型层
  - 涉及他人的信息：规则层 + 小模型层 + 推送用户
  - 其他类型：仅规则层

验收场景:
  场景1: 日报生成 -> reviewer = "small_model"
  场景2: 涉及他人信息 -> reviewer = "small_model" + 推送用户
  场景3: 普通任务 -> reviewer = "rule"

边界条件:
  - 小模型审核失败 -> 仅保留规则层结果
  - 小模型审核超时 -> 仅保留规则层结果
```

**F2.4.3（大模型审核）**：

```
行为描述:
  - 复杂摘要（> 500 字）或涉及多人的报告，按需调用大模型审核
  - 大模型审核在小模型审核之后触发

验收场景:
  场景1: 500 字以上摘要 -> 小模型审核后调大模型
  场景2: 涉及 3 人以上的报告 -> 小模型审核后调大模型
  场景3: 简短任务 -> 仅规则层

边界条件:
  - 大模型审核失败 -> 使用小模型结果
  - Token 预算不足 -> 降级到小模型
```

**验收标准**：
- [ ] 3 个 Feature 的验收标准已填写
- [ ] 每个 Feature 有 3+ 个验收场景
- [ ] 每个 Feature 有边界条件说明

#### 任务 2.4.2：确认 L4 测试场景

| 场景 | 产品行为是否正确 | 边界条件是否覆盖 | 签字 |
|------|----------------|----------------|------|
| 场景1：含截止日期的消息 | 是/否 | 是/否 | |
| 场景2：普通飞书消息 | 是/否 | 是/否 | |
| 场景3：含隐式任务的消息 | 是/否 | 是/否 | |
| 场景4：审批消息 | 是/否 | 是/否 | |
| 场景5：会议待办 | 是/否 | 是/否 | |
| 场景6：低置信度事件 | 是/否 | 是/否 | |

**验收标准**：
- [ ] 6 个场景全部确认「是」
- [ ] 如有「否」，说明原因并要求 Test 修改

#### 任务 2.4.3：更新 PRD 补充逻辑缺陷

**修改 `docs/product/prd.md`，补充 3 处缺失的逻辑描述**：

| 缺陷 | 补充内容 |
|------|---------|
| Task Discovery 触发时机 | 明确：在 Extraction 之后、ReviewAgent 之前触发 |
| Classifier 与 Task Discovery 的关系 | 明确：Classifier 决定路由，Task Discovery 在 Instant 路由中触发 |
| ReviewAgent 成本控制 | 明确：小模型审核使用小模型，大模型审核按需触发，共享 TokenBudget |

**验收标准**：
- [ ] PRD 中补充了 3 处逻辑描述
- [ ] 补充内容与实现一致

### 阶段二总验收门禁

```
□ Dev: Classifier 的 run_classify() 在 pipeline 中被调用
□ Dev: TaskDiscovery 在 pipeline 中被调用
□ Dev: ReviewAgent 的 reviewer 字段不再永远是 "rule"
□ Dev: L1/L2 测试全绿（新增 10+ 个）
□ Test: L3 集成测试 3 个全绿
□ Test: L4 验收测试 6 个全绿
□ Test: 回归测试无失败
□ Reviewer: Classifier AI 接入审查无 CRITICAL
□ Reviewer: Task Discovery 接入审查无 CRITICAL
□ Reviewer: ReviewAgent 小模型层审查无 CRITICAL
□ Reviewer: 安全审查无 CRITICAL
□ Product: F2.1.1/F2.4.2/F2.4.3 验收标准已填写
□ Product: 6 个 L4 场景全部确认
□ Product: PRD 补充 3 处逻辑缺陷
□ Product: 签字确认进入阶段三
```

---

## 阶段三：验证闭环

### 目标

用真实数据验证端到端流程，确认产品行为与 PRD 一致。

### 3.1 Dev 任务

#### 任务 3.1.1：E2E 测试环境准备

**内容**：在 `.dev-data/` 下准备 E2E 测试环境。

**准备内容**：
- 确保 `.dev-data/` 目录结构正确
- 确保 SQLite 数据库为空
- 确保 Obsidian vault 目录为空
- 确保 config.json 配置正确

**验收标准**：
- [ ] `scripts/dev.sh` 可以启动
- [ ] 启动后所有数据在 `.dev-data/` 下

#### 任务 3.1.2：E2E 测试脚本

**内容**：编写 E2E 测试脚本，从飞书采集真实消息到 Obsidian 输出。

**脚本流程**：
1. 启动应用（`.dev-data/` 环境）
2. 触发飞书消息采集（手动或定时）
3. 等待 pipeline 处理完成
4. 检查 SQLite 数据库中的 WorkRecord
5. 检查 Obsidian 目录中的文件
6. 输出测试报告

**验收标准**：
- [ ] 脚本可执行
- [ ] 脚本能采集到至少 5 条消息
- [ ] 脚本能输出测试报告

### 3.2 Test 任务

#### 任务 3.2.1：L5 E2E 测试执行

**测试场景**：

| # | 场景 | 输入 | 预期输出 | 验证点 |
|---|------|------|---------|--------|
| 1 | 飞书消息采集 | 触发飞书采集 | EventLog 中有新事件 | 采集器正常工作 |
| 2 | Pipeline 处理 | Event 进入 pipeline | WorkRecord 写入 SQLite | 完整链路执行 |
| 3 | Task Discovery | 消息含任务信息 | WorkRecord.category = Task | AI 发现任务 |
| 4 | Obsidian 输出 | WorkRecord 持久化 | Obsidian 目录有新文件 | 文件内容正确 |
| 5 | 审计链路 | 任意事件 | 审计记录完整 | trace_id 贯穿 |

**验收标准**：
- [ ] 5 个场景全部通过
- [ ] 每个场景有具体的验证数据

#### 任务 3.2.2：测试报告

**交付物**：`docs/testing/e2e-test-report.md`

**报告内容**：

```markdown
# E2E 测试报告

## 测试环境
- 环境: .dev-data/
- 时间: YYYY-MM-DD HH:MM
- 飞书消息数量: N

## 测试结果

### 场景1: 飞书消息采集
- 输入: 触发飞书采集
- 输出: EventLog 中新增 N 条事件
- 结果: PASS/FAIL
- 证据: [SQLite 查询结果]

### 场景2: Pipeline 处理
- 输入: N 条事件进入 pipeline
- 输出: WorkRecord 写入 SQLite
- 结果: PASS/FAIL
- 证据: [WorkRecord 字段截图]

...

## 性能基线
- 平均处理耗时: Xms/条
- 平均 token 消耗: X tokens/条
- 预估日成本: $X

## 问题清单
- [问题1]: 描述 + 影响 + 建议
```

**验收标准**：
- [ ] 报告覆盖 5 个场景
- [ ] 每个场景有具体证据
- [ ] 性能基线有数据

#### 任务 3.2.3：审计链路验证

**验证方法**：
1. 查询 SQLite 中的审计记录
2. 按 trace_id 分组
3. 检查每个 trace_id 是否包含所有必要步骤

**预期链路**：

| 路由 | 预期步骤 |
|------|---------|
| Instant | classify -> extract -> discover -> review -> persist |
| Aggregate | classify -> extract -> review -> persist |
| Archive | classify |

**验收标准**：
- [ ] 所有 Instant 路由的 trace_id 有 5 个步骤
- [ ] 所有 Aggregate 路由的 trace_id 有 4 个步骤
- [ ] 所有 Archive 路由的 trace_id 有 1 个步骤
- [ ] 无断裂的 trace_id

### 3.3 Reviewer 任务

#### 任务 3.3.1：审计链路审查

| 检查项 | 标准 | 结果 |
|--------|------|------|
| trace_id 完整性 | 每个事件的所有步骤共享同一 trace_id | pass/fail |
| 步骤完整性 | Instant 路由有 5 个步骤 | pass/fail |
| 审计内容 | 每个步骤记录了 input/output/model_used/duration_ms | pass/fail |
| 敏感信息 | 审计日志不包含密码、token 等敏感信息 | pass/fail |

#### 任务 3.3.2：安全审查

| 检查项 | 标准 | 结果 |
|--------|------|------|
| 飞书数据处理 | 敏感信息脱敏后传入 pipeline | pass/fail |
| Obsidian 输出 | 文件内容不包含敏感信息 | pass/fail |
| LLM prompt | 用户输入转义后传入 prompt | pass/fail |

#### 任务 3.3.3：最终代码审查

| 检查项 | 标准 | 结果 |
|--------|------|------|
| 代码质量 | 函数 < 50 行，文件 < 800 行 | pass/fail |
| 错误处理 | 所有错误显式处理 | pass/fail |
| 不可变性 | 无就地修改 | pass/fail |
| 测试覆盖 | 新增代码有对应测试 | pass/fail |

**验收标准**：
- [ ] 审查报告无 CRITICAL 项
- [ ] 所有 FAIL 项已修复

### 3.4 Product 任务

#### 任务 3.4.1：最终验收

**验收方法**：
1. 在 `.dev-data/` 环境中启动应用
2. 手动触发飞书采集
3. 观察 pipeline 处理过程
4. 检查 Obsidian 输出文件
5. 确认行为符合 PRD

**验收清单**：

| 检查项 | PRD 要求 | 实际行为 | 签字 |
|--------|---------|---------|------|
| 消息分类 | 规则 + AI | | |
| 任务发现 | AI 提取任务候选项 | | |
| 截止日期 | 从消息中提取截止日期 | | |
| 审核质量 | 三层审核 | | |
| Obsidian 输出 | 结构化文件 | | |

**验收标准**：
- [ ] 所有检查项签字确认
- [ ] 无产品行为与 PRD 不一致的情况

#### 任务 3.4.2：Feature Index 最终更新

**修改 `docs/features/index.md`**：

| Feature | 状态 | E2E 证据 |
|---------|------|---------|
| F2.1.1 | done | E2E 测试报告 场景X |
| F2.4.2 | done | E2E 测试报告 场景X |
| F2.4.3 | done | E2E 测试报告 场景X |

**验收标准**：
- [ ] 所有相关 Feature 标记为 done
- [ ] 每个 done 项附 E2E 证据链接

#### 任务 3.4.3：产品验收签字

**交付物**：`docs/testing/product-acceptance.md`

**内容**：

```markdown
# 产品验收报告

## 验收范围
- F2.1.1: 分类器（规则 + 小模型）
- F2.4.2: 小模型审核
- F2.4.3: 大模型审核
- Task Discovery AI 化
- Task Discovery 接入 pipeline

## 验收结果
- [ ] 所有 L4 场景通过
- [ ] 所有 L5 E2E 场景通过
- [ ] 审计链路完整
- [ ] 性能基线可接受
- [ ] 无产品行为与 PRD 不一致

## 签字
- Product: [签名] [日期]
- Test: [签名] [日期]
- Reviewer: [签名] [日期]
```

**验收标准**：
- [ ] 验收报告已填写
- [ ] 四方签字完成

### 阶段三总验收门禁

```
□ Dev: E2E 测试环境准备完成
□ Dev: E2E 测试脚本可执行
□ Test: L5 E2E 测试 5 个场景全绿
□ Test: 测试报告已输出
□ Test: 审计链路验证通过
□ Reviewer: 审计链路审查无 CRITICAL
□ Reviewer: 安全审查无 CRITICAL
□ Reviewer: 最终代码审查无 CRITICAL
□ Product: 最终验收签字
□ Product: Feature Index 最终更新
□ Product: 产品验收报告签字
```

---

## 全流程时间估算

| 阶段 | Dev | Test | Reviewer | Product | 总计 |
|------|-----|------|----------|---------|------|
| 一：测试基础设施 | 3h | 2h | 1h | 1h | 7h |
| 二：核心链路修复 | 8h | 4h | 2h | 2h | 16h |
| 三：验证闭环 | 2h | 2h | 1h | 1h | 6h |
| **合计** | **13h** | **8h** | **4h** | **4h** | **29h** |

---

---

## Phase 4 完成说明 (2026-06-14)

### 完成内容

Phase 4 为质量打磨阶段，修复代码审查发现的 MEDIUM 问题：

| 修复项 | ID | 状态 | 说明 |
|--------|-----|------|------|
| UserConfirmPush 错误处理 | M2 | done | `reviewer.rs`: `let _ =` 改为 `match` + `tracing::warn!`，重复推送不再静默丢弃 |
| AI 内容输入净化 | M4 | done | `pipeline.rs`: `due_date` 截断到 100 字符，防止 AI 生成超长内容污染 WorkRecord |
| Source 参数化 | M5 | done | `discovery_ai.rs`: `create_synthetic_event` 接收 `source` 参数，不再硬编码 `FeishuMessage` |

### 遗留问题（design_debt，不阻塞发布）

| 问题 | ID | 优先级 | 说明 |
|------|-----|--------|------|
| 文件长度超限 | M1 | low | pipeline.rs 1105 行、reviewer.rs 885 行，需拆分测试代码 |
| 无界目录扫描 | M3 | low | `find_existing_path_by_id` 遍历目录无条数限制，vault 规模可控时不影响 |

### 验证

- M2/M4/M5 修复均有对应单元测试覆盖
- Phase 3 测试报告：1425 tests 全绿
- 产品审查报告：`.workflow/artifacts/fix-ai-pipeline-phase4/product-review.json`

---

## 关联文档

- [产品需求文档](../product/prd.md)
- [用户使用规则](../product/main-usage-rules.md)
- [Phase 4 计划](../superpowers/plans/2026-06-06-phase4-task-intelligence.md)
- [Phase 3 计划](../superpowers/plans/2026-06-06-phase3-deep-insight.md)
- [处理层架构](../architecture/modules/processing.md)
- [特性索引](../features/index.md)
- [测试架构](../testing/architecture.md)
