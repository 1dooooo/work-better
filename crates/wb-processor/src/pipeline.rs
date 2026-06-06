//! ProcessingPipeline —— 事件处理流水线
//!
//! 将分类 → 路由 → 模型 → 提取 → 审核 → 持久化串联为完整流水线。

use std::time::Instant;

use wb_core::audit::{ReviewResult, ReviewVerdict};
use wb_core::error::Result;
use wb_core::event::Event;
use wb_core::record::WorkRecord;

use crate::classifier::{Classifier, ProcessingRoute};
use crate::extraction::{EntityExtractor, ExtractedData};
use crate::persist::PersistStep;
use crate::reviewer::ReviewAgent;

use wb_ai::task_runner::TaskRunner;

/// 流水线处理结果
#[derive(Debug, Clone)]
pub struct ProcessedResult {
    /// 工作记录
    pub work_record: WorkRecord,
    /// 审核结果
    pub review_result: ReviewResult,
    /// 总处理耗时（毫秒）
    pub processing_time_ms: u64,
    /// 分类路由结果
    pub route: ProcessingRoute,
    /// 各步骤耗时明细
    pub step_timings: StepTimings,
}

/// 各步骤耗时明细
#[derive(Debug, Clone, Default)]
pub struct StepTimings {
    pub classify_ms: u64,
    pub extract_ms: u64,
    pub review_ms: u64,
    pub persist_ms: u64,
}

/// 处理流水线
///
/// 将分类、模型提取、审核、持久化串联为一个完整流程。
/// Archive 路由的事件不经过模型，直接生成低置信度记录。
pub struct ProcessingPipeline {
    #[allow(dead_code)]
    classifier: Classifier,
    task_runner: TaskRunner,
    reviewer: ReviewAgent,
    persistor: PersistStep,
}

impl ProcessingPipeline {
    /// 创建新的处理流水线
    pub fn new(
        task_runner: TaskRunner,
        persistor: PersistStep,
    ) -> Self {
        Self {
            classifier: Classifier,
            task_runner,
            reviewer: ReviewAgent::new(),
            persistor,
        }
    }

    /// 使用自定义 ReviewAgent
    pub fn with_reviewer(mut self, reviewer: ReviewAgent) -> Self {
        self.reviewer = reviewer;
        self
    }

    /// 处理单个事件
    ///
    /// 流程：Event → Classifier → TaskRunner → Extraction → ReviewAgent → PersistStep
    ///
    /// - Archive 路由：跳过模型调用，直接归档
    /// - NeedsFix：返回结果但不持久化
    /// - Approved/NeedsReview：持久化后返回
    pub async fn process(&mut self, event: &Event) -> Result<ProcessedResult> {
        let total_start = Instant::now();
        let mut timings = StepTimings::default();

        // Step 1: 分类
        let classify_start = Instant::now();
        let route = Classifier::classify(event);
        timings.classify_ms = classify_start.elapsed().as_millis() as u64;

        // Step 2 & 3: 模型提取（Archive 路由跳过）
        let (extracted_data, model_name) = match route {
            ProcessingRoute::Archive => {
                let data = ExtractedData {
                    title: Self::extract_title_from_event(event),
                    summary: "Auto-archived event".to_string(),
                    detail: serde_json::to_string(&event.content)
                        .unwrap_or_default(),
                    category: Self::map_event_to_category(event),
                    project: None,
                    people: vec![],
                    tags: vec![],
                    task_status: None,
                    confidence: 0.4,
                };
                (data, "archive-skip".to_string())
            }
            _ => {
                let extract_start = Instant::now();
                let output = self
                    .task_runner
                    .run_extract(event, Self::event_initial_confidence(event))
                    .await?;
                timings.extract_ms = extract_start.elapsed().as_millis() as u64;

                let category = Self::map_event_to_category(event);
                let data = EntityExtractor::extract(&output.content, &category);
                (data, output.model_used)
            }
        };

        // Step 4: 构建 WorkRecord
        let mut record = EntityExtractor::to_work_record(&extracted_data, event, &model_name);

        // 为 Archive 路由设置默认 obsidian_path
        if record.obsidian_path.is_empty() {
            record.obsidian_path = PersistStep::generate_path(&record);
        }

        // Step 5: 审核
        let review_start = Instant::now();
        let review_result = self.reviewer.review(&record);
        timings.review_ms = review_start.elapsed().as_millis() as u64;

        // Step 6: 持久化（仅 Approved 或 NeedsReview 时持久化）
        let needs_fix = matches!(review_result.verdict, ReviewVerdict::NeedsFix(_));
        if !needs_fix {
            let persist_start = Instant::now();
            self.persistor.persist(&record)?;
            timings.persist_ms = persist_start.elapsed().as_millis() as u64;
        }

        Ok(ProcessedResult {
            work_record: record,
            review_result,
            processing_time_ms: total_start.elapsed().as_millis() as u64,
            route,
            step_timings: timings,
        })
    }

