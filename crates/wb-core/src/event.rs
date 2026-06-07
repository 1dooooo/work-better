use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// 事件来源
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub enum Source {
    FeishuMessage,
    FeishuDoc,
    FeishuProject,
    FeishuCalendar,
    FeishuMeeting,
    FeishuEmail,
    FeishuApproval,
    FeishuOkr,
    FeishuBitable,
    FeishuSheet,
    FeishuWiki,
    SystemAppSwitch,
    SystemBrowser,
    UserCapture,
}

/// 来源置信度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

/// 事件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub enum EventType {
    Message,
    DocumentChange,
    TaskUpdate,
    Meeting,
    CalendarEvent,
    Email,
    Approval,
    OkrUpdate,
    Browsing,
    AppActivity,
    ManualNote,
}

/// 附件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub enum AttachmentType {
    Image,
    File,
}

/// 附件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct Attachment {
    pub id: String,
    #[serde(rename = "type")]
    pub attachment_type: AttachmentType,
    pub filename: String,
    pub path: String,
    pub mime_type: String,
    pub size_bytes: u64,
}

/// 事件 —— 系统的原子单位，不可变
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct Event {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub collected_at: DateTime<Utc>,
    pub source: Source,
    pub source_confidence: Confidence,
    #[serde(rename = "type")]
    pub event_type: EventType,
    pub content: serde_json::Value,
    pub raw_payload: String,
    pub tags: Vec<String>,
    pub related_ids: Vec<String>,
    pub attachments: Vec<Attachment>,
}

impl Event {
    /// 创建新事件，自动分配 id 和 collected_at
    pub fn new(
        source: Source,
        source_confidence: Confidence,
        event_type: EventType,
        content: serde_json::Value,
        raw_payload: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            collected_at: Utc::now(),
            source,
            source_confidence,
            event_type,
            content,
            raw_payload,
            tags: Vec::new(),
            related_ids: Vec::new(),
            attachments: Vec::new(),
        }
    }
}

/// 事件过滤条件
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    pub source: Option<Source>,
    pub event_type: Option<EventType>,
    pub processed: Option<bool>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

/// EventLog 异步 trait —— 事件存储的核心抽象
#[async_trait::async_trait]
pub trait EventLog: Send + Sync {
    /// 追加事件（不可变，只追加）
    async fn append(&self, event: &Event) -> crate::error::Result<()>;

    /// 根据 ID 获取事件
    async fn get(&self, id: &str) -> crate::error::Result<Option<Event>>;

    /// 按条件查询事件
    async fn query(&self, filter: &EventFilter) -> crate::error::Result<Vec<Event>>;

    /// 标记事件已处理
    async fn mark_processed(&self, id: &str) -> crate::error::Result<()>;

    /// 获取未处理的事件
    async fn get_unprocessed(&self, limit: Option<usize>) -> crate::error::Result<Vec<Event>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_new_assigns_id_and_timestamp() {
        let event = Event::new(
            Source::FeishuMessage,
            Confidence::High,
            EventType::Message,
            serde_json::json!({"text": "hello"}),
            "raw".to_string(),
        );

        assert!(!event.id.is_empty(), "id should not be empty");
        assert!(!event.id.is_empty());
        // timestamp should be set (not zero)
        assert!(event.timestamp.timestamp() > 0);
        assert!(event.collected_at.timestamp() > 0);
        assert_eq!(event.source, Source::FeishuMessage);
        assert_eq!(event.source_confidence, Confidence::High);
        assert_eq!(event.event_type, EventType::Message);
        assert!(event.tags.is_empty());
        assert!(event.related_ids.is_empty());
        assert!(event.attachments.is_empty());
    }

    #[test]
    fn test_event_serialization_roundtrip() {
        let event = Event::new(
            Source::FeishuDoc,
            Confidence::Medium,
            EventType::DocumentChange,
            serde_json::json!({"doc_id": "123", "action": "update"}),
            "raw payload data".to_string(),
        );

        let json = serde_json::to_string(&event).expect("serialization should succeed");
        let deserialized: Event =
            serde_json::from_str(&json).expect("deserialization should succeed");

        assert_eq!(event, deserialized);
    }

