# Mock 系统设计

## Mock 策略总览

系统有两类外部依赖需要 Mock：

| 依赖 | Mock 方式 | 原因 |
|------|---------|------|
| 飞书 API | MSW 网络层拦截 | 契约测试，验证请求格式和响应解析 |
| AI 模型 | Fake 实现 | 确定性输出，可控置信度 |
| Obsidian Vault | 临时文件系统 | 隔离测试环境，不污染真实 Vault |
| VectorDB | Fake 实现 | 内存存储，快速验证同步逻辑 |
| SQLite | 内存数据库 | 每次测试独立，无需清理 |

## 飞书 API Mock（MSW）

### 设计原则

- 按 API 端点组织 handlers
- 每个端点准备 4 种场景：成功、空响应、分页、错误
- 使用 MSW v2 的 `http`/`HttpResponse` API
- 不依赖飞书服务可用性

### Handler 组织

```
test/_foundation/handlers/feishu/
├── messages.ts         # /open-apis/im/v1/messages
├── calendar.ts         # /open-apis/calendar/v4/calendars
├── tasks.ts            # /open-apis/task/v2/tasks
├── docs.ts             # /open-apis/docx/v1/documents
├── meetings.ts         # /open-apis/vc/v1/meetings
└── index.ts            # 组合导出所有 handlers
```

### 每个端点的 Handler 模式

```typescript
// test/_foundation/handlers/feishu/messages.ts
import { http, HttpResponse, delay } from 'msw';

// 成功响应
export const messagesSuccess = http.get(
  '*/open-apis/im/v1/messages',
  async ({ request }) => {
    await delay(50); // 模拟网络延迟
    const url = new URL(request.url);
    const containerId = url.searchParams.get('container_id');

    return HttpResponse.json({
      code: 0,
      msg: 'success',
      data: {
        items: [
          {
            message_id: `om_${crypto.randomUUID()}`,
            msg_type: 'text',
            content: JSON.stringify({ text: '测试消息' }),
            sender: {
              sender_type: 'user',
              sender_id: { open_id: 'ou_test_user' },
            },
            create_time: String(Date.now()),
          },
        ],
        has_more: false,
      },
    });
  }
);

// 空响应
export const messagesEmpty = http.get(
  '*/open-apis/im/v1/messages',
  () => {
    return HttpResponse.json({
      code: 0,
      msg: 'success',
      data: { items: [], has_more: false },
    });
  }
);

// 分页响应
export const messagesPaginated = (() => {
  let callCount = 0;

  return http.get('*/open-apis/im/v1/messages', ({ request }) => {
    callCount++;
    const url = new URL(request.url);
    const pageToken = url.searchParams.get('page_token');

    if (!pageToken) {
      // 第一页
      return HttpResponse.json({
        code: 0,
        data: {
          items: [{ message_id: 'om_page1' }],
          has_more: true,
          page_token: 'page2_token',
        },
      });
    }

    // 第二页
    return HttpResponse.json({
      code: 0,
      data: {
        items: [{ message_id: 'om_page2' }],
        has_more: false,
      },
    });
  });
})();

// 401 错误（token 过期）
export const messagesAuthError = http.get(
  '*/open-apis/im/v1/messages',
  () => {
    return HttpResponse.json(
      { code: 99991400, msg: 'Invalid token', data: null },
      { status: 401 }
    );
  }
);

// 429 错误（频率限制）
export const messagesRateLimit = http.get(
  '*/open-apis/im/v1/messages',
  () => {
    return HttpResponse.json(
      { code: 99991429, msg: 'Rate limit exceeded', data: null },
      { status: 429 }
    );
  }
);

// 500 错误（服务端错误）
export const messagesServerError = http.get(
  '*/open-apis/im/v1/messages',
  () => {
    return HttpResponse.json(
      { code: 99991500, msg: 'Internal server error', data: null },
      { status: 500 }
    );
  }
);
```

