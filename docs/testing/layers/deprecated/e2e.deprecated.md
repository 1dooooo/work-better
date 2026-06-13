---
title: E2E 测试指南
type: guide
domain: testing
created: 2026-06-06
updated: 2026-06-06
status: deprecated
---

# E2E 测试指南

## 定义

E2E 测试验证完整的用户场景，从信息输入到最终产出，模拟真实使用流程。

## 文件命名

- 后缀：`*.e2e.test.ts`
- 位置：`test/e2e/` 目录下
- 示例：`test/e2e/full-pipeline.e2e.test.ts`

## 配置

使用独立的 Vitest 配置：

```typescript
// vitest.config.e2e.ts
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    include: ['**/*.e2e.test.ts'],
    testTimeout: 120_000,
    setupFiles: ['./test/setup.ts'],
  },
});
```

## 测试场景

### 场景 1：完整处理管道

飞书消息 → Event → WorkRecord → Obsidian 文档

```typescript
describe('完整处理管道', () => {
  let testVault: TestVault;

  beforeEach(async () => {
    testVault = new TestVault();
    await testVault.setup();
  });

  afterEach(async () => {
    await testVault.teardown();
  });

  it('飞书消息最终写入 Obsidian', async () => {
    // 1. 模拟飞书消息到达
    server.use(feishu.messages.success);

    // 2. 采集层处理
    const collector = new FeishuCollector(config);
    const eventLog = new FakeEventLog();
    await collector.collect(eventLog);

    // 3. 处理层处理
    const processor = new EventProcessor({
      modelRouter: new FakeModelRouter(),
      reviewAgent: new FakeReviewAgent(),
      vaultPath: testVault.getPath(),
    });
    const records = await processor.process(eventLog.getUnprocessed());

    // 4. 验证 Obsidian 文件
    const files = await testVault.listFiles();
    expect(files.length).toBeGreaterThan(0);

    const content = await testVault.readFile(files[0]);
    expect(content).toContain(records[0].title);
  });
});
```

### 场景 2：快记窗口流程

用户输入 → EventLog → 自动分类 → 归档

```typescript
describe('快记窗口流程', () => {
  it('文本输入自动归档到对应项目', async () => {
    const eventLog = new FakeEventLog();

    // 1. 模拟用户输入
    const capture = new UserCaptureCollector();
    await capture.captureText('完成需求文档 #项目A', eventLog);

    // 2. 处理层分类
    const processor = new EventProcessor({
      modelRouter: new FakeModelRouter(),
    });
    const records = await processor.process(eventLog.getUnprocessed());

    // 3. 验证分类结果
    expect(records[0].project).toBe('项目A');
    expect(records[0].category).toBe(WorkRecordCategory.document);
  });

  it('图片输入触发大模型处理', async () => {
    const eventLog = new FakeEventLog();
    const capture = new UserCaptureCollector();

    // 1. 模拟图片粘贴
    await capture.captureImage(Buffer.from('fake-image'), eventLog);

    // 2. 验证 Event 类型
    const events = eventLog.getAll();
    expect(events[0].type).toBe(EventType.image);

    // 3. 处理层应该使用大模型
    const modelRouter = new FakeModelRouter();
    const processor = new EventProcessor({ modelRouter });
    await processor.process(eventLog.getUnprocessed());

    // 图片只能用大模型处理
    expect(modelRouter.getCallHistory()[0].model).toBe('large');
  });
});
```

### 场景 3：任务生命周期

任务创建 → 状态变更 → 完成 → 同步

