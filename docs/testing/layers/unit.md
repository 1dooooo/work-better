---
title: 单元测试指南
type: guide
domain: testing
created: 2026-06-06
updated: 2026-06-06
status: active
---

# 单元测试指南

## 定义

单元测试验证单个函数或类的逻辑正确性，不依赖外部服务、文件系统或网络。

## 文件命名

- 后缀：`*.test.ts`
- 位置：`test/<module>/` 目录下
- 示例：`test/processing/classifier.test.ts`

## 测试范围

### 采集层

| 被测模块 | 测试重点 |
|---------|---------|
| `CollectorManager` | 注册/注销、启用/禁用、父子开关联动、健康检查 |
| `EventLog` | 追加、查询、标记已处理、获取未处理、重放 |
| `FeishuCollector`（解析逻辑） | 消息解析、事件构建、字段映射 |
| `SystemCollector`（解析逻辑） | 应用切换阈值（30秒）、浏览器历史过滤 |

**注意：** Collector 的 HTTP 请求部分不在此测试，由集成测试 + MSW 覆盖。

### 处理层

| 被测模块 | 测试重点 |
|---------|---------|
| `Classifier` | 路由决策表（14 Source × 11 EventType → 4 路径） |
| `ModelRouter` | 升级阈值判定、Token 预算检查 |
| `ReviewAgent` | 三层审查策略、verdict 判定 |
| `TokenBudget` | 预算分配、超载策略（queue_tomorrow/degrade_to_small/allow_overflow） |
| `SLAManager` | 优先级超时判定（P0:5min/P1:30min/P2:4hr/P3:daily） |

### 存储层

| 被测模块 | 测试重点 |
|---------|---------|
| `ObsidianWriter`（模板渲染） | Markdown 模板生成、双向链接创建 |
| `VectorDBSync` | 嵌入触发、延迟重嵌入（5分钟）、删除清理 |
| `SQLiteQueries` | 查询条件、分页、聚合 |
| `FreshnessEngine` | 六种衰减检测、策略匹配 |

### 呈现层

| 被测模块 | 测试重点 |
|---------|---------|
| 通知分类 | needs-confirmation/light-reminder/click-to-open 判定 |
| 快记窗口 | 输入验证、项目/标签选择 |

### 调度器

| 被测模块 | 测试重点 |
|---------|---------|
| `CronParser` | cron 表达式解析 |
| `DependencyGraph` | 依赖链排序、循环检测 |
| `RetryStrategy` | 重试次数、间隔递增 |
| `TaskRunner` | 超时控制、并发控制 |

## 编写模式

### 决策表测试（Classifier）

```typescript
describe('Classifier 路由规则', () => {
  const classifier = new Classifier();

  it.each([
    [Source.feishu_project, EventType.task_update, 'immediate'],
    [Source.feishu_approval, EventType.approval, 'immediate'],
    [Source.feishu_message, EventType.message, 'aggregation'],
    [Source.browser, EventType.browsing, 'aggregation'],
    [Source.user_capture, EventType.manual_capture, 'immediate'],
    [Source.feishu_message, EventType.low_confidence, 'archive'],
  ])('source=%s, type=%s → %s', (source, type, expectedRoute) => {
    const event = eventFactory.build({ source, type });
    expect(classifier.classify(event)).toBe(expectedRoute);
  });
});
```

### 状态机测试（Task）

```typescript
describe('Task 状态机', () => {
  it.each([
    [TaskStatus.todo, TaskStatus.in_progress, true],
    [TaskStatus.in_progress, TaskStatus.done, true],
    [TaskStatus.in_progress, TaskStatus.blocked, true],
    [TaskStatus.blocked, TaskStatus.in_progress, true],
    [TaskStatus.todo, TaskStatus.cancelled, true],
    [TaskStatus.in_progress, TaskStatus.cancelled, true],
    [TaskStatus.blocked, TaskStatus.cancelled, true],
    // 非法转换
    [TaskStatus.todo, TaskStatus.done, false],
    [TaskStatus.done, TaskStatus.in_progress, false],
    [TaskStatus.blocked, TaskStatus.done, false],
    [TaskStatus.done, TaskStatus.cancelled, false],
    [TaskStatus.cancelled, TaskStatus.todo, false],
  ])('%s → %s 合法=%s', (from, to, expected) => {
    const task = taskFactory.build({ status: from });
    expect(isValidTransition(task.status, to)).toBe(expected);
  });
});
```

### 阈值测试（ModelRouter）

```typescript
describe('ModelRouter 升级阈值', () => {
  const router = new ModelRouter({
    thresholds: {
      entity_extraction: 0.7,
      task_identification: 0.6,
      summary_generation: 0.6,
    },
  });

  it.each([
    ['entity_extraction', 0.6, 'upgrade'],
    ['entity_extraction', 0.8, 'use_small'],
    ['task_identification', 0.5, 'upgrade'],
    ['task_identification', 0.7, 'use_small'],
  ])('task=%s, confidence=%f → %s', (taskType, confidence, expected) => {
    const result = router.decide({ taskType, confidence });
    expect(result.action).toBe(expected);
  });
});
```

### TokenBudget 测试

```typescript
describe('TokenBudget 预算分配', () => {
  it('P0 任务允许超支', () => {
    const budget = new TokenBudget({ dailyLimit: 100000, dailyUsed: 95000 });
    const decision = budget.allocate({ priority: 'P0', estimatedTokens: 10000 });
    expect(decision.action).toBe('allow_overflow');
  });

  it('P2 任务排队到明天', () => {
    const budget = new TokenBudget({ dailyLimit: 100000, dailyUsed: 95000 });
    const decision = budget.allocate({ priority: 'P2', estimatedTokens: 10000 });
    expect(decision.action).toBe('queue_tomorrow');
  });

  it('P1 任务降级到小模型', () => {
    const budget = new TokenBudget({ dailyLimit: 100000, dailyUsed: 95000 });
    const decision = budget.allocate({ priority: 'P1', estimatedTokens: 10000 });
    expect(decision.action).toBe('degrade_to_small');
  });
});
```

### 调度器测试

```typescript
describe('RetryStrategy 重试策略', () => {
  it('最多重试 3 次', () => {
    const strategy = new RetryStrategy({ maxRetries: 3 });
    expect(strategy.shouldRetry(0)).toBe(true);
    expect(strategy.shouldRetry(1)).toBe(true);
    expect(strategy.shouldRetry(2)).toBe(true);
    expect(strategy.shouldRetry(3)).toBe(false);
  });

  it('重试间隔递增', () => {
    const strategy = new RetryStrategy({ maxRetries: 3 });
    expect(strategy.getDelay(0)).toBe(1000);   // 1s
    expect(strategy.getDelay(1)).toBe(5000);   // 5s
    expect(strategy.getDelay(2)).toBe(30000);  // 30s
  });
});
```

## 关键路径覆盖率

以下模块必须达到 100% 分支覆盖率：

- Classifier 路由决策
- Task 状态机转换判定
- SLA 超时判定
- ModelRouter 升级阈值
- ReviewAgent verdict 判定
