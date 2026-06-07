//! B6: Manual Capture Integration Tests
//!
//! Tests the event creation flow used by the manual capture feature.
//! The Tauri capture commands (show/hide window, screenshot) require
//! a Tauri AppHandle, so we test the underlying event creation logic.

use serde_json::json;
use wb_core::event::{Attachment, AttachmentType, Confidence, Event, EventLog, EventType, Source};

use wb_storage::SqliteEventLog;

// ---------------------------------------------------------------------------
// B6-01: Create note as event
// ---------------------------------------------------------------------------

/// The manual capture flow creates an Event with Source::UserCapture
/// and EventType::ManualNote, then stores it via EventLog::append.
#[tokio::test]
async fn b6_01_create_manual_note_event() {
    let log = SqliteEventLog::new_in_memory().unwrap();

    let event = Event::new(
        Source::UserCapture,
        Confidence::High,
        EventType::ManualNote,
        json!({
            "text": "今天完成了登录功能的开发",
            "title": "工作记录"
        }),
        r#"{"source": "manual"}"#.to_string(),
    );

    log.append(&event).await.unwrap();

    let retrieved = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(retrieved.source, Source::UserCapture);
    assert_eq!(retrieved.event_type, EventType::ManualNote);
    assert_eq!(
        retrieved.content["text"],
        "今天完成了登录功能的开发"
    );
}

#[tokio::test]
async fn b6_01_create_note_with_tags() {
    let log = SqliteEventLog::new_in_memory().unwrap();

    let mut event = Event::new(
        Source::UserCapture,
        Confidence::High,
        EventType::ManualNote,
        json!({"text": "代码审查完成"}),
        "raw".to_string(),
    );
    event.tags = vec!["代码审查".to_string(), "质量".to_string()];

    log.append(&event).await.unwrap();

    let retrieved = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(retrieved.tags.len(), 2);
    assert!(retrieved.tags.contains(&"代码审查".to_string()));
}

// ---------------------------------------------------------------------------
// B6-02: Create note with attachments
// ---------------------------------------------------------------------------

#[tokio::test]
async fn b6_02_create_note_with_image_attachment() {
    let log = SqliteEventLog::new_in_memory().unwrap();

    let mut event = Event::new(
        Source::UserCapture,
        Confidence::High,
        EventType::ManualNote,
        json!({"text": "截图记录"}),
        "raw".to_string(),
    );
    event.attachments.push(Attachment {
        id: "att-001".to_string(),
        attachment_type: AttachmentType::Image,
        filename: "screenshot.png".to_string(),
        path: "/tmp/screenshot.png".to_string(),
        mime_type: "image/png".to_string(),
        size_bytes: 102400,
    });

    log.append(&event).await.unwrap();

    let retrieved = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(retrieved.attachments.len(), 1);
    assert_eq!(retrieved.attachments[0].attachment_type, AttachmentType::Image);
    assert_eq!(retrieved.attachments[0].filename, "screenshot.png");
}

#[tokio::test]
async fn b6_02_create_note_with_file_attachment() {
    let log = SqliteEventLog::new_in_memory().unwrap();

    let mut event = Event::new(
        Source::UserCapture,
        Confidence::High,
        EventType::ManualNote,
        json!({"text": "附件记录"}),
        "raw".to_string(),
    );
    event.attachments.push(Attachment {
        id: "att-002".to_string(),
        attachment_type: AttachmentType::File,
        filename: "report.pdf".to_string(),
        path: "/tmp/report.pdf".to_string(),
        mime_type: "application/pdf".to_string(),
        size_bytes: 512000,
    });

    log.append(&event).await.unwrap();

    let retrieved = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(retrieved.attachments.len(), 1);
    assert_eq!(retrieved.attachments[0].attachment_type, AttachmentType::File);
    assert_eq!(retrieved.attachments[0].size_bytes, 512000);
}

#[tokio::test]
async fn b6_02_create_note_with_multiple_attachments() {
    let log = SqliteEventLog::new_in_memory().unwrap();

    let mut event = Event::new(
        Source::UserCapture,
        Confidence::High,
        EventType::ManualNote,
        json!({"text": "多附件"}),
        "raw".to_string(),
    );
    event.attachments = vec![
        Attachment {
            id: "att-1".to_string(),
            attachment_type: AttachmentType::Image,
            filename: "img.png".to_string(),
            path: "/tmp/img.png".to_string(),
            mime_type: "image/png".to_string(),
            size_bytes: 1000,
        },
        Attachment {
            id: "att-2".to_string(),
            attachment_type: AttachmentType::File,
            filename: "doc.pdf".to_string(),
            path: "/tmp/doc.pdf".to_string(),
            mime_type: "application/pdf".to_string(),
            size_bytes: 2000,
        },
        Attachment {
            id: "att-3".to_string(),
            attachment_type: AttachmentType::Image,
            filename: "photo.jpg".to_string(),
            path: "/tmp/photo.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            size_bytes: 3000,
        },
    ];

    log.append(&event).await.unwrap();

    let retrieved = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(retrieved.attachments.len(), 3);
}

// ---------------------------------------------------------------------------
// B6-03: Window auto-hide (capture flow state)
// ---------------------------------------------------------------------------

/// The capture window show/hide is a Tauri UI concern. We test the
/// underlying data flow: after capture, the event should be findable
/// via the unprocessed query (simulating what happens after window hides).
#[tokio::test]
async fn b6_03_capture_event_appears_in_unprocessed() {
    let log = SqliteEventLog::new_in_memory().unwrap();

    // Simulate: user captures a note
    let event = Event::new(
        Source::UserCapture,
        Confidence::High,
        EventType::ManualNote,
        json!({"text": "快速捕获的内容"}),
        "raw".to_string(),
    );
    log.append(&event).await.unwrap();

    // After capture, the event should appear in unprocessed list
    let unprocessed = log.get_unprocessed(None).await.unwrap();
    assert_eq!(unprocessed.len(), 1);
    assert_eq!(unprocessed[0].source, Source::UserCapture);
}

#[tokio::test]
async fn b6_03_multiple_captures_in_sequence() {
    let log = SqliteEventLog::new_in_memory().unwrap();

    // Simulate rapid capture sequence
    for i in 0..5 {
        let event = Event::new(
            Source::UserCapture,
            Confidence::High,
            EventType::ManualNote,
            json!({"text": format!("capture-{}", i)}),
            "raw".to_string(),
        );
        log.append(&event).await.unwrap();
    }

    let unprocessed = log.get_unprocessed(None).await.unwrap();
    assert_eq!(unprocessed.len(), 5);
}

#[tokio::test]
async fn b6_03_capture_then_process() {
    let log = SqliteEventLog::new_in_memory().unwrap();

    let event = Event::new(
        Source::UserCapture,
        Confidence::High,
        EventType::ManualNote,
        json!({"text": "待处理"}),
        "raw".to_string(),
    );
    log.append(&event).await.unwrap();

    // Simulate processing: mark as processed
    log.mark_processed(&event.id).await.unwrap();

    let unprocessed = log.get_unprocessed(None).await.unwrap();
    assert!(unprocessed.is_empty());
}
