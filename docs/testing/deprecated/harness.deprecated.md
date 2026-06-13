---
title: 测试夹具
type: structural
domain: testing
created: 2026-06-06
updated: 2026-06-06
status: deprecated
---

# 测试 Harness 设计

## 什么是 Harness

测试 Harness 是测试的测试框架——它提供 Fake 实现、数据工厂、自定义断言、MSW handlers 等基础设施，让各功能域的测试可以专注于业务逻辑验证，而非重复搭建测试环境。

类比关系：
- `src/` 之于业务代码 = `_foundation/` 之于测试代码
- 应用框架提供路由、中间件、ORM = Harness 提供 Fake、Factory、Matcher、Handler

## 架构

```
test/_foundation/
├── fakes/              # Fake 实现（替代真实依赖）
├── factories/          # 数据工厂（生成测试数据）
├── matchers/           # 自定义断言（语义化验证）
├── handlers/           # MSW handlers（外部 API Mock）
└── helpers/            # 工具函数（通用辅助）
```

## Fake 实现

参考 LangChain.js 的 `FakeLLM`/`FakeChatModel` 模式，为系统核心依赖提供可配置的内存实现。

### FakeEventLog

EventLog 的内存实现，用于解耦采集层和处理层测试。

```typescript
// test/_foundation/fakes/fake-event-log.ts
export class FakeEventLog {
  private events: Event[] = [];

  append(event: Event): void;
  get(eventId: string): Event | null;
  query(filter: EventFilter): Event[];
  getUnprocessed(): Event[];
  markProcessed(eventId: string): void;
  replay(fromTimestamp: string): Event[];

  // 测试辅助
  getAll(): Event[];
  clear(): void;
  count(): number;
}
```

**使用场景：**
- 采集层测试：验证 Collector 输出的 Event 格式正确
- 处理层测试：注入预设 Event，验证处理逻辑
- 集成测试：验证 Event 从采集到处理的完整流转

### FakeModelRouter

AI 模型路由的可配置 Fake，用于控制置信度和升级路径。

```typescript
// test/_foundation/fakes/fake-model-router.ts
export class FakeModelRouter {
  // 配置每个任务类型的返回结果
  configure(taskType: string, response: ModelResponse): void;

  // 配置升级行为（当置信度低于阈值时）
  configureUpgrade(taskType: string, upgradedResponse: ModelResponse): void;

  // 注入错误（测试错误处理）
  throwError(error: Error): void;

  // 执行分类/提取
  classify(event: Event): Promise<ClassifierResult>;
  extract(event: Event): Promise<ExtractionResult>;

  // 记录调用历史（用于断言）
  getCallHistory(): CallRecord[];
  getCallCount(): number;
}
```

**使用场景：**
- 测试 Classifier 路由逻辑（不依赖真实模型）
- 测试 ModelRouter 升级阈值判定
- 测试 TokenBudget 预算超限时的降级策略

### FakeReviewAgent

审查代理的可配置 Fake。

```typescript
// test/_foundation/fakes/fake-review-agent.ts
export class FakeReviewAgent {
  // 配置审查结果
  configure(verdict: ReviewVerdict, issues?: Issue[]): void;

  // 按 record 类型配置不同结果
  configureFor(category: WorkRecordCategory, verdict: ReviewVerdict): void;

  // 执行审查
  review(record: WorkRecord): Promise<ReviewResult>;

  // 记录调用历史
  getCallHistory(): ReviewCallRecord[];
}
```

### FakeTokenBudget

Token 预算的可控 Fake。

```typescript
// test/_foundation/fakes/fake-token-budget.ts
export class FakeTokenBudget {
  // 初始化预算
  constructor(options: { dailyLimit: number; dailyUsed: number });

  // 模拟消耗
  consume(tokens: number): void;

  // 查询状态
  getRemaining(): number;
  getUsed(): number;
  isOverloaded(): boolean;

  // 测试辅助：直接设置已用量
  setUsed(amount: number): void;
}
```

## 数据工厂

使用 fishery 库，提供类型安全的测试数据生成。

### 设计原则

- 每个核心数据类型一个工厂
- 工厂提供合理的默认值（可直接 `.build()` 使用）
- 支持覆盖任意字段
- 支持关联（如 Event → WorkRecord）

### 工厂列表

