//! 飞书文档采集器

use serde::{Deserialize, Serialize};
use wb_core::error::Result;
use wb_core::event::{Confidence, Event, EventType, Source};

use crate::runner;

/// lark-cli 文档列表响应
#[derive(Debug, Deserialize)]
struct LarkDocsResponse {
    data: Option<LarkDocsData>,
}

#[derive(Debug, Deserialize)]
struct LarkDocsData {
    items: Option<Vec<LarkDoc>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LarkDoc {
    document_id: Option<String>,
    title: Option<String>,
    owner: Option<String>,
    edit_time: Option<String>,
    create_time: Option<String>,
}

/// 飞书文档采集器
pub struct FeishuDocsCollector;

impl FeishuDocsCollector {
    /// 采集文档列表
    ///
    /// # Arguments
    /// * `limit` - 最大采集数量
    pub fn collect(limit: u32) -> Result<Vec<Event>> {
        let limit_str = limit.to_string();
        let args = vec!["docx", "list", "--page-size", &limit_str];

        let response: LarkDocsResponse = runner::execute_json("lark-cli", &args)?;

        let items = response.data.and_then(|d| d.items).unwrap_or_default();

        let events: Vec<Event> = items
            .into_iter()
            .filter_map(|doc| Self::convert_doc(doc))
            .collect();

        Ok(events)
    }

    /// 将 lark-cli 文档转换为 Event
    fn convert_doc(doc: LarkDoc) -> Option<Event> {
        let document_id = doc.document_id.clone()?;

        let raw_payload = serde_json::to_string(&doc).ok()?;

        let content = serde_json::json!({
            "document_id": doc.document_id,
            "title": doc.title,
            "owner": doc.owner,
            "edit_time": doc.edit_time,
        });

        let mut event = Event::new(
            Source::FeishuDoc,
            Confidence::Medium,
            EventType::DocumentChange,
            content,
            raw_payload,
        );

        // 使用 document_id 作为事件 id（保证幂等）
        event.id = document_id;

        Some(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_doc() {
        let doc = LarkDoc {
            document_id: Some("doc-001".to_string()),
            title: Some("设计文档".to_string()),
            owner: Some("user-001".to_string()),
            edit_time: Some("1717689600".to_string()),
            create_time: Some("1717600000".to_string()),
        };

        let event = FeishuDocsCollector::convert_doc(doc);
        assert!(event.is_some());

        let event = event.unwrap();
        assert_eq!(event.id, "doc-001");
        assert_eq!(event.source, Source::FeishuDoc);
        assert_eq!(event.source_confidence, Confidence::Medium);
        assert_eq!(event.event_type, EventType::DocumentChange);
        assert_eq!(event.content["title"], "设计文档");
        assert_eq!(event.content["owner"], "user-001");
    }

    #[test]
    fn test_convert_doc_no_id_returns_none() {
        let doc = LarkDoc {
            document_id: None,
            title: Some("无 ID 文档".to_string()),
            owner: None,
            edit_time: None,
            create_time: None,
        };

        let event = FeishuDocsCollector::convert_doc(doc);
        assert!(event.is_none());
    }

    #[test]
    fn test_convert_doc_missing_optional_fields() {
        let doc = LarkDoc {
            document_id: Some("doc-002".to_string()),
            title: None,
            owner: None,
            edit_time: None,
            create_time: None,
        };

        let event = FeishuDocsCollector::convert_doc(doc).unwrap();
        assert_eq!(event.id, "doc-002");
        assert_eq!(event.content["title"], serde_json::Value::Null);
    }
}
