//! 事件标记处理功能真实后端测试
//!
//! 测试场景：调用 mark_event_processed → 事件状态正确更新
//!
//! 验证目标：
//! 1. 事件初始状态为未处理
//! 2. 标记处理后状态正确更新
//! 3. 已处理事件不出现在未处理列表
//! 4. 已处理事件仍可通过 ID 查询

use wb_core::event::{Event, EventLog, Source, Confidence, EventType};
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

/// 测试事件初始状态为未处理
///
/// 场景：创建事件后，检查处理状态
/// 预期：事件默认未处理
#[tokio::test]
async fn test_event_initial_unprocessed() {
    let event_log = SqliteEventLog::new_in_memory().unwrap();

    let event = create_test_event("测试事件");
    event_log.append(&event).await.unwrap();

    // 查询未处理事件
    let unprocessed = event_log.get_unprocessed(None).await.unwrap();

    // 验证：事件在未处理列表中
    assert_eq!(unprocessed.len(), 1, "应该有 1 个未处理事件");
    assert_eq!(unprocessed[0].id, event.id, "事件 ID 应该匹配");
}

/// 测试标记事件为已处理
///
/// 场景：标记事件后，检查状态
/// 预期：事件状态更新为已处理
#[tokio::test]
async fn test_event_mark_processed() {
    let event_log = SqliteEventLog::new_in_memory().unwrap();

    let event = create_test_event("待处理事件");
    event_log.append(&event).await.unwrap();

    // 标记为已处理
    event_log.mark_processed(&event.id).await.unwrap();

    // 验证：事件不在未处理列表中
    let unprocessed = event_log.get_unprocessed(None).await.unwrap();
    assert!(
        unprocessed.is_empty(),
        "标记处理后事件不应该出现在未处理列表"
    );
}

/// 测试已处理事件仍可通过 ID 查询
///
/// 场景：标记处理后，通过 ID 查询
/// 预期：仍然可以找到该事件
#[tokio::test]
async fn test_processed_event_queryable_by_id() {
    let event_log = SqliteEventLog::new_in_memory().unwrap();

    let event = create_test_event("已处理事件");
    event_log.append(&event).await.unwrap();

    // 标记为已处理
    event_log.mark_processed(&event.id).await.unwrap();

    // 通过 ID 查询
    let retrieved = event_log.get(&event.id).await.unwrap();
    assert!(
        retrieved.is_some(),
        "标记处理后仍然应该能通过 ID 查询到事件"
    );

    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, event.id, "事件 ID 应该匹配");
}

/// 测试多个事件的标记处理
///
/// 场景：创建多个事件，标记部分为已处理
/// 预期：只有未处理的事件出现在列表中
#[tokio::test]
async fn test_multiple_events_partial_processing() {
    let event_log = SqliteEventLog::new_in_memory().unwrap();

    // 创建多个事件
    let event1 = create_test_event("事件 1");
    let event2 = create_test_event("事件 2");
    let event3 = create_test_event("事件 3");

    event_log.append(&event1).await.unwrap();
    event_log.append(&event2).await.unwrap();
    event_log.append(&event3).await.unwrap();

    // 标记第一个事件为已处理
    event_log.mark_processed(&event1.id).await.unwrap();

    // 查询未处理事件
    let unprocessed = event_log.get_unprocessed(None).await.unwrap();

    // 验证：只有 2 个未处理事件
    assert_eq!(unprocessed.len(), 2, "应该有 2 个未处理事件");

    // 验证：未处理事件是 event2 和 event3
    let unprocessed_ids: Vec<String> = unprocessed.iter().map(|e| e.id.clone()).collect();
    assert!(
        unprocessed_ids.contains(&event2.id),
        "event2 应该在未处理列表中"
    );
    assert!(
        unprocessed_ids.contains(&event3.id),
        "event3 应该在未处理列表中"
    );
    assert!(
        !unprocessed_ids.contains(&event1.id),
        "event1 不应该在未处理列表中"
    );
}

/// 测试标记不存在的事件
///
/// 场景：标记一个不存在的事件 ID
/// 预期：返回错误
#[tokio::test]
async fn test_mark_nonexistent_event() {
    let event_log = SqliteEventLog::new_in_memory().unwrap();

    // 尝试标记不存在的事件
    let result = event_log.mark_processed("nonexistent-id").await;

    // 验证：返回错误
    assert!(
        result.is_err(),
        "标记不存在的事件应该返回错误"
    );
}

/// 测试标记处理后的事件状态持久化
///
/// 场景：标记处理后，重新查询
/// 预期：状态保持为已处理
#[tokio::test]
async fn test_processed_state_persistence() {
    let event_log = SqliteEventLog::new_in_memory().unwrap();

    let event = create_test_event("持久化测试");
    event_log.append(&event).await.unwrap();

    // 标记为已处理
    event_log.mark_processed(&event.id).await.unwrap();

    // 多次查询未处理列表
    for _ in 0..3 {
        let unprocessed = event_log.get_unprocessed(None).await.unwrap();
        assert!(
            unprocessed.is_empty(),
            "已处理事件不应该出现在未处理列表中"
        );
    }
}

/// 测试所有事件标记处理后列表为空
///
/// 场景：将所有事件标记为已处理
/// 预期：未处理列表为空
#[tokio::test]
async fn test_all_events_processed() {
    let event_log = SqliteEventLog::new_in_memory().unwrap();

    // 创建多个事件
    let events = vec![
        create_test_event("事件 A"),
        create_test_event("事件 B"),
        create_test_event("事件 C"),
    ];

    for event in &events {
        event_log.append(event).await.unwrap();
    }

    // 标记所有事件为已处理
    for event in &events {
        event_log.mark_processed(&event.id).await.unwrap();
    }

    // 验证：未处理列表为空
    let unprocessed = event_log.get_unprocessed(None).await.unwrap();
    assert!(
        unprocessed.is_empty(),
        "所有事件处理后未处理列表应该为空"
    );

    // 验证：所有事件仍可通过 ID 查询
    for event in &events {
        let retrieved = event_log.get(&event.id).await.unwrap();
        assert!(
            retrieved.is_some(),
            "已处理事件应该仍可通过 ID 查询到"
        );
    }
}
