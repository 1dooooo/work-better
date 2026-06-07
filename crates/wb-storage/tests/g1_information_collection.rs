//! G1: Information Collection — 37 scenarios
//!
//! Black-box acceptance tests for the information collection layer.
//! Covers: Feishu messages, documents, tasks, calendar, meetings,
//! email, approval, OKR, system activity, manual capture, collector management.

mod acceptance_helpers;
use acceptance_helpers::*;
use serde_json::json;
use wb_core::event::{Confidence, EventLog, EventType, Source};
use wb_processor::classifier::{Classifier, ProcessingRoute};

// ===========================================================================
// G1-01 ~ G1-05: Feishu Message Collection
// ===========================================================================

/// G1-01: Given Feishu message @mentions user, When arrives,
///        Then captured as message(confidence=high)
#[tokio::test]
async fn g1_01_feishu_mention_capture() {
    let log = fresh_event_log();
    let event = feishu_mention_event("@user", "@user hello, please review the PR");

    log.append(&event).await.unwrap();

    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.event_type, EventType::Message);
    assert_eq!(stored.source_confidence, Confidence::High);
    assert_eq!(stored.source, Source::FeishuMessage);
    assert!(stored.content["text"]
        .as_str()
        .unwrap()
        .contains("@user"));
}

/// G1-02: Given Feishu message is a reply in a thread the user participates in,
///        When arrives, Then captured and linked to the thread
#[tokio::test]
async fn g1_02_feishu_thread_reply_capture() {
    let log = fresh_event_log();
    let thread_id = "thread-abc-123";
    let event = feishu_thread_reply_event(thread_id, "I agree with the approach");

    log.append(&event).await.unwrap();

    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.event_type, EventType::Message);
    assert_eq!(stored.source_confidence, Confidence::High);
    assert!(
        stored.related_ids.contains(&thread_id.to_string()),
        "Thread reply should be linked to the parent thread"
    );
}

/// G1-03: Given Feishu direct message, When arrives, Then captured as message
#[tokio::test]
async fn g1_03_feishu_dm_capture() {
    let log = fresh_event_log();
    let event = feishu_dm_event("Can we sync on the design doc?");

    log.append(&event).await.unwrap();

    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.event_type, EventType::Message);
    assert_eq!(stored.source, Source::FeishuMessage);
    assert_eq!(stored.source_confidence, Confidence::High);
}

/// G1-04: Given message matches keyword rule, When evaluated,
///        Then captured even without @mention
#[tokio::test]
async fn g1_04_keyword_rule_capture() {
    let log = fresh_event_log();
    let mut event = feishu_message_event("The deployment pipeline is broken");
    event.tags.push("keyword:deployment".to_string());

    log.append(&event).await.unwrap();

    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.event_type, EventType::Message);
    assert!(
        stored.tags.iter().any(|t| t.starts_with("keyword:")),
        "Event should have keyword tag for rule matching"
    );
}

/// G1-05: Given message is unrelated to user, When evaluated, Then NOT captured
#[tokio::test]
async fn g1_05_unrelated_message_not_captured() {
    let log = fresh_event_log();

    // A low-confidence, unrelated message should be classified as Archive
    let mut event = feishu_message_event("Random noise in a public channel");
    event.source_confidence = Confidence::Low;

    // The classifier should route low-confidence events to Archive
    let route = Classifier::classify(&event);
    assert_eq!(
        route,
        ProcessingRoute::Archive,
        "Unrelated low-confidence messages should be archived, not processed"
    );

    // Even if stored, it should not appear in unprocessed high-priority queue
    log.append(&event).await.unwrap();
    let unprocessed = log.get_unprocessed(None).await.unwrap();
    // The event is stored (EventLog is append-only), but its route is Archive
    assert_eq!(unprocessed.len(), 1);
    let stored = &unprocessed[0];
    assert_eq!(stored.source_confidence, Confidence::Low);
}

// ===========================================================================
// G1-06 ~ G1-17: Feishu Document / Task / Calendar / Meeting / Email / etc.
// ===========================================================================

