//! AI 驱动的任务发现
//!
//! 使用 ModelAdapter.extract() 分析消息内容，识别潜在任务。
//! AI 优先：AI 返回有效结果则使用 AI 结果，否则降级到关键词匹配。
//!
//! 支持任务上下文：传入已有 Pending/Open 任务列表，AI 可判断新消息是
//! "新任务"还是"已有任务的状态更新"。

use wb_ai::{ModelAdapter, TaskContext};

use super::discovery::PendingTask;
use super::discovery_message;
use super::model::{TaskPriority, TaskSource};
use wb_core::event::{Confidence, EventType, Source};

/// AI 置信度阈值：低于此值认为 AI 不确定是任务
const AI_CONFIDENCE_THRESHOLD: f64 = 0.5;

/// 紧急关键词 —— 匹配时提升优先级
const URGENT_KEYWORDS: &[&str] = &[
    "尽快", "紧急", "马上", "立即", "ASAP", "urgent", "今天内", "今天之内",
];

/// AI 驱动的任务发现
///
/// 调用 adapter.extract() 分析消息内容，将 AI 返回的 Extraction 转换为 PendingTask。
/// AI 优先：AI 返回有效候选则使用 AI 结果。
/// AI 返回空或失败时，降级到关键词匹配 `discovery_message::discover_from_message(text)`。
/// 两者都为空则确认无任务。
///
/// # Arguments
/// - `text`: 待分析文本
/// - `adapter`: AI 模型适配器
/// - `source`: 事件来源类型（M5: 参数化替代硬编码）
/// - `existing_tasks`: 已有任务上下文列表，用于判断状态更新
pub async fn discover_with_ai(
    text: &str,
    adapter: &dyn ModelAdapter,
    source: Source,
    existing_tasks: &[TaskContext],
) -> Vec<PendingTask> {
    // Step 1: 尝试 AI 提取（带上下文）
    let ai_candidates = try_ai_extraction(text, adapter, source, existing_tasks).await;

    // Step 2: AI 返回有效候选则使用 AI 结果
    if !ai_candidates.is_empty() {
        return ai_candidates;
    }

    // Step 3: 降级到关键词匹配
    discovery_message::discover_from_message(text)
}

