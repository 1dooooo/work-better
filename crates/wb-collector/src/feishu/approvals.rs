//! 飞书审批采集器

use serde::{Deserialize, Serialize};
use wb_core::error::Result;
use wb_core::event::{Confidence, Event, EventType, Source};

use crate::runner;

/// lark-cli approval instances initiated 响应
#[derive(Debug, Deserialize)]
struct LarkApprovalsResponse {
    data: Option<LarkApprovalsData>,
}

/// 审批实例数据（注意：使用 instances 而非 items）
#[derive(Debug, Deserialize)]
struct LarkApprovalsData {
    instances: Option<Vec<LarkApproval>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LarkApproval {
    instance_code: Option<String>,
    definition_name: Option<String>,
    instance_status: Option<String>,
    initiator_name: Option<String>,
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
        let params = format!(r#"{{"page_size":{}}}"#, limit);
        let args = vec![
            "approval",
            "instances",
            "initiated",
            "--params",
            &params,
            "--format",
            "json",
        ];

        let response: LarkApprovalsResponse = runner::execute_json("lark-cli", &args)?;

        let items = response.data.and_then(|d| d.instances).unwrap_or_default();

        let events: Vec<Event> = items
            .into_iter()
            .filter_map(Self::convert_approval)
            .collect();

        Ok(events)
    }

    /// 将 lark-cli 审批转换为 Event
    fn convert_approval(approval: LarkApproval) -> Option<Event> {
        let instance_code = approval.instance_code.clone()?;

        let raw_payload = serde_json::to_string(&approval).ok()?;

        let content = serde_json::json!({
            "instance_code": approval.instance_code,
            "definition_name": approval.definition_name,
            "instance_status": approval.instance_status,
            "initiator_name": approval.initiator_name,
        });

        let mut event = Event::new(
            Source::FeishuApproval,
            Confidence::Medium,
            EventType::Approval,
            content,
            raw_payload,
        );

        // 使用 instance_code 作为事件 id（保证幂等）
        event.id = instance_code;

        Some(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_approval() {
        let approval = LarkApproval {
            instance_code: Some("apv-001".to_string()),
            definition_name: Some("请假审批".to_string()),
            instance_status: Some("approved".to_string()),
            initiator_name: Some("user-001".to_string()),
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
        assert_eq!(event.content["definition_name"], "请假审批");
        assert_eq!(event.content["instance_status"], "approved");
        assert_eq!(event.content["initiator_name"], "user-001");
    }

    #[test]
    fn test_convert_approval_no_id_returns_none() {
        let approval = LarkApproval {
            instance_code: None,
            definition_name: Some("无 ID 审批".to_string()),
            instance_status: None,
            initiator_name: None,
            create_time: None,
            update_time: None,
        };

        let event = FeishuApprovalsCollector::convert_approval(approval);
        assert!(event.is_none());
    }

    #[test]
    fn test_convert_approval_missing_optional_fields() {
        let approval = LarkApproval {
            instance_code: Some("apv-002".to_string()),
            definition_name: None,
            instance_status: None,
            initiator_name: None,
            create_time: None,
            update_time: None,
        };

        let event = FeishuApprovalsCollector::convert_approval(approval).unwrap();
        assert_eq!(event.id, "apv-002");
        assert_eq!(event.content["definition_name"], serde_json::Value::Null);
    }
}
