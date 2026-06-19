//! 任务发现 L2 集成测试
//!
//! 测试场景：验证完整的任务发现链路
//! - 消息 → AI 提取 → 任务创建/状态更新 → pending 列表
//!
//! 核心验证：两条消息的完整流程
//! 1. "我今天要发邮件给lily" → 创建任务
//! 2. "给Lily的邮件已经发送了" → 状态更新，不创建新任务

use std::collections::HashMap;
use wb_ai::{Extraction, MockAdapter, ModelRouter, TaskRunner, TokenBudget};
use wb_core::event::Source;
use wb_processor::task::discovery::TaskDiscovery;
use wb_processor::task::model::TaskSource;

/// 辅助函数：创建 TaskRunner
fn make_runner(extraction: Extraction) -> TaskRunner {
    let router = ModelRouter::new();
    let budget = TokenBudget::new(100_000);
    let mut adapters: HashMap<wb_ai::ModelSize, Box<dyn wb_ai::ModelAdapter>> = HashMap::new();
    adapters.insert(
        wb_ai::ModelSize::Small,
        Box::new(MockAdapter::new().with_extraction(extraction.clone())),
    );
    adapters.insert(
        wb_ai::ModelSize::Large,
        Box::new(MockAdapter::new().with_extraction(extraction)),
    );
    let mut adapter_names = HashMap::new();
    adapter_names.insert(wb_ai::ModelSize::Small, "mock-small".to_string());
    adapter_names.insert(wb_ai::ModelSize::Large, "mock-large".to_string());
    TaskRunner::new(router, budget, adapters, adapter_names)
}

/// 辅助函数：创建新任务 runner
fn new_task_runner(title: &str, people: Vec<String>) -> TaskRunner {
    make_runner(Extraction {
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

/// 辅助函数：创建状态更新 runner
fn status_update_runner(related_task_id: String) -> TaskRunner {
    make_runner(Extraction {
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
    let mut runner1 = new_task_runner("发邮件给lily", vec!["lily".to_string()]);
    let tasks1 = discovery
        .discover_with_ai("我今天要发邮件给lily", &mut runner1, Source::UserCapture)
        .await;

    assert_eq!(tasks1.len(), 1, "第一条消息应发现一个任务");
    assert_eq!(tasks1[0].title, "发邮件给lily");
    assert_eq!(tasks1[0].source, TaskSource::Message);
    assert_eq!(discovery.pending_count(), 1, "pending 列表应有 1 个任务");

    // 第二条消息：状态更新
    let mut runner2 = status_update_runner(tasks1[0].id.clone());
    let tasks2 = discovery
        .discover_with_ai("给Lily的邮件已经发送了", &mut runner2, Source::UserCapture)
        .await;

    assert!(tasks2.is_empty(), "状态更新不应创建新任务");
    assert_eq!(discovery.pending_count(), 1, "pending 数量不应增加");
}

// ─── 多条消息的复杂场景 ─────────────────────────────────────

#[tokio::test]
async fn test_l2_multiple_messages_mixed() {
    let mut discovery = TaskDiscovery::new();

    // 消息 1：新任务
    let mut runner1 = new_task_runner("发邮件给lily", vec!["lily".to_string()]);
    let tasks1 = discovery
        .discover_with_ai("我今天要发邮件给lily", &mut runner1, Source::UserCapture)
        .await;
    assert_eq!(tasks1.len(), 1);
    assert_eq!(discovery.pending_count(), 1);

    // 消息 2：另一个新任务
    let mut runner2 = new_task_runner("写周报", vec![]);
    let tasks2 = discovery
        .discover_with_ai("我需要写周报", &mut runner2, Source::UserCapture)
        .await;
    assert_eq!(tasks2.len(), 1);
    assert_eq!(discovery.pending_count(), 2);

    // 消息 3：状态更新（完成第一个任务）
    let mut runner3 = status_update_runner(tasks1[0].id.clone());
    let tasks3 = discovery
        .discover_with_ai("给Lily的邮件已经发送了", &mut runner3, Source::UserCapture)
        .await;
    assert!(tasks3.is_empty(), "状态更新不应创建新任务");
    assert_eq!(discovery.pending_count(), 2, "pending 数量不应变化");

    // 消息 4：状态更新（完成第二个任务）
    let mut runner4 = status_update_runner(tasks2[0].id.clone());
    let tasks4 = discovery
        .discover_with_ai("周报已经写完了", &mut runner4, Source::UserCapture)
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
        let mut runner = new_task_runner(&task_name, vec![]);
        let tasks = discovery
            .discover_with_ai(
                &format!("需要完成任务{}", i + 1),
                &mut runner,
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
    let mut runner1 = new_task_runner("发邮件给lily", vec!["lily".to_string()]);
    let tasks = discovery
        .discover_with_ai("我今天要发邮件给lily", &mut runner1, Source::UserCapture)
        .await;
    assert_eq!(tasks.len(), 1);
    assert_eq!(discovery.pending_count(), 1);

    let pending_id = &tasks[0].id;

    // 确认任务
    let confirmed = discovery.confirm(pending_id);
    assert!(confirmed.is_ok(), "确认任务应成功");
    assert_eq!(discovery.pending_count(), 0, "确认后 pending 应清空");

    // 创建另一个任务并拒绝
    let mut runner2 = new_task_runner("写周报", vec![]);
    let tasks2 = discovery
        .discover_with_ai("我需要写周报", &mut runner2, Source::UserCapture)
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

    let mut runner = new_task_runner("发邮件给lily", vec!["lily".to_string()]);
    let tasks = discovery
        .discover_with_ai("", &mut runner, Source::UserCapture)
        .await;

    // 空文本不应崩溃
    assert!(discovery.pending_count() <= 1);
}

// ─── 边界情况：低置信度 ─────────────────────────────────────

#[tokio::test]
async fn test_l2_low_confidence() {
    let mut runner = make_runner(Extraction {
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
        .discover_with_ai("可能是个任务", &mut runner, Source::UserCapture)
        .await;

    // 低置信度不应崩溃
    assert!(discovery.pending_count() <= 1);
}