| 工厂 | 生成对象 | 说明 |
|------|---------|------|
| `eventFactory` | `Event` | 14 种 Source × 11 种 EventType 组合 |
| `workRecordFactory` | `WorkRecord` | 8 种 Category |
| `taskFactory` | `Task` | 5 种 Status × 4 种 Priority |
| `auditFactory` | `ProcessingAudit` | 6 种 AuditStep |

### 使用模式

```typescript
// 最简用法：全部默认值
const event = eventFactory.build();

// 覆盖关注字段
const taskEvent = eventFactory.build({
  type: EventType.task_update,
  source: Source.feishu_project,
});

// 批量生成
const events = eventFactory.buildList(10);

// 关联生成
const { event, workRecord } = eventWithWorkRecord();
```

## 自定义断言

扩展 Vitest 的 `expect`，提供领域语义化断言。

### 断言列表

| 断言 | 验证内容 |
|------|---------|
| `toBeValidEvent()` | 对象符合 Event 结构 |
| `toBeValidWorkRecord()` | 对象符合 WorkRecord 结构 |
| `toBeValidTask()` | 对象符合 Task 结构 |
| `toHaveConfidenceAbove(threshold)` | 置信度高于阈值 |
| `toBeValidTransition(targetStatus)` | Task 状态转换合法 |
| `toHaveAuditStep(step)` | 审计链包含指定步骤 |
| `toHaveTraceId()` | 审计记录有有效的 trace_id |

### 实现模式

```typescript
// test/_foundation/matchers/index.ts
import { expect } from 'vitest';

expect.extend({
  toBeValidEvent(received: unknown) {
    const pass =
      received !== null &&
      typeof received === 'object' &&
      'id' in received &&
      'timestamp' in received &&
      'source' in received &&
      'type' in received;

    return {
      pass,
      message: () => `expected ${JSON.stringify(received)} to be a valid Event`,
    };
  },

  toHaveConfidenceAbove(received: { confidence: number }, threshold: number) {
    return {
      pass: received.confidence >= threshold,
      message: () =>
        `expected confidence ${received.confidence} to be >= ${threshold}`,
    };
  },
});
```

### 类型声明

```typescript
// test/_foundation/matchers/types.d.ts
interface CustomMatchers<R = unknown> {
  toBeValidEvent(): R;
  toBeValidWorkRecord(): R;
  toBeValidTask(): R;
  toHaveConfidenceAbove(threshold: number): R;
  toBeValidTransition(targetStatus: string): R;
  toHaveAuditStep(step: string): R;
  toHaveTraceId(): R;
}

declare module 'vitest' {
  interface Assertion<T = any> extends CustomMatchers<T> {}
  interface AsymmetricMatchersContaining extends CustomMatchers {}
}
```

## MSW Handlers

外部 API 的契约测试基础设施。详见 [mocking.md](mocking.md)。

## 工具函数

### run-pipeline

封装处理管道的测试辅助函数，简化集成测试编写：

```typescript
// test/_foundation/helpers/run-pipeline.ts
export async function runProcessingPipeline(
  events: Event[],
  options?: {
    modelRouter?: FakeModelRouter;
    reviewAgent?: FakeReviewAgent;
    tokenBudget?: FakeTokenBudget;
  }
): Promise<PipelineResult> {
  const eventLog = new FakeEventLog();
  events.forEach(e => eventLog.append(e));

  return startProcessing(eventLog, {
    modelRouter: options?.modelRouter ?? new FakeModelRouter(),
    reviewAgent: options?.reviewAgent ?? new FakeReviewAgent(),
    tokenBudget: options?.tokenBudget ?? new FakeTokenBudget({ dailyLimit: 100000, dailyUsed: 0 }),
  });
}
```

### load-fixture

加载 fixture 文件的工具函数：

```typescript
// test/_foundation/helpers/load-fixture.ts
export function loadFixture<T>(path: string): T;
export function loadFixtureLines<T>(path: string): T[];  // JSONL
```

## 依赖关系

```
fakes/        ← 不依赖其他 _foundation 模块
factories/    ← 依赖 models/ 类型定义
matchers/    ← 不依赖其他 _foundation 模块
handlers/    ← 不依赖其他 _foundation 模块
helpers/     ← 依赖 fakes/
```

原则：`_foundation/` 内部模块之间保持最小依赖，避免循环引用。
