//! B1: SQLite EventLog Integration Tests
//!
//! Tests the EventLog storage layer against in-memory SQLite.
//! Each test is self-contained: setup + execute + assert.

use std::sync::Arc;

use chrono::{TimeZone, Utc};
use serde_json::json;
use tokio::task::JoinHandle;
use wb_core::event::{
    Attachment, AttachmentType, Confidence, Event, EventFilter, EventLog, EventType, Source,
};
use wb_storage::SqliteEventLog;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_event(source: Source, event_type: EventType) -> Event {
    Event::new(
        source,
        Confidence::High,
        event_type,
        json!({"text": "test content"}),
        r#"{"raw": "data"}"#.to_string(),
    )
}

fn make_full_event() -> Event {
    Event {
        id: "full-event-001".to_string(),
        timestamp: Utc.with_ymd_and_hms(2026, 6, 6, 10, 30, 0).unwrap(),
        collected_at: Utc.with_ymd_and_hms(2026, 6, 6, 10, 30, 1).unwrap(),
        source: Source::FeishuMessage,
        source_confidence: Confidence::High,
        event_type: EventType::Message,
        content: json!({
            "text": "Hello world",
            "sender": "user-001",
            "chat_id": "oc_test123",
            "nested": {"key": "value", "number": 42}
        }),
        raw_payload: r#"{"message_id":"msg-001","text":"Hello world"}"#.to_string(),
        tags: vec![
            "meeting".to_string(),
            "product".to_string(),
            "q3-planning".to_string(),
        ],
        related_ids: vec!["evt-aaa".to_string(), "evt-bbb".to_string()],
        attachments: vec![
            Attachment {
                id: "att-001".to_string(),
                attachment_type: AttachmentType::Image,
                filename: "screenshot.png".to_string(),
                path: "/tmp/screenshot.png".to_string(),
                mime_type: "image/png".to_string(),
                size_bytes: 102400,
            },
            Attachment {
                id: "att-002".to_string(),
                attachment_type: AttachmentType::File,
                filename: "report.pdf".to_string(),
                path: "/tmp/report.pdf".to_string(),
                mime_type: "application/pdf".to_string(),
                size_bytes: 512000,
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// B1-01: Append event
// ---------------------------------------------------------------------------

#[tokio::test]
async fn b1_01_append_event() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    let event = make_event(Source::FeishuMessage, EventType::Message);

    log.append(&event).await.unwrap();

    // Verify the event was stored by retrieving it
    let retrieved = log.get(&event.id).await.unwrap();
    assert!(retrieved.is_some(), "Event should be retrievable after append");
    assert_eq!(retrieved.unwrap().id, event.id);
}

#[tokio::test]
async fn b1_01_append_multiple_events() {
    let log = SqliteEventLog::new_in_memory().unwrap();

    let e1 = make_event(Source::FeishuMessage, EventType::Message);
    let e2 = make_event(Source::FeishuDoc, EventType::DocumentChange);
    let e3 = make_event(Source::UserCapture, EventType::ManualNote);

    log.append(&e1).await.unwrap();
    log.append(&e2).await.unwrap();
    log.append(&e3).await.unwrap();

    let all = log.query(&EventFilter::default()).await.unwrap();
    assert_eq!(all.len(), 3, "Should have 3 events after appending 3");
}

#[tokio::test]
async fn b1_01_append_idempotent() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    let event = make_event(Source::FeishuMessage, EventType::Message);

    // INSERT OR IGNORE means duplicate id is silently ignored
    log.append(&event).await.unwrap();
    log.append(&event).await.unwrap();

    let all = log.query(&EventFilter::default()).await.unwrap();
    assert_eq!(all.len(), 1, "Duplicate append should not create duplicate rows");
}

// ---------------------------------------------------------------------------
// B1-02: Retrieve by id
// ---------------------------------------------------------------------------

#[tokio::test]
async fn b1_02_retrieve_by_id_found() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    let event = make_event(Source::FeishuDoc, EventType::DocumentChange);

    log.append(&event).await.unwrap();
    let retrieved = log.get(&event.id).await.unwrap().unwrap();

    assert_eq!(retrieved.id, event.id);
    assert_eq!(retrieved.source, Source::FeishuDoc);
    assert_eq!(retrieved.event_type, EventType::DocumentChange);
    assert_eq!(retrieved.content, json!({"text": "test content"}));
    assert_eq!(retrieved.raw_payload, r#"{"raw": "data"}"#);
}

#[tokio::test]
async fn b1_02_retrieve_by_id_not_found() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    let result = log.get("nonexistent-id").await.unwrap();
    assert!(result.is_none(), "Non-existent id should return None");
}

#[tokio::test]
async fn b1_02_retrieve_preserves_timestamps() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    let event = make_event(Source::FeishuMessage, EventType::Message);
    let original_ts = event.timestamp;

    log.append(&event).await.unwrap();
    let retrieved = log.get(&event.id).await.unwrap().unwrap();

    // Timestamps should be preserved (within RFC3339 precision)
    let diff = (retrieved.timestamp - original_ts).num_seconds().abs();
    assert!(diff <= 1, "Timestamp should be preserved within 1 second");
}

