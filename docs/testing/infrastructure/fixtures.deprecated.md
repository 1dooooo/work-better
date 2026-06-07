---
title: 测试数据
type: structural
domain: testing
created: 2026-06-06
updated: 2026-06-06
status: deprecated
---

# 测试数据管理

## 数据来源

测试数据有三种来源，按优先级使用：

| 来源 | 适用场景 | 优先级 |
|------|---------|--------|
| 工厂生成 | 大部分测试数据 | 最高 |
| Fixture 文件 | API 响应快照、事件流 | 中 |
| 手写 | 边界值、特殊场景 | 最低 |

## 工厂模式（fishery）

### 为什么用工厂

- 类型安全：TypeScript 原生支持
- 可组合：支持关联、序列、临时参数
- 可维护：修改默认值只需改一处
- 可读性：`eventFactory.build()` 比手写 20+ 字段清晰

### 工厂实现模式

```typescript
// test/_foundation/factories/event.factory.ts
import { Factory } from 'fishery';
import { Event, EventType, Source, Confidence } from '../../../src/models/event';
import { faker } from '@faker-js/faker/locale/zh_CN';

export const eventFactory = Factory.define<Event>(({ sequence, params }) => ({
  id: params.id ?? `evt-${sequence}-${faker.string.uuid()}`,
  timestamp: params.timestamp ?? new Date().toISOString(),
  collected_at: params.collected_at ?? new Date().toISOString(),
  source: params.source ?? Source.feishu_message,
  source_confidence: params.source_confidence ?? Confidence.high,
  type: params.type ?? EventType.message,
  content: params.content ?? { text: faker.lorem.sentence() },
  raw_payload: params.raw_payload ?? '{}',
  tags: params.tags ?? [],
  related_ids: params.related_ids ?? [],
  attachments: params.attachments ?? [],
}));

// 预设变体
export const taskUpdateEventFactory = eventFactory.params({
  type: EventType.task_update,
  source: Source.feishu_project,
  content: {
    task_id: 'task-123',
    field: 'status',
    old_value: 'todo',
    new_value: 'in_progress',
  },
});

export const meetingEventFactory = eventFactory.params({
  type: EventType.meeting,
  source: Source.feishu_meeting,
  content: {
    meeting_id: 'meeting-456',
    title: '周会',
    start_time: new Date().toISOString(),
    participants: ['user-1', 'user-2'],
  },
});

export const manualCaptureEventFactory = eventFactory.params({
  type: EventType.manual_capture,
  source: Source.user_capture,
  source_confidence: Confidence.high,
  content: {
    text: '手动记录的内容',
  },
});
```

### 工厂列表

#### eventFactory

生成 `Event` 对象。支持 14 种 Source × 11 种 EventType 的组合。

```typescript
// 默认事件
const event = eventFactory.build();

// 指定类型
const taskEvent = eventFactory.build({
  type: EventType.task_update,
  source: Source.feishu_project,
});

// 批量生成
const events = eventFactory.buildList(10);

// 使用预设变体
const meeting = meetingEventFactory.build();
```

#### workRecordFactory

生成 `WorkRecord` 对象。支持 8 种 Category。

```typescript
const record = workRecordFactory.build({
  category: WorkRecordCategory.task,
  title: '完成需求文档',
});

const meetingRecord = workRecordFactory.build({
  category: WorkRecordCategory.meeting,
});
```

#### taskFactory

生成 `Task` 对象。支持 5 种 Status × 4 种 Priority。

```typescript
const task = taskFactory.build({
  status: TaskStatus.todo,
  priority: TaskPriority.P0,
});

const doneTask = taskFactory.build({
  status: TaskStatus.done,
  completed_at: new Date().toISOString(),
});
```

#### auditFactory

生成 `ProcessingAudit` 对象。支持 6 种 AuditStep。

```typescript
const audit = auditFactory.build({
  step: AuditStep.classifier,
  confidence: 0.95,
});

const auditChain = auditFactory.buildList(4, [
  { step: AuditStep.classifier },
  { step: AuditStep.extract },
  { step: AuditStep.upgrade },
  { step: AuditStep.review },
]);
```

### 关联生成

工厂之间可以关联，生成完整的数据链路：

```typescript
// test/_foundation/factories/relations.ts
import { eventFactory } from './event.factory';
import { workRecordFactory } from './work-record.factory';
import { auditFactory } from './audit.factory';

export function eventWithWorkRecord() {
  const event = eventFactory.build();
  const workRecord = workRecordFactory.build({
    source_event_ids: [event.id],
  });
  const audits = auditFactory.buildList(4, {
    event_id: event.id,
    record_id: workRecord.id,
  });

  return { event, workRecord, audits };
}

export function eventChain(count: number) {
  const events = eventFactory.buildList(count);
  const records = events.map(event =>
    workRecordFactory.build({ source_event_ids: [event.id] })
  );

  return { events, records };
}
```

## Fixture 文件

### 适用场景

- 飞书 API 响应快照（结构复杂，手写容易遗漏字段）
- 事件流（JSONL 格式，模拟真实采集数据）
- AI 模型输出快照（用于回归测试）

### 目录结构

