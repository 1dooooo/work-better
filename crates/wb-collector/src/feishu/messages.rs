//! 飞书消息采集器

use serde::{Deserialize, Serialize};
use wb_core::error::Result;
use wb_core::event::{Confidence, Event, EventType, Source};

use crate::runner;

/// lark-cli 消息响应
#[derive(Debug, Deserialize)]
struct LarkMessagesResponse {
    data: Option<LarkMessagesData>,
}

#[derive(Debug, Deserialize)]
struct LarkMessagesData {
    items: Option<Vec<LarkMessage>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LarkMessage {
    message_id: Option<String>,
    msg_type: Option<String>,
    content: Option<String>,
    sender: Option<LarkSender>,
    create_time: Option<String>,
    update_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LarkSender {
    sender_id: Option<String>,
    sender_type: Option<String>,
    tenant_key: Option<String>,
}

/// 飞书消息采集器
pub struct FeishuMessageCollector;

impl FeishuMessageCollector {
    /// 采集指定会话的消息
    ///
    /// # Arguments
    /// * `chat_id` - 飞书会话 ID
    /// * `limit` - 最大采集数量
    pub fn collect(chat_id: &str, limit: u32) -> Result<Vec<Event>> {
        let limit_str = limit.to_string();
        let args = vec![
            "im",
            "messages",
            "list",
            "--chat-id",
            chat_id,
            "--page-size",
            &limit_str,
        ];

        let response: LarkMessagesResponse = runner::execute_json("lark-cli", &args)?;

        let items = response.data.and_then(|d| d.items).unwrap_or_default();

        let events: Vec<Event> = items
            .into_iter()
            .filter_map(|msg| Self::convert_message(msg))
            .collect();

        Ok(events)
    }

    /// 将 lark-cli 消息转换为 Event
    fn convert_message(msg: LarkMessage) -> Option<Event> {
        let message_id = msg.message_id.clone()?;

        // 构造 raw_payload
        let raw_payload = serde_json::to_string(&msg).ok()?;

        let content = msg.content.unwrap_or_default();

        // 解析 content 为 JSON
        let content_json: serde_json::Value =
            serde_json::from_str(&content).unwrap_or(serde_json::Value::String(content));

        let mut event = Event::new(
            Source::FeishuMessage,
            Confidence::Medium,
            EventType::Message,
            content_json,
            raw_payload,
        );

        // 使用 message_id 作为事件 id（保证幂等）
        event.id = message_id;

        Some(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_message() {
        let msg = LarkMessage {
            message_id: Some("msg-001".to_string()),
            msg_type: Some("text".to_string()),
            content: Some(r#"{"text":"Hello, world!"}"#.to_string()),
            sender: Some(LarkSender {
                sender_id: Some("user-001".to_string()),
                sender_type: Some("user".to_string()),
                tenant_key: None,
            }),
            create_time: Some("1717689600".to_string()),
            update_time: None,
        };

        let event = FeishuMessageCollector::convert_message(msg);
        assert!(event.is_some());

        let event = event.unwrap();
        assert_eq!(event.id, "msg-001");
        assert_eq!(event.source, Source::FeishuMessage);
        assert_eq!(event.source_confidence, Confidence::Medium);
        assert_eq!(event.event_type, EventType::Message);
    }

    #[test]
    fn test_convert_message_no_id_returns_none() {
        let msg = LarkMessage {
            message_id: None,
            msg_type: Some("text".to_string()),
            content: Some("{}".to_string()),
            sender: None,
            create_time: None,
            update_time: None,
        };

        let event = FeishuMessageCollector::convert_message(msg);
        assert!(event.is_none());
    }

    #[test]
    fn test_convert_message_invalid_content() {
        let msg = LarkMessage {
            message_id: Some("msg-002".to_string()),
            msg_type: Some("text".to_string()),
            content: Some("not-json".to_string()),
            sender: None,
            create_time: None,
            update_time: None,
        };

        let event = FeishuMessageCollector::convert_message(msg).unwrap();
        // content 应该回退为 JSON string
        assert_eq!(
            event.content,
            serde_json::Value::String("not-json".to_string())
        );
    }
}
