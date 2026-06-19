//! AI 驱动的任务发现
//!
//! 使用 ModelAdapter.extract() 分析消息内容，识别潜在任务。
//! 所有内容理解统一走 AI 模型，不使用关键词匹配。
//!
//! 支持任务上下文：传入已有 Pending/Open 任务列表，AI 可判断新消息是
//! "新任务"还是"已有任务的状态更新"。

use wb_ai::{ModelAdapter, TaskContext};

use super::discovery::PendingTask;
use super::model::{TaskPriority, TaskSource};
use wb_core::event::{Confidence, EventType, Source};

/// AI 置信度阈值：低于此值认为 AI 不确定是任务
const AI_CONFIDENCE_THRESHOLD: f64 = 0.5;

/// AI 驱动的任务发现
///
/// AI 返回有效候选则使用 AI 结果。
/// AI 判断为状态更新则返回空（不创建新任务）。
/// AI 返回空或失败则返回空（不降级到关键词）。
pub async fn discover_with_ai(
    text: &str,
    adapter: &dyn ModelAdapter,
    source: Source,
    existing_tasks: &[TaskContext],
) -> Vec<PendingTask> {
    let event = create_synthetic_event(text, source, existing_tasks);
    match adapter.extract(&event).await {
        Ok(extraction) => {
            eprintln!(
                "[discovery_ai] extraction: title='{}', confidence={}, is_status_update={}, related_task_id={:?}",
                extraction.title, extraction.confidence, extraction.is_status_update, extraction.related_task_id
            );
            // AI 判断为状态更新 → 不创建新任务
            if extraction.is_status_update {
                tracing::info!(
                    "task_status_update_detected: related_task_id={:?}",
                    extraction.related_task_id
                );
                return vec![];
            }
            // AI 返回有效任务
            if is_valid_task_extraction(&extraction) {
                vec![extraction_to_pending_task(&extraction, text)]
            } else {
                // AI 认为不是任务 → 返回空
                vec![]
            }
        }
        Err(e) => {
            // AI 调用失败 → 返回空，不降级
            tracing::warn!("AI extraction failed: {}", e);
            vec![]
        }
    }
}

/// 判断 AI 提取结果是否为有效任务
fn is_valid_task_extraction(extraction: &wb_ai::Extraction) -> bool {
    !extraction.title.is_empty() && extraction.confidence >= AI_CONFIDENCE_THRESHOLD
}

/// 将 AI Extraction 转换为 PendingTask
fn extraction_to_pending_task(
    extraction: &wb_ai::Extraction,
    origin_text: &str,
) -> PendingTask {
    let priority = if extraction.tags.iter().any(|t| {
        ["紧急", "urgent", "ASAP", "尽快", "马上", "立即"].contains(&t.as_str())
    }) {
        TaskPriority::P1
    } else {
        TaskPriority::P2
    };

    let description = if extraction.summary.is_empty() {
        None
    } else {
        Some(extraction.summary.as_str())
    };

    PendingTask::new(
        &extraction.title,
        description,
        TaskSource::Message,
        priority,
        extraction.due_date.clone(),
        origin_text,
    )
}

