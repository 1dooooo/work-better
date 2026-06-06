# 集成测试指南

## 定义

集成测试验证模块之间的协作和数据流转，关注层间接口的正确性。

## 文件命名

- 后缀：`*.int.test.ts`
- 位置：`test/<module>/` 目录下
- 示例：`test/processing/event-pipeline.int.test.ts`

## 配置

使用独立的 Vitest 配置：

```typescript
// vitest.config.int.ts
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    include: ['**/*.int.test.ts'],
    testTimeout: 60_000,
    setupFiles: ['./test/setup.ts'],
  },
});
```

## 测试范围

### 采集层 → EventLog

验证 Collector 正确产出 Event 并写入 EventLog。

```typescript
describe('采集层集成', () => {
  it('飞书消息采集器写入正确格式的 Event', async () => {
    const eventLog = new FakeEventLog();
    const collector = new FeishuCollector(config);

    await collector.collect(eventLog);

    const events = eventLog.getAll();
    expect(events).toHaveLength(1);
    expect(events[0]).toBeValidEvent();
    expect(events[0].source).toBe(Source.feishu_message);
  });

  it('多个采集器的 Event 按时间顺序写入', async () => {
    const eventLog = new FakeEventLog();
    const messageCollector = new FeishuMessageCollector(config);
    const calendarCollector = new FeishuCalendarCollector(config);

    await messageCollector.collect(eventLog);
    await calendarCollector.collect(eventLog);

    const events = eventLog.getAll();
    const timestamps = events.map(e => new Date(e.timestamp).getTime());
    expect(timestamps).toEqual([...timestamps].sort());
  });
});
```

### EventLog → 处理层

验证处理层从 EventLog 读取 Event 并产出正确的 WorkRecord。

```typescript
describe('处理层集成', () => {
  it('task_update 事件经处理产出 Task 类型 WorkRecord', async () => {
    const eventLog = new FakeEventLog();
    eventLog.append(taskUpdateEventFactory.build());

    const processor = new EventProcessor({
      modelRouter: new FakeModelRouter(),
      reviewAgent: new FakeReviewAgent(),
    });

    const records = await processor.process(eventLog.getUnprocessed());

    expect(records).toHaveLength(1);
    expect(records[0].category).toBe(WorkRecordCategory.task);
    expect(records[0].source_event_ids).toContain(expect.any(String));
  });

  it('低置信度事件触发模型升级', async () => {
    const eventLog = new FakeEventLog();
    eventLog.append(eventFactory.build({ type: EventType.message }));

    const modelRouter = new FakeModelRouter();
    modelRouter.configure('summary_generation', { confidence: 0.3 });

    const processor = new EventProcessor({ modelRouter });
    await processor.process(eventLog.getUnprocessed());

    // 验证升级发生
    expect(modelRouter.getCallCount()).toBe(2); // 小模型 + 大模型
  });

  it('审查不通过的记录标记 needs_fix', async () => {
    const reviewAgent = new FakeReviewAgent();
    reviewAgent.configure(ReviewVerdict.needs_fix, [
      { type: 'missing_field', severity: 'high', description: '缺少 title' },
    ]);

    const processor = new EventProcessor({ reviewAgent });
    const records = await processor.process(eventLog.getUnprocessed());

    expect(records[0].needs_review).toBe(true);
  });
});
```

### 处理层 → 存储层

验证 WorkRecord 正确写入三层存储。