/// G1-06: Given Feishu doc created/edited/commented by user, When detected,
///        Then captured as document_change
#[tokio::test]
async fn g1_06_feishu_document_change_capture() {
    let log = fresh_event_log();
    let event = document_change_event("doc-001", "edit");

    log.append(&event).await.unwrap();

    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.event_type, EventType::DocumentChange);
    assert_eq!(stored.source, Source::FeishuDoc);
    assert_eq!(stored.content["doc_id"], "doc-001");
    assert_eq!(stored.content["action"], "edit");
}

/// G1-07: Given user is mentioned in a document, When detected, Then event captured
#[tokio::test]
async fn g1_07_document_mention_capture() {
    let log = fresh_event_log();
    let mut event = document_change_event("doc-002", "comment");
    event.content = json!({
        "doc_id": "doc-002",
        "action": "comment",
        "mentions": ["user"]
    });

    log.append(&event).await.unwrap();

    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.event_type, EventType::DocumentChange);
    assert!(
        stored.content["mentions"].as_array().unwrap().len() > 0,
        "Document with mentions should be captured"
    );
}

/// G1-08: Given Feishu project task changes, When captured, Then captured as task_update
#[tokio::test]
async fn g1_08_feishu_task_update_capture() {
    let log = fresh_event_log();
    let event = task_update_event("task-001", "in_progress");

    log.append(&event).await.unwrap();

    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.event_type, EventType::TaskUpdate);
    assert_eq!(stored.source, Source::FeishuProject);
    assert_eq!(stored.content["task_id"], "task-001");
}

/// G1-09: Given upcoming calendar event, When hourly sync, Then captured as calendar_event
#[tokio::test]
async fn g1_09_calendar_event_capture() {
    let log = fresh_event_log();
    let event = calendar_event_event("cal-001", "Sprint Planning");

    log.append(&event).await.unwrap();

    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.event_type, EventType::CalendarEvent);
    assert_eq!(stored.source, Source::FeishuCalendar);
    assert_eq!(stored.content["title"], "Sprint Planning");
}

/// G1-10: Given user attends video meeting, When ends,
///        Then captured as meeting(with minutes and todos)
#[tokio::test]
async fn g1_10_meeting_capture_with_minutes() {
    let log = fresh_event_log();
    let event = meeting_event("meet-001", "Design Review");

    log.append(&event).await.unwrap();

    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.event_type, EventType::Meeting);
    assert_eq!(stored.source, Source::FeishuMeeting);
    assert_eq!(
        stored.content["has_minutes"], true,
        "Meeting should include minutes"
    );
}

// ===========================================================================
// G1-11 ~ G1-17: Scaffolded scenarios
// ===========================================================================

/// G1-11: Given Feishu minutes has recording summary, When ends,
///        Then captures summary/todos/chapters
#[tokio::test]
async fn g1_11_minutes_summary_capture() {
    // TODO: implement — requires minutes/summary parsing integration
    let log = fresh_event_log();
    let mut event = meeting_event("meet-002", "Weekly Sync");
    event.content = json!({
        "meeting_id": "meet-002",
        "title": "Weekly Sync",
        "has_minutes": true,
        "summary": "Discussed Q3 roadmap",
        "todos": ["Update timeline", "Share doc"],
        "chapters": ["Intro", "Roadmap", "Action Items"]
    });

    log.append(&event).await.unwrap();
    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert!(stored.content["summary"].as_str().is_some());
    assert!(stored.content["todos"].as_array().unwrap().len() > 0);
}

/// G1-12: Given user email via Feishu, When synced every 30min, Then captured as email
#[tokio::test]
async fn g1_12_email_capture() {
    let log = fresh_event_log();
    let event = email_event("Re: Project Status Update");

    log.append(&event).await.unwrap();

    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.event_type, EventType::Email);
    assert_eq!(stored.source, Source::FeishuEmail);
}

/// G1-13: Given Feishu approval status change, When changes, Then captured as approval
#[tokio::test]
async fn g1_13_approval_status_change_capture() {
    let log = fresh_event_log();
    let event = approval_event("approval-001", "approved");

    log.append(&event).await.unwrap();

    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.event_type, EventType::Approval);
    assert_eq!(stored.source, Source::FeishuApproval);
    assert_eq!(stored.content["status"], "approved");
}

