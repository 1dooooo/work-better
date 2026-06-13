//! 真实后端 happy-path 集成测试
//!
//! 验证 5 个核心功能在真实 crate 层的行为，不 mock 任何内部组件。

use std::sync::Arc;
use wb_core::event::{Confidence, Event, EventFilter, EventLog, EventType, Source};
use wb_collector::traits::{Collector, HealthLevel, HealthStatus};
use wb_processor::task::model::{TaskFilter, TaskPriority, TaskSource};
use wb_real_backend_tests::TestHarness;

// ── Mock Collector ─────────────────────────────────────────

struct MockCollector {
    id: String,
}

impl MockCollector {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl Collector for MockCollector {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> &str {
        &self.id
    }
    fn group_id(&self) -> &str {
        "test"
    }
    fn group_name(&self) -> &str {
        "测试"
    }
    fn version(&self) -> &str {
        "0.1.0-test"
    }
    async fn collect(&self) -> wb_core::error::Result<Vec<Event>> {
        Ok(vec![])
    }
    async fn health_check(&self) -> HealthStatus {
        HealthStatus::healthy()
    }
}

// ── Test 1: 手动捕获 → 事件出现在事件列表 ──────────────────

#[tokio::test]
async fn manual_capture_event_appears_in_event_list() {
    let harness = TestHarness::new();
    let event = Event::new(
        Source::UserCapture,
        Confidence::High,
        EventType::ManualNote,
        serde_json::json!({"text": "手动捕获的笔记"}),
        "{}".to_string(),
    );
    let event_id = event.id.clone();

    harness
        .event_log
        .append(&event)
        .await
        .expect("append should succeed");

    let events = harness
        .event_log
        .query(&EventFilter::default())
        .await
        .expect("query should succeed");
    assert_eq!(events.len(), 1, "应有 1 条事件");
    assert_eq!(events[0].id, event_id);
    assert_eq!(events[0].source, Source::UserCapture);
    assert_eq!(events[0].event_type, EventType::ManualNote);
}

// ── Test 2: 创建任务 → 任务出现在任务列表 ──────────────────

#[tokio::test]
async fn create_task_appears_in_task_list() {
    let harness = TestHarness::new();
    let task = harness
        .task_manager
        .create("整理周报", TaskPriority::P1, TaskSource::Manual)
        .await
        .expect("create should succeed");

    assert_eq!(task.title, "整理周报");
    assert_eq!(task.priority, TaskPriority::P1);
    assert_eq!(task.source, TaskSource::Manual);

    let tasks = harness
        .task_manager
        .list(TaskFilter::default())
        .await
        .expect("list should succeed");
    assert_eq!(tasks.len(), 1, "应有 1 个任务");
    assert_eq!(tasks[0].id, task.id);
}

// ── Test 3: 采集器开关 → 状态正确变化 ──────────────────────

#[tokio::test]
async fn collector_enable_disable_state_changes() {
    let harness = TestHarness::new();
    let collector = Arc::new(MockCollector::new("feishu"));
    harness.collector_manager.register(collector).await;

    assert!(
        harness.collector_manager.is_enabled("feishu").await,
        "注册后默认启用"
    );

    harness.collector_manager.disable("feishu").await;
    assert!(
        !harness.collector_manager.is_enabled("feishu").await,
        "禁用后应为 false"
    );

    harness.collector_manager.enable("feishu").await;
    assert!(
        harness.collector_manager.is_enabled("feishu").await,
        "重新启用后应为 true"
    );
}

// ── Test 4: 调度器暂停/恢复 → 状态正确切换 ─────────────────

#[tokio::test]
async fn scheduler_pause_resume_state_toggle() {
    let harness = TestHarness::new();
    assert!(!harness.scheduler.is_paused().await, "初始非暂停");

    harness.scheduler.pause_all().await;
    assert!(harness.scheduler.is_paused().await, "暂停后应为 true");

    harness.scheduler.resume_all().await;
    assert!(
        !harness.scheduler.is_paused().await,
        "恢复后应为 false"
    );
}

// ── Test 5: 事件标记处理 → 状态正确更新 ────────────────────

#[tokio::test]
async fn event_mark_processed_state_updates() {
    let harness = TestHarness::new();
    let event = Event::new(
        Source::FeishuMessage,
        Confidence::High,
        EventType::Message,
        serde_json::json!({"text": "待处理消息"}),
        "{}".to_string(),
    );
    let event_id = event.id.clone();

    harness
        .event_log
        .append(&event)
        .await
        .expect("append should succeed");

    let unprocessed = harness
        .event_log
        .get_unprocessed(None)
        .await
        .expect("get_unprocessed");
    assert_eq!(unprocessed.len(), 1, "应有 1 条未处理事件");

    harness
        .event_log
        .mark_processed(&event_id)
        .await
        .expect("mark_processed");

    let unprocessed = harness
        .event_log
        .get_unprocessed(None)
        .await
        .expect("get_unprocessed");
    assert_eq!(unprocessed.len(), 0, "标记后应为 0 条未处理");

    let stored = harness
        .event_log
        .get(&event_id)
        .await
        .expect("get");
    assert!(stored.is_some(), "事件应仍存在于数据库中");
}
