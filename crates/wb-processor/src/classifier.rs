//! 规则分类器

use wb_core::event::{Confidence, Event, EventType};

/// 处理路由
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingRoute {
    /// 即时处理：高优先级，立即调用 AI 提取
    Instant,
    /// 聚合处理：批量收集后统一处理
    Aggregate,
    /// 模式分析：长周期数据分析
    Pattern,
    /// 直接归档：低价值事件
    Archive,
}

/// 分类器
pub struct Classifier;

impl Classifier {
    /// 对事件进行分类，返回处理路由
    pub fn classify(event: &Event) -> ProcessingRoute {
        // 低置信度 → 归档
        if event.source_confidence == Confidence::Low {
            return ProcessingRoute::Archive;
        }

        match event.event_type {
            // 高价值事件 → 即时处理
            EventType::TaskUpdate | EventType::Approval | EventType::ManualNote => {
                ProcessingRoute::Instant
            }

            // 消息类：检查是否包含 @mention
            EventType::Message => {
                if Self::has_mention(event) {
                    ProcessingRoute::Instant
                } else {
                    ProcessingRoute::Aggregate
                }
            }

            // 文档/浏览/应用活动 → 聚合处理
            EventType::DocumentChange | EventType::Browsing | EventType::AppActivity => {
                ProcessingRoute::Aggregate
            }

            // OKR 变更 → 模式分析
            EventType::OkrUpdate => ProcessingRoute::Pattern,

            // 会议、日历、邮件 → 即时处理
            EventType::Meeting | EventType::CalendarEvent | EventType::Email => {
                ProcessingRoute::Instant
            }
        }
    }

    /// 检查消息内容是否包含 @mention
    fn has_mention(event: &Event) -> bool {
        let content_str = match &event.content {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Object(obj) => {
                // 常见的飞书消息格式：{"text": "...@..."}
                obj.get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            }
            _ => serde_json::to_string(&event.content).unwrap_or_default(),
        };

        content_str.contains('@')
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wb_core::event::{Confidence, Event, EventType, Source};

    fn make_event(
        source: Source,
        confidence: Confidence,
        event_type: EventType,
        content: serde_json::Value,
    ) -> Event {
        Event::new(source, confidence, event_type, content, "{}".to_string())
    }

    #[test]
    fn test_low_confidence_archives() {
        let event = make_event(
            Source::SystemAppSwitch,
            Confidence::Low,
            EventType::AppActivity,
            json!({"app": "Safari"}),
        );
        assert_eq!(Classifier::classify(&event), ProcessingRoute::Archive);
    }

    #[test]
    fn test_task_update_is_instant() {
        let event = make_event(
            Source::FeishuProject,
            Confidence::High,
            EventType::TaskUpdate,
            json!({"status": "done"}),
        );
        assert_eq!(Classifier::classify(&event), ProcessingRoute::Instant);
    }

    #[test]
    fn test_approval_is_instant() {
        let event = make_event(
            Source::FeishuApproval,
            Confidence::High,
            EventType::Approval,
            json!({"approved": true}),
        );
        assert_eq!(Classifier::classify(&event), ProcessingRoute::Instant);
    }

    #[test]
    fn test_manual_note_is_instant() {
        let event = make_event(
            Source::UserCapture,
            Confidence::High,
            EventType::ManualNote,
            json!({"text": "记一下今天的进展"}),
        );
        assert_eq!(Classifier::classify(&event), ProcessingRoute::Instant);
    }

    #[test]
    fn test_message_with_mention_is_instant() {
        let event = make_event(
            Source::FeishuMessage,
            Confidence::Medium,
            EventType::Message,
            json!({"text": "@张三 请review一下这个PR"}),
        );
        assert_eq!(Classifier::classify(&event), ProcessingRoute::Instant);
    }

    #[test]
    fn test_message_without_mention_is_aggregate() {
        let event = make_event(
            Source::FeishuMessage,
            Confidence::Medium,
            EventType::Message,
            json!({"text": "收到了，我看一下"}),
        );
        assert_eq!(Classifier::classify(&event), ProcessingRoute::Aggregate);
    }

    #[test]
    fn test_document_change_is_aggregate() {
        let event = make_event(
            Source::FeishuDoc,
            Confidence::Medium,
            EventType::DocumentChange,
            json!({"doc_id": "doc-001"}),
        );
        assert_eq!(Classifier::classify(&event), ProcessingRoute::Aggregate);
    }

    #[test]
    fn test_browsing_is_aggregate() {
        let event = make_event(
            Source::SystemBrowser,
            Confidence::Medium,
            EventType::Browsing,
            json!({"url": "https://example.com"}),
        );
        assert_eq!(Classifier::classify(&event), ProcessingRoute::Aggregate);
    }

    #[test]
    fn test_okr_update_is_pattern() {
        let event = make_event(
            Source::FeishuOkr,
            Confidence::High,
            EventType::OkrUpdate,
            json!({"okr_id": "okr-001"}),
        );
        assert_eq!(Classifier::classify(&event), ProcessingRoute::Pattern);
    }

    #[test]
    fn test_meeting_is_instant() {
        let event = make_event(
            Source::FeishuMeeting,
            Confidence::High,
            EventType::Meeting,
            json!({"meeting_id": "meet-001"}),
        );
        assert_eq!(Classifier::classify(&event), ProcessingRoute::Instant);
    }

    #[test]
    fn test_email_is_instant() {
        let event = make_event(
            Source::FeishuEmail,
            Confidence::Medium,
            EventType::Email,
            json!({"subject": "Re: 项目进展"}),
        );
        assert_eq!(Classifier::classify(&event), ProcessingRoute::Instant);
    }
}