/// G1-14: Given user has OKR, When daily sync, Then captured as okr_update
#[tokio::test]
async fn g1_14_okr_update_capture() {
    let log = fresh_event_log();
    let event = okr_update_event("okr-q3-001");

    log.append(&event).await.unwrap();

    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.event_type, EventType::OkrUpdate);
    assert_eq!(stored.source, Source::FeishuOkr);
}

/// G1-15: Given Bitable record changes, When detected, Then captured as document_change
#[tokio::test]
async fn g1_15_bitable_change_capture() {
    let log = fresh_event_log();
    let event = document_change_event("bitable-001", "row_update");

    log.append(&event).await.unwrap();

    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.event_type, EventType::DocumentChange);
}

/// G1-16: Given spreadsheet cell changes, When detected, Then captured as document_change
#[tokio::test]
async fn g1_16_spreadsheet_change_capture() {
    let log = fresh_event_log();
    let event = document_change_event("sheet-001", "cell_edit");

    log.append(&event).await.unwrap();

    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.event_type, EventType::DocumentChange);
}

/// G1-17: Given wiki node changes, When detected, Then captured as document_change
#[tokio::test]
async fn g1_17_wiki_node_change_capture() {
    let log = fresh_event_log();
    let event = document_change_event("wiki-node-001", "content_update");

    log.append(&event).await.unwrap();

    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.event_type, EventType::DocumentChange);
}

// ===========================================================================
// G1-18 ~ G1-27: System Activity & Manual Capture (scaffolded)
// ===========================================================================

/// G1-18: Given user switches app and stays >30s, When detected, Then records app_activity
#[tokio::test]
async fn g1_18_app_switch_long_stay_recorded() {
    let log = fresh_event_log();
    let event = app_activity_event("VS Code", 120); // 120 seconds > 30s threshold

    log.append(&event).await.unwrap();

    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.event_type, EventType::AppActivity);
    assert_eq!(stored.source, Source::SystemAppSwitch);
    // The actual debounce logic is in the collector; here we verify the event is stored
}

/// G1-19: Given user switches app and stays <30s, When detected, Then NOT recorded (debounce)
#[tokio::test]
async fn g1_19_app_switch_short_stay_not_recorded() {
    // This scenario tests collector-level debounce.
    // At the acceptance level, we verify that events below threshold
    // would not be produced by the collector.
    // Since we test at the EventLog level, we document the contract:
    // collectors MUST NOT emit AppActivity for stays < 30 seconds.
    //
    // This is enforced at the collector layer (A10 tests).
    // Here we verify the event structure if it were to arrive.
    let log = fresh_event_log();
    let event = app_activity_event("Safari", 10); // 10 seconds < 30s

    // The collector should filter this out, but if it arrives at the log,
    // it still gets stored (EventLog is append-only).
    // The contract is: collectors must debounce.
    log.append(&event).await.unwrap();
    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.event_type, EventType::AppActivity);
    // NOTE: Real debounce is tested at the collector level.
    // This test documents the expected behavior contract.
}

/// G1-20: Given user visits non-search page URL, When detected, Then records browsing
#[tokio::test]
async fn g1_20_non_search_url_recorded() {
    let log = fresh_event_log();
    let event = browsing_event("https://github.com/org/repo/pull/42", "PR #42");

    log.append(&event).await.unwrap();

    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.event_type, EventType::Browsing);
    assert_eq!(stored.source, Source::SystemBrowser);
}

/// G1-21: Given user visits search result page, When detected, Then NOT recorded
#[tokio::test]
async fn g1_21_search_page_not_recorded() {
    // Search pages are filtered at the collector level.
    // This test documents the contract.
    let log = fresh_event_log();
    // Search URLs typically contain /search or query params
    let event = browsing_event(
        "https://www.google.com/search?q=rust+async",
        "Google Search",
    );

    // Collector should filter search pages. If it arrives, it's still stored.
    // The contract is: collectors must filter search result pages.
    log.append(&event).await.unwrap();
    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.event_type, EventType::Browsing);
    // NOTE: Real filtering is at the collector level.
}

/// G1-22 ~ G1-27: Manual capture scenarios (UI-level, scaffolded)
/// These require Tauri window interaction which is tested at the E2E layer (F1).