```typescript
describe('任务生命周期', () => {
  it('任务状态变更全链路', async () => {
    const eventLog = new FakeEventLog();

    // 1. 任务创建事件
    eventLog.append(eventFactory.build({
      type: EventType.task_update,
      content: { task_id: 'task-1', field: 'status', new_value: 'todo' },
    }));

    // 2. 任务开始事件
    eventLog.append(eventFactory.build({
      type: EventType.task_update,
      content: { task_id: 'task-1', field: 'status', old_value: 'todo', new_value: 'in_progress' },
    }));

    // 3. 任务完成事件
    eventLog.append(eventFactory.build({
      type: EventType.task_update,
      content: { task_id: 'task-1', field: 'status', old_value: 'in_progress', new_value: 'done' },
    }));

    // 4. 处理所有事件
    const processor = new EventProcessor({
      modelRouter: new FakeModelRouter(),
    });
    const records = await processor.process(eventLog.getUnprocessed());

    // 5. 验证最终状态
    const task = records.find(r => r.category === WorkRecordCategory.task);
    expect(task.status).toBe(TaskStatus.done);
  });
});
```

### 场景 4：报告生成

定时触发 → 数据聚合 → 报告生成

```typescript
describe('报告生成', () => {
  let testVault: TestVault;

  beforeEach(async () => {
    testVault = new TestVault();
    await testVault.setup();
  });

  afterEach(async () => {
    await testVault.teardown();
  });

  it('日报生成包含今日所有工作记录', async () => {
    // 1. 准备今日数据
    const eventLog = new FakeEventLog();
    eventLog.append(eventFactory.build({
      timestamp: new Date().toISOString(),
      type: EventType.task_update,
    }));
    eventLog.append(eventFactory.build({
      timestamp: new Date().toISOString(),
      type: EventType.meeting,
    }));

    // 2. 处理事件
    const processor = new EventProcessor({
      modelRouter: new FakeModelRouter(),
    });
    await processor.process(eventLog.getUnprocessed());

    // 3. 生成日报
    const reporter = new DailyReporter({
      vaultPath: testVault.getPath(),
    });
    await reporter.generate();

    // 4. 验证报告
    const files = await testVault.listFiles();
    const reportFile = files.find(f => f.includes('日报'));
    expect(reportFile).toBeDefined();

    const content = await testVault.readFile(reportFile!);
    expect(content).toContain('任务');
    expect(content).toContain('会议');
  });
});
```

### 场景 5：SLA 违规处理

超时检测 → 自动升级 → 通知

```typescript
describe('SLA 违规处理', () => {
  it('P0 任务超时 5 分钟后自动升级', async () => {
    const eventLog = new FakeEventLog();
    eventLog.append(eventFactory.build({
      type: EventType.task_update,
      content: { priority: 'P0' },
      timestamp: new Date(Date.now() - 6 * 60 * 1000).toISOString(), // 6 分钟前
    }));

    const slaManager = new SLAManager();
    const violations = await slaManager.scan(eventLog);

    expect(violations).toHaveLength(1);
    expect(violations[0].priority).toBe('P0');
    expect(violations[0].overdueMinutes).toBeGreaterThan(5);
  });
});
```

## 测试数据管理

### 使用 Fixture 文件

E2E 测试可以使用预录制的 fixture 数据：

```typescript
import { loadFixtureLines } from '../_foundation/helpers/load-fixture';

it('处理真实事件流', async () => {
  const events = await loadFixtureLines<Event>('events/mixed-events.jsonl');

  const eventLog = new FakeEventLog();
  events.forEach(e => eventLog.append(e));

  const processor = new EventProcessor({
    modelRouter: new FakeModelRouter(),
  });
  const records = await processor.process(eventLog.getUnprocessed());

  expect(records.length).toBeGreaterThan(0);
});
```

### 使用工厂批量生成

```typescript
it('处理大量事件', async () => {
  const events = eventFactory.buildList(100);

  const eventLog = new FakeEventLog();
  events.forEach(e => eventLog.append(e));

  const processor = new EventProcessor({
    modelRouter: new FakeModelRouter(),
  });
  const records = await processor.process(eventLog.getUnprocessed());

  expect(records).toHaveLength(100);
});
```

## 注意事项

- E2E 测试不调用真实飞书 API 或 AI 模型
- 文件系统操作使用临时目录
- 每个测试完全独立，不共享状态
- 测试超时设置为 120 秒
