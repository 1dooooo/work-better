//! 飞书知识库采集器

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use wb_core::error::Result;
use wb_core::event::{Confidence, Event, EventType, Source};

use crate::runner;
use crate::traits::{HealthStatus, Collector};

/// lark-cli wiki spaces list 响应
#[derive(Debug, Deserialize)]
struct LarkWikiResponse {
    data: Option<LarkWikiData>,
}

#[derive(Debug, Deserialize)]
struct LarkWikiData {
    items: Option<Vec<LarkWiki>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LarkWiki {
    space_id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    visibility: Option<String>,
    open_sharing: Option<String>,
}

/// 飞书知识库采集器
pub struct FeishuWikiCollector;

impl FeishuWikiCollector {
    /// 将 lark-cli 知识库页面转换为 Event
    fn convert_wiki(wiki: LarkWiki) -> Option<Event> {
        let space_id = wiki.space_id.clone()?;

        let raw_payload = serde_json::to_string(&wiki).ok()?;

        let content = serde_json::json!({
            "space_id": wiki.space_id,
            "name": wiki.name,
            "description": wiki.description,
            "visibility": wiki.visibility,
        });

        let mut event = Event::new(
            Source::FeishuWiki,
            Confidence::Medium,
            EventType::DocumentChange,
            content,
            raw_payload,
        );

        // 使用 space_id 作为事件 id（保证幂等）
        event.id = space_id;

        Some(event)
    }
}

#[async_trait]
impl Collector for FeishuWikiCollector {
    fn id(&self) -> &str {
        "feishu-wiki"
    }

    fn name(&self) -> &str {
        "飞书知识库采集器"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn collect(&self) -> Result<Vec<Event>> {
        // wiki spaces list --format json
        let args = vec!["wiki", "spaces", "list", "--format", "json"];

        let response: LarkWikiResponse = runner::execute_json("lark-cli", &args)?;

        let items = response.data.and_then(|d| d.items).unwrap_or_default();

        let events: Vec<Event> = items
            .into_iter()
            .filter_map(Self::convert_wiki)
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
    fn test_convert_wiki() {
        let wiki = LarkWiki {
            space_id: Some("wiki-001".to_string()),
            name: Some("产品设计规范".to_string()),
            description: Some("产品设计规范知识库".to_string()),
            visibility: Some("private".to_string()),
            open_sharing: Some("closed".to_string()),
        };

        let event = FeishuWikiCollector::convert_wiki(wiki);
        assert!(event.is_some());

        let event = event.unwrap();
        assert_eq!(event.id, "wiki-001");
        assert_eq!(event.source, Source::FeishuWiki);
        assert_eq!(event.source_confidence, Confidence::Medium);
        assert_eq!(event.event_type, EventType::DocumentChange);
        assert_eq!(event.content["name"], "产品设计规范");
        assert_eq!(event.content["description"], "产品设计规范知识库");
    }

    #[test]
    fn test_convert_wiki_no_id_returns_none() {
        let wiki = LarkWiki {
            space_id: None,
            name: Some("无 ID 知识库".to_string()),
            description: None,
            visibility: None,
            open_sharing: None,
        };

        let event = FeishuWikiCollector::convert_wiki(wiki);
        assert!(event.is_none());
    }

    #[test]
    fn test_convert_wiki_missing_optional_fields() {
        let wiki = LarkWiki {
            space_id: Some("wiki-002".to_string()),
            name: None,
            description: None,
            visibility: None,
            open_sharing: None,
        };

        let event = FeishuWikiCollector::convert_wiki(wiki).unwrap();
        assert_eq!(event.id, "wiki-002");
        assert_eq!(event.content["name"], serde_json::Value::Null);
        assert_eq!(event.content["description"], serde_json::Value::Null);
    }
}
