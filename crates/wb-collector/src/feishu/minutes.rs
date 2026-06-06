//! 飞书妙记采集器

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use wb_core::error::Result;
use wb_core::event::{Confidence, Event, EventType, Source};

use crate::runner;
use crate::traits::{HealthStatus, Collector};

/// lark-cli 妙记列表响应
#[derive(Debug, Deserialize)]
struct LarkMinutesResponse {
    data: Option<LarkMinutesData>,
}

#[derive(Debug, Deserialize)]
struct LarkMinutesData {
    items: Option<Vec<LarkMinute>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LarkMinute {
    minute_token: Option<String>,
    title: Option<String>,
    owner: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
    create_time: Option<String>,
}

/// 飞书妙记采集器
pub struct FeishuMinutesCollector;

impl FeishuMinutesCollector {
    /// 将 lark-cli 妙记转换为 Event
    fn convert_minute(minute: LarkMinute) -> Option<Event> {
        let minute_token = minute.minute_token.clone()?;

        let raw_payload = serde_json::to_string(&minute).ok()?;

        let content = serde_json::json!({
            "minute_token": minute.minute_token,
            "title": minute.title,
            "owner": minute.owner,
            "start_time": minute.start_time,
            "end_time": minute.end_time,
        });

        let mut event = Event::new(
            Source::FeishuMeeting,
            Confidence::Medium,
            EventType::Meeting,
            content,
            raw_payload,
        );

        // 使用 minute_token 作为事件 id（保证幂等）
        event.id = minute_token;

        Some(event)
    }
}

#[async_trait]
impl Collector for FeishuMinutesCollector {
    fn id(&self) -> &str {
        "feishu-minutes"
    }

    fn name(&self) -> &str {
        "飞书妙记采集器"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn collect(&self) -> Result<Vec<Event>> {
        let args = vec!["minutes", "list"];

        let response: LarkMinutesResponse = runner::execute_json("lark-cli", &args)?;

        let items = response.data.and_then(|d| d.items).unwrap_or_default();

        let events: Vec<Event> = items
            .into_iter()
            .filter_map(|m| Self::convert_minute(m))
            .collect();

        Ok(events)
    }

    async fn health_check(&self) -> HealthStatus {
        HealthStatus::healthy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_minute() {
        let minute = LarkMinute {
            minute_token: Some("min-001".to_string()),
            title: Some("产品评审会议纪要".to_string()),
            owner: Some("user-001".to_string()),
            start_time: Some("1717689600".to_string()),
            end_time: Some("1717693200".to_string()),
            create_time: Some("1717693300".to_string()),
        };

        let event = FeishuMinutesCollector::convert_minute(minute);
        assert!(event.is_some());

        let event = event.unwrap();
        assert_eq!(event.id, "min-001");
        assert_eq!(event.source, Source::FeishuMeeting);
        assert_eq!(event.source_confidence, Confidence::Medium);
        assert_eq!(event.event_type, EventType::Meeting);
        assert_eq!(event.content["title"], "产品评审会议纪要");
        assert_eq!(event.content["owner"], "user-001");
    }

    #[test]
    fn test_convert_minute_no_id_returns_none() {
        let minute = LarkMinute {
            minute_token: None,
            title: Some("无 token 妙记".to_string()),
            owner: None,
            start_time: None,
            end_time: None,
            create_time: None,
        };

        let event = FeishuMinutesCollector::convert_minute(minute);
        assert!(event.is_none());
    }

    #[test]
    fn test_convert_minute_missing_optional_fields() {
        let minute = LarkMinute {
            minute_token: Some("min-002".to_string()),
            title: None,
            owner: None,
            start_time: None,
            end_time: None,
            create_time: None,
        };

        let event = FeishuMinutesCollector::convert_minute(minute).unwrap();
        assert_eq!(event.id, "min-002");
        assert_eq!(event.content["title"], serde_json::Value::Null);
        assert_eq!(event.content["owner"], serde_json::Value::Null);
    }
}
