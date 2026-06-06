//! 模型适配器 trait 定义及实现

use serde::{Deserialize, Serialize};
use wb_core::event::Event;

/// 分类结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Classification {
    /// 分类标签（如 task, meeting, communication 等）
    pub category: String,
    /// 置信度 0-1
    pub confidence: f64,
    /// 理由
    pub reasoning: String,
}

/// 提取结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extraction {
    /// 标题
    pub title: String,
    /// 摘要
    pub summary: String,
    /// 详细内容（markdown）
    pub detail: String,
    /// 涉及的人
    pub people: Vec<String>,
    /// 标签
    pub tags: Vec<String>,
    /// 项目
    pub project: Option<String>,
    /// 置信度
    pub confidence: f64,
}

/// 模型适配器异步 trait
#[async_trait::async_trait]
pub trait ModelAdapter: Send + Sync {
    /// 对事件进行分类
    async fn classify(&self, event: &Event) -> wb_core::error::Result<Classification>;

    /// 从事件中提取结构化信息
    async fn extract(&self, event: &Event) -> wb_core::error::Result<Extraction>;

    /// 生成摘要
    async fn summarize(&self, text: &str) -> wb_core::error::Result<String>;
}

/// Mock 适配器 —— 用于测试，返回固定响应
pub struct MockAdapter {
    /// 固定分类结果
    pub classification: Classification,
    /// 固定提取结果
    pub extraction: Extraction,
    /// 固定摘要文本
    pub summary: String,
    /// 模拟的模型名称
    pub model_name: String,
}

impl MockAdapter {
    pub fn new() -> Self {
        Self {
            classification: Classification {
                category: "task".to_string(),
                confidence: 0.9,
                reasoning: "mock classification".to_string(),
            },
            extraction: Extraction {
                title: "Mock Title".to_string(),
                summary: "Mock summary".to_string(),
                detail: "Mock detail".to_string(),
                people: vec![],
                tags: vec!["mock".to_string()],
                project: None,
                confidence: 0.95,
            },
            summary: "Mock summary text".to_string(),
            model_name: "mock-model".to_string(),
        }
    }

    /// 设置固定的分类结果
    pub fn with_classification(mut self, classification: Classification) -> Self {
        self.classification = classification;
        self
    }

    /// 设置固定的提取结果
    pub fn with_extraction(mut self, extraction: Extraction) -> Self {
        self.extraction = extraction;
        self
    }

    /// 设置固定的摘要
    pub fn with_summary(mut self, summary: String) -> Self {
        self.summary = summary;
        self
    }

    /// 设置模型名称
    pub fn with_model_name(mut self, name: String) -> Self {
        self.model_name = name;
        self
    }
}

impl Default for MockAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ModelAdapter for MockAdapter {
    async fn classify(&self, _event: &Event) -> wb_core::error::Result<Classification> {
        Ok(self.classification.clone())
    }

    async fn extract(&self, _event: &Event) -> wb_core::error::Result<Extraction> {
        Ok(self.extraction.clone())
    }

    async fn summarize(&self, _text: &str) -> wb_core::error::Result<String> {
        Ok(self.summary.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wb_core::event::{Confidence, EventType, Source};

    fn make_test_event() -> Event {
        Event::new(
            Source::FeishuMessage,
            Confidence::High,
            EventType::Message,
            serde_json::json!({"text": "test event content"}),
            "raw payload".to_string(),
        )
    }

    #[tokio::test]
    async fn test_mock_adapter_classify() {
        let adapter = MockAdapter::new();
        let event = make_test_event();
        let result = adapter.classify(&event).await.unwrap();
        assert_eq!(result.category, "task");
        assert_eq!(result.confidence, 0.9);
    }

    #[tokio::test]
    async fn test_mock_adapter_extract() {
        let adapter = MockAdapter::new();
        let event = make_test_event();
        let result = adapter.extract(&event).await.unwrap();
        assert_eq!(result.title, "Mock Title");
        assert_eq!(result.confidence, 0.95);
    }

    #[tokio::test]
    async fn test_mock_adapter_summarize() {
        let adapter = MockAdapter::new();
        let result = adapter.summarize("some text").await.unwrap();
        assert_eq!(result, "Mock summary text");
    }

    #[tokio::test]
    async fn test_mock_adapter_with_custom_classification() {
        let adapter = MockAdapter::new().with_classification(Classification {
            category: "meeting".to_string(),
            confidence: 0.75,
            reasoning: "custom".to_string(),
        });
        let event = make_test_event();
        let result = adapter.classify(&event).await.unwrap();
        assert_eq!(result.category, "meeting");
        assert_eq!(result.confidence, 0.75);
    }

    #[tokio::test]
    async fn test_mock_adapter_with_custom_extraction() {
        let adapter = MockAdapter::new().with_extraction(Extraction {
            title: "Custom Title".to_string(),
            summary: "Custom summary".to_string(),
            detail: "Custom detail".to_string(),
            people: vec!["Alice".to_string()],
            tags: vec!["custom".to_string()],
            project: Some("ProjectX".to_string()),
            confidence: 0.88,
        });
        let event = make_test_event();
        let result = adapter.extract(&event).await.unwrap();
        assert_eq!(result.title, "Custom Title");
        assert_eq!(result.people, vec!["Alice".to_string()]);
        assert_eq!(result.project, Some("ProjectX".to_string()));
    }
}