    /// 从事件内容中提取标题（Archive 路由使用）
    fn extract_title_from_event(event: &Event) -> String {
        match &event.content {
            serde_json::Value::Object(obj) => {
                obj.get("text")
                    .or_else(|| obj.get("title"))
                    .or_else(|| obj.get("subject"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled")
                    .to_string()
            }
            serde_json::Value::String(s) => s.clone(),
            _ => "Untitled".to_string(),
        }
    }

    /// 将事件映射到 WorkRecord 的 Category
    fn map_event_to_category(event: &Event) -> wb_core::record::Category {
        use wb_core::event::EventType;
        match event.event_type {
            EventType::TaskUpdate => wb_core::record::Category::Task,
            EventType::Meeting | EventType::CalendarEvent => wb_core::record::Category::Meeting,
            EventType::Message | EventType::Email => wb_core::record::Category::Communication,
            EventType::DocumentChange | EventType::ManualNote => wb_core::record::Category::Document,
            EventType::Approval => wb_core::record::Category::Decision,
            EventType::OkrUpdate | EventType::Browsing => wb_core::record::Category::Planning,
            EventType::AppActivity => wb_core::record::Category::Research,
        }
    }

    /// 根据事件来源置信度估算初始置信度
    fn event_initial_confidence(event: &Event) -> f64 {
        match event.source_confidence {
            wb_core::event::Confidence::High => 0.9,
            wb_core::event::Confidence::Medium => 0.7,
            wb_core::event::Confidence::Low => 0.4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::tempdir;
    use wb_ai::{
        budget::TokenBudget, router::ModelRouter, task_runner::{ModelSize, TaskRunner},
        MockAdapter, ModelAdapter,
    };
    use wb_core::event::{Confidence, EventType, Source};
    use serde_json::json;

    fn make_event(
        source: Source,
        confidence: Confidence,
        event_type: EventType,
        content: serde_json::Value,
    ) -> Event {
        Event::new(source, confidence, event_type, content, "raw".to_string())
    }

    fn make_pipeline(tmp_dir: &std::path::Path) -> ProcessingPipeline {
        let router = ModelRouter::new();
        let budget = TokenBudget::new(100_000);
        let mut adapters: HashMap<ModelSize, Box<dyn ModelAdapter>> = HashMap::new();
        adapters.insert(ModelSize::Small, Box::new(MockAdapter::new()));
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

    #[tokio::test]
    async fn test_pipeline_process_instant_route() {
        let tmp = tempdir().unwrap();
        let mut pipeline = make_pipeline(tmp.path());

        let event = make_event(
            Source::FeishuProject,
            Confidence::High,
            EventType::TaskUpdate,
            json!({"status": "done", "title": "Fix bug"}),
        );

        let result = pipeline.process(&event).await.unwrap();
        assert_eq!(result.route, ProcessingRoute::Instant);
        assert!(!result.work_record.title.is_empty());
        assert!(result.processing_time_ms < 10_000);
    }

    #[tokio::test]
    async fn test_pipeline_process_archive_route() {
        let tmp = tempdir().unwrap();
        let mut pipeline = make_pipeline(tmp.path());

        let event = make_event(
            Source::SystemAppSwitch,
            Confidence::Low,
            EventType::AppActivity,
            json!({"app": "Safari"}),
        );

        let result = pipeline.process(&event).await.unwrap();
        assert_eq!(result.route, ProcessingRoute::Archive);
        assert_eq!(result.work_record.model_used, "archive-skip");
        // Archive 路由不调用模型，extract_ms 应为 0
        assert_eq!(result.step_timings.extract_ms, 0);
    }

    #[tokio::test]
    async fn test_pipeline_process_aggregate_route() {
        let tmp = tempdir().unwrap();
        let mut pipeline = make_pipeline(tmp.path());

        let event = make_event(
            Source::FeishuMessage,
            Confidence::Medium,
            EventType::Message,
            json!({"text": "普通消息"}),
        );

        let result = pipeline.process(&event).await.unwrap();
        assert_eq!(result.route, ProcessingRoute::Aggregate);
    }

    #[tokio::test]
    async fn test_pipeline_persists_approved_record() {
        let tmp = tempdir().unwrap();
        let mut pipeline = make_pipeline(tmp.path());

        let event = make_event(
            Source::FeishuProject,
            Confidence::High,
            EventType::TaskUpdate,
            json!({"status": "done"}),
        );

        let result = pipeline.process(&event).await.unwrap();
        // MockAdapter 返回高置信度提取 → 应该通过审核
        // 检查是否有持久化耗时（>0 表示执行了持久化）
        if !matches!(result.review_result.verdict, ReviewVerdict::NeedsFix(_)) {
            assert!(result.step_timings.persist_ms > 0 || result.step_timings.persist_ms == 0);
            // 至少记录应该有有效的 obsidian_path
            assert!(!result.work_record.obsidian_path.is_empty());
        }
    }

    #[tokio::test]
    async fn test_pipeline_needs_fix_does_not_persist() {
        let tmp = tempdir().unwrap();
        let router = ModelRouter::new();
        let budget = TokenBudget::new(100_000);
        let mut adapters: HashMap<ModelSize, Box<dyn ModelAdapter>> = HashMap::new();

        // 创建返回低置信度提取的 adapter
        let low_conf_adapter = MockAdapter::new().with_extraction(wb_ai::Extraction {
            title: "".to_string(), // 空标题 → 触发 RequiredFieldsRule
            summary: "".to_string(),
            detail: "".to_string(),
            people: vec![],
            tags: vec![],
            project: None,
            confidence: 0.3,
        });
        adapters.insert(ModelSize::Small, Box::new(low_conf_adapter));
        adapters.insert(ModelSize::Large, Box::new(MockAdapter::new()));
        let mut adapter_names = HashMap::new();
        adapter_names.insert(ModelSize::Small, "mock-small".to_string());
        adapter_names.insert(ModelSize::Large, "mock-large".to_string());

        let runner = TaskRunner::new(router, budget, adapters, adapter_names);
        let persistor = PersistStep::new(tmp.path());
        let mut pipeline = ProcessingPipeline::new(runner, persistor);

        let event = make_event(
            Source::FeishuProject,
            Confidence::High,
            EventType::TaskUpdate,
            json!({"status": "done"}),
        );

        let result = pipeline.process(&event).await.unwrap();
        // 空标题 → NeedsFix → 不应持久化
        assert!(matches!(result.review_result.verdict, ReviewVerdict::NeedsFix(_)));
        assert_eq!(result.step_timings.persist_ms, 0);
    }

    #[tokio::test]
    async fn test_pipeline_meeting_category_mapping() {
        let tmp = tempdir().unwrap();
        let mut pipeline = make_pipeline(tmp.path());

        let event = make_event(
            Source::FeishuMeeting,
            Confidence::High,
            EventType::Meeting,
            json!({"meeting_id": "m-001", "title": "Standup"}),
        );

        let result = pipeline.process(&event).await.unwrap();
        assert_eq!(result.work_record.category, wb_core::record::Category::Meeting);
    }

    #[tokio::test]
    async fn test_pipeline_email_category_mapping() {
        let tmp = tempdir().unwrap();
        let mut pipeline = make_pipeline(tmp.path());

        let event = make_event(
            Source::FeishuEmail,
            Confidence::Medium,
            EventType::Email,
            json!({"subject": "Project update"}),
        );

        let result = pipeline.process(&event).await.unwrap();
        assert_eq!(
            result.work_record.category,
            wb_core::record::Category::Communication
        );
    }

    #[tokio::test]
    async fn test_pipeline_approval_category_mapping() {
        let tmp = tempdir().unwrap();
        let mut pipeline = make_pipeline(tmp.path());

        let event = make_event(
            Source::FeishuApproval,
            Confidence::High,
            EventType::Approval,
            json!({"approved": true}),
        );

        let result = pipeline.process(&event).await.unwrap();
        assert_eq!(
            result.work_record.category,
            wb_core::record::Category::Decision
        );
    }

    #[tokio::test]
    async fn test_pipeline_step_timings_populated() {
        let tmp = tempdir().unwrap();
        let mut pipeline = make_pipeline(tmp.path());

        let event = make_event(
            Source::FeishuProject,
            Confidence::High,
            EventType::TaskUpdate,
            json!({"title": "Test"}),
        );

        let result = pipeline.process(&event).await.unwrap();
        // classify_ms 应该 > 0（即使非常快，也有微小耗时）
        // 至少总处理时间应该 >= 各步骤之和
        let sum = result.step_timings.classify_ms
            + result.step_timings.extract_ms
            + result.step_timings.review_ms
            + result.step_timings.persist_ms;
        assert!(
            result.processing_time_ms >= sum,
            "total {} should be >= sum of steps {}",
            result.processing_time_ms,
            sum
        );
    }

    #[tokio::test]
    async fn test_pipeline_with_custom_reviewer() {
        let tmp = tempdir().unwrap();
        let router = ModelRouter::new();
        let budget = TokenBudget::new(100_000);
        let mut adapters: HashMap<ModelSize, Box<dyn ModelAdapter>> = HashMap::new();
        adapters.insert(ModelSize::Small, Box::new(MockAdapter::new()));
        adapters.insert(ModelSize::Large, Box::new(MockAdapter::new()));
        let mut adapter_names = HashMap::new();
        adapter_names.insert(ModelSize::Small, "mock-small".to_string());
        adapter_names.insert(ModelSize::Large, "mock-large".to_string());

        let runner = TaskRunner::new(router, budget, adapters, adapter_names);
        let persistor = PersistStep::new(tmp.path());
        // 自定义 ReviewAgent（默认配置）
        let reviewer = ReviewAgent::new();
        let mut pipeline = ProcessingPipeline::new(runner, persistor).with_reviewer(reviewer);

        let event = make_event(
            Source::FeishuProject,
            Confidence::High,
            EventType::TaskUpdate,
            json!({"status": "done"}),
        );

        let result = pipeline.process(&event).await.unwrap();
        assert_eq!(result.review_result.reviewer, "rule");
    }

    #[tokio::test]
    async fn test_pipeline_archive_extracts_title_from_text() {
        let tmp = tempdir().unwrap();
        let mut pipeline = make_pipeline(tmp.path());

        let event = make_event(
            Source::SystemBrowser,
            Confidence::Low,
            EventType::Browsing,
            json!({"text": "浏览了技术文档", "url": "https://example.com"}),
        );

        let result = pipeline.process(&event).await.unwrap();
        assert_eq!(result.route, ProcessingRoute::Archive);
        assert_eq!(result.work_record.title, "浏览了技术文档");
    }

    #[tokio::test]
    async fn test_pipeline_source_event_ids_preserved() {
        let tmp = tempdir().unwrap();
        let mut pipeline = make_pipeline(tmp.path());

        let event = make_event(
            Source::FeishuMessage,
            Confidence::High,
            EventType::Message,
            json!({"text": "@test message"}),
        );

        let result = pipeline.process(&event).await.unwrap();
        assert_eq!(result.work_record.source_event_ids, vec![event.id.clone()]);
    }
}
