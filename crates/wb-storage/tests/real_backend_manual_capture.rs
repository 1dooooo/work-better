//! 手动捕获功能真实后端测试
//!
//! 测试场景：用户输入文本 → 调用 trigger_manual_capture → 事件出现在 EventLog
//!
//! 验证目标：
//! 1. 事件被正确创建并持久化
//! 2. 事件字段（source, type, content）正确
//! 3. 事件可以通过 EventLog 查询到

use wb_core::event::{Confidence, Event, EventLog, EventType, Source};
use wb_storage::SqliteEventLog;

/// 创建测试事件
fn create_test_event(content: &str) -> Event {
    Event::new(
        Source::UserCapture,
        Confidence::High,
        EventType::ManualNote,
        serde_json::json!({"text": content}),
        format!(r#"{{"raw": "{}"}}"#, content),
    )
}

/// 测试手动捕获创建事件并持久化到 EventLog
///
/// 场景：用户在速记窗口输入文本，点击提交
/// 预期：事件出现在 EventLog 中，字段正确
#[tokio::test]
async fn test_manual_capture_creates_event_in_log() {
    let event_log = SqliteEventLog::new_in_memory().unwrap();

    // 模拟手动捕获：创建事件
    let content = "这是一条测试笔记";
    let event = Event::new(
        Source::UserCapture,
        Confidence::High,
        EventType::ManualNote,
        serde_json::json!({"text": content}),
        format!(r#"{{"raw": "{}"}}"#, content),
    );

    // 执行：追加事件到 EventLog
    event_log.append(&event).await.unwrap();

    // 验证：事件被持久化
    let events = event_log.get_unprocessed(None).await.unwrap();
    assert_eq!(events.len(), 1, "应该有 1 个未处理的事件");

    let retrieved = &events[0];
    assert_eq!(retrieved.id, event.id, "事件 ID 应该匹配");
    assert_eq!(retrieved.source, Source::UserCapture, "来源应该是 UserCapture");
    assert_eq!(retrieved.event_type, EventType::ManualNote, "类型应该是 ManualNote");
    assert_eq!(
        retrieved.content,
        serde_json::json!({"text": content}),
        "内容应该匹配"
    );
    assert!(retrieved.tags.is_empty(), "新事件应该没有标签");
    assert!(retrieved.related_ids.is_empty(), "新事件应该没有关联 ID");
    assert!(retrieved.attachments.is_empty(), "新事件应该没有附件");
}

/// 测试手动捕获的事件可以通过 ID 查询
///
/// 场景：创建事件后，通过 ID 查询
/// 预期：可以找到该事件
#[tokio::test]
async fn test_manual_capture_event_queryable_by_id() {
    let event_log = SqliteEventLog::new_in_memory().unwrap();

    let event = create_test_event("可查询的笔记");
    event_log.append(&event).await.unwrap();

    // 通过 ID 查询
    let retrieved = event_log.get(&event.id).await.unwrap();
    assert!(retrieved.is_some(), "应该能通过 ID 找到事件");

    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.content, serde_json::json!({"text": "可查询的笔记"}));
}

/// 测试多个手动捕获事件的顺序
///
/// 场景：连续创建多个事件
/// 预期：事件按时间倒序排列（最新的在前）
#[tokio::test]
async fn test_manual_capture_events_order() {
    let event_log = SqliteEventLog::new_in_memory().unwrap();

    // 创建多个事件
    let event1 = create_test_event("第一条笔记");
    let event2 = create_test_event("第二条笔记");
    let event3 = create_test_event("第三条笔记");

    event_log.append(&event1).await.unwrap();
    event_log.append(&event2).await.unwrap();
    event_log.append(&event3).await.unwrap();

    // 查询所有事件
    let events = event_log.get_unprocessed(None).await.unwrap();
    assert_eq!(events.len(), 3, "应该有 3 个事件");

    // 验证顺序（按时间倒序）
    assert_eq!(events[0].content, serde_json::json!({"text": "第三条笔记"}));
    assert_eq!(events[1].content, serde_json::json!({"text": "第二条笔记"}));
    assert_eq!(events[2].content, serde_json::json!({"text": "第一条笔记"}));
}

/// 测试手动捕获事件的标记处理
///
/// 场景：创建事件后，标记为已处理
/// 预期：事件不再出现在未处理列表中
#[tokio::test]
async fn test_manual_capture_event_mark_processed() {
    let event_log = SqliteEventLog::new_in_memory().unwrap();

    let event = create_test_event("待处理的笔记");
    event_log.append(&event).await.unwrap();

    // 验证初始状态：未处理
    let unprocessed = event_log.get_unprocessed(None).await.unwrap();
    assert_eq!(unprocessed.len(), 1, "应该有 1 个未处理事件");

    // 标记为已处理
    event_log.mark_processed(&event.id).await.unwrap();

    // 验证：不再出现在未处理列表
    let unprocessed = event_log.get_unprocessed(None).await.unwrap();
    assert!(unprocessed.is_empty(), "标记处理后不应该出现在未处理列表");

    // 验证：仍然可以通过 ID 查询到
    let retrieved = event_log.get(&event.id).await.unwrap();
    assert!(retrieved.is_some(), "标记处理后仍然应该能通过 ID 查询到");
}

/// 测试手动捕获事件的字段完整性
///
/// 场景：创建事件，验证所有字段
/// 预期：所有字段都有正确的值
#[tokio::test]
async fn test_manual_capture_event_fields() {
    let event_log = SqliteEventLog::new_in_memory().unwrap();

    let content = "完整的字段测试";
    let event = Event::new(
        Source::UserCapture,
        Confidence::High,
        EventType::ManualNote,
        serde_json::json!({"text": content, "extra": "data"}),
        r#"{"raw": "payload"}"#.to_string(),
    );

    event_log.append(&event).await.unwrap();

    let retrieved = event_log.get(&event.id).await.unwrap().unwrap();

    // 验证所有字段
    assert!(!retrieved.id.is_empty(), "ID 不应该为空");
    assert!(retrieved.timestamp.timestamp() > 0, "时间戳应该有效");
    assert!(retrieved.collected_at.timestamp() > 0, "采集时间应该有效");
    assert_eq!(retrieved.source, Source::UserCapture);
    assert_eq!(retrieved.source_confidence, Confidence::High);
    assert_eq!(retrieved.event_type, EventType::ManualNote);
    assert_eq!(
        retrieved.content,
        serde_json::json!({"text": content, "extra": "data"})
    );
    assert_eq!(retrieved.raw_payload, r#"{"raw": "payload"}"#);
    assert!(retrieved.tags.is_empty());
    assert!(retrieved.related_ids.is_empty());
    assert!(retrieved.attachments.is_empty());
}