// ---------------------------------------------------------------------------
// B1-03: Mark processed
// ---------------------------------------------------------------------------

#[tokio::test]
async fn b1_03_mark_processed() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    let event = make_event(Source::FeishuMessage, EventType::Message);

    log.append(&event).await.unwrap();

    // Initially unprocessed
    let unprocessed = log.get_unprocessed(None).await.unwrap();
    assert_eq!(unprocessed.len(), 1);

    // Mark processed
    log.mark_processed(&event.id).await.unwrap();

    // Now should be empty
    let unprocessed = log.get_unprocessed(None).await.unwrap();
    assert!(unprocessed.is_empty(), "No unprocessed events after marking");
}

#[tokio::test]
async fn b1_03_mark_processed_nonexistent() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    let result = log.mark_processed("nonexistent").await;
    assert!(result.is_err(), "Marking non-existent event should return error");
}

#[tokio::test]
async fn b1_03_mark_processed_selective() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    let e1 = make_event(Source::FeishuMessage, EventType::Message);
    let e2 = make_event(Source::FeishuDoc, EventType::DocumentChange);

    log.append(&e1).await.unwrap();
    log.append(&e2).await.unwrap();

    log.mark_processed(&e1.id).await.unwrap();

    let unprocessed = log.get_unprocessed(None).await.unwrap();
    assert_eq!(unprocessed.len(), 1);
    assert_eq!(unprocessed[0].id, e2.id);
}

// ---------------------------------------------------------------------------
// B1-04: Filter by type
// ---------------------------------------------------------------------------

#[tokio::test]
async fn b1_04_filter_by_event_type() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    log.append(&make_event(Source::FeishuMessage, EventType::Message))
        .await
        .unwrap();
    log.append(&make_event(Source::FeishuDoc, EventType::DocumentChange))
        .await
        .unwrap();
    log.append(&make_event(Source::FeishuMessage, EventType::Message))
        .await
        .unwrap();

    let filter = EventFilter {
        event_type: Some(EventType::Message),
        ..Default::default()
    };
    let results = log.query(&filter).await.unwrap();
    assert_eq!(results.len(), 2, "Should find 2 Message events");
    assert!(results.iter().all(|e| e.event_type == EventType::Message));
}

#[tokio::test]
async fn b1_04_filter_by_source() {
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

    let filter = EventFilter {
        source: Some(Source::FeishuDoc),
        ..Default::default()
    };
    let results = log.query(&filter).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].source, Source::FeishuDoc);
}

#[tokio::test]
async fn b1_04_filter_by_processed_status() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    let e1 = make_event(Source::FeishuMessage, EventType::Message);
    let e2 = make_event(Source::FeishuDoc, EventType::DocumentChange);

    log.append(&e1).await.unwrap();
    log.append(&e2).await.unwrap();
    log.mark_processed(&e1.id).await.unwrap();

    let filter = EventFilter {
        processed: Some(true),
        ..Default::default()
    };
    let results = log.query(&filter).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, e1.id);
}