```typescript
describe('存储层集成', () => {
  let tempDir: string;
  let testVault: TestVault;

  beforeEach(async () => {
    testVault = new TestVault();
    await testVault.setup();
    tempDir = testVault.getPath();
  });

  afterEach(async () => {
    await testVault.teardown();
  });

  it('WorkRecord 写入 Obsidian 后触发 VectorDB 嵌入', async () => {
    const vectorDb = new FakeVectorDB();
    const writer = new ObsidianWriter({ vaultPath: tempDir, vectorDb });

    const record = workRecordFactory.build();
    await writer.write(record);

    // 验证文件创建
    const files = await testVault.listFiles();
    expect(files.length).toBeGreaterThan(0);

    // 验证 VectorDB 嵌入
    expect(await vectorDb.count()).toBe(1);
  });

  it('三层一致性检查发现不一致', async () => {
    const checker = new ConsistencyChecker({
      vaultPath: tempDir,
      vectorDb: new FakeVectorDB(),
      sqlite: new TestDatabase().getDb(),
    });

    // 人为制造不一致：Obsidian 有文件但 SQLite 无记录
    await testVault.writeFile('test.md', '# Test');

    const report = await checker.check();

    expect(report.inconsistencies).toHaveLength(1);
    expect(report.inconsistencies[0].type).toBe('obsidian_without_sqlite');
  });
});
```

### 调度器依赖链

验证定时任务按依赖顺序执行。

```typescript
describe('调度器依赖链', () => {
  it('P-02 在 P-01 完成后执行', async () => {
    const executionOrder: string[] = [];

    const scheduler = new TaskScheduler({
      tasks: [
        createTask('P-01', { execute: () => { executionOrder.push('P-01'); } }),
        createTask('P-02', { dependencies: ['P-01'], execute: () => { executionOrder.push('P-02'); } }),
      ],
    });

    await scheduler.runAll();

    expect(executionOrder).toEqual(['P-01', 'P-02']);
  });

  it('失败任务触发重试', async () => {
    let attempts = 0;
    const task = createTask('P-01', {
      execute: () => {
        attempts++;
        if (attempts < 3) throw new Error('临时失败');
      },
    });

    const scheduler = new TaskScheduler({ tasks: [task] });
    await scheduler.runAll();

    expect(attempts).toBe(3);
    expect(task.result.status).toBe('success');
  });

  it('全局暂停后任务不执行', async () => {
    let executed = false;
    const scheduler = new TaskScheduler({
      tasks: [createTask('P-01', { execute: () => { executed = true; } })],
    });

    scheduler.pause();
    await scheduler.runAll();

    expect(executed).toBe(false);
  });
});
```

### Audit 链完整性

验证处理全流程产出完整的审计链。

```typescript
describe('Audit 链完整性', () => {
  it('完整处理流程产出 4 步审计记录', async () => {
    const eventLog = new FakeEventLog();
    eventLog.append(eventFactory.build());

    const processor = new EventProcessor({
      modelRouter: new FakeModelRouter(),
      reviewAgent: new FakeReviewAgent(),
    });

    await processor.process(eventLog.getUnprocessed());

    const audits = processor.getAudits();
    expect(audits).toHaveLength(4);
    expect(audits.map(a => a.step)).toEqual([
      AuditStep.classifier,
      AuditStep.extract,
      AuditStep.upgrade,
      AuditStep.review,
    ]);

    // 所有审计记录共享 trace_id
    const traceIds = new Set(audits.map(a => a.trace_id));
    expect(traceIds.size).toBe(1);

    // 时间戳递增
    const timestamps = audits.map(a => new Date(a.timestamp).getTime());
    expect(timestamps).toEqual([...timestamps].sort());
  });
});
```

## 测试隔离

### 临时文件系统

涉及 Obsidian Vault 的测试使用临时目录：

```typescript
let testVault: TestVault;

beforeEach(async () => {
  testVault = new TestVault();
  await testVault.setup();
});

afterEach(async () => {
  await testVault.teardown();
});
```

### FakeEventLog

每个测试创建独立的 FakeEventLog 实例：

```typescript
it('测试场景', () => {
  const eventLog = new FakeEventLog();
  // 不共享，不依赖其他测试
});
```

### MSW Handlers

集成测试可以使用 MSW 切换不同的 API 响应场景：

```typescript
import { server } from '../setup';
import { feishu } from '../_foundation/handlers/feishu';

it('处理分页响应', async () => {
  server.use(feishu.messages.paginated);
  // ...
});
```
