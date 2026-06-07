---
title: 测试层级定义
type: structural
domain: testing
created: 2026-06-06
updated: 2026-06-06
status: draft
---

# 测试层级定义

> **维护说明**：当新增测试层级、修改边界定义、或调整 Mock 策略时更新本文档。场景目录见 `03-scenario-catalog.md`。

## 层级总览

| 层级 | 代号 | 场景数 | 运行时间 | 框架 | 触发条件 |
|------|------|--------|----------|------|----------|
| 纯单元测试 | A+D | 158 | <10s | rstest / Vitest | 每次代码变更 |
| 集成测试 | B+E | 63 | <30s | rstest / Vitest | 每次代码变更 |
| 契约测试 | C | 9 | <10s | insta / httpmock | 外部依赖变更 / 夜间 |
| 跨层 E2E | F | 20 | <2min | Playwright + cargo test | PR / 发布 |
| 黑盒验收 | G | 182 | <10min (并行) | cucumber-rs | PR / 发布 |

---

## A 层：纯 Rust 单元测试 (136 scenarios)

### 边界定义

- **纯函数，无 I/O**
- 毫秒级运行
- 每个 crate 内的 `#[cfg(test)]` 模块
- 不依赖 SQLite、文件系统、网络、外部进程

### 子层分类

| 子层 | 场景数 | 所属模块 | 测试焦点 |
|------|--------|---------|---------|
| A1 | 16 | wb-processor/classifier.rs | 分类路由规则 |
| A2 | 16 | wb-ai/router.rs | 模型升级阈值 |
| A3 | 20 | wb-core/task.rs + wb-processor/task/lifecycle.rs | Task 状态机转换 |
| A4 | 16 | wb-processor/sla.rs | SLA 超时计算 |
| A5 | 16 | wb-ai/budget.rs | Token 预算管理 |
| A6 | 22 | wb-processor/review_rules.rs + reviewer.rs | Review Agent 规则 |
| A7 | 7 | wb-scheduler/dependency.rs | 调度器依赖图 |
| A8 | 6 | wb-scheduler/resource.rs | 调度器资源推迟 |
| A9 | 4 | wb-scheduler/scheduler.rs | 调度器重试/超时 |
| A10 | 5 | wb-collector/manager.rs | CollectorManager 状态 |
| A11 | 4 | wb-collector/feishu/messages.rs | 飞书消息转换 |
| A12 | 4 | src-tauri/commands/settings.rs | 配置构建 |

### 框架使用

- rstest `#[fixture]` 构建共享测试夹具
- rstest `#[case]` / `#[values]` 参数化测试
- rstest `#[context]` 提供 Agent 可解析的测试元数据
- 每个测试函数 < 50 行

### 代码示例：分类器路由参数化测试

```rust
use rstest::*;
use wb_processor::classifier::{Classifier, ProcessingRoute};
use wb_core::event::{Event, EventType, Source};

#[fixture]
fn classifier() -> Classifier {
    Classifier::new()
}

#[rstest]
#[case(EventType::TaskUpdate, ProcessingRoute::Instant)]
#[case(EventType::Approval, ProcessingRoute::Instant)]
#[case(EventType::ManualNote, ProcessingRoute::Instant)]
#[case(EventType::Meeting, ProcessingRoute::Instant)]
#[case(EventType::Email, ProcessingRoute::Instant)]
#[case(EventType::DocumentChange, ProcessingRoute::Aggregate)]
#[case(EventType::Browsing, ProcessingRoute::Aggregate)]
#[case(EventType::AppActivity, ProcessingRoute::Aggregate)]
#[case(EventType::OkrUpdate, ProcessingRoute::Pattern)]
fn test_classifier_routing(
    classifier: Classifier,
    #[case] event_type: EventType,
    #[case] expected: ProcessingRoute,
) {
    let event = Event::builder()
        .event_type(event_type)
        .source(Source::FeishuMessage)
        .build();
    let result = classifier.classify(&event);
    assert_eq!(result.route, expected);
}
```

### 代码示例：Task 状态机合法/非法转换