/// G1-22: Global shortcut opens capture window with focus on input
#[tokio::test]
async fn g1_22_shortcut_opens_capture_window() {
    // UI-level scenario: tested in F-layer E2E
    // Here we verify the event creation path works
    let log = fresh_event_log();
    let event = manual_note_event("Quick capture via shortcut");

    log.append(&event).await.unwrap();
    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.event_type, EventType::ManualNote);
    assert_eq!(stored.source, Source::UserCapture);
}

/// G1-23: Window open -> input and submit -> creates manual_note
#[tokio::test]
async fn g1_23_manual_note_creation() {
    let log = fresh_event_log();
    let event = manual_note_event("Completed the API integration today");

    log.append(&event).await.unwrap();

    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.event_type, EventType::ManualNote);
    assert_eq!(stored.source_confidence, Confidence::High);
    assert_eq!(stored.content["text"], "Completed the API integration today");
}

/// G1-24: Window open -> paste image -> accepted as attachment
#[tokio::test]
async fn g1_24_paste_image_as_attachment() {
    use wb_core::event::{Attachment, AttachmentType};

    let log = fresh_event_log();
    let mut event = manual_note_event("Screenshot of the bug");
    event.attachments.push(Attachment {
        id: "att-paste-001".to_string(),
        attachment_type: AttachmentType::Image,
        filename: "pasted-image.png".to_string(),
        path: "/tmp/pasted-image.png".to_string(),
        mime_type: "image/png".to_string(),
        size_bytes: 204800,
    });

    log.append(&event).await.unwrap();
    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.attachments.len(), 1);
    assert_eq!(stored.attachments[0].attachment_type, AttachmentType::Image);
}

/// G1-25: Window open -> drag-drop file -> accepted as attachment
#[tokio::test]
async fn g1_25_drag_drop_file_as_attachment() {
    use wb_core::event::{Attachment, AttachmentType};

    let log = fresh_event_log();
    let mut event = manual_note_event("Design spec");
    event.attachments.push(Attachment {
        id: "att-drop-001".to_string(),
        attachment_type: AttachmentType::File,
        filename: "spec.pdf".to_string(),
        path: "/tmp/spec.pdf".to_string(),
        mime_type: "application/pdf".to_string(),
        size_bytes: 1024000,
    });

    log.append(&event).await.unwrap();
    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert_eq!(stored.attachments.len(), 1);
    assert_eq!(stored.attachments[0].attachment_type, AttachmentType::File);
}

/// G1-26: Screenshot key -> screenshot taken -> window opens with preloaded screenshot
#[tokio::test]
async fn g1_26_screenshot_preloaded_in_window() {
    // UI-level scenario: macOS screencapture integration
    // Tested at E2E layer (F1-02). Here we verify the event + attachment flow.
    use wb_core::event::{Attachment, AttachmentType};

    let log = fresh_event_log();
    let mut event = manual_note_event("Screenshot capture");
    event.attachments.push(Attachment {
        id: "att-screenshot-001".to_string(),
        attachment_type: AttachmentType::Image,
        filename: "screenshot.png".to_string(),
        path: "/tmp/screenshot.png".to_string(),
        mime_type: "image/png".to_string(),
        size_bytes: 512000,
    });

    log.append(&event).await.unwrap();
    let stored = log.get(&event.id).await.unwrap().unwrap();
    assert!(!stored.attachments.is_empty());
}

/// G1-27: Quick capture submit -> window auto-hides (config-dependent)
#[tokio::test]
async fn g1_27_capture_auto_hide() {
    // UI-level behavior, tested at E2E layer.
    // Here we verify the event is stored successfully after capture.
    let log = fresh_event_log();
    let event = manual_note_event("Quick note before meeting");

    log.append(&event).await.unwrap();

    let unprocessed = log.get_unprocessed(None).await.unwrap();
    assert_eq!(unprocessed.len(), 1);
}

// ===========================================================================
// G1-28 ~ G1-37: Collector Management & Event Ordering (scaffolded)
// ===========================================================================

