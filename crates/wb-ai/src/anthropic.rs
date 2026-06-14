//! Anthropic Messages API 适配器

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::adapter::{Classification, Extraction, ModelAdapter};
use crate::config::ModelConfig;

/// Anthropic API 请求
#[derive(Serialize)]
struct MessagesRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

/// Anthropic API 响应
#[derive(Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

/// Anthropic 模型适配器
pub struct AnthropicAdapter {
    config: ModelConfig,
    client: Client,
}

impl AnthropicAdapter {
    pub fn new(config: ModelConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    /// 调用 Anthropic Messages API
    async fn call_messages(&self, prompt: &str) -> wb_core::error::Result<String> {
        let request = MessagesRequest {
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens,
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let response = self
            .client
            .post(format!("{}/v1/messages", self.config.base_url))
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| wb_core::error::WbError::Ai(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(wb_core::error::WbError::Ai(format!("API error {}", status)));
        }

        let result: MessagesResponse = response
            .json()
            .await
            .map_err(|e| wb_core::error::WbError::Ai(format!("Failed to parse response: {}", e)))?;

        result
            .content
            .first()
            .map(|c| c.text.clone())
            .ok_or_else(|| wb_core::error::WbError::Ai("Empty response".to_string()))
    }
}

#[async_trait::async_trait]
impl ModelAdapter for AnthropicAdapter {
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

        let text = self.call_messages(&prompt).await?;

        // 尝试从响应中提取 JSON
        let json_str = extract_json(&text);
        serde_json::from_str::<Classification>(json_str).map_err(|e| {
            wb_core::error::WbError::Ai(format!("Failed to parse classification: {}", e))
        })
    }

    async fn extract(&self, event: &wb_core::event::Event) -> wb_core::error::Result<Extraction> {
        // 从 event.content 中提取 task_context（如果存在）
        let task_context_section = build_task_context_section(&event.content);

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

        let text = self.call_messages(&prompt).await?;

        let json_str = extract_json(&text);
        serde_json::from_str::<Extraction>(json_str)
            .map_err(|e| wb_core::error::WbError::Ai(format!("Failed to parse extraction: {}", e)))
    }

    async fn summarize(&self, text: &str) -> wb_core::error::Result<String> {
        let prompt = format!(
            r#"请对以下内容生成简洁的一句话摘要（不超过100字）：

{}"#,
            text
        );

        self.call_messages(&prompt).await
    }
}

/// 从 event.content 中提取 task_context 并构建提示词段落
///
/// 如果 content 中包含 "task_context" 字段，将其格式化为已有任务列表。
/// 否则返回空字符串。
pub(crate) fn build_task_context_section(content: &serde_json::Value) -> String {
    let context_array = match content.get("task_context") {
        Some(serde_json::Value::Array(arr)) if !arr.is_empty() => arr,
        _ => return String::new(),
    };

    let mut lines = vec![String::from("\n已有任务列表（请判断当前消息是否为以下任务的状态更新）：")];
    for item in context_array {
        let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("");
        lines.push(format!("- [{}] {} (状态: {})", id, title, status));
    }
    lines.push(String::new());
    lines.join("\n")
}

/// 从响应文本中提取 JSON（处理可能的 markdown 代码块包裹）
pub fn extract_json(text: &str) -> &str {
    let trimmed = text.trim();

    // 处理 ```json ... ``` 格式
    if trimmed.starts_with("```") {
        let start = trimmed.find('{').unwrap_or(0);
        let end = trimmed.rfind('}').map(|i| i + 1).unwrap_or(trimmed.len());
        return &trimmed[start..end];
    }

    // 直接找 JSON 对象
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            return &trimmed[start..=end];
        }
    }

    trimmed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_from_code_block() {
        let text = r#"```json
{"category": "task", "confidence": 0.9, "reasoning": "test"}
```"#;
        let json = extract_json(text);
        let result: Classification = serde_json::from_str(json).unwrap();
        assert_eq!(result.category, "task");
        assert_eq!(result.confidence, 0.9);
    }

    #[test]
    fn test_extract_json_direct() {
        let text = r#"{"category": "meeting", "confidence": 0.85, "reasoning": "讨论了项目"}"#;
        let json = extract_json(text);
        let result: Classification = serde_json::from_str(json).unwrap();
        assert_eq!(result.category, "meeting");
    }

    #[test]
    fn test_extract_json_with_surrounding_text() {
        let text = r#"根据分析，结果如下：
{"category": "research", "confidence": 0.7, "reasoning": "调研报告"}
以上是分类结果。"#;
        let json = extract_json(text);
        let result: Classification = serde_json::from_str(json).unwrap();
        assert_eq!(result.category, "research");
    }

    #[test]
    fn test_model_config_builders() {
        let config = ModelConfig::anthropic("test-key".to_string());
        assert_eq!(config.model, "claude-sonnet-4-6");
        assert_eq!(config.base_url, "https://api.anthropic.com");

        let config = ModelConfig::openai("test-key".to_string(), None);
        assert_eq!(config.model, "gpt-4o");

        let config =
            ModelConfig::openai("key".to_string(), Some("http://localhost:8080".to_string()));
        assert_eq!(config.base_url, "http://localhost:8080");
    }

    #[test]
    fn test_build_task_context_section_empty() {
        let content = serde_json::json!({"text": "hello"});
        let section = build_task_context_section(&content);
        assert!(section.is_empty());
    }

    #[test]
    fn test_build_task_context_section_with_tasks() {
        let content = serde_json::json!({
            "text": "给Lily的邮件已经发送了",
            "task_context": [
                {"id": "task-1", "title": "发邮件给lily", "status": "Open"}
            ]
        });
        let section = build_task_context_section(&content);
        assert!(section.contains("已有任务列表"));
        assert!(section.contains("task-1"));
        assert!(section.contains("发邮件给lily"));
        assert!(section.contains("Open"));
    }

    #[test]
    fn test_build_task_context_section_empty_array() {
        let content = serde_json::json!({
            "text": "hello",
            "task_context": []
        });
        let section = build_task_context_section(&content);
        assert!(section.is_empty());
    }
}
