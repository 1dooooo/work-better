//! 飞书消息采集器

use serde::{Deserialize, Serialize};
use wb_core::error::Result;
use wb_core::event::{Confidence, Event, EventType, Source};

use crate::runner;

/// lark-cli 消息响应（+chat-messages-list 格式）
#[derive(Debug, Deserialize)]
struct LarkMessagesResponse {
    #[allow(dead_code)]
    ok: Option<bool>,
    data: Option<LarkMessagesData>,
}

#[derive(Debug, Deserialize)]
struct LarkMessagesData {
    messages: Option<Vec<LarkMessage>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LarkMessage {
    message_id: Option<String>,
    msg_type: Option<String>,
    content: Option<String>,
    sender: Option<LarkSender>,
    create_time: Option<String>,
    chat_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LarkSender {
    id: Option<String>,
    name: Option<String>,
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
            "+chat-messages-list",
            "--chat-id",
            chat_id,
            "--page-size",
            &limit_str,
        ];

        let response: LarkMessagesResponse = runner::execute_json("lark-cli", &args)?;

        let items = response.data.and_then(|d| d.messages).unwrap_or_default();

        let events: Vec<Event> = items
            .into_iter()
            .filter_map(Self::convert_message)
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
                id: Some("user-001".to_string()),
                name: Some("testuser".to_string()),
                sender_type: Some("user".to_string()),
                tenant_key: None,
            }),
            create_time: Some("1717689600".to_string()),
            chat_id: Some("oc_test".to_string()),
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
            chat_id: None,
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
            chat_id: None,
        };

        let event = FeishuMessageCollector::convert_message(msg).unwrap();
        // content 应该回退为 JSON string
        assert_eq!(
            event.content,
            serde_json::Value::String("not-json".to_string())
        );
    }

    // ============================================================
    // C1: Lark API Response Contracts (snapshot tests)
    // ============================================================

    #[test]
    fn c1_01_lark_messages_response_deserialize() {
        let json = include_str!("../../fixtures/lark_messages_response.json");
        let response: LarkMessagesResponse = serde_json::from_str(json).unwrap();
        assert!(response.ok.unwrap());
        let messages = response.data.unwrap().messages.unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].message_id.as_deref(), Some("om_msg_001"));
        assert_eq!(messages[1].message_id.as_deref(), Some("om_msg_002"));
        insta::assert_json_snapshot!("lark_messages_response", messages);
    }

    #[test]
    fn c1_02_lark_message_field_matching() {
        let json = include_str!("../../fixtures/lark_message.json");
        let msg: LarkMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.message_id.as_deref(), Some("om_msg_single"));
        assert_eq!(msg.msg_type.as_deref(), Some("text"));
        assert!(msg.content.is_some());
        assert!(msg.sender.is_some());
        assert_eq!(msg.create_time.as_deref(), Some("1717776000"));
        assert_eq!(msg.chat_id.as_deref(), Some("oc_chat_002"));
        insta::assert_json_snapshot!("lark_message", msg);
    }

    #[test]
    fn c1_03_lark_sender_field_matching() {
        let json = include_str!("../../fixtures/lark_sender.json");
        let sender: LarkSender = serde_json::from_str(json).unwrap();
        assert_eq!(sender.id.as_deref(), Some("ou_sender_001"));
        assert_eq!(sender.name.as_deref(), Some("Diana"));
        assert_eq!(sender.sender_type.as_deref(), Some("app"));
        assert_eq!(sender.tenant_key.as_deref(), Some("tenant_app"));
        insta::assert_json_snapshot!("lark_sender", sender);
    }

    #[test]
    fn c1_04_empty_data_messages_fallback() {
        let json = include_str!("../../fixtures/lark_messages_empty.json");
        let response: LarkMessagesResponse = serde_json::from_str(json).unwrap();
        assert!(response.ok.unwrap());
        // Verify fallback: unwrap_or_default produces empty vec when data.messages is empty
        let messages = response.data.and_then(|d| d.messages).unwrap_or_default();
        assert!(messages.is_empty());
        insta::assert_json_snapshot!("lark_messages_empty", messages);
    }
}
