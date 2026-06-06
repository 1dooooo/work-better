//! 飞书电子表格采集器

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use wb_core::error::Result;
use wb_core::event::{Confidence, Event, EventType, Source};

use crate::runner;
use crate::traits::{HealthStatus, Collector};

/// lark-cli sheets +workbook-info 响应
#[derive(Debug, Deserialize)]
struct LarkSpreadsheetsResponse {
    data: Option<LarkSpreadsheetsData>,
}

#[derive(Debug, Deserialize)]
struct LarkSpreadsheetsData {
    items: Option<Vec<LarkSpreadsheet>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LarkSpreadsheet {
    spreadsheet_token: Option<String>,
    title: Option<String>,
    owner: Option<String>,
    edit_time: Option<String>,
    create_time: Option<String>,
}

/// 飞书电子表格采集器
pub struct FeishuSpreadsheetCollector;

impl FeishuSpreadsheetCollector {
    /// 将 lark-cli 电子表格转换为 Event
    fn convert_spreadsheet(sheet: LarkSpreadsheet) -> Option<Event> {
        let spreadsheet_token = sheet.spreadsheet_token.clone()?;

        let raw_payload = serde_json::to_string(&sheet).ok()?;

        let content = serde_json::json!({
            "spreadsheet_token": sheet.spreadsheet_token,
            "title": sheet.title,
            "owner": sheet.owner,
            "edit_time": sheet.edit_time,
        });

        let mut event = Event::new(
            Source::FeishuSheet,
            Confidence::Medium,
            EventType::DocumentChange,
            content,
            raw_payload,
        );

        // 使用 spreadsheet_token 作为事件 id（保证幂等）
        event.id = spreadsheet_token;

        Some(event)
    }
}

#[async_trait]
impl Collector for FeishuSpreadsheetCollector {
    fn id(&self) -> &str {
        "feishu-spreadsheet"
    }

    fn name(&self) -> &str {
        "飞书电子表格采集器"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn collect(&self) -> Result<Vec<Event>> {
        // sheets +workbook-info 需要 --url，使用默认空值
        let default_url = "";
        let args = vec!["sheets", "+workbook-info", "--url", default_url, "--format", "json"];

        let response: LarkSpreadsheetsResponse = runner::execute_json("lark-cli", &args)?;

        let items = response.data.and_then(|d| d.items).unwrap_or_default();

        let events: Vec<Event> = items
            .into_iter()
            .filter_map(Self::convert_spreadsheet)
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
    fn test_convert_spreadsheet() {
        let sheet = LarkSpreadsheet {
            spreadsheet_token: Some("sht001".to_string()),
            title: Some("Q2 数据表".to_string()),
            owner: Some("user-001".to_string()),
            edit_time: Some("1717689600".to_string()),
            create_time: Some("1717600000".to_string()),
        };

        let event = FeishuSpreadsheetCollector::convert_spreadsheet(sheet);
        assert!(event.is_some());

        let event = event.unwrap();
        assert_eq!(event.id, "sht001");
        assert_eq!(event.source, Source::FeishuSheet);
        assert_eq!(event.source_confidence, Confidence::Medium);
        assert_eq!(event.event_type, EventType::DocumentChange);
        assert_eq!(event.content["title"], "Q2 数据表");
        assert_eq!(event.content["owner"], "user-001");
    }

    #[test]
    fn test_convert_spreadsheet_no_id_returns_none() {
        let sheet = LarkSpreadsheet {
            spreadsheet_token: None,
            title: Some("无 token 表格".to_string()),
            owner: None,
            edit_time: None,
            create_time: None,
        };

        let event = FeishuSpreadsheetCollector::convert_spreadsheet(sheet);
        assert!(event.is_none());
    }

    #[test]
    fn test_convert_spreadsheet_missing_optional_fields() {
        let sheet = LarkSpreadsheet {
            spreadsheet_token: Some("sht002".to_string()),
            title: None,
            owner: None,
            edit_time: None,
            create_time: None,
        };

        let event = FeishuSpreadsheetCollector::convert_spreadsheet(sheet).unwrap();
        assert_eq!(event.id, "sht002");
        assert_eq!(event.content["title"], serde_json::Value::Null);
        assert_eq!(event.content["owner"], serde_json::Value::Null);
    }
}