```rust
#[rstest]
#[case(TaskStatus::Todo, TaskStatus::InProgress, true)]
#[case(TaskStatus::Todo, TaskStatus::Cancelled, true)]
#[case(TaskStatus::InProgress, TaskStatus::Done, true)]
#[case(TaskStatus::InProgress, TaskStatus::Blocked, true)]
#[case(TaskStatus::Blocked, TaskStatus::InProgress, true)]
#[case(TaskStatus::Blocked, TaskStatus::Cancelled, true)]
#[case(TaskStatus::Todo, TaskStatus::Done, false)]
#[case(TaskStatus::Done, TaskStatus::InProgress, false)]
#[case(TaskStatus::Blocked, TaskStatus::Done, false)]
#[case(TaskStatus::Cancelled, TaskStatus::InProgress, false)]
fn test_task_transition(
    #[case] from: TaskStatus,
    #[case] to: TaskStatus,
    #[case] should_succeed: bool,
) {
    let task = Task::builder().status(from).build();
    let result = task.transition(to);
    if should_succeed {
        assert!(result.is_ok(), "{:?} -> {:?} should succeed", from, to);
    } else {
        assert!(result.is_err(), "{:?} -> {:?} should fail", from, to);
    }
}
```

---

## B 层：Rust 集成测试 (46 scenarios)

### 边界定义

- 涉及 I/O，但外部系统用 mock/in-memory 替代
- 秒级运行
- 在 `crates/*/tests/` 目录下

### 子层分类

| 子层 | 场景数 | 所属模块 | Mock 策略 |
|------|--------|---------|----------|
| B1 | 8 | wb-storage/sqlite/event_log.rs | `SqliteEventLog::new_in_memory()` |
| B2 | 3 | src-tauri/commands/events.rs | in-memory SQLite |
| B3 | 9 | src-tauri/commands/settings.rs | `tempfile::tempdir()` |
| B4 | 4 | src-tauri/commands/collectors.rs | Mock CollectorManager |
| B5 | 4 | src-tauri/commands/scheduler.rs | Mock Scheduler |
| B6 | 3 | src-tauri/commands/collect.rs | in-memory SQLite |
| B7 | 6 | wb-storage/obsidian/ | `tempfile::tempdir()` |
| B8 | 5 | wb-storage/freshness/ | in-memory stores |
| B9 | 4 | wb-storage/vector/ | InMemoryVectorStore |

### Mock 策略

| 外部系统 | Mock 方式 | 说明 |
|---------|----------|------|
| SQLite | `SqliteEventLog::new_in_memory()` | 已实现 |
| 文件系统 | `tempfile::tempdir()` | 测试结束自动清理 |
| Vector DB | `InMemoryVectorStore` | 内存向量存储 |
| lark-cli | Canned JSON responses | 录制的真实响应 |
| AI 模型 | Mock adapter | 固定返回值 |

### 代码示例：EventLog 集成测试

```rust
#[fixture]
async fn event_log() -> SqliteEventLog {
    SqliteEventLog::new_in_memory().await.unwrap()
}

#[rstest]
#[tokio::test]
async fn test_append_and_retrieve(#[future] event_log: SqliteEventLog) {
    let log = event_log.await;
    let event = Event::builder()
        .source(Source::UserCapture)
        .event_type(EventType::ManualNote)
        .content("test content".into())
        .build();
    log.append(event.clone()).await.unwrap();
    let retrieved = log.get_by_id(&event.id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().content, event.content);
}
```

### 代码示例：Obsidian Writer 集成测试

```rust
#[tokio::test]
async fn test_write_daily_note() {
    let vault = tempdir().unwrap();
    let journal = DailyJournal::new(vault.path());
    journal.append("## 09:00\n会议记录内容").await.unwrap();

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let file = vault.path().join("diary").join(format!("{}.md", today));
    assert!(file.exists());

    let content = tokio::fs::read_to_string(&file).await.unwrap();
    assert!(content.contains("会议记录内容"));
    assert!(content.contains("---")); // YAML frontmatter
}
```

