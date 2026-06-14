//! 任务发现 L2 集成测试
//!
//! 测试场景：验证完整的任务发现链路
//! - 消息 → AI 提取 → 任务创建/状态更新 → pending 列表
//!
//! 核心验证：两条消息的完整流程
//! 1. "我今天要发邮件给lily" → 创建任务
//! 2. "给Lily的邮件已经发送了" → 状态更新，不创建新任务

use wb_ai::{Extraction, MockAdapter};
use wb_core::event::Source;
use wb_processor::task::discovery::TaskDiscovery;
use wb_processor::task::model::TaskSource;

/// 辅助函数：创建新任务 adapter
fn new_task_adapter(title: &str, people: Vec<String>) -> MockAdapter {
    MockAdapter::new().with_extraction(Extraction {
        title: title.to_string(),
        summary: format!("{}的摘要", title),
        detail: String::new(),
        people,
        tags: vec![],
        project: None,
        due_date: None,
        confidence: 0.9,
        is_status_update: false,
        related_task_id: None,
    })
}

/// 辅助函数：创建状态更新 adapter
fn status_update_adapter(related_task_id: String) -> MockAdapter {
    MockAdapter::new().with_extraction(Extraction {
        title: String::new(),
        summary: String::new(),
        detail: String::new(),
        people: vec![],
        tags: vec![],
        project: None,
        due_date: None,
        confidence: 0.9,
        is_status_update: true,
        related_task_id: Some(related_task_id),
    })
}

// ─── 核心场景：两条消息的完整流程 ─────────────────────────────

#[tokio::test]
async fn test_l2_two_messages_no_duplicate_task() {
    let mut discovery = TaskDiscovery::new();

    // 第一条消息：发现任务
    let adapter1 = new_task_adapter("发邮件给lily", vec!["lily".to_string()]);
    let tasks1 = discovery
        .discover_with_ai("我今天要发邮件给lily", &adapter1, Source::UserCapture)
        .await;

    assert_eq!(tasks1.len(), 1, "第一条消息应发现一个任务");
    assert_eq!(tasks1[0].title, "发邮件给lily");
    assert_eq!(tasks1[0].source, TaskSource::Message);
    assert_eq!(discovery.pending_count(), 1, "pending 列表应有 1 个任务");

    // 第二条消息：状态更新
    let adapter2 = status_update_adapter(tasks1[0].id.clone());
    let tasks2 = discovery
        .discover_with_ai("给Lily的邮件已经发送了", &adapter2, Source::UserCapture)
        .await;

    assert!(tasks2.is_empty(), "状态更新不应创建新任务");
    assert_eq!(discovery.pending_count(), 1, "pending 数量不应增加");
}

// ─── 多条消息的复杂场景 ─────────────────────────────────────

#[tokio::test]
async fn test_l2_multiple_messages_mixed() {
    let mut discovery = TaskDiscovery::new();

    // 消息 1：新任务
    let adapter1 = new_task_adapter("发邮件给lily", vec!["lily".to_string()]);
    let tasks1 = discovery
        .discover_with_ai("我今天要发邮件给lily", &adapter1, Source::UserCapture)
        .await;
    assert_eq!(tasks1.len(), 1);
    assert_eq!(discovery.pending_count(), 1);

    // 消息 2：另一个新任务
    let adapter2 = new_task_adapter("写周报", vec![]);
    let tasks2 = discovery
        .discover_with_ai("我需要写周报", &adapter2, Source::UserCapture)
        .await;
    assert_eq!(tasks2.len(), 1);
    assert_eq!(discovery.pending_count(), 2);

    // 消息 3：状态更新（完成第一个任务）
    let adapter3 = status_update_adapter(tasks1[0].id.clone());
    let tasks3 = discovery
        .discover_with_ai("给Lily的邮件已经发送了", &adapter3, Source::UserCapture)
        .await;
    assert!(tasks3.is_empty(), "状态更新不应创建新任务");
    assert_eq!(discovery.pending_count(), 2, "pending 数量不应变化");

    // 消息 4：状态更新（完成第二个任务）
    let adapter4 = status_update_adapter(tasks2[0].id.clone());
    let tasks4 = discovery
        .discover_with_ai("周报已经写完了", &adapter4, Source::UserCapture)
        .await;
    assert!(tasks4.is_empty(), "状态更新不应创建新任务");
    assert_eq!(discovery.pending_count(), 2, "pending 数量不应变化");
}