### 组合导出

```typescript
// test/_foundation/handlers/feishu/index.ts
import {
  messagesSuccess,
  messagesEmpty,
  messagesPaginated,
  messagesAuthError,
  messagesRateLimit,
  messagesServerError,
} from './messages';

import {
  calendarSuccess,
  calendarEmpty,
  calendarAuthError,
} from './calendar';

// 默认 handlers（成功场景）
export const defaultHandlers = [
  messagesSuccess,
  calendarSuccess,
  // ... 其他端点的默认 handler
];

// 按场景分组
export const errorHandlers = {
  auth: [messagesAuthError, calendarAuthError],
  rateLimit: [messagesRateLimit],
  server: [messagesServerError],
};

// 按端点导出
export const feishu = {
  messages: {
    success: messagesSuccess,
    empty: messagesEmpty,
    paginated: messagesPaginated,
    authError: messagesAuthError,
    rateLimit: messagesRateLimit,
    serverError: messagesServerError,
  },
  calendar: {
    success: calendarSuccess,
    empty: calendarEmpty,
    authError: calendarAuthError,
  },
};
```

### 测试中使用

```typescript
import { setupServer } from 'msw/node';
import { defaultHandlers, feishu } from '../_foundation/handlers/feishu';

const server = setupServer(...defaultHandlers);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

describe('FeishuCollector', () => {
  it('正常获取消息', async () => {
    // 使用默认 handler（成功）
    const collector = new FeishuCollector(config);
    const events = await collector.collect();
    expect(events).toHaveLength(1);
  });

  it('处理 token 过期', async () => {
    // 切换到 401 handler
    server.use(feishu.messages.authError);

    const collector = new FeishuCollector(config);
    const result = await collector.collect();

    expect(result.error).toBe('token_expired');
  });

  it('处理分页响应', async () => {
    server.use(feishu.messages.paginated);

    const collector = new FeishuCollector(config);
    const events = await collector.collect();

    expect(events).toHaveLength(2);
  });
});
```

### 需要 Mock 的飞书 API 端点

| 端点 | 采集器 | Handler 文件 |
|------|--------|-------------|
| `/open-apis/im/v1/messages` | 消息采集器 | `messages.ts` |
| `/open-apis/calendar/v4/calendars/:id/events` | 日历采集器 | `calendar.ts` |
| `/open-apis/task/v2/tasks` | 任务采集器 | `tasks.ts` |
| `/open-apis/docx/v1/documents/:id` | 文档采集器 | `docs.ts` |
| `/open-apis/vc/v1/meetings` | 会议采集器 | `meetings.ts` |
| `/open-apis/mail/v1/messages` | 邮件采集器 | `mail.ts` |
| `/open-apis/approval/v4/instances` | 审批采集器 | `approval.ts` |
| `/open-apis/okr/v1/okrs` | OKR 采集器 | `okr.ts` |
| `/open-apis/bitable/v1/apps/:id/records` | 多维表格采集器 | `bitable.ts` |
| `/open-apis/sheets/v3/spreadsheets/:id` | 电子表格采集器 | `sheets.ts` |
| `/open-apis/wiki/v2/spaces` | 知识库采集器 | `wiki.ts` |

每个端点都需要准备 success、empty、paginated、authError、rateLimit、serverError 六种 handler。

## AI 模型 Mock（Fake 实现）

### 设计原则

- 使用 Fake 实现而非 Mock 函数（参考 LangChain.js）
- 支持按任务类型配置不同返回结果
- 支持注入置信度来控制升级路径
- 记录调用历史用于断言

### Mock 层次

```
FakeModelRouter（路由层）
  ├── FakeClassifier（分类器）
  ├── FakeExtractor（提取器）
  └── FakeSummarizer（摘要器）
```

### 配置模式

