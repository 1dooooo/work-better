//! 飞书多维表格采集器

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use wb_core::error::Result;
use wb_core::event::{Confidence, Event, EventType, Source};

use crate::runner;
use crate::traits::{HealthStatus, Collector};

/// lark-cli base +base-block-list 响应
#[derive(Debug, Deserialize)]
struct LarkBitableResponse {
    data: Option<LarkBitableData>,
}

#[derive(Debug, Deserialize)]
struct LarkBitableData {
    items: Option<Vec<LarkBitable>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LarkBitable {
    app_token: Option<String>,
    name: Option<String>,
    owner: Option<String>,
    edit_time: Option<String>,
    create_time: Option<String>,
}

/// 飞书多维表格采集器
pub struct FeishuBitableCollector;

impl FeishuBitableCollector {
    /// 将 lark-cli 多维表格转换为 Event
    fn convert_bitable(bitable: LarkBitable) -> Option<Event> {
        let app_token = bitable.app_token.clone()?;

        let raw_payload = serde_json::to_string(&bitable).ok()?;

        let content = serde_json::json!({
            "app_token": bitable.app_token,
            "name": bitable.name,
            "owner": bitable.owner,
            "edit_time": bitable.edit_time,
        });

        let mut event = Event::new(
            Source::FeishuBitable,
            Confidence::Medium,
            EventType::DocumentChange,
            content,
            raw_payload,
        );

        // 使用 app_token 作为事件 id（保证幂等）
        event.id = app_token;

        Some(event)
    }
}

#[async_trait]
impl Collector for FeishuBitableCollector {
    fn id(&self) -> &str {
        "feishu-bitable"
    }

    fn name(&self) -> &str {
        "飞书多维表格采集器"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn collect(&self) -> Result<Vec<Event>> {
        // base +base-block-list 需要 --app-token，使用默认空值
        let default_token = "";
        let args = vec!["base", "+base-block-list", "--app-token", default_token, "--format", "json"];

        let response: LarkBitableResponse = runner::execute_json("lark-cli", &args)?;

        let items = response.data.and_then(|d| d.items).unwrap_or_default();

        let events: Vec<Event> = items
            .into_iter()
            .filter_map(Self::convert_bitable)
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
    fn test_convert_bitable() {
        let bitable = LarkBitable {
            app_token: Some("bascn001".to_string()),
            name: Some("项目跟踪表".to_string()),
            owner: Some("user-001".to_string()),
            edit_time: Some("1717689600".to_string()),
            create_time: Some("1717600000".to_string()),
        };

        let event = FeishuBitableCollector::convert_bitable(bitable);
        assert!(event.is_some());

        let event = event.unwrap();
        assert_eq!(event.id, "bascn001");
        assert_eq!(event.source, Source::FeishuBitable);
        assert_eq!(event.source_confidence, Confidence::Medium);
        assert_eq!(event.event_type, EventType::DocumentChange);
        assert_eq!(event.content["name"], "项目跟踪表");
        assert_eq!(event.content["owner"], "user-001");
    }

    #[test]
    fn test_convert_bitable_no_id_returns_none() {
        let bitable = LarkBitable {
            app_token: None,
            name: Some("无 token 表格".to_string()),
            owner: None,
            edit_time: None,
            create_time: None,
        };

        let event = FeishuBitableCollector::convert_bitable(bitable);
        assert!(event.is_none());
    }

    #[test]
    fn test_convert_bitable_missing_optional_fields() {
        let bitable = LarkBitable {
            app_token: Some("bascn002".to_string()),
            name: None,
            owner: None,
            edit_time: None,
            create_time: None,
        };

        let event = FeishuBitableCollector::convert_bitable(bitable).unwrap();
        assert_eq!(event.id, "bascn002");
        assert_eq!(event.content["name"], serde_json::Value::Null);
        assert_eq!(event.content["owner"], serde_json::Value::Null);
    }
}