---

## C 层：契约测试 (9 scenarios)

### 边界定义

- 验证代码对外部系统的假设是否与真实世界一致
- 使用 insta 快照 + httpmock record/replay
- 外部依赖变更时触发

### 子层分类

| 子层 | 场景数 | 验证目标 | 方法 |
|------|--------|---------|------|
| C1 | 4 | lark-cli 输出格式 | insta JSON 快照 |
| C2 | 2 | 飞书 API 响应 schema | httpmock record/replay |
| C3 | 3 | 文件系统行为 | 平台特定断言 |

### 快照管理

- `.snap` 文件存储在版本控制中
- `cargo insta review` 审核变更
- redaction 处理动态字段 (timestamp, uuid, token)

### 代码示例

```rust
use insta::assert_json_snapshot;

#[test]
fn test_lark_messages_response_format() {
    let raw_json = include_str!("../fixtures/lark_messages_response.json");
    let response: LarkMessagesResponse = serde_json::from_str(raw_json).unwrap();
    assert_json_snapshot!("lark_messages_response", response, {
        ".data.messages[].create_time" => "[timestamp]",
        ".data.messages[].message_id" => "[message_id]",
        ".data.messages[].sender.id" => "[user_id]"
    });
}
```

---

## D 层：TypeScript 单元测试 (22 scenarios)

### 边界定义

- React 组件渲染、UI 工具函数、状态管理
- 不涉及 Tauri IPC
- jsdom 环境

### 子层分类

| 子层 | 场景数 | 测试焦点 |
|------|--------|---------|
| D1 | 14 | React 组件渲染 |
| D2 | 5 | UI 工具函数 |
| D3 | 3 | 状态管理 |

### 代码示例

```typescript
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi } from 'vitest';
import { CaptureWindow } from '@/capture/CaptureWindow';

describe('CaptureWindow', () => {
  it('renders textarea ready for input', () => {
    render(<CaptureWindow />);
    const textarea = screen.getByRole('textbox');
    expect(textarea).toBeInTheDocument();
    expect(textarea).toHaveFocus();
  });

  it('submits text via invoke', async () => {
    const invokeSpy = vi.spyOn(window.__TAURI__.core, 'invoke');
    render(<CaptureWindow />);
    await userEvent.type(screen.getByRole('textbox'), '测试笔记');
    await userEvent.keyboard('{Meta>}');
    expect(invokeSpy).toHaveBeenCalledWith(
      'trigger_manual_capture',
      expect.objectContaining({ text: '测试笔记' })
    );
  });
});
```

---

## E 层：TypeScript 集成测试 (17 scenarios)

### 边界定义

- 测试 `invoke()` 调用与 Tauri 命令层的交互
- Mock Tauri IPC
- 验证参数传递和返回值类型

### 子层分类

| 子层 | 场景数 | 测试焦点 |
|------|--------|---------|
| E1 | 15 | Tauri invoke 调用 |
| E2 | 2 | 事件监听器 |

### 代码示例

```typescript
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';
const mockInvoke = vi.mocked(invoke);

describe('Tauri invoke calls', () => {
  it('getEvents passes correct args', async () => {
    mockInvoke.mockResolvedValueOnce([]);
    await getEvents({ limit: 50, offset: 0 });
    expect(mockInvoke).toHaveBeenCalledWith('get_events', { limit: 50, offset: 0 });
  });

  it('triggerManualCapture passes text', async () => {
    mockInvoke.mockResolvedValueOnce({ id: 'test-id' });
    await triggerManualCapture('测试内容');
    expect(mockInvoke).toHaveBeenCalledWith('trigger_manual_capture', { text: '测试内容' });
  });
});
```

---

## F 层：跨层 E2E 测试 (20 scenarios)

### 边界定义

- 从前端 UI 到 Rust 后端到存储的完整链路
- Playwright 驱动 Tauri app
- Mock 外部系统，真实文件系统

### 子层分类