/// 尝试通过 AI 提取任务（带已有任务上下文）
async fn try_ai_extraction(
    text: &str,
    adapter: &dyn ModelAdapter,
    source: Source,
    existing_tasks: &[TaskContext],
) -> Vec<PendingTask> {
    let event = create_synthetic_event(text, source, existing_tasks);
    match adapter.extract(&event).await {
        Ok(extraction) => {
            // 如果 AI 判断为状态更新，返回空（不创建新任务）
            if extraction.is_status_update {
                tracing::info!(
                    "task_status_update_detected: related_task_id={:?}",
                    extraction.related_task_id
                );
                return vec![];
            }
            if is_valid_task_extraction(&extraction) {
                vec![extraction_to_pending_task(&extraction, text)]
            } else {
                vec![]
            }
        }
        _ => vec![],
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
    let priority = if has_urgent_keywords(origin_text) {
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

/// 检查文本是否包含紧急关键词
fn has_urgent_keywords(text: &str) -> bool {
    URGENT_KEYWORDS.iter().any(|k| text.contains(k))
}

/// 从纯文本创建合成 Event，用于调用 ModelAdapter.extract()
///
/// M5: Source 参数化，不再硬编码 FeishuMessage。
/// 如果有已有任务上下文，将其嵌入 event.content 的 task_context 字段。
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

    /// 创建返回指定 Extraction 的 MockAdapter
    fn adapter_with_extraction(extraction: Extraction) -> MockAdapter {
        MockAdapter::new().with_extraction(extraction)
    }

    /// 创建返回错误的 Adapter
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
        assert_eq!(tasks.len(), 1, "AI 应发现 1 个任务");
        assert_eq!(tasks[0].title, "完成报告");
        assert_eq!(
            tasks[0].due_date,
            Some("明天".to_string()),
            "应保留 AI 返回的截止日期"
        );
        assert_eq!(tasks[0].source, TaskSource::Message);
    }

    #[tokio::test]
    async fn test_ai_returns_empty_for_non_task() {
        // AI 返回低置信度空标题 → 不是任务
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

        // 文本也不包含任务关键词 → 两者都为空
        let tasks = discover_with_ai("今天天气真好", &adapter, Source::FeishuMessage, &[]).await;
        assert!(tasks.is_empty(), "非任务文本 + AI 无结果 → 空");
    }

    #[tokio::test]
    async fn test_ai_fallback_to_keywords() {
        // AI 返回 Err → 降级到关键词匹配
        let adapter = ErrorAdapter;

        let tasks = discover_with_ai("请你帮忙检查一下登录接口", &adapter, Source::FeishuMessage, &[]).await;
        assert_eq!(tasks.len(), 1, "应降级到关键词匹配发现任务");
        assert_eq!(tasks[0].source, TaskSource::Message);
    }

    #[tokio::test]
    async fn test_ai_urgent_keyword_sets_p1() {
        let adapter = adapter_with_extraction(Extraction {
            title: "修复崩溃".to_string(),
            summary: "紧急修复".to_string(),
            detail: String::new(),
            people: vec![],
            tags: vec![],
            project: None,
            due_date: None,
            confidence: 0.9,
            is_status_update: false,
            related_task_id: None,
        });

        let tasks = discover_with_ai("尽快修复生产环境崩溃", &adapter, Source::FeishuMessage, &[]).await;
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].priority, TaskPriority::P1, "含紧急关键词应为 P1");
    }

    #[tokio::test]
    async fn test_ai_normal_priority_is_p2() {
        let adapter = adapter_with_extraction(Extraction {
            title: "更新文档".to_string(),
            summary: "更新 API 文档".to_string(),
            detail: String::new(),
            people: vec![],
            tags: vec![],
            project: None,
            due_date: None,
            confidence: 0.8,
            is_status_update: false,
            related_task_id: None,
        });

        let tasks = discover_with_ai("更新一下 API 文档", &adapter, Source::FeishuMessage, &[]).await;
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].priority, TaskPriority::P2, "无紧急关键词应为 P2");
    }

    #[tokio::test]
    async fn test_ai_low_confidence_falls_back() {
        // AI 返回有效标题但置信度低于阈值 → 降级
        let adapter = adapter_with_extraction(Extraction {
            title: "可能的任务".to_string(),
            summary: String::new(),
            detail: String::new(),
            people: vec![],
            tags: vec![],
            project: None,
            due_date: None,
            confidence: 0.3, // < 0.5 阈值
            is_status_update: false,
            related_task_id: None,
        });

        // 文本含关键词 → 降级后关键词能匹配
        let tasks = discover_with_ai("请你帮忙检查 API", &adapter, Source::FeishuMessage, &[]).await;
        assert_eq!(tasks.len(), 1, "低置信度应降级到关键词");
        // 降级后由关键词发现，title 来自关键词解析
    }

    #[tokio::test]
    async fn test_synthetic_event_uses_provided_source() {
        // M5: create_synthetic_event 应使用传入的 Source 而非硬编码
        let adapter = adapter_with_extraction(Extraction {
            title: "用户捕获的任务".to_string(),
            summary: "从屏幕捕获".to_string(),
            detail: String::new(),
            people: vec![],
            tags: vec![],
            project: None,
            due_date: None,
            confidence: 0.9,
            is_status_update: false,
            related_task_id: None,
        });

        // 使用 UserCapture 作为 source
        let tasks = discover_with_ai("请完成这个任务", &adapter, Source::UserCapture, &[]).await;
        assert_eq!(tasks.len(), 1, "应发现任务");
    }

    #[tokio::test]
    async fn test_synthetic_event_different_sources() {
        // M5: 验证不同 Source 都能正确传递
        let adapter = adapter_with_extraction(Extraction {
            title: "测试任务".to_string(),
            summary: String::new(),
            detail: String::new(),
            people: vec![],
            tags: vec![],
            project: None,
            due_date: None,
            confidence: 0.8,
            is_status_update: false,
            related_task_id: None,
        });

        // 测试多种 Source 类型
        for source in [Source::FeishuMessage, Source::UserCapture, Source::SystemBrowser] {
            let tasks = discover_with_ai("检查一下这个", &adapter, source.clone(), &[]).await;
            assert_eq!(tasks.len(), 1, "Source {:?} 应能发现任务", source);
        }
    }

    // ─── 状态更新检测测试 ─────────────────────────────────────

    #[tokio::test]
    async fn test_ai_status_update_returns_empty() {
        // AI 判断为状态更新 → 不创建新任务
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

        let tasks = discover_with_ai(
            "给Lily的邮件已经发送了",
            &adapter,
            Source::FeishuMessage,
            &existing,
        )
        .await;
        assert!(tasks.is_empty(), "状态更新不应创建新任务");
    }

    #[tokio::test]
    async fn test_ai_new_task_with_context() {
        // AI 判断为新任务（有上下文但不匹配）→ 正常创建
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

        let tasks = discover_with_ai(
            "今天要写周报",
            &adapter,
            Source::FeishuMessage,
            &existing,
        )
        .await;
        assert_eq!(tasks.len(), 1, "新任务应正常创建");
        assert_eq!(tasks[0].title, "写周报");
    }

    #[tokio::test]
    async fn test_create_synthetic_event_with_context() {
        // 验证合成事件包含 task_context
        let existing = vec![TaskContext {
            id: "task-1".to_string(),
            title: "发邮件给lily".to_string(),
            status: "Open".to_string(),
        }];

        let event = create_synthetic_event("给Lily的邮件已经发送了", Source::FeishuMessage, &existing);
        let context = event.content.get("task_context").unwrap();
        assert!(context.is_array(), "task_context 应为数组");
        assert_eq!(context.as_array().unwrap().len(), 1);
        assert_eq!(context[0]["id"], "task-1");
    }

    #[tokio::test]
    async fn test_create_synthetic_event_without_context() {
        // 无上下文时，content 不应包含 task_context
        let event = create_synthetic_event("普通消息", Source::FeishuMessage, &[]);
        assert!(
            event.content.get("task_context").is_none(),
            "无上下文时不应有 task_context 字段"
        );
    }
}