/// 从纯文本创建合成 Event，用于调用 ModelAdapter.extract()
fn create_synthetic_event(
    text: &str,
    source: Source,
    existing_tasks: &[TaskContext],
) -> wb_core::event::Event {
    let content = if existing_tasks.is_empty() {
        serde_json::json!({"text": text})
    } else {
        let context_array: Vec<serde_json::Value> = existing_tasks
            .iter()
            .map(|t| {
                serde_json::json!({
                    "id": t.id,
                    "title": t.title,
                    "status": t.status,
                })
            })
            .collect();
        serde_json::json!({
            "text": text,
            "task_context": context_array,
        })
    };

    wb_core::event::Event::new(source, Confidence::Medium, EventType::Message, content, text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wb_ai::{Extraction, MockAdapter};

    fn adapter_with_extraction(extraction: Extraction) -> MockAdapter {
        MockAdapter::new().with_extraction(extraction)
    }

    struct ErrorAdapter;

    #[async_trait::async_trait]
    impl ModelAdapter for ErrorAdapter {
        async fn classify(
            &self,
            _event: &wb_core::event::Event,
        ) -> wb_core::error::Result<wb_ai::Classification> {
            Err(wb_core::error::WbError::Ai("mock error".to_string()))
        }

        async fn extract(
            &self,
            _event: &wb_core::event::Event,
        ) -> wb_core::error::Result<Extraction> {
            Err(wb_core::error::WbError::Ai("mock error".to_string()))
        }

        async fn summarize(&self, _text: &str) -> wb_core::error::Result<String> {
            Err(wb_core::error::WbError::Ai("mock error".to_string()))
        }
    }

    #[tokio::test]
    async fn test_ai_discovers_task_with_deadline() {
        let adapter = adapter_with_extraction(Extraction {
            title: "完成报告".to_string(),
            summary: "明天完成报告".to_string(),
            detail: String::new(),
            people: vec![],
            tags: vec![],
            project: None,
            due_date: Some("明天".to_string()),
            confidence: 0.9,
            is_status_update: false,
            related_task_id: None,
        });

        let tasks = discover_with_ai("请帮忙明天完成报告", &adapter, Source::FeishuMessage, &[]).await;
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "完成报告");
        assert_eq!(tasks[0].due_date, Some("明天".to_string()));
    }

    #[tokio::test]
    async fn test_ai_returns_empty_for_non_task() {
        let adapter = adapter_with_extraction(Extraction {
            title: String::new(),
            summary: String::new(),
            detail: String::new(),
            people: vec![],
            tags: vec![],
            project: None,
            due_date: None,
            confidence: 0.2,
            is_status_update: false,
            related_task_id: None,
        });

        let tasks = discover_with_ai("今天天气真好", &adapter, Source::FeishuMessage, &[]).await;
        assert!(tasks.is_empty(), "非任务文本 + AI 无结果 → 空");
    }

    #[tokio::test]
    async fn test_ai_error_returns_empty() {
        let adapter = ErrorAdapter;
        let tasks = discover_with_ai("请你帮忙检查一下登录接口", &adapter, Source::FeishuMessage, &[]).await;
        assert!(tasks.is_empty(), "AI 失败应返回空，不降级");
    }

    #[tokio::test]
    async fn test_ai_low_confidence_returns_empty() {
        let adapter = adapter_with_extraction(Extraction {
            title: "可能的任务".to_string(),
            summary: String::new(),
            detail: String::new(),
            people: vec![],
            tags: vec![],
            project: None,
            due_date: None,
            confidence: 0.3,
            is_status_update: false,
            related_task_id: None,
        });

        let tasks = discover_with_ai("请你帮忙检查 API", &adapter, Source::FeishuMessage, &[]).await;
        assert!(tasks.is_empty(), "低置信度应返回空");
    }

    #[tokio::test]
    async fn test_ai_status_update_returns_empty() {
        let adapter = adapter_with_extraction(Extraction {
            title: String::new(),
            summary: String::new(),
            detail: String::new(),
            people: vec![],
            tags: vec![],
            project: None,
            due_date: None,
            confidence: 0.9,
            is_status_update: true,
            related_task_id: Some("existing-task-1".to_string()),
        });

        let existing = vec![TaskContext {
            id: "existing-task-1".to_string(),
            title: "发邮件给lily".to_string(),
            status: "Open".to_string(),
        }];

        let tasks = discover_with_ai("给Lily的邮件已经发送了", &adapter, Source::FeishuMessage, &existing).await;
        assert!(tasks.is_empty(), "状态更新不应创建新任务");
    }

    #[tokio::test]
    async fn test_ai_new_task_with_context() {
        let adapter = adapter_with_extraction(Extraction {
            title: "写周报".to_string(),
            summary: "本周工作总结".to_string(),
            detail: String::new(),
            people: vec![],
            tags: vec![],
            project: None,
            due_date: None,
            confidence: 0.9,
            is_status_update: false,
            related_task_id: None,
        });

        let existing = vec![TaskContext {
            id: "existing-task-1".to_string(),
            title: "发邮件给lily".to_string(),
            status: "Open".to_string(),
        }];

        let tasks = discover_with_ai("今天要写周报", &adapter, Source::FeishuMessage, &existing).await;
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "写周报");
    }

    #[tokio::test]
    async fn test_synthetic_event_with_context() {
        let existing = vec![TaskContext {
            id: "task-1".to_string(),
            title: "发邮件给lily".to_string(),
            status: "Open".to_string(),
        }];

        let event = create_synthetic_event("给Lily的邮件已经发送了", Source::FeishuMessage, &existing);
        let context = event.content.get("task_context").unwrap();
        assert!(context.is_array());
        assert_eq!(context.as_array().unwrap().len(), 1);
        assert_eq!(context[0]["id"], "task-1");
    }

    #[tokio::test]
    async fn test_synthetic_event_without_context() {
        let event = create_synthetic_event("普通消息", Source::FeishuMessage, &[]);
        assert!(event.content.get("task_context").is_none());
    }
}