```typescript
// 测试分类路由
const modelRouter = new FakeModelRouter();
modelRouter.configure('task_update', {
  confidence: 0.95,
  route: 'immediate',
});

// 测试升级路径
modelRouter.configure('entity_extraction', {
  confidence: 0.3, // 低于阈值
  output: null,    // 小模型无法处理
});
modelRouter.configureUpgrade('entity_extraction', {
  confidence: 0.9,
  output: { entities: ['张三', '项目A'] },
});

// 测试错误处理
modelRouter.throwError(new Error('模型服务不可用'));
```

## Obsidian Vault Mock

### 临时目录策略

```typescript
import { mkdtemp, rm, readdir, readFile, writeFile } from 'fs/promises';
import { tmpdir } from 'os';
import { join } from 'path';

class TestVault {
  private dir: string;

  async setup(): Promise<void> {
    this.dir = await mkdtemp(join(tmpdir(), 'work-better-vault-'));
  }

  async teardown(): Promise<void> {
    await rm(this.dir, { recursive: true, force: true });
  }

  getPath(): string {
    return this.dir;
  }

  async readFile(relativePath: string): Promise<string> {
    return readFile(join(this.dir, relativePath), 'utf-8');
  }

  async writeFile(relativePath: string, content: string): Promise<void> {
    const fullPath = join(this.dir, relativePath);
    await writeFile(fullPath, content, 'utf-8');
  }

  async listFiles(): Promise<string[]> {
    return readdir(this.dir, { recursive: true });
  }
}
```

## VectorDB Mock

### 内存实现

```typescript
class FakeVectorDB {
  private embeddings: Map<string, { vector: number[]; metadata: unknown }> = new Map();

  async embed(id: string, text: string, metadata: unknown): Promise<void> {
    const vector = this.simpleHash(text); // 简单的伪向量
    this.embeddings.set(id, { vector, metadata });
  }

  async search(query: string, limit: number): Promise<SearchResult[]> {
    const queryVector = this.simpleHash(query);
    // 简化的相似度计算
    return Array.from(this.embeddings.entries())
      .slice(0, limit)
      .map(([id, { metadata }]) => ({ id, score: 0.8, metadata }));
  }

  async remove(id: string): Promise<void> {
    this.embeddings.delete(id);
  }

  async count(): Promise<number> {
    return this.embeddings.size;
  }

  private simpleHash(text: string): number[] {
    // 简化的伪向量生成，仅用于测试
    return Array.from(text).map(c => c.charCodeAt(0) / 255);
  }
}
```

## SQLite Mock

### 内存数据库

```typescript
import Database from 'better-sqlite3';

class TestDatabase {
  private db: Database.Database;

  constructor() {
    this.db = new Database(':memory:');
    this.runMigrations();
  }

  private runMigrations(): void {
    this.db.exec(`
      CREATE TABLE IF NOT EXISTS events (
        id TEXT PRIMARY KEY,
        source TEXT NOT NULL,
        type TEXT NOT NULL,
        collected_at TEXT NOT NULL,
        processed INTEGER DEFAULT 0
      );
      CREATE TABLE IF NOT EXISTS work_records (
        id TEXT PRIMARY KEY,
        source_event_ids TEXT,
        category TEXT,
        created_at TEXT NOT NULL
      );
      -- ... 其他表
    `);
  }

  getDb(): Database.Database {
    return this.db;
  }

  close(): void {
    this.db.close();
  }
}
```

## Global Setup

```typescript
// test/setup.ts
import { setupServer } from 'msw/node';
import { defaultHandlers } from './_foundation/handlers/feishu';
import './_foundation/matchers'; // 注册自定义断言

// MSW 服务器
const server = setupServer(...defaultHandlers);

beforeAll(() => server.listen({ onUnhandledRequest: 'warn' }));
afterEach(() => {
  server.resetHandlers();
  vi.restoreAllMocks();
});
afterAll(() => server.close());

// 导出供需要自定义 handler 的测试使用
export { server };
```