```
test/fixtures/
├── api-responses/              # 飞书 API 响应快照
│   ├── feishu-messages-success.json
│   ├── feishu-messages-empty.json
│   ├── feishu-messages-paginated.json
│   ├── feishu-calendar-success.json
│   ├── feishu-tasks-success.json
│   └── feishu-auth-error.json
│
├── events/                     # 事件流
│   ├── single-message.jsonl
│   ├── mixed-events.jsonl      # 多种类型混合
│   ├── task-lifecycle.jsonl    # 任务状态变更链
│   └── meeting-with-todos.jsonl
│
├── models/                     # AI 模型输出快照
│   ├── classifier-results.json
│   ├── extraction-results.json
│   └── summary-results.json
│
└── scenarios/                  # 完整业务场景
    ├── daily-report.json       # 日报生成场景的数据
    ├── task-discovery.json     # 任务发现场景的数据
    └── sla-violation.json      # SLA 违规场景的数据
```

### Fixture 文件格式

#### JSON 文件（API 响应）

```json
{
  "_description": "飞书消息 API 成功响应",
  "_source": "录制于 2024-01-15",
  "code": 0,
  "msg": "success",
  "data": {
    "items": [
      {
        "message_id": "om_test_001",
        "msg_type": "text",
        "content": "{\"text\":\"测试消息内容\"}",
        "sender": {
          "sender_type": "user",
          "sender_id": { "open_id": "ou_test_user" }
        },
        "create_time": "1705334400"
      }
    ],
    "has_more": false
  }
}
```

#### JSONL 文件（事件流）

每行一个完整的 Event JSON 对象：

```jsonl
{"id":"evt-001","timestamp":"2024-01-15T10:00:00Z","source":"feishu_message","type":"message","content":{"text":"消息1"}}
{"id":"evt-002","timestamp":"2024-01-15T10:05:00Z","source":"feishu_project","type":"task_update","content":{"task_id":"t-1","field":"status","new_value":"in_progress"}}
{"id":"evt-003","timestamp":"2024-01-15T10:10:00Z","source":"user_capture","type":"manual_capture","content":{"text":"手动记录"}}
```

### Fixture 加载工具

```typescript
// test/_foundation/helpers/load-fixture.ts
import { readFile } from 'fs/promises';
import { resolve } from 'path';

const FIXTURES_DIR = resolve(__dirname, '../../fixtures');

export async function loadFixture<T>(relativePath: string): Promise<T> {
  const fullPath = resolve(FIXTURES_DIR, relativePath);
  const content = await readFile(fullPath, 'utf-8');
  return JSON.parse(content) as T;
}

export async function loadFixtureLines<T>(relativePath: string): Promise<T[]> {
  const fullPath = resolve(FIXTURES_DIR, relativePath);
  const content = await readFile(fullPath, 'utf-8');
  return content
    .split('\n')
    .filter(line => line.trim() !== '')
    .map(line => JSON.parse(line) as T);
}
```

### 使用示例

```typescript
import { loadFixture, loadFixtureLines } from '../_foundation/helpers/load-fixture';
import { Event } from '../../src/models/event';

describe('FeishuCollector', () => {
  it('解析飞书消息响应', async () => {
    const response = await loadFixture<FeishuResponse>('api-responses/feishu-messages-success.json');

    const events = collector.parseResponse(response);

    expect(events).toHaveLength(1);
    expect(events[0]).toBeValidEvent();
  });

  it('处理事件流', async () => {
    const events = await loadFixtureLines<Event>('events/mixed-events.jsonl');

    expect(events).toHaveLength(3);
    expect(events[0].type).toBe(EventType.message);
    expect(events[1].type).toBe(EventType.task_update);
  });
});
```

## Fixture 录制

### 录制飞书 API 响应

开发阶段可以录制真实 API 响应作为 fixture：

```typescript
// scripts/record-feishu-fixtures.ts
import { writeFile } from 'fs/promises';

async function recordMessages() {
  const response = await fetch('https://open.feishu.cn/open-apis/im/v1/messages', {
    headers: { Authorization: `Bearer ${process.env.FEISHU_TOKEN}` },
  });
  const data = await response.json();

  await writeFile(
    'test/fixtures/api-responses/feishu-messages-success.json',
    JSON.stringify(data, null, 2)
  );
}
```

### 录制 AI 模型输出

```typescript
// scripts/record-model-fixtures.ts
async function recordClassifierOutput() {
  const events = await loadFixtureLines<Event>('events/mixed-events.jsonl');
  const results = [];

  for (const event of events) {
    const result = await classifier.classify(event);
    results.push({ input: event, output: result });
  }

  await writeFile(
    'test/fixtures/models/classifier-results.json',
    JSON.stringify(results, null, 2)
  );
}
```

## 数据清理

### 测试间隔离

- 工厂每次调用生成新对象，不共享状态
- Fixture 文件只读，不修改
- 临时文件系统在 `afterEach` 中清理

### 大规模数据

对于需要大量数据的性能测试，使用工厂批量生成：

```typescript
// 生成 1000 个事件用于性能测试
const events = eventFactory.buildList(1000);

// 生成特定分布的数据
const mixedEvents = [
  ...eventFactory.buildList(500, { type: EventType.message }),
  ...eventFactory.buildList(300, { type: EventType.task_update }),
  ...eventFactory.buildList(200, { type: EventType.meeting }),
];
```
