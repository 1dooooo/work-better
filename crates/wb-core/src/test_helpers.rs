//! 测试辅助工具 —— 提供常用测试构造函数和重导出

use crate::event::{Confidence, Event, EventType, Source};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 简化的应用配置，用于测试
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestAppConfig {
    pub vault_path: String,
    pub db_path: String,
    pub model_endpoint: String,
    pub model_token_budget: u32,
    pub collectors_enabled: HashMap<String, bool>,
}

impl Default for TestAppConfig {
    fn default() -> Self {
        Self {
            vault_path: "/tmp/test-vault".into(),
            db_path: "/tmp/test.db".into(),
            model_endpoint: "https://api.openai.com/v1".into(),
            model_token_budget: 4096,
            collectors_enabled: HashMap::new(),
        }
    }
}

/// 创建一个用于测试的 Event，使用飞书消息来源
pub fn create_test_event() -> Event {
    Event::new(
        Source::FeishuMessage,
        Confidence::High,
        EventType::Message,
        serde_json::json!({
            "text": "测试消息内容",
            "sender": "test_user",
            "chat_id": "oc_test123"
        }),
        r#"{"text":"测试消息内容","sender":"test_user"}"#.to_string(),
    )
}

/// 创建一个带有自定义内容的测试 Event
pub fn create_test_event_with_content(content: serde_json::Value) -> Event {
    Event::new(
        Source::FeishuMessage,
        Confidence::High,
        EventType::Message,
        content.clone(),
        content.to_string(),
    )
}

/// 创建一个用于测试的 AppConfig（简化版本）
pub fn create_test_config() -> TestAppConfig {
    TestAppConfig::default()
}

/// 创建一个带有指定标签的测试 Event
pub fn create_test_event_with_tags(tags: Vec<String>) -> Event {
    let mut event = create_test_event();
    event.tags = tags;
    event
}

/// 生成唯一测试 ID
pub fn test_id(prefix: &str) -> String {
    format!("{prefix}-{}", Uuid::new_v4())
}

/// 创建固定时间戳的测试 Event（用于快照测试）
pub fn create_test_event_fixed() -> Event {
    Event {
        id: "test-event-001".to_string(),
        timestamp: "2024-01-15T10:30:00Z".parse().unwrap(),
        collected_at: "2024-01-15T10:30:01Z".parse().unwrap(),
        source: Source::FeishuMessage,
        source_confidence: Confidence::High,
        event_type: EventType::Message,
        content: serde_json::json!({"text": "固定事件"}),
        raw_payload: r#"{"text":"固定事件"}"#.to_string(),
        tags: vec!["test".to_string()],
        related_ids: vec![],
        attachments: vec![],
    }
}

// 重导出常用测试依赖
pub use insta;
pub use rstest;
