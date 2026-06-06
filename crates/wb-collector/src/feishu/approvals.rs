//! 飞书审批采集器

use serde::{Deserialize, Serialize};
use wb_core::error::Result;
use wb_core::event::{Confidence, Event, EventType, Source};

use crate::runner;

/// lark-cli 审批列表响应
#[derive(Debug, Deserialize)]
struct LarkApprovalsResponse {
    data: Option<LarkApprovalsData>,
}

#[derive(Debug, Deserialize)]
struct LarkApprovalsData {
    items: Option<Vec<LarkApproval>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LarkApproval {
    approval_id: Option<String>,
    title: Option<String>,
    status: Option<String>,
    submitter: Option<String>,
    create_time: Option<String>,
    update_time: Option<String>,
}

/// 飞书审批采集器
pub struct FeishuApprovalsCollector;

impl FeishuApprovalsCollector {
    /// 采集审批列表
    ///
    /// # Arguments
    /// * `limit` - 最大采集数量
    pub fn collect(limit: u32) -> Result<Vec<Event>> {
        let limit_str = limit.to_string();
        let args = vec!["approval", "list", "--page-size", &limit_str];

        let response: LarkApprovalsResponse = runner::execute_json("lark-cli", &args)?;

        let items = response.data.and_then(|d| d.items).unwrap_or_default();

        let events: Vec<Event> = items
            .into_iter()
            .filter_map(Self::convert_approval)
            .collect();

        Ok(events)
    }

    /// 将 lark-cli 审批转换为 Event
    fn convert_approval(approval: LarkApproval) -> Option<Event> {
        let approval_id = approval.approval_id.clone()?;

        let raw_payload = serde_json::to_string(&approval).ok()?;

        let content = serde_json::json!({
            "approval_id": approval.approval_id,
            "title": approval.title,
            "status": approval.status,
            "submitter": approval.submitter,
        });

        let mut event = Event::new(
            Source::FeishuApproval,
            Confidence::Medium,
            EventType::Approval,
            content,
            raw_payload,
        );

        // 使用 approval_id 作为事件 id（保证幂等）
        event.id = approval_id;

        Some(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_approval() {
        let approval = LarkApproval {
            approval_id: Some("apv-001".to_string()),
            title: Some("请假审批".to_string()),
            status: Some("approved".to_string()),
            submitter: Some("user-001".to_string()),
            create_time: Some("1717689600".to_string()),
            update_time: Some("1717689700".to_string()),
        };

        let event = FeishuApprovalsCollector::convert_approval(approval);
        assert!(event.is_some());

        let event = event.unwrap();
        assert_eq!(event.id, "apv-001");
        assert_eq!(event.source, Source::FeishuApproval);
        assert_eq!(event.source_confidence, Confidence::Medium);
        assert_eq!(event.event_type, EventType::Approval);
        assert_eq!(event.content["title"], "请假审批");
        assert_eq!(event.content["status"], "approved");
        assert_eq!(event.content["submitter"], "user-001");
    }

    #[test]
    fn test_convert_approval_no_id_returns_none() {
        let approval = LarkApproval {
            approval_id: None,
            title: Some("无 ID 审批".to_string()),
            status: None,
            submitter: None,
            create_time: None,
            update_time: None,
        };

        let event = FeishuApprovalsCollector::convert_approval(approval);
        assert!(event.is_none());
    }

    #[test]
    fn test_convert_approval_missing_optional_fields() {
        let approval = LarkApproval {
            approval_id: Some("apv-002".to_string()),
            title: None,
            status: None,
            submitter: None,
            create_time: None,
            update_time: None,
        };

        let event = FeishuApprovalsCollector::convert_approval(approval).unwrap();
        assert_eq!(event.id, "apv-002");
        assert_eq!(event.content["title"], serde_json::Value::Null);
    }
}