// ─── 不同来源的消息 ─────────────────────────────────────────

#[tokio::test]
async fn test_l2_different_sources() {
    let mut discovery = TaskDiscovery::new();

    let sources = vec![
        Source::UserCapture,
        Source::FeishuMessage,
        Source::SystemBrowser,
    ];

    for (i, source) in sources.iter().enumerate() {
        let task_name = format!("任务{}", i + 1);
        let adapter = new_task_adapter(&task_name, vec![]);
        let tasks = discovery
            .discover_with_ai(
                &format!("需要完成任务{}", i + 1),
                &adapter,
                source.clone(),
            )
            .await;
        assert_eq!(tasks.len(), 1, "来源 {:?} 应能发现任务", source);
    }

    assert_eq!(discovery.pending_count(), 3, "应有 3 个待确认任务");
}

// ─── 任务确认流程 ─────────────────────────────────────────

#[tokio::test]
async fn test_l2_confirm_reject_flow() {
    let mut discovery = TaskDiscovery::new();

    // 创建任务
    let adapter = new_task_adapter("发邮件给lily", vec!["lily".to_string()]);
    let tasks = discovery
        .discover_with_ai("我今天要发邮件给lily", &adapter, Source::UserCapture)
        .await;
    assert_eq!(tasks.len(), 1);
    assert_eq!(discovery.pending_count(), 1);

    let pending_id = &tasks[0].id;

    // 确认任务
    let confirmed = discovery.confirm(pending_id);
    assert!(confirmed.is_ok(), "确认任务应成功");
    assert_eq!(discovery.pending_count(), 0, "确认后 pending 应清空");

    // 创建另一个任务并拒绝
    let adapter2 = new_task_adapter("写周报", vec![]);
    let tasks2 = discovery
        .discover_with_ai("我需要写周报", &adapter2, Source::UserCapture)
        .await;
    assert_eq!(tasks2.len(), 1);
    assert_eq!(discovery.pending_count(), 1);

    let rejected = discovery.reject(&tasks2[0].id);
    assert!(rejected.is_ok(), "拒绝任务应成功");
    assert_eq!(discovery.pending_count(), 0, "拒绝后 pending 应清空");
}

// ─── 边界情况：空文本 ─────────────────────────────────────

#[tokio::test]
async fn test_l2_empty_text() {
    let mut discovery = TaskDiscovery::new();

    let adapter = new_task_adapter("发邮件给lily", vec!["lily".to_string()]);
    let tasks = discovery
        .discover_with_ai("", &adapter, Source::UserCapture)
        .await;

    // 空文本不应崩溃
    assert!(discovery.pending_count() <= 1);
}

// ─── 边界情况：低置信度 ─────────────────────────────────────

#[tokio::test]
async fn test_l2_low_confidence() {
    let adapter = MockAdapter::new().with_extraction(Extraction {
        title: "可能的任务".to_string(),
        summary: "低置信度".to_string(),
        detail: String::new(),
        people: vec![],
        tags: vec![],
        project: None,
        due_date: None,
        confidence: 0.3,
        is_status_update: false,
        related_task_id: None,
    });

    let mut discovery = TaskDiscovery::new();
    let tasks = discovery
        .discover_with_ai("可能是个任务", &adapter, Source::UserCapture)
        .await;

    // 低置信度不应崩溃
    assert!(discovery.pending_count() <= 1);
}
