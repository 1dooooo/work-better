---
title: 验收标准模板
date: 2026-06-14
status: active
goal: 为每个功能定义明确的验收标准，确保测试验证的是设计要求的行为
---

# 验收标准模板

> 为每个功能定义明确的验收标准，确保测试验证的是设计要求的行为。

## 模板格式

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
  [从哪个入口触发] -> [经过哪些组件] -> [产出什么结果]
```

---

## F2.1.1：分类器（规则 + 小模型）

**PRD 引用**: docs/product/prd.md - 智能处理 > 事件分类

**行为描述**:
  - 当事件进入 pipeline 时，Classifier 先用规则分类
  - 对非 Archive 路由，调用 AI 做二次确认
  - AI 与规则不一致时，取 AI 结果并记录审计
  - AI 失败时，降级到规则结果

**验收场景**:
  场景1: 普通消息 + AI 确认为 Aggregate
    输入: Event { source: FeishuMessage, content: "今天天气不错" }
    预期输出: route = Aggregate, confidence > 0.5
    验证点: ProcessingRoute 枚举值

  场景2: 普通消息 + AI 判定为 Instant
    输入: Event { source: FeishuMessage, content: "请帮忙处理一下这个bug" }
    预期输出: route = Instant
    验证点: 审计记录包含 "classify_ai_override"

  场景3: AI 调用失败
    输入: Event { source: FeishuMessage, content: "测试消息" } + MockAdapter 返回 Err
    预期输出: route = 规则分类结果
    验证点: 审计记录包含 "classify_ai_fallback"

**边界条件**:
  - AI 返回异常 JSON -> 降级到规则结果
  - AI 返回超时 -> 降级到规则结果
  - Archive 路由不调 AI -> 直接归档

**E2E 测试路径**:
  飞书消息 -> Collector -> Event -> Classifier(规则) -> Classifier(AI确认) -> Extraction -> ReviewAgent -> PersistStep -> WorkRecord

---

## F2.4.2：小模型审核

**PRD 引用**: docs/product/prd.md - 智能处理 > 审核代理

**行为描述**:
  - 报告/摘要类输出：规则层 + 小模型层
  - 涉及他人的信息：规则层 + 小模型层 + 推送用户
  - 其他类型：仅规则层

**验收场景**:
  场景1: 日报生成
    输入: ProcessorOutput { category: Report, content: "今日完成..." }
    预期输出: reviewer = "small_model", verdict = Approved 或 NeedsFix
    验证点: ReviewResult.reviewer 字段

  场景2: 涉及他人信息
    输入: ProcessorOutput { involves_others: true, content: "张三负责..." }
    预期输出: reviewer = "small_model" + 推送用户确认
    验证点: ReviewResult.reviewer 字段 + 通知列表

  场景3: 普通任务
    输入: ProcessorOutput { category: Task, involves_others: false }
    预期输出: reviewer = "rule"
    验证点: ReviewResult.reviewer 字段

**边界条件**:
  - 小模型审核失败 -> 仅保留规则层结果
  - 小模型审核超时 -> 仅保留规则层结果

**E2E 测试路径**:
  Event -> Classifier -> Extraction -> ReviewAgent(规则层) -> ReviewAgent(小模型层) -> PersistStep

---

## F2.4.3：大模型审核

**PRD 引用**: docs/product/prd.md - 智能处理 > 审核代理

**行为描述**:
  - 复杂摘要（> 500 字）或涉及多人的报告，按需调用大模型审核
  - 大模型审核在小模型审核之后触发

**验收场景**:
  场景1: 500 字以上摘要
    输入: ProcessorOutput { content: "..." (500+ 字) }
    预期输出: 小模型审核后调大模型
    验证点: 审计记录包含两次审核

  场景2: 涉及 3 人以上的报告
    输入: ProcessorOutput { people: ["张三", "李四", "王五"], category: Report }
    预期输出: 小模型审核后调大模型
    验证点: 审计记录包含两次审核

  场景3: 简短任务
    输入: ProcessorOutput { category: Task, content: "修复bug" }
    预期输出: 仅规则层
    验证点: ReviewResult.reviewer = "rule"

**边界条件**:
  - 大模型审核失败 -> 使用小模型结果
  - Token 预算不足 -> 降级到小模型

**E2E 测试路径**:
  Event -> Classifier -> Extraction -> ReviewAgent(规则层) -> ReviewAgent(小模型层) -> ReviewAgent(大模型层) -> PersistStep
