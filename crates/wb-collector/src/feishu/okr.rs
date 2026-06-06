//! 飞书 OKR 采集器

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use wb_core::error::Result;
use wb_core::event::{Confidence, Event, EventType, Source};

use crate::runner;
use crate::traits::{HealthStatus, Collector};

/// lark-cli okr +cycle-list 响应
#[derive(Debug, Deserialize)]
struct LarkOkrResponse {
    data: Option<LarkOkrData>,
}

#[derive(Debug, Deserialize)]
struct LarkOkrData {
    items: Option<Vec<LarkOkr>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LarkOkr {
    okr_id: Option<String>,
    title: Option<String>,
    owner: Option<String>,
    progress: Option<f64>,
    period: Option<String>,
    update_time: Option<String>,
}

/// 飞书 OKR 采集器
pub struct FeishuOkrCollector;

impl FeishuOkrCollector {
    /// 将 lark-cli OKR 转换为 Event
    fn convert_okr(okr: LarkOkr) -> Option<Event> {
        let okr_id = okr.okr_id.clone()?;

        let raw_payload = serde_json::to_string(&okr).ok()?;

        let content = serde_json::json!({
            "okr_id": okr.okr_id,
            "title": okr.title,
            "owner": okr.owner,
            "progress": okr.progress,
            "period": okr.period,
        });

        let mut event = Event::new(
            Source::FeishuOkr,
            Confidence::Medium,
            EventType::TaskUpdate,
            content,
            raw_payload,
        );

        // 使用 okr_id 作为事件 id（保证幂等）
        event.id = okr_id;

        Some(event)
    }
}

#[async_trait]
impl Collector for FeishuOkrCollector {
    fn id(&self) -> &str {
        "feishu-okr"
    }

    fn name(&self) -> &str {
        "飞书 OKR 采集器"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn collect(&self) -> Result<Vec<Event>> {
        // okr +cycle-list 需要 --user-id，使用默认空值
        let default_user_id = "";
        let args = vec!["okr", "+cycle-list", "--user-id", default_user_id, "--format", "json"];

        let response: LarkOkrResponse = runner::execute_json("lark-cli", &args)?;

        let items = response.data.and_then(|d| d.items).unwrap_or_default();

        let events: Vec<Event> = items
            .into_iter()
            .filter_map(Self::convert_okr)
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
    fn test_convert_okr() {
        let okr = LarkOkr {
            okr_id: Some("okr-001".to_string()),
            title: Some("Q2 产品目标".to_string()),
            owner: Some("user-001".to_string()),
            progress: Some(0.75),
            period: Some("2026-Q2".to_string()),
            update_time: Some("1717689600".to_string()),
        };

        let event = FeishuOkrCollector::convert_okr(okr);
        assert!(event.is_some());

        let event = event.unwrap();
        assert_eq!(event.id, "okr-001");
        assert_eq!(event.source, Source::FeishuOkr);
        assert_eq!(event.source_confidence, Confidence::Medium);
        assert_eq!(event.event_type, EventType::TaskUpdate);
        assert_eq!(event.content["title"], "Q2 产品目标");
        assert_eq!(event.content["progress"], 0.75);
    }

    #[test]
    fn test_convert_okr_no_id_returns_none() {
        let okr = LarkOkr {
            okr_id: None,
            title: Some("无 ID OKR".to_string()),
            owner: None,
            progress: None,
            period: None,
            update_time: None,
        };

        let event = FeishuOkrCollector::convert_okr(okr);
        assert!(event.is_none());
    }

    #[test]
    fn test_convert_okr_missing_optional_fields() {
        let okr = LarkOkr {
            okr_id: Some("okr-002".to_string()),
            title: None,
            owner: None,
            progress: None,
            period: None,
            update_time: None,
        };

        let event = FeishuOkrCollector::convert_okr(okr).unwrap();
        assert_eq!(event.id, "okr-002");
        assert_eq!(event.content["title"], serde_json::Value::Null);
        assert_eq!(event.content["progress"], serde_json::Value::Null);
    }
}
