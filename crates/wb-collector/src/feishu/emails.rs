//! 飞书邮件采集器

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use wb_core::error::Result;
use wb_core::event::{Confidence, Event, EventType, Source};

use crate::runner;
use crate::traits::{HealthStatus, Collector};

/// lark-cli mail user_mailbox.messages list 响应
#[derive(Debug, Deserialize)]
struct LarkEmailsResponse {
    data: Option<LarkEmailsData>,
}

#[derive(Debug, Deserialize)]
struct LarkEmailsData {
    items: Option<Vec<LarkEmail>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LarkEmail {
    message_id: Option<String>,
    subject: Option<String>,
    sender: Option<String>,
    recipients: Option<Vec<String>>,
    received_time: Option<String>,
}

/// 飞书邮件采集器
pub struct FeishuEmailCollector;

impl FeishuEmailCollector {
    /// 将 lark-cli 邮件转换为 Event
    fn convert_email(email: LarkEmail) -> Option<Event> {
        let message_id = email.message_id.clone()?;

        let raw_payload = serde_json::to_string(&email).ok()?;

        let content = serde_json::json!({
            "message_id": email.message_id,
            "subject": email.subject,
            "sender": email.sender,
            "recipients": email.recipients,
            "received_time": email.received_time,
        });

        let mut event = Event::new(
            Source::FeishuEmail,
            Confidence::Medium,
            EventType::Email,
            content,
            raw_payload,
        );

        // 使用 message_id 作为事件 id（保证幂等）
        event.id = message_id;

        Some(event)
    }
}

#[async_trait]
impl Collector for FeishuEmailCollector {
    fn id(&self) -> &str {
        "feishu-email"
    }

    fn name(&self) -> &str {
        "飞书邮件采集器"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn collect(&self) -> Result<Vec<Event>> {
        let params = r#"{"user_mailbox_id":"me","page_size":50}"#;
        let args = vec!["mail", "user_mailbox.messages", "list", "--params", params, "--format", "json"];

        let response: LarkEmailsResponse = runner::execute_json("lark-cli", &args)?;

        let items = response.data.and_then(|d| d.items).unwrap_or_default();

        let events: Vec<Event> = items
            .into_iter()
            .filter_map(Self::convert_email)
            .collect();

        Ok(events)
    }

    async fn health_check(&self) -> HealthStatus {
        match crate::runner::execute("lark-cli", &["--version"]) {
            Ok(_) => HealthStatus::healthy(),
            Err(e) => HealthStatus::unhealthy(format!("lark-cli not available: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_email() {
        let email = LarkEmail {
            message_id: Some("eml-001".to_string()),
            subject: Some("项目进度通知".to_string()),
            sender: Some("user-001@example.com".to_string()),
            recipients: Some(vec!["user-002@example.com".to_string()]),
            received_time: Some("1717689600".to_string()),
        };

        let event = FeishuEmailCollector::convert_email(email);
        assert!(event.is_some());

        let event = event.unwrap();
        assert_eq!(event.id, "eml-001");
        assert_eq!(event.source, Source::FeishuEmail);
        assert_eq!(event.source_confidence, Confidence::Medium);
        assert_eq!(event.event_type, EventType::Email);
        assert_eq!(event.content["subject"], "项目进度通知");
        assert_eq!(event.content["sender"], "user-001@example.com");
    }

    #[test]
    fn test_convert_email_no_id_returns_none() {
        let email = LarkEmail {
            message_id: None,
            subject: Some("无 ID 邮件".to_string()),
            sender: None,
            recipients: None,
            received_time: None,
        };

        let event = FeishuEmailCollector::convert_email(email);
        assert!(event.is_none());
    }

    #[test]
    fn test_convert_email_missing_optional_fields() {
        let email = LarkEmail {
            message_id: Some("eml-002".to_string()),
            subject: None,
            sender: None,
            recipients: None,
            received_time: None,
        };

        let event = FeishuEmailCollector::convert_email(email).unwrap();
        assert_eq!(event.id, "eml-002");
        assert_eq!(event.content["subject"], serde_json::Value::Null);
        assert_eq!(event.content["sender"], serde_json::Value::Null);
    }
}
