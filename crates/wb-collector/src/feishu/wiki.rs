//! 飞书知识库采集器

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use wb_core::error::Result;
use wb_core::event::{Confidence, Event, EventType, Source};

use crate::runner;
use crate::traits::{HealthStatus, Collector};

/// lark-cli 知识库列表响应
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
    wiki_token: Option<String>,
    title: Option<String>,
    owner: Option<String>,
    edit_time: Option<String>,
    create_time: Option<String>,
}

/// 飞书知识库采集器
pub struct FeishuWikiCollector;

impl FeishuWikiCollector {
    /// 将 lark-cli 知识库页面转换为 Event
    fn convert_wiki(wiki: LarkWiki) -> Option<Event> {
        let wiki_token = wiki.wiki_token.clone()?;

        let raw_payload = serde_json::to_string(&wiki).ok()?;

        let content = serde_json::json!({
            "wiki_token": wiki.wiki_token,
            "title": wiki.title,
            "owner": wiki.owner,
            "edit_time": wiki.edit_time,
        });

        let mut event = Event::new(
            Source::FeishuWiki,
            Confidence::Medium,
            EventType::DocumentChange,
            content,
            raw_payload,
        );

        // 使用 wiki_token 作为事件 id（保证幂等）
        event.id = wiki_token;

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
        let args = vec!["wiki", "list"];

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
            wiki_token: Some("wiki-001".to_string()),
            title: Some("产品设计规范".to_string()),
            owner: Some("user-001".to_string()),
            edit_time: Some("1717689600".to_string()),
            create_time: Some("1717600000".to_string()),
        };

        let event = FeishuWikiCollector::convert_wiki(wiki);
        assert!(event.is_some());

        let event = event.unwrap();
        assert_eq!(event.id, "wiki-001");
        assert_eq!(event.source, Source::FeishuWiki);
        assert_eq!(event.source_confidence, Confidence::Medium);
        assert_eq!(event.event_type, EventType::DocumentChange);
        assert_eq!(event.content["title"], "产品设计规范");
        assert_eq!(event.content["owner"], "user-001");
    }

    #[test]
    fn test_convert_wiki_no_id_returns_none() {
        let wiki = LarkWiki {
            wiki_token: None,
            title: Some("无 token 知识库".to_string()),
            owner: None,
            edit_time: None,
            create_time: None,
        };

        let event = FeishuWikiCollector::convert_wiki(wiki);
        assert!(event.is_none());
    }

    #[test]
    fn test_convert_wiki_missing_optional_fields() {
        let wiki = LarkWiki {
            wiki_token: Some("wiki-002".to_string()),
            title: None,
            owner: None,
            edit_time: None,
            create_time: None,
        };

        let event = FeishuWikiCollector::convert_wiki(wiki).unwrap();
        assert_eq!(event.id, "wiki-002");
        assert_eq!(event.content["title"], serde_json::Value::Null);
        assert_eq!(event.content["owner"], serde_json::Value::Null);
    }
}
