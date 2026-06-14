//! L3 集成测试 —— 完整链路
//!
//! 使用 MockAdapter 模拟 AI 调用，使用真实内部组件
//! （ProcessingPipeline、Classifier、TaskRunner、ReviewAgent）。
//!
//! 测试场景：
//! 1. 含任务的消息走完整链路（Instant route）
//! 2. 普通消息走 Aggregate 链路
//! 3. 低置信度直接归档

use std::collections::HashMap;

use tempfile::tempdir;
use wb_ai::{
    budget::TokenBudget,
    router::ModelRouter,
    task_runner::{ModelSize, TaskRunner},
    Classification, Extraction, MockAdapter, ModelAdapter,
};
use wb_core::event::{Confidence, Event, EventType, Source};
use wb_core::record::Category;
use wb_processor::classifier::ProcessingRoute;
use wb_processor::persist::PersistStep;
use wb_processor::pipeline::ProcessingPipeline;

/// 辅助函数：创建 Event
fn make_event(
    source: Source,
    confidence: Confidence,
    event_type: EventType,
    content: serde_json::Value,
) -> Event {
    Event::new(source, confidence, event_type, content, "raw".to_string())
}

/// 辅助函数：使用自定义 MockAdapter 创建 ProcessingPipeline
fn make_pipeline_with_adapter(
    tmp_dir: &std::path::Path,
    small_adapter: MockAdapter,
) -> ProcessingPipeline {
    let router = ModelRouter::new();
    let budget = TokenBudget::new(100_000);
    let mut adapters: HashMap<ModelSize, Box<dyn ModelAdapter>> = HashMap::new();
    adapters.insert(ModelSize::Small, Box::new(small_adapter));
    adapters.insert(
        ModelSize::Large,
        Box::new(MockAdapter::new().with_model_name("mock-large".to_string())),
    );
    let mut adapter_names = HashMap::new();
    adapter_names.insert(ModelSize::Small, "mock-small".to_string());
    adapter_names.insert(ModelSize::Large, "mock-large".to_string());

    let runner = TaskRunner::new(router, budget, adapters, adapter_names);
    let persistor = PersistStep::new(tmp_dir);
    ProcessingPipeline::new(runner, persistor)
}

/// 辅助函数：使用默认 MockAdapter 创建 ProcessingPipeline
fn make_pipeline(tmp_dir: &std::path::Path) -> ProcessingPipeline {
    make_pipeline_with_adapter(tmp_dir, MockAdapter::new())
}

// ─── 测试用例 1：含任务的消息走完整链路 ─────────────────────────

/// Event: { source: FeishuMessage, content: "明天下午5点完成代码发布" }
/// MockAdapter: classify→Instant(task), extract→title="完成代码发布", due_date="明天下午5点"
/// 断言: route==Instant, title 包含 "完成代码发布", category==Task,
///       task_due.is_some(), model_used 不是 archive-skip（证明提取已执行）,
///       review_ms >= 0（证明审核已执行）
#[tokio::test]
async fn l3_task_message_full_pipeline() {
    let tmp = tempdir().unwrap();

    // 配置 MockAdapter：分类为 task（Instant），提取包含任务信息和截止日期
    let adapter = MockAdapter::new()
        .with_classification(Classification {
            category: "task".to_string(),
            confidence: 0.9,
            reasoning: "contains task with deadline".to_string(),
        })
        .with_extraction(Extraction {
            title: "完成代码发布".to_string(),
            summary: "明天下午5点完成代码发布".to_string(),
            detail: "需要在明天下午5点前完成代码发布流程".to_string(),
            people: vec![],
            tags: vec!["发布".to_string()],
            project: None,
            due_date: Some("明天下午5点".to_string()),
            confidence: 0.92,
        });

    let mut pipeline = make_pipeline_with_adapter(tmp.path(), adapter);

    let event = make_event(
        Source::FeishuMessage,
        Confidence::Medium,
        EventType::Message,
        serde_json::json!({"text": "明天下午5点完成代码发布"}),
    );

    let result = pipeline.process(&event).await.unwrap();

    // 断言：路由应为 Instant（AI 分类为 task → Instant）
    assert_eq!(
        result.route,
        ProcessingRoute::Instant,
        "含任务的消息应走 Instant 路由"
    );

    // 断言：标题应包含 "完成代码发布"
    assert!(
        result.work_record.title.contains("完成代码发布"),
        "标题应包含 '完成代码发布'，实际: {}",
        result.work_record.title
    );

    // 断言：分类应为 Task（Discovery 识别 + AI 提取）
    assert_eq!(
        result.work_record.category,
        Category::Task,
        "含任务的消息应归类为 Task"
    );

    // 断言：应有截止日期
    assert!(
        result.work_record.task_due.is_some(),
        "含截止日期的任务应有 task_due"
    );

    // 断言：模型提取已执行（model_used 不是 archive-skip，证明非 Archive 路由走了模型）
    assert_ne!(
        result.work_record.model_used, "archive-skip",
        "Instant 路由应调用模型提取，model_used 不应为 archive-skip"
    );

    // 断言：审核已执行（review_result 应有有效的 reviewer）
    assert!(
        !result.review_result.reviewer.is_empty(),
        "应经过 ReviewAgent 审核"
    );
}

