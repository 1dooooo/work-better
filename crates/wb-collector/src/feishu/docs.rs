//! 飞书文档采集器

use serde::{Deserialize, Serialize};
use wb_core::error::Result;
use wb_core::event::{Confidence, Event, EventType, Source};

use crate::runner;

/// lark-cli docs +search 响应
#[derive(Debug, Deserialize)]
struct LarkDocsResponse {
    data: Option<LarkDocsData>,
}

#[derive(Debug, Deserialize)]
struct LarkDocsData {
    results: Option<Vec<LarkDocResult>>,
}

/// docs +search 返回的单条搜索结果
#[derive(Debug, Deserialize)]
struct LarkDocResult {
    entity_type: Option<String>,
    result_meta: Option<LarkDocResultMeta>,
    title_highlighted: Option<String>,
}

/// 搜索结果元数据
#[derive(Debug, Deserialize)]
struct LarkDocResultMeta {
    token: Option<String>,
    owner_name: Option<String>,
    update_time_iso: Option<String>,
}

/// 用于序列化的扁平文档结构
#[derive(Debug, Serialize)]
struct LarkDoc {
    document_id: String,
    entity_type: Option<String>,
    title: Option<String>,
    owner: Option<String>,
    edit_time: Option<String>,
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
        let args = vec!["docs", "+search", "--query", "", "--page-size", &limit_str, "--format", "json"];

        let response: LarkDocsResponse = runner::execute_json("lark-cli", &args)?;

        let results = response.data.and_then(|d| d.results).unwrap_or_default();

        let events: Vec<Event> = results
            .into_iter()
            .filter_map(Self::convert_doc_result)
            .collect();

        Ok(events)
    }

    /// 将 lark-cli 搜索结果转换为 Event
    fn convert_doc_result(result: LarkDocResult) -> Option<Event> {
        let meta = result.result_meta?;
        let document_id = meta.token?;

        let doc = LarkDoc {
            document_id: document_id.clone(),
            entity_type: result.entity_type.clone(),
            title: result.title_highlighted.clone(),
            owner: meta.owner_name.clone(),
            edit_time: meta.update_time_iso.clone(),
        };

        let raw_payload = serde_json::to_string(&doc).ok()?;

        let content = serde_json::json!({
            "document_id": doc.document_id,
            "entity_type": doc.entity_type,
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
    fn test_convert_doc_result() {
        let result = LarkDocResult {
            entity_type: Some("docx".to_string()),
            result_meta: Some(LarkDocResultMeta {
                token: Some("doc-001".to_string()),
                owner_name: Some("user-001".to_string()),
                update_time_iso: Some("2024-06-06T12:00:00Z".to_string()),
            }),
            title_highlighted: Some("设计文档".to_string()),
        };

        let event = FeishuDocsCollector::convert_doc_result(result);
        assert!(event.is_some());

        let event = event.unwrap();
        assert_eq!(event.id, "doc-001");
        assert_eq!(event.source, Source::FeishuDoc);
        assert_eq!(event.source_confidence, Confidence::Medium);
        assert_eq!(event.event_type, EventType::DocumentChange);
        assert_eq!(event.content["title"], "设计文档");
        assert_eq!(event.content["owner"], "user-001");
        assert_eq!(event.content["entity_type"], "docx");
    }

    #[test]
    fn test_convert_doc_result_no_token_returns_none() {
        let result = LarkDocResult {
            entity_type: Some("docx".to_string()),
            result_meta: Some(LarkDocResultMeta {
                token: None,
                owner_name: None,
                update_time_iso: None,
            }),
            title_highlighted: Some("无 token 文档".to_string()),
        };

        let event = FeishuDocsCollector::convert_doc_result(result);
        assert!(event.is_none());
    }

    #[test]
    fn test_convert_doc_result_no_meta_returns_none() {
        let result = LarkDocResult {
            entity_type: Some("docx".to_string()),
            result_meta: None,
            title_highlighted: Some("无元数据文档".to_string()),
        };

        let event = FeishuDocsCollector::convert_doc_result(result);
        assert!(event.is_none());
    }

    #[test]
    fn test_convert_doc_result_missing_optional_fields() {
        let result = LarkDocResult {
            entity_type: None,
            result_meta: Some(LarkDocResultMeta {
                token: Some("doc-002".to_string()),
                owner_name: None,
                update_time_iso: None,
            }),
            title_highlighted: None,
        };

        let event = FeishuDocsCollector::convert_doc_result(result).unwrap();
        assert_eq!(event.id, "doc-002");
        assert_eq!(event.content["title"], serde_json::Value::Null);
        assert_eq!(event.content["owner"], serde_json::Value::Null);
    }
}