#[tokio::test]
async fn b1_04_filter_combined_source_and_type() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    log.append(&make_event(Source::FeishuMessage, EventType::Message))
        .await
        .unwrap();
    log.append(&make_event(Source::FeishuMessage, EventType::DocumentChange))
        .await
        .unwrap();
    log.append(&make_event(Source::FeishuDoc, EventType::Message))
        .await
        .unwrap();

    let filter = EventFilter {
        source: Some(Source::FeishuMessage),
        event_type: Some(EventType::Message),
        ..Default::default()
    };
    let results = log.query(&filter).await.unwrap();
    assert_eq!(results.len(), 1);
}

#[tokio::test]
async fn b1_04_filter_no_match() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    log.append(&make_event(Source::FeishuMessage, EventType::Message))
        .await
        .unwrap();

    let filter = EventFilter {
        source: Some(Source::FeishuCalendar),
        ..Default::default()
    };
    let results = log.query(&filter).await.unwrap();
    assert!(results.is_empty());
}

// ---------------------------------------------------------------------------
// B1-05: Query with pagination
// ---------------------------------------------------------------------------

#[tokio::test]
async fn b1_05_query_with_limit() {
    let log = SqliteEventLog::new_in_memory().unwrap();

    for i in 0..10 {
        let mut event = make_event(Source::FeishuMessage, EventType::Message);
        event.id = format!("evt-{:03}", i);
        log.append(&event).await.unwrap();
    }

    let filter = EventFilter {
        limit: Some(3),
        ..Default::default()
    };
    let results = log.query(&filter).await.unwrap();
    assert_eq!(results.len(), 3, "Limit 3 should return exactly 3 results");
}

#[tokio::test]
async fn b1_05_query_with_time_range() {
    let log = SqliteEventLog::new_in_memory().unwrap();

    // Event from 2026-01-01
    let mut old_event = make_event(Source::FeishuMessage, EventType::Message);
    old_event.id = "old-event".to_string();
    old_event.timestamp = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    log.append(&old_event).await.unwrap();

    // Event from 2026-06-06
    let mut new_event = make_event(Source::FeishuMessage, EventType::Message);
    new_event.id = "new-event".to_string();
    new_event.timestamp = Utc.with_ymd_and_hms(2026, 6, 6, 0, 0, 0).unwrap();
    log.append(&new_event).await.unwrap();

    // Query since March
    let filter = EventFilter {
        since: Some(Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap()),
        ..Default::default()
    };
    let results = log.query(&filter).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "new-event");
}

#[tokio::test]
async fn b1_05_query_ordered_by_timestamp_desc() {
    let log = SqliteEventLog::new_in_memory().unwrap();

    let mut e1 = make_event(Source::FeishuMessage, EventType::Message);
    e1.id = "first".to_string();
    e1.timestamp = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    log.append(&e1).await.unwrap();

    let mut e2 = make_event(Source::FeishuMessage, EventType::Message);
    e2.id = "second".to_string();
    e2.timestamp = Utc.with_ymd_and_hms(2026, 6, 6, 0, 0, 0).unwrap();
    log.append(&e2).await.unwrap();

    let results = log.query(&EventFilter::default()).await.unwrap();
    assert_eq!(results.len(), 2);
    // Results should be ordered by timestamp DESC
    assert_eq!(results[0].id, "second");
    assert_eq!(results[1].id, "first");
}

#[tokio::test]
async fn b1_05_query_empty_result() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    let results = log.query(&EventFilter::default()).await.unwrap();
    assert!(results.is_empty());
}

// ---------------------------------------------------------------------------
// B1-06: Round-trip serialization
// ---------------------------------------------------------------------------

