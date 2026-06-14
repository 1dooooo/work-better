//! OpenAI 兼容 API 适配器
//!
//! 支持 OpenAI Chat Completions API 格式，兼容第三方 OpenAI-compatible 端点。

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::adapter::{Classification, Extraction, ModelAdapter};
use crate::config::ModelConfig;

/// OpenAI Chat Completions 请求
#[derive(Serialize)]
struct ChatRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

/// OpenAI Chat Completions 响应
#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

/// OpenAI 兼容模型适配器
pub struct OpenAIAdapter {
    config: ModelConfig,
    client: Client,
}

impl OpenAIAdapter {
    pub fn new(config: ModelConfig) -> Self {
        let client = Client::builder()
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { config, client }
    }

    /// 调用 OpenAI Chat Completions API
    async fn call_chat(&self, prompt: &str) -> wb_core::error::Result<String> {
        let url = format!("{}/v1/chat/completions", self.config.base_url);

        let request = ChatRequest {
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens,
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| wb_core::error::WbError::Ai(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(wb_core::error::WbError::Ai(format!("API error {}", status)));
        }

        let result: ChatResponse = response
            .json()
            .await
            .map_err(|e| wb_core::error::WbError::Ai(format!("Failed to parse response: {}", e)))?;

        result
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| wb_core::error::WbError::Ai("Empty response".to_string()))
    }
}

#[async_trait::async_trait]
impl ModelAdapter for OpenAIAdapter {
    async fn classify(
        &self,
        event: &wb_core::event::Event,
    ) -> wb_core::error::Result<Classification> {
        let prompt = format!(
            r#"你是一个工作事件分类器。请对以下事件进行分类。

事件类型: {:?}
来源: {:?}
内容: {}

请以 JSON 格式返回：
{{"category": "task|meeting|communication|research|review|planning|document|decision", "confidence": 0.0-1.0, "reasoning": "理由"}}

只返回 JSON，不要其他内容。"#,
            event.event_type, event.source, event.content
        );

        let text = self.call_chat(&prompt).await?;
        let json_str = crate::anthropic::extract_json(&text);
        serde_json::from_str::<Classification>(json_str).map_err(|e| {
            wb_core::error::WbError::Ai(format!("Failed to parse classification: {}", e))
        })
    }

    async fn extract(&self, event: &wb_core::event::Event) -> wb_core::error::Result<Extraction> {
        // 从 event.content 中提取 task_context（如果存在）
        let task_context_section = crate::anthropic::build_task_context_section(&event.content);

        let prompt = format!(
            r#"从以下工作事件中提取结构化信息。

事件类型: {:?}
来源: {:?}
内容: {}
原始数据: {}
{task_context_section}

请以 JSON 格式返回：
{{
  "title": "简洁标题",
  "summary": "一句话摘要",
  "detail": "详细内容（markdown格式）",
  "people": ["人名列表"],
  "tags": ["标签列表"],
  "project": "项目名或null",
  "due_date": "截止时间（如'明天10点'、'下周五'、'2024-01-15'），无则null",
  "confidence": 0.0-1.0,
  "is_status_update": false,
  "related_task_id": null
}}

提取规则：
- title 应反映事件的核心动作或意图，而非字面复述
- 如果事件描述了某个任务的进展、完成或状态变更（如"已完成"、"开始做"、"推迟到"），
  title 应以该任务为核心命名，而非以状态变更为核心
- 如果提供了已有任务列表，请判断当前消息是否是对某个已有任务的状态更新：
  - 如果是状态更新，设置 is_status_update=true，related_task_id=对应任务的id，title=""
  - 如果是全新任务，设置 is_status_update=false，related_task_id=null

只返回 JSON，不要其他内容。"#,
            event.event_type, event.source, event.content, event.raw_payload
        );

        let text = self.call_chat(&prompt).await?;
        let json_str = crate::anthropic::extract_json(&text);
        serde_json::from_str::<Extraction>(json_str)
            .map_err(|e| wb_core::error::WbError::Ai(format!("Failed to parse extraction: {}", e)))
    }

    async fn summarize(&self, text: &str) -> wb_core::error::Result<String> {
        let prompt = format!(
            r#"请对以下内容生成简洁的一句话摘要（不超过100字）：

{}"#,
            text
        );

        self.call_chat(&prompt).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_config_creation() {
        let config = ModelConfig::openai("test-key".to_string(), None);
        let adapter = OpenAIAdapter::new(config);
        assert_eq!(adapter.config.model, "gpt-4o");
        assert_eq!(adapter.config.base_url, "https://api.openai.com");
    }

    #[test]
    fn test_openai_config_custom_base_url() {
        let config = ModelConfig::openai(
            "key".to_string(),
            Some("http://localhost:11434".to_string()),
        );
        let adapter = OpenAIAdapter::new(config);
        assert_eq!(adapter.config.base_url, "http://localhost:11434");
    }

    #[test]
    fn test_openai_config_custom_model() {
        let config = ModelConfig::openai("key".to_string(), None).with_model("gpt-4o-mini");
        let adapter = OpenAIAdapter::new(config);
        assert_eq!(adapter.config.model, "gpt-4o-mini");
    }
}
