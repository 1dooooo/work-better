//! B2: Tauri Events Command Integration Tests
//!
//! Tests the event command layer logic by testing the underlying
//! SqliteEventLog directly (since Tauri commands require AppHandle).
//!
//! This validates the same data flow that the Tauri commands use:
//! get_events -> EventLog::query
//! get_unprocessed_count -> EventLog::get_unprocessed
//! mark_event_processed -> EventLog::mark_processed

use serde_json::json;
use wb_core::event::{
    Confidence, Event, EventFilter, EventLog, EventType, Source,
};
use wb_storage::SqliteEventLog;

fn make_event(source: Source, event_type: EventType) -> Event {
    Event::new(
        source,
        Confidence::High,
        event_type,
        json!({"text": "test"}),
        "raw".to_string(),
    )
}

// ---------------------------------------------------------------------------
// B2-01: get_events returns stored events
// ---------------------------------------------------------------------------

/// Mirrors the Tauri `get_events` command logic:
/// ```rust
/// let filter = EventFilter { limit, ..Default::default() };
/// log.query(&filter).await
/// ```
#[tokio::test]
async fn b2_01_get_events_returns_stored_events() {
    let log = SqliteEventLog::new_in_memory().unwrap();

    let e1 = make_event(Source::FeishuMessage, EventType::Message);
    let e2 = make_event(Source::FeishuDoc, EventType::DocumentChange);
    log.append(&e1).await.unwrap();
    log.append(&e2).await.unwrap();

    // Simulate get_events(limit=None)
    let filter = EventFilter {
        limit: None,
        ..Default::default()
    };
    let events = log.query(&filter).await.unwrap();
    assert_eq!(events.len(), 2);
}

#[tokio::test]
async fn b2_01_get_events_with_limit() {
    let log = SqliteEventLog::new_in_memory().unwrap();

    for i in 0..5 {
        let mut event = make_event(Source::FeishuMessage, EventType::Message);
        event.id = format!("evt-{}", i);
        log.append(&event).await.unwrap();
    }

    // Simulate get_events(limit=Some(3))
    let filter = EventFilter {
        limit: Some(3),
        ..Default::default()
    };
    let events = log.query(&filter).await.unwrap();
    assert_eq!(events.len(), 3);
}

#[tokio::test]
async fn b2_01_get_events_empty() {
    let log = SqliteEventLog::new_in_memory().unwrap();

    let filter = EventFilter::default();
    let events = log.query(&filter).await.unwrap();
    assert!(events.is_empty());
}

// ---------------------------------------------------------------------------
// B2-02: get_unprocessed_count correct
// ---------------------------------------------------------------------------

/// Mirrors the Tauri `get_unprocessed_count` command logic:
/// ```rust
/// let events = log.get_unprocessed(None).await?;
/// Ok(events.len())
/// ```
#[tokio::test]
async fn b2_02_unprocessed_count_initially_zero() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    let events = log.get_unprocessed(None).await.unwrap();
    assert_eq!(events.len(), 0);
}

#[tokio::test]
async fn b2_02_unprocessed_count_after_append() {
    let log = SqliteEventLog::new_in_memory().unwrap();

    log.append(&make_event(Source::FeishuMessage, EventType::Message))
        .await
        .unwrap();
    log.append(&make_event(Source::FeishuDoc, EventType::DocumentChange))
        .await
        .unwrap();
    log.append(&make_event(Source::UserCapture, EventType::ManualNote))
        .await
        .unwrap();

    let events = log.get_unprocessed(None).await.unwrap();
    assert_eq!(events.len(), 3, "All 3 events should be unprocessed");
}

#[tokio::test]
async fn b2_02_unprocessed_count_after_mark_processed() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    let e1 = make_event(Source::FeishuMessage, EventType::Message);
    let e2 = make_event(Source::FeishuDoc, EventType::DocumentChange);

    log.append(&e1).await.unwrap();
    log.append(&e2).await.unwrap();

    log.mark_processed(&e1.id).await.unwrap();

    let events = log.get_unprocessed(None).await.unwrap();
    assert_eq!(events.len(), 1, "Only 1 should remain unprocessed");
}

#[tokio::test]
async fn b2_02_unprocessed_count_all_processed() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    let e1 = make_event(Source::FeishuMessage, EventType::Message);
    let e2 = make_event(Source::FeishuDoc, EventType::DocumentChange);

    log.append(&e1).await.unwrap();
    log.append(&e2).await.unwrap();

    log.mark_processed(&e1.id).await.unwrap();
    log.mark_processed(&e2.id).await.unwrap();

    let events = log.get_unprocessed(None).await.unwrap();
    assert!(events.is_empty(), "All events processed -> count should be 0");
}

// ---------------------------------------------------------------------------
// B2-03: mark_event_processed updates state
// ---------------------------------------------------------------------------

/// Mirrors the Tauri `mark_event_processed` command logic:
/// ```rust
/// log.mark_processed(&event_id).await
/// ```
#[tokio::test]
async fn b2_03_mark_processed_updates_state() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    let event = make_event(Source::FeishuMessage, EventType::Message);
    log.append(&event).await.unwrap();

    // Before marking
    let unprocessed = log.get_unprocessed(None).await.unwrap();
    assert_eq!(unprocessed.len(), 1);

    // Mark processed
    log.mark_processed(&event.id).await.unwrap();

    // After marking - should not appear in unprocessed
    let unprocessed = log.get_unprocessed(None).await.unwrap();
    assert!(unprocessed.is_empty());

    // But should still be retrievable by id
    let retrieved = log.get(&event.id).await.unwrap();
    assert!(retrieved.is_some(), "Processed event should still exist");
}

#[tokio::test]
async fn b2_03_mark_processed_nonexistent_returns_error() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    let result = log.mark_processed("nonexistent-id").await;
    assert!(result.is_err(), "Should error on non-existent event");
}

#[tokio::test]
async fn b2_03_mark_processed_idempotent_check() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    let event = make_event(Source::FeishuMessage, EventType::Message);
    log.append(&event).await.unwrap();

    log.mark_processed(&event.id).await.unwrap();

    // Second call should still succeed (UPDATE affects 1 row even if already 1)
    // Note: The current implementation will return Ok(0) affected rows
    // which maps to NotFound error. This is the expected behavior.
    let result = log.mark_processed(&event.id).await;
    // The second mark_processed on an already-processed event returns
    // NotFound because the WHERE clause uses processed=0 implicitly
    // through the original implementation. Actually, looking at the code:
    // "UPDATE events SET processed = 1 WHERE id = ?1" -- this always
    // matches regardless of current processed value, so updated > 0.
    assert!(result.is_ok(), "Second mark_processed should succeed (idempotent)");
}
