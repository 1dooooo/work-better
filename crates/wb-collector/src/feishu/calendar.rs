//! 飞书日历采集器

use serde::{Deserialize, Serialize};
use wb_core::error::Result;
use wb_core::event::{Confidence, Event, EventType, Source};

use crate::runner;

/// lark-cli 日历事件列表响应（calendar +agenda 格式）
#[derive(Debug, Deserialize)]
struct LarkCalendarResponse {
    ok: Option<bool>,
    data: Option<Vec<LarkCalendarEvent>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LarkCalendarEvent {
    event_id: Option<String>,
    summary: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
    attendees: Option<Vec<String>>,
}

/// 飞书日历采集器
pub struct FeishuCalendarCollector;

impl FeishuCalendarCollector {
    /// 采集日历事件列表
    ///
    /// # Arguments
    /// * `limit` - 最大采集数量
    pub fn collect(_limit: u32) -> Result<Vec<Event>> {
        let args = vec!["calendar", "+agenda"];

        let response: LarkCalendarResponse = runner::execute_json("lark-cli", &args)?;

        let items = response.data.unwrap_or_default();

        let events: Vec<Event> = items
            .into_iter()
            .filter_map(Self::convert_event)
            .collect();

        Ok(events)
    }

    /// 将 lark-cli 日历事件转换为 Event
    fn convert_event(evt: LarkCalendarEvent) -> Option<Event> {
        let event_id = evt.event_id.clone()?;

        let raw_payload = serde_json::to_string(&evt).ok()?;

        let content = serde_json::json!({
            "event_id": evt.event_id,
            "summary": evt.summary,
            "start_time": evt.start_time,
            "end_time": evt.end_time,
            "attendees": evt.attendees,
        });

        let mut event = Event::new(
            Source::FeishuCalendar,
            Confidence::Medium,
            EventType::CalendarEvent,
            content,
            raw_payload,
        );

        // 使用 event_id 作为事件 id（保证幂等）
        event.id = event_id;

        Some(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_calendar_event() {
        let evt = LarkCalendarEvent {
            event_id: Some("cal-001".to_string()),
            summary: Some("周会".to_string()),
            start_time: Some("1717689600".to_string()),
            end_time: Some("1717693200".to_string()),
            attendees: Some(vec!["user-001".to_string(), "user-002".to_string()]),
        };

        let event = FeishuCalendarCollector::convert_event(evt);
        assert!(event.is_some());

        let event = event.unwrap();
        assert_eq!(event.id, "cal-001");
        assert_eq!(event.source, Source::FeishuCalendar);
        assert_eq!(event.source_confidence, Confidence::Medium);
        assert_eq!(event.event_type, EventType::CalendarEvent);
        assert_eq!(event.content["summary"], "周会");
        assert_eq!(event.content["attendees"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_convert_calendar_event_no_id_returns_none() {
        let evt = LarkCalendarEvent {
            event_id: None,
            summary: Some("无 ID 事件".to_string()),
            start_time: None,
            end_time: None,
            attendees: None,
        };

        let event = FeishuCalendarCollector::convert_event(evt);
        assert!(event.is_none());
    }

    #[test]
    fn test_convert_calendar_event_no_attendees() {
        let evt = LarkCalendarEvent {
            event_id: Some("cal-002".to_string()),
            summary: Some("个人时间".to_string()),
            start_time: Some("1717689600".to_string()),
            end_time: Some("1717693200".to_string()),
            attendees: None,
        };

        let event = FeishuCalendarCollector::convert_event(evt).unwrap();
        assert_eq!(event.id, "cal-002");
        assert_eq!(event.content["attendees"], serde_json::Value::Null);
    }
}
