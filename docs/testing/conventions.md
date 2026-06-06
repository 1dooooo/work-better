# 测试规范与约束

## 文件命名规范

### 测试文件

| 类型 | 命名模式 | 示例 |
|------|---------|------|
| 单元测试 | `<module>.test.ts` | `classifier.test.ts` |
| 集成测试 | `<module>.int.test.ts` | `event-pipeline.int.test.ts` |
| E2E 测试 | `<scenario>.e2e.test.ts` | `full-pipeline.e2e.test.ts` |

### 基础设施文件

| 类型 | 命名模式 | 示例 |
|------|---------|------|
| Fake 实现 | `fake-<module>.ts` | `fake-event-log.ts` |
| 数据工厂 | `<entity>.factory.ts` | `event.factory.ts` |
| MSW handlers | 按 API 端点分文件 | `messages.ts` |
| 自定义 matcher | `index.ts`（统一导出） | — |
| 工具函数 | `run-<action>.ts` / `load-<type>.ts` | `run-pipeline.ts` |

## 测试编写规范

### Describe-It 结构

```typescript
describe('Classifier', () => {
  describe('路由规则', () => {
    it('将 task_update 事件路由到 immediate 路径', () => {
      // Arrange
      const event = eventFactory.build({ type: EventType.task_update });

      // Act
      const route = classifier.classify(event);

      // Assert
      expect(route).toBe('immediate');
    });
  });
});
```

### AAA 模式（Arrange-Act-Assert）

每个测试用例遵循 AAA 结构：

- **Arrange**：准备测试数据和环境
- **Act**：执行被测行为
- **Assert**：验证结果

```typescript
it('当置信度低于阈值时触发模型升级', () => {
  // Arrange
  const router = new FakeModelRouter();
  router.configure('entity_extraction', { confidence: 0.5 });

  // Act
  const decision = router.decide({ taskType: 'entity_extraction', confidence: 0.5 });

  // Assert
  expect(decision.action).toBe('upgrade_to_large');
});
```

### 测试命名规范

使用中文描述行为，保持可读性：

```typescript
// ✅ 好：描述行为
it('当父级采集器禁用时，所有子采集器自动禁用')
it('飞书 token 过期时返回 401 错误并标记 unhealthy')
it('P0 任务 SLA 超时 5 分钟后自动升级处理优先级')

// ❌ 做：描述实现
it('should call disable on children')
it('returns 401')
it('SLA timeout')
```

### 断言规范

优先使用语义化断言：

```typescript
// ✅ 语义化断言
expect(event).toBeValidEvent();
expect(result).toHaveConfidenceAbove(0.6);
expect(task.status).toBeValidTransition('in_progress');

// ❌ 魔法值断言
expect(event.id).toBeTruthy();
expect(result.confidence).toBeGreaterThan(0.6);
```

## 测试隔离规范

### 每个测试独立

- 测试之间不共享状态
- 每个测试自己创建 Fake 实例和测试数据
- 不依赖执行顺序

```typescript
// ✅ 每个测试独立
it('场景 A', () => {
  const eventLog = new FakeEventLog();
  // ...
});

it('场景 B', () => {
  const eventLog = new FakeEventLog();
  // ...
});
```

### 共享基础设施 vs 共享状态

**共享基础设施是允许的**（如 MSW server、全局 setup），但每个测试必须通过 `afterEach` 清理状态：

```typescript
// ✅ 共享基础设施 + 状态清理
import { server } from '../setup'; // 共享 MSW server

afterEach(() => {
  server.resetHandlers(); // 清理 handler 状态
  vi.restoreAllMocks();   // 清理 mock 状态
});

// ❌ 共享可变状态
let sharedEvent: Event; // 跨测试共享的可变变量
```

### Mock 清理

Vitest 配置 `restoreMocks: true`，每个测试后自动恢复：

```typescript
// vitest.config.ts
export default defineConfig({
  test: {
    restoreMocks: true,
  },
});
```

### 文件系统隔离

涉及 Obsidian Vault 的测试使用临时目录：

```typescript
import { mkdtemp, rm } from 'fs/promises';
import { tmpdir } from 'os';
import { join } from 'path';

let tempDir: string;

beforeEach(async () => {
  tempDir = await mkdtemp(join(tmpdir(), 'work-better-test-'));
});

afterEach(async () => {
  await rm(tempDir, { recursive: true, force: true });
});
```

## 测试数据规范

### 使用工厂而非手写

```typescript
// ✅ 使用工厂
const event = eventFactory.build({ type: EventType.task_update });

// ❌ 手写完整对象
const event: Event = {
  id: 'evt-1',
  timestamp: '2024-01-01T00:00:00Z',
  collected_at: '2024-01-01T00:00:00Z',
  source: Source.feishu_message,
  // ... 20+ 字段
};
```

### Fixture 文件规范

- JSON 文件用于 API 响应快照
- JSONL 文件用于事件流（每行一个 Event）
- 文件名描述场景：`feishu-messages-success.json`、`task-update-event.jsonl`

## 禁止事项

| 禁止 | 原因 |
|------|------|
| 测试间共享可变状态 | 导致测试顺序依赖 |
| 硬编码 sleep/wait | 导致测试不稳定和慢 |
| 测试中调用真实飞书 API | 外部依赖不可控 |
| 测试中调用真实 AI 模型 | 结果不确定、成本高 |
| 跳过失败的测试（`.skip`） | 应修复或删除 |
| 测试中使用 `console.log` 调试 | 使用 Vitest 的 `--reporter` |

## 包脚本

```json
{
  "scripts": {
    "test": "vitest",
    "test:unit": "vitest run",
    "test:int": "vitest run --config vitest.config.int.ts",
    "test:e2e": "vitest run --config vitest.config.e2e.ts",
    "test:all": "vitest run && vitest run --config vitest.config.int.ts",
    "test:coverage": "vitest run --coverage",
    "test:watch": "vitest watch",
    "test:ui": "vitest --ui"
  }
}
```