| 子层 | 场景数 | 测试焦点 |
|------|--------|---------|
| F1 | 2 | 快捷记录流程 |
| F2 | 3 | 飞书采集流程 |
| F3 | 4 | 事件处理流程 |
| F4 | 4 | 设置变更传播 |
| F5 | 4 | 调度器集成 |
| F6 | 3 | 菜单栏数据流 |

### 代码示例

```typescript
import { test, expect } from '@playwright/test';

test('快捷记录写入 EventLog', async ({ page }) => {
  await page.evaluate(() =>
    window.__TAURI__.core.invoke('show_capture_window')
  );
  const textarea = page.locator('textarea');
  await textarea.fill('E2E 测试笔记');
  await page.keyboard.press('Meta+Enter');

  const events = await page.evaluate(() =>
    window.__TAURI__.core.invoke('get_events', { limit: 10, offset: 0 })
  );
  const found = events.find((e: any) => e.content.includes('E2E 测试笔记'));
  expect(found).toBeDefined();
  expect(found.source).toBe('user_capture');
});
```

---

## G 层：黑盒验收测试 (182 scenarios)

### 边界定义

- 纯产品视角，无实现知识
- Given/When/Then 格式
- cucumber-rs 纯 Rust API
- 1:1 映射到产品场景

### Domain 分组

| Domain | 场景数 | 产品场景编号 |
|--------|--------|------------|
| G1 信息采集 | 37 | 1-37 |
| G2 智能处理 | 34 | 38-71 |
| G3 数据存储 | 28 | 72-99 |
| G4 任务管理 | 20 | 100-119 |
| G5 报告生成 | 12 | 120-131 |
| G6 系统能力 | 38 | 132-169 |
| G7 横切关注 | 13 | 170-182 |

### World 状态管理

```rust
#[derive(Debug, World)]
pub struct TestWorld {
    event_log: SqliteEventLog,
    processor: Processor,
    task_manager: TaskManager,
    config: AppConfig,
    vault_path: PathBuf,
    last_event: Option<Event>,
    last_record: Option<WorkRecord>,
    last_task: Option<Task>,
    last_error: Option<String>,
}

impl Default for TestWorld {
    fn default() -> Self {
        // 用 tokio runtime 初始化 async 资源
        let vault = tempfile::tempdir().unwrap();
        Self {
            event_log: /* in-memory */,
            processor: Processor::new_with_mock_ai(),
            // ...
        }
    }
}
```

### 代码示例：验收场景

```rust
#[given(regex = r"^用户提交了一条内容为「(.+)」的快捷记录$")]
async fn user_submits_note(world: &mut TestWorld, content: String) {
    let event = Event::builder()
        .source(Source::UserCapture)
        .event_type(EventType::ManualNote)
        .content(content)
        .build();
    world.event_log.append(event.clone()).await.unwrap();
    world.last_event = Some(event);
}

#[when("系统处理该事件")]
async fn system_processes_event(world: &mut TestWorld) {
    let event = world.last_event.as_ref().unwrap();
    let record = world.processor.process(event.clone()).await.unwrap();
    world.last_record = Some(record);
}

#[then(regex = r"^该记录应被分类为「(.+)」$")]
fn record_classified(world: &mut TestWorld, expected: String) {
    let record = world.last_record.as_ref().unwrap();
    assert_eq!(record.category.to_string(), expected);
}
```

---

## 层级间依赖关系

```
A 层 (纯单元) ──无依赖──→ 可独立运行
    │
    ▼
B 层 (集成) ──依赖 A 层通过──→ 验证模块间协作
    │
    ▼
C 层 (契约) ──独立──→ 验证外部系统假设
    │
    ▼
F 层 (E2E) ──依赖 B 层通过──→ 验证端到端数据流
    │
    ▼
G 层 (验收) ──依赖 F 层通过──→ 验证产品行为
```

**失败传播规则**：
- A 层失败 → 阻塞 B、F、G 层
- B 层失败 → 阻塞 F、G 层
- C 层失败 → 不阻塞开发流程（仅夜间/外部变更触发）
- F 层失败 → 阻塞 G 层
- G 层失败 → 直接阻塞合并/发布