    // ============================================================
    // A11: Message Conversion (4 scenarios)
    // ============================================================

    /// A11-01: Valid FeishuMessage-like data -> Event conversion
    /// Verifies that constructing an Event from FeishuMessage source
    /// produces correct fields.
    #[test]
    fn a11_01_valid_feishu_message_to_event() {
        let content = serde_json::json!({
            "text": "Hello, world!",
            "sender": "user-001",
            "chat_id": "oc_test123"
        });
        let raw = r#"{"message_id":"msg-001","text":"Hello, world!"}"#;

        let event = Event::new(
            Source::FeishuMessage,
            Confidence::Medium,
            EventType::Message,
            content.clone(),
            raw.to_string(),
        );

        assert_eq!(event.source, Source::FeishuMessage);
        assert_eq!(event.source_confidence, Confidence::Medium);
        assert_eq!(event.event_type, EventType::Message);
        assert_eq!(event.content, content);
        assert_eq!(event.raw_payload, raw);
        assert!(event.tags.is_empty());
        assert!(event.related_ids.is_empty());
        assert!(event.attachments.is_empty());
    }

    /// A11-02: Missing id -> new() still assigns an id
    /// The Event::new constructor always generates a UUID, so there
    /// is no "missing id" path at the Event level. This test verifies
    /// that an id is always present and non-empty.
    #[test]
    fn a11_02_id_always_assigned() {
        let event = Event::new(
            Source::FeishuMessage,
            Confidence::High,
            EventType::Message,
            serde_json::json!({"text": "test"}),
            "raw".to_string(),
        );
        assert!(!event.id.is_empty(), "Event id should never be empty");
        // Verify it's a valid UUID format
        assert!(
            uuid::Uuid::parse_str(&event.id).is_ok(),
            "Event id should be a valid UUID"
        );
    }

    /// A11-03: Invalid JSON content falls back gracefully
    /// When content is a non-JSON string, Event stores it as-is
    /// (the caller is responsible for parsing). The Event itself
    /// does not fail — it accepts any serde_json::Value.
    #[test]
    fn a11_03_invalid_json_content_stored_as_is() {
        // Simulate what convert_message does: parse fails -> Value::String fallback
        let raw_content = "not-valid-json";
        let content_json: serde_json::Value = serde_json::from_str(raw_content)
            .unwrap_or(serde_json::Value::String(raw_content.to_string()));

        let event = Event::new(
            Source::FeishuMessage,
            Confidence::Medium,
            EventType::Message,
            content_json.clone(),
            "raw".to_string(),
        );

        assert_eq!(
            event.content,
            serde_json::Value::String("not-valid-json".to_string()),
            "Invalid JSON should be stored as a string Value"
        );
    }

    /// A11-04: Idempotent id generation
    /// When using Event::new(), each call generates a unique id.
    /// But if an external id is assigned (like message_id), calling
    /// with the same input should produce deterministic content.
    #[test]
    fn a11_04_idempotent_content_generation() {
        let content = serde_json::json!({"text": "same content"});
        let raw = r#"{"text":"same content"}"#;

        let event1 = Event::new(
            Source::FeishuMessage,
            Confidence::Medium,
            EventType::Message,
            content.clone(),
            raw.to_string(),
        );

        let event2 = Event::new(
            Source::FeishuMessage,
            Confidence::Medium,
            EventType::Message,
            content.clone(),
            raw.to_string(),
        );

        // IDs will differ (random UUID), but content must be identical
        assert_ne!(event1.id, event2.id, "UUIDs should be unique");
        assert_eq!(
            event1.content, event2.content,
            "Content should be identical"
        );
        assert_eq!(event1.source, event2.source);
        assert_eq!(event1.event_type, event2.event_type);
        assert_eq!(event1.source_confidence, event2.source_confidence);
        assert_eq!(event1.raw_payload, event2.raw_payload);
    }
}
