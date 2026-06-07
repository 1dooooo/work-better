//! G-layer acceptance test helpers
//!
//! Shared utilities for black-box acceptance tests (G1-G7).
//! These helpers construct domain objects for test scenarios without
//! depending on internal implementation details.

use serde_json::{json, Value};
use wb_core::event::{Confidence, Event, EventFilter, EventLog, EventType, Source};
use wb_core::record::{Category, WorkRecord};
use wb_core::task::{Priority, Task, TaskStatus};
use wb_storage::SqliteEventLog;

// ─── Event Builders ──────────────────────────────────────────────────────

/// Create a Feishu message event with @mention
pub fn feishu_mention_event(mention_target: &str, text: &str) -> Event {
    Event::new(
        Source::FeishuMessage,
        Confidence::High,
        EventType::Message,
        json!({"text": text, "mention": mention_target}),
        format!(r#"{{"type":"message","text":"{}"}}"#, text),
    )
}

/// Create a plain Feishu message (no @mention)
pub fn feishu_message_event(text: &str) -> Event {
    Event::new(
        Source::FeishuMessage,
        Confidence::Medium,
        EventType::Message,
        json!({"text": text}),
        format!(r#"{{"type":"message","text":"{}"}}"#, text),
    )
}

/// Create a Feishu thread reply event
pub fn feishu_thread_reply_event(thread_id: &str, text: &str) -> Event {
    let mut event = Event::new(
        Source::FeishuMessage,
        Confidence::High,
        EventType::Message,
        json!({"text": text, "thread_id": thread_id}),
        format!(r#"{{"type":"message","thread_id":"{}"}}"#, thread_id),
    );
    event.related_ids.push(thread_id.to_string());
    event
}

/// Create a Feishu DM event
pub fn feishu_dm_event(text: &str) -> Event {
    Event::new(
        Source::FeishuMessage,
        Confidence::High,
        EventType::Message,
        json!({"text": text, "chat_type": "p2p"}),
        format!(r#"{{"type":"dm","text":"{}"}}"#, text),
    )
}

/// Create a keyword-matching message event
pub fn keyword_match_event(keyword: &str, text: &str) -> Event {
    let mut event = Event::new(
        Source::FeishuMessage,
        Confidence::Medium,
        EventType::Message,
        json!({"text": text}),
        format!(r#"{{"type":"message","text":"{}"}}"#, text),
    );
    event.tags.push(format!("keyword:{}", keyword));
    event
}

/// Create a document change event
pub fn document_change_event(doc_id: &str, action: &str) -> Event {
    Event::new(
        Source::FeishuDoc,
        Confidence::High,
        EventType::DocumentChange,
        json!({"doc_id": doc_id, "action": action}),
        format!(r#"{{"type":"doc_change","doc_id":"{}"}}"#, doc_id),
    )
}

/// Create a task update event
pub fn task_update_event(task_id: &str, status: &str) -> Event {
    Event::new(
        Source::FeishuProject,
        Confidence::High,
        EventType::TaskUpdate,
        json!({"task_id": task_id, "status": status}),
        format!(r#"{{"type":"task_update","task_id":"{}"}}"#, task_id),
    )
}

/// Create a calendar event
pub fn calendar_event_event(event_id: &str, title: &str) -> Event {
    Event::new(
        Source::FeishuCalendar,
        Confidence::High,
        EventType::CalendarEvent,
        json!({"event_id": event_id, "title": title}),
        format!(r#"{{"type":"calendar","event_id":"{}"}}"#, event_id),
    )
}

/// Create a meeting event
pub fn meeting_event(meeting_id: &str, title: &str) -> Event {
    Event::new(
        Source::FeishuMeeting,
        Confidence::High,
        EventType::Meeting,
        json!({"meeting_id": meeting_id, "title": title, "has_minutes": true}),
        format!(r#"{{"type":"meeting","meeting_id":"{}"}}"#, meeting_id),
    )
}

/// Create an email event
pub fn email_event(subject: &str) -> Event {
    Event::new(
        Source::FeishuEmail,
        Confidence::Medium,
        EventType::Email,
        json!({"subject": subject}),
        format!(r#"{{"type":"email","subject":"{}"}}"#, subject),
    )
}

/// Create an approval event
pub fn approval_event(approval_id: &str, status: &str) -> Event {
    Event::new(
        Source::FeishuApproval,
        Confidence::High,
        EventType::Approval,
        json!({"approval_id": approval_id, "status": status}),
        format!(r#"{{"type":"approval","id":"{}"}}"#, approval_id),
    )
}

/// Create an OKR update event
pub fn okr_update_event(okr_id: &str) -> Event {
    Event::new(
        Source::FeishuOkr,
        Confidence::High,
        EventType::OkrUpdate,
        json!({"okr_id": okr_id}),
        format!(r#"{{"type":"okr_update","okr_id":"{}"}}"#, okr_id),
    )
}

/// Create a manual note event (user capture)
pub fn manual_note_event(text: &str) -> Event {
    Event::new(
        Source::UserCapture,
        Confidence::High,
        EventType::ManualNote,
        json!({"text": text}),
        format!(r#"{{"type":"manual_note","text":"{}"}}"#, text),
    )
}

/// Create a browsing event
pub fn browsing_event(url: &str, title: &str) -> Event {
    Event::new(
        Source::SystemBrowser,
        Confidence::Medium,
        EventType::Browsing,
        json!({"url": url, "title": title}),
        format!(r#"{{"type":"browsing","url":"{}"}}"#, url),
    )
}

/// Create an app activity event
pub fn app_activity_event(app_name: &str, duration_secs: u64) -> Event {
    Event::new(
        Source::SystemAppSwitch,
        Confidence::Medium,
        EventType::AppActivity,
        json!({"app": app_name, "duration_secs": duration_secs}),
        format!(r#"{{"type":"app_activity","app":"{}"}}"#, app_name),
    )
}

// ─── Task Builders ───────────────────────────────────────────────────────

/// Create a task with specific status and source platform
pub fn make_task(title: &str, status: TaskStatus, platform: &str) -> Task {
    let mut task = Task::new(title, status);
    task.source_platform = platform.to_string();
    task
}

/// Create a task that needs review (AI-extracted)
pub fn make_ai_task(title: &str) -> Task {
    let mut task = Task::new(title, TaskStatus::Todo);
    task.needs_review = true;
    task.confidence = 0.6;
    task.source_platform = "ai".to_string();
    task
}

// ─── WorkRecord Builders ────────────────────────────────────────────────

/// Create a work record with given category
pub fn make_record(
    title: &str,
    summary: &str,
    category: Category,
    source_event_ids: Vec<String>,
) -> WorkRecord {
    WorkRecord::new(
        title.to_string(),
        summary.to_string(),
        String::new(),
        category,
        source_event_ids,
        "test-model".to_string(),
        0.9,
    )
}

// ─── Database Helpers ────────────────────────────────────────────────────

/// Create a fresh in-memory SQLite event log
pub fn fresh_event_log() -> SqliteEventLog {
    SqliteEventLog::new_in_memory().expect("failed to create in-memory event log")
}