#[tokio::test]
async fn b1_06_roundtrip_all_fields() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    let event = make_full_event();

    log.append(&event).await.unwrap();
    let retrieved = log.get(&event.id).await.unwrap().unwrap();

    assert_eq!(retrieved.id, event.id);
    assert_eq!(retrieved.source, event.source);
    assert_eq!(retrieved.source_confidence, event.source_confidence);
    assert_eq!(retrieved.event_type, event.event_type);
    assert_eq!(retrieved.content, event.content);
    assert_eq!(retrieved.raw_payload, event.raw_payload);
    assert_eq!(retrieved.tags, event.tags);
    assert_eq!(retrieved.related_ids, event.related_ids);
    assert_eq!(retrieved.attachments, event.attachments);
}

#[tokio::test]
async fn b1_06_roundtrip_empty_collections() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    let mut event = make_event(Source::FeishuMessage, EventType::Message);
    event.tags = vec![];
    event.related_ids = vec![];
    event.attachments = vec![];

    log.append(&event).await.unwrap();
    let retrieved = log.get(&event.id).await.unwrap().unwrap();

    assert!(retrieved.tags.is_empty());
    assert!(retrieved.related_ids.is_empty());
    assert!(retrieved.attachments.is_empty());
}

#[tokio::test]
async fn b1_06_roundtrip_complex_json_content() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    let mut event = make_event(Source::FeishuMessage, EventType::Message);
    event.content = json!({
        "nested": {"deep": {"value": 42}},
        "array": [1, 2, 3],
        "unicode": "你好世界",
        "special": "line\nbreak\ttab"
    });

    log.append(&event).await.unwrap();
    let retrieved = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(retrieved.content, event.content);
}