// ─── 测试用例 2：普通消息走 Aggregate 链路 ─────────────────────

/// Event: { content: "今天天气不错" }
/// MockAdapter: classify→Aggregate(message), extract→title="天气讨论"
/// 断言: route==Aggregate, category!=Task, task_due.is_none(),
///       model_used 不是 archive-skip（证明提取已执行）
#[tokio::test]
async fn l3_normal_message_aggregate_route() {
    let tmp = tempdir().unwrap();

    // 配置 MockAdapter：分类为 message（Aggregate），提取普通内容
    let adapter = MockAdapter::new()
        .with_classification(Classification {
            category: "message".to_string(),
            confidence: 0.8,
            reasoning: "普通聊天消息".to_string(),
        })
        .with_extraction(Extraction {
            title: String::new(), // 空标题 → AI Discovery 不认为是任务
            summary: "讨论今天天气".to_string(),
            detail: "今天天气不错".to_string(),
            people: vec![],
            tags: vec![],
            project: None,
            due_date: None,
            confidence: 0.3, // 低于 AI_CONFIDENCE_THRESHOLD(0.5) → 不触发任务发现
        });

    let mut pipeline = make_pipeline_with_adapter(tmp.path(), adapter);

    let event = make_event(
        Source::FeishuMessage,
        Confidence::Medium,
        EventType::Message,
        serde_json::json!({"text": "今天天气不错"}),
    );

    let result = pipeline.process(&event).await.unwrap();

    // 断言：路由应为 Aggregate
    assert_eq!(
        result.route,
        ProcessingRoute::Aggregate,
        "普通消息应走 Aggregate 路由"
    );

    // 断言：分类不应为 Task
    assert_ne!(
        result.work_record.category,
        Category::Task,
        "普通天气消息不应归类为 Task"
    );

    // 断言：不应有截止日期
    assert!(
        result.work_record.task_due.is_none(),
        "普通消息不应有 task_due"
    );

    // 断言：模型提取已执行（model_used 不是 archive-skip）
    assert_ne!(
        result.work_record.model_used, "archive-skip",
        "Aggregate 路由应调用模型提取，model_used 不应为 archive-skip"
    );
}

// ─── 测试用例 3：低置信度直接归档 ───────────────────────────────

/// Event: { source: SystemAppSwitch, confidence: Low }
/// 断言: route==Archive, model_used=="archive-skip"（证明未调用模型提取）
#[tokio::test]
async fn l3_low_confidence_archive() {
    let tmp = tempdir().unwrap();
    let mut pipeline = make_pipeline(tmp.path());

    let event = make_event(
        Source::SystemAppSwitch,
        Confidence::Low,
        EventType::AppActivity,
        serde_json::json!({"app": "Safari", "duration_min": 5}),
    );

    let result = pipeline.process(&event).await.unwrap();

    // 断言：路由应为 Archive
    assert_eq!(
        result.route,
        ProcessingRoute::Archive,
        "低置信度事件应直接归档"
    );

    // 断言：model_used 应为 archive-skip（Archive 路由跳过模型调用）
    assert_eq!(
        result.work_record.model_used, "archive-skip",
        "Archive 路由不应调用模型提取，model_used 应为 archive-skip"
    );
}