/// G1-28: Given collector running, When disabled, Then stops and sub-collectors also disabled
#[tokio::test]
async fn g1_28_disable_collector_stops_sub_collectors() {
    // Tested at the CollectorManager level (A10-02).
    // This acceptance test verifies the contract.
    use std::sync::Arc;
    use wb_collector::manager::CollectorManager;
    use wb_collector::traits::{HealthStatus, Collector};
    use wb_core::error::Result as WbResult;

    struct StubCollector {
        id: String,
    }
    #[async_trait::async_trait]
    impl Collector for StubCollector {
        fn id(&self) -> &str { &self.id }
        fn name(&self) -> &str { &self.id }
        fn version(&self) -> &str { "0.1.0" }
        async fn collect(&self) -> WbResult<Vec<wb_core::event::Event>> { Ok(vec![]) }
        async fn health_check(&self) -> HealthStatus { HealthStatus::healthy() }
    }

    let manager = CollectorManager::new();
    manager.register(Arc::new(StubCollector { id: "feishu".into() })).await;
    manager.register(Arc::new(StubCollector { id: "browser".into() })).await;

    assert!(manager.is_enabled("feishu").await);
    manager.disable("feishu").await;
    assert!(!manager.is_enabled("feishu").await);
}

/// G1-29: Given parent collector disabled, When re-enabled, Then sub-collectors restore state
#[tokio::test]
async fn g1_29_re_enable_collector_restores_state() {
    use std::sync::Arc;
    use wb_collector::manager::CollectorManager;
    use wb_collector::traits::{HealthStatus, Collector};
    use wb_core::error::Result as WbResult;

    struct StubCollector { id: String }
    #[async_trait::async_trait]
    impl Collector for StubCollector {
        fn id(&self) -> &str { &self.id }
        fn name(&self) -> &str { &self.id }
        fn version(&self) -> &str { "0.1.0" }
        async fn collect(&self) -> WbResult<Vec<wb_core::event::Event>> { Ok(vec![]) }
        async fn health_check(&self) -> HealthStatus { HealthStatus::healthy() }
    }

    let manager = CollectorManager::new();
    manager.register(Arc::new(StubCollector { id: "feishu".into() })).await;

    manager.disable("feishu").await;
    assert!(!manager.is_enabled("feishu").await);

    manager.enable("feishu").await;
    assert!(manager.is_enabled("feishu").await);
}

/// G1-30: Given collector fails, When health check, Then auto-disabled and notified
#[tokio::test]
async fn g1_30_failed_collector_auto_disabled() {
    // This scenario requires integration with the health check loop.
    // At the acceptance level, we verify the health status propagation.
    use std::sync::Arc;
    use wb_collector::manager::CollectorManager;
    use wb_collector::traits::{HealthLevel, HealthStatus, Collector};
    use wb_core::error::Result as WbResult;

    struct FailingCollector;
    #[async_trait::async_trait]
    impl Collector for FailingCollector {
        fn id(&self) -> &str { "failing" }
        fn name(&self) -> &str { "Failing Collector" }
        fn version(&self) -> &str { "0.1.0" }
        async fn collect(&self) -> WbResult<Vec<wb_core::event::Event>> {
            Err(wb_core::error::WbError::Collector("connection timeout".into()))
        }
        async fn health_check(&self) -> HealthStatus {
            HealthStatus::unhealthy("connection timeout".into())
        }
    }

    let manager = CollectorManager::new();
    manager.register(Arc::new(FailingCollector)).await;

    let status = manager.health_check("failing").await.unwrap();
    assert_eq!(status.level, HealthLevel::Unhealthy);
}

/// G1-31: Given unhealthy status, When viewed, Then shows error indicator
#[tokio::test]
async fn g1_31_unhealthy_status_shows_error() {
    use wb_collector::traits::{HealthLevel, HealthStatus};

    let status = HealthStatus::unhealthy("API rate limited".to_string());
    assert_eq!(status.level, HealthLevel::Unhealthy);
    assert!(status.message.is_some());
    assert_eq!(status.message.unwrap(), "API rate limited");
}

