//! 飞书会议采集器

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use wb_core::error::Result;
use wb_core::event::{Confidence, Event, EventType, Source};

use crate::runner;
use crate::traits::{HealthStatus, Collector};

/// lark-cli 会议列表响应
#[derive(Debug, Deserialize)]
struct LarkMeetingsResponse {
    data: Option<LarkMeetingsData>,
}

#[derive(Debug, Deserialize)]
struct LarkMeetingsData {
    items: Option<Vec<LarkMeeting>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LarkMeeting {
    meeting_id: Option<String>,
    topic: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
    organizer: Option<String>,
    participants: Option<Vec<String>>,
}

/// 飞书会议采集器
pub struct FeishuMeetingCollector;

impl FeishuMeetingCollector {
    /// 将 lark-cli 会议转换为 Event
    fn convert_meeting(meeting: LarkMeeting) -> Option<Event> {
        let meeting_id = meeting.meeting_id.clone()?;

        let raw_payload = serde_json::to_string(&meeting).ok()?;

        let content = serde_json::json!({
            "meeting_id": meeting.meeting_id,
            "topic": meeting.topic,
            "start_time": meeting.start_time,
            "end_time": meeting.end_time,
            "organizer": meeting.organizer,
            "participants": meeting.participants,
        });

        let mut event = Event::new(
            Source::FeishuMeeting,
            Confidence::Medium,
            EventType::Meeting,
            content,
            raw_payload,
        );

        // 使用 meeting_id 作为事件 id（保证幂等）
        event.id = meeting_id;

        Some(event)
    }
}

#[async_trait]
impl Collector for FeishuMeetingCollector {
    fn id(&self) -> &str {
        "feishu-meeting"
    }

    fn name(&self) -> &str {
        "飞书会议采集器"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn collect(&self) -> Result<Vec<Event>> {
        let args = vec!["meeting", "list"];

        let response: LarkMeetingsResponse = runner::execute_json("lark-cli", &args)?;

        let items = response.data.and_then(|d| d.items).unwrap_or_default();

        let events: Vec<Event> = items
            .into_iter()
            .filter_map(|m| Self::convert_meeting(m))
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
    fn test_convert_meeting() {
        let meeting = LarkMeeting {
            meeting_id: Some("mtg-001".to_string()),
            topic: Some("周会".to_string()),
            start_time: Some("1717689600".to_string()),
            end_time: Some("1717693200".to_string()),
            organizer: Some("user-001".to_string()),
            participants: Some(vec!["user-001".to_string(), "user-002".to_string()]),
        };

        let event = FeishuMeetingCollector::convert_meeting(meeting);
        assert!(event.is_some());

        let event = event.unwrap();
        assert_eq!(event.id, "mtg-001");
        assert_eq!(event.source, Source::FeishuMeeting);
        assert_eq!(event.source_confidence, Confidence::Medium);
        assert_eq!(event.event_type, EventType::Meeting);
        assert_eq!(event.content["topic"], "周会");
        assert_eq!(event.content["organizer"], "user-001");
    }

    #[test]
    fn test_convert_meeting_no_id_returns_none() {
        let meeting = LarkMeeting {
            meeting_id: None,
            topic: Some("无 ID 会议".to_string()),
            start_time: None,
            end_time: None,
            organizer: None,
            participants: None,
        };

        let event = FeishuMeetingCollector::convert_meeting(meeting);
        assert!(event.is_none());
    }

    #[test]
    fn test_convert_meeting_missing_optional_fields() {
        let meeting = LarkMeeting {
            meeting_id: Some("mtg-002".to_string()),
            topic: None,
            start_time: None,
            end_time: None,
            organizer: None,
            participants: None,
        };

        let event = FeishuMeetingCollector::convert_meeting(meeting).unwrap();
        assert_eq!(event.id, "mtg-002");
        assert_eq!(event.content["topic"], serde_json::Value::Null);
        assert_eq!(event.content["organizer"], serde_json::Value::Null);
    }
}