// ---------------------------------------------------------------------------
// B1-07: Concurrent writes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn b1_07_concurrent_writes() {
    let log = Arc::new(SqliteEventLog::new_in_memory().unwrap());
    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    for i in 0..20 {
        let log_clone = Arc::clone(&log);
        let handle = tokio::spawn(async move {
            let mut event = make_event(Source::FeishuMessage, EventType::Message);
            event.id = format!("concurrent-{:03}", i);
            log_clone.append(&event).await.unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let all = log.query(&EventFilter::default()).await.unwrap();
    assert_eq!(all.len(), 20, "All 20 concurrent writes should succeed");
}

#[tokio::test]
async fn b1_07_concurrent_read_write() {
    let log = Arc::new(SqliteEventLog::new_in_memory().unwrap());

    // Pre-populate some data
    for i in 0..5 {
        let mut event = make_event(Source::FeishuMessage, EventType::Message);
        event.id = format!("pre-{:03}", i);
        log.append(&event).await.unwrap();
    }

    let mut handles: Vec<JoinHandle<usize>> = Vec::new();

    // Spawn concurrent readers
    for _ in 0..5 {
        let log_clone = Arc::clone(&log);
        let handle = tokio::spawn(async move {
            let results = log_clone.query(&EventFilter::default()).await.unwrap();
            results.len()
        });
        handles.push(handle);
    }

    // Spawn concurrent writers
    for i in 0..5 {
        let log_clone = Arc::clone(&log);
        let handle = tokio::spawn(async move {
            let mut event = make_event(Source::FeishuDoc, EventType::DocumentChange);
            event.id = format!("write-{:03}", i);
            log_clone.append(&event).await.unwrap();
            0
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let final_count = log.query(&EventFilter::default()).await.unwrap().len();
    assert_eq!(final_count, 10, "Should have 5 pre-existing + 5 new events");
}

// ---------------------------------------------------------------------------
// B1-08: Event with all field types
// ---------------------------------------------------------------------------

#[tokio::test]
async fn b1_08_all_source_variants() {
    let log = SqliteEventLog::new_in_memory().unwrap();

    let sources = vec![
        Source::FeishuMessage,
        Source::FeishuDoc,
        Source::FeishuProject,
        Source::FeishuCalendar,
        Source::FeishuMeeting,
        Source::FeishuEmail,
        Source::FeishuApproval,
        Source::FeishuOkr,
        Source::FeishuBitable,
        Source::FeishuSheet,
        Source::FeishuWiki,
        Source::SystemAppSwitch,
        Source::SystemBrowser,
        Source::UserCapture,
    ];

    for (i, source) in sources.iter().enumerate() {
        let mut event = make_event(source.clone(), EventType::Message);
        event.id = format!("src-{:03}", i);
        log.append(&event).await.unwrap();
    }

    let all = log.query(&EventFilter::default()).await.unwrap();
    assert_eq!(all.len(), 14, "All 14 source variants should be stored");

    // Verify each source round-trips correctly
    for source in &sources {
        let filter = EventFilter {
            source: Some(source.clone()),
            ..Default::default()
        };
        let results = log.query(&filter).await.unwrap();
        assert_eq!(results.len(), 1, "Each source should have exactly 1 event");
        assert_eq!(results[0].source, *source);
    }
}

#[tokio::test]
async fn b1_08_all_event_type_variants() {
    let log = SqliteEventLog::new_in_memory().unwrap();

    let types = vec![
        EventType::Message,
        EventType::DocumentChange,
        EventType::TaskUpdate,
        EventType::Meeting,
        EventType::CalendarEvent,
        EventType::Email,
        EventType::Approval,
        EventType::OkrUpdate,
        EventType::Browsing,
        EventType::AppActivity,
        EventType::ManualNote,
    ];

    for (i, etype) in types.iter().enumerate() {
        let mut event = make_event(Source::UserCapture, etype.clone());
        event.id = format!("type-{:03}", i);
        log.append(&event).await.unwrap();
    }

    let all = log.query(&EventFilter::default()).await.unwrap();
    assert_eq!(all.len(), 11, "All 11 event type variants should be stored");
}

#[tokio::test]
async fn b1_08_confidence_levels() {
    let log = SqliteEventLog::new_in_memory().unwrap();

    let confidences = [Confidence::High, Confidence::Medium, Confidence::Low];

    for (i, conf) in confidences.iter().enumerate() {
        let event = Event::new(
            Source::FeishuMessage,
            conf.clone(),
            EventType::Message,
            json!({"text": "test"}),
            "raw".to_string(),
        );
        let mut event = event;
        event.id = format!("conf-{:03}", i);
        log.append(&event).await.unwrap();
    }

    let all = log.query(&EventFilter::default()).await.unwrap();
    assert_eq!(all.len(), 3);

    // Verify confidence round-trips
    let high = log.get("conf-000").await.unwrap().unwrap();
    assert_eq!(high.source_confidence, Confidence::High);

    let medium = log.get("conf-001").await.unwrap().unwrap();
    assert_eq!(medium.source_confidence, Confidence::Medium);

    let low = log.get("conf-002").await.unwrap().unwrap();
    assert_eq!(low.source_confidence, Confidence::Low);
}

#[tokio::test]
async fn b1_08_attachment_types() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    let event = make_full_event();

    log.append(&event).await.unwrap();
    let retrieved = log.get(&event.id).await.unwrap().unwrap();

    assert_eq!(retrieved.attachments.len(), 2);
    assert_eq!(retrieved.attachments[0].attachment_type, AttachmentType::Image);
    assert_eq!(retrieved.attachments[1].attachment_type, AttachmentType::File);
    assert_eq!(retrieved.attachments[0].size_bytes, 102400);
    assert_eq!(retrieved.attachments[1].mime_type, "application/pdf");
}

#[tokio::test]
async fn b1_08_unicode_content() {
    let log = SqliteEventLog::new_in_memory().unwrap();
    let mut event = make_event(Source::FeishuMessage, EventType::Message);
    event.content = json!({"text": "你好世界 🌍 Emoji 测试"});
    event.tags = vec!["会议".to_string(), "产品规划".to_string()];

    log.append(&event).await.unwrap();
    let retrieved = log.get(&event.id).await.unwrap().unwrap();

    assert_eq!(retrieved.content, json!({"text": "你好世界 🌍 Emoji 测试"}));
    assert_eq!(retrieved.tags, vec!["会议".to_string(), "产品规划".to_string()]);
}