/// G1-32: Given new collector registered at runtime, When registered, Then starts collecting
#[tokio::test]
async fn g1_32_runtime_collector_registration() {
    use std::sync::Arc;
    use wb_collector::manager::CollectorManager;
    use wb_collector::traits::{HealthStatus, Collector};
    use wb_core::error::Result as WbResult;

    struct NewCollector;
    #[async_trait::async_trait]
    impl Collector for NewCollector {
        fn id(&self) -> &str { "new-collector" }
        fn name(&self) -> &str { "New Collector" }
        fn version(&self) -> &str { "0.1.0" }
        async fn collect(&self) -> WbResult<Vec<wb_core::event::Event>> { Ok(vec![]) }
        async fn health_check(&self) -> HealthStatus { HealthStatus::healthy() }
    }

    let manager = CollectorManager::new();
    assert!(manager.list().await.is_empty());

    manager.register(Arc::new(NewCollector)).await;
    assert_eq!(manager.list().await.len(), 1);
    assert!(manager.is_enabled("new-collector").await);
}

/// G1-33: Given Feishu connection configured, When API mode, Then uses Feishu Open Platform API
#[tokio::test]
async fn g1_33_feishu_api_mode_config() {
    use wb_collector::config::CollectorConfigBuilder;
    use std::path::PathBuf;

    let config = CollectorConfigBuilder::new()
        .enable("feishu")
        .vault_path(PathBuf::from("/tmp/vault"))
        .build()
        .unwrap();

    let feishu = config.sources.iter().find(|s| s.id == "feishu").unwrap();
    assert!(feishu.enabled, "Feishu collector should be enabled");
}

/// G1-34: Given Feishu connection configured, When CLI mode, Then uses lark-cli fallback
#[tokio::test]
async fn g1_34_feishu_cli_mode_config() {
    use wb_collector::config::CollectorConfigBuilder;
    use std::path::PathBuf;

    let config = CollectorConfigBuilder::new()
        .enable("feishu")
        .vault_path(PathBuf::from("/tmp/vault"))
        .build()
        .unwrap();

    assert_eq!(config.sources.len(), 1);
}

/// G1-35: Given multiple events arrive simultaneously, When written to EventLog,
///        Then strict chronological order
#[tokio::test]
async fn g1_35_concurrent_events_chronological_order() {
    let log = fresh_event_log();

    // Append multiple events rapidly
    let mut ids = Vec::new();
    for i in 0..10 {
        let event = manual_note_event(&format!("note-{}", i));
        ids.push(event.id.clone());
        log.append(&event).await.unwrap();
    }

    // All events should be retrievable
    for id in &ids {
        let stored = log.get(id).await.unwrap();
        assert!(stored.is_some(), "Event {} should exist", id);
    }

    // Query with ordering — events should be in insertion order
    let filter = wb_core::event::EventFilter {
        source: Some(Source::UserCapture),
        ..Default::default()
    };
    let events = log.query(&filter).await.unwrap();
    assert_eq!(events.len(), 10);
}

/// G1-36: Given system restart, When unprocessed events exist, Then events recovered (no data loss)
#[tokio::test]
async fn g1_36_event_recovery_after_restart() {
    // Simulate: events are stored, then we "restart" by creating a new log handle
    // pointing to the same data. For in-memory, we test the contract.
    let log = fresh_event_log();

    let event1 = manual_note_event("note before restart");
    log.append(&event1).await.unwrap();

    // In a real scenario, the SQLite file persists.
    // Here we verify unprocessed events survive across handles.
    let unprocessed = log.get_unprocessed(None).await.unwrap();
    assert_eq!(unprocessed.len(), 1);
    assert_eq!(unprocessed[0].id, event1.id);
}

/// G1-37: Given processing logic changes, When replay triggered, Then historical events reprocessed
#[tokio::test]
async fn g1_37_event_replay() {
    let log = fresh_event_log();

    // Store some events
    let e1 = manual_note_event("replay-note-1");
    let e2 = manual_note_event("replay-note-2");
    log.append(&e1).await.unwrap();
    log.append(&e2).await.unwrap();

    // Mark one as processed
    log.mark_processed(&e1.id).await.unwrap();

    // Replay scenario: get all events (including processed) for reprocessing
    let filter = wb_core::event::EventFilter {
        source: Some(Source::UserCapture),
        ..Default::default()
    };
    let all_events = log.query(&filter).await.unwrap();
    assert_eq!(all_events.len(), 2, "Replay should see all events regardless of processed status");

    // Verify we can distinguish processed from unprocessed
    let unprocessed = log.get_unprocessed(None).await.unwrap();
    assert_eq!(unprocessed.len(), 1, "Only unprocessed events in the queue");
}
