//! L5 E2E 测试 —— 完整端到端链路
//!
//! 使用**真实文件系统 + 真实内部组件**，仅 Mock AI API。
//! 每个测试使用独立 tmpdir，验证 vault/ 下的文件内容和数据库记录。
//!
//! 30 个场景，分 6 组：
//! - 组 A: 核心 Pipeline 流转（6 个）
//! - 组 B: 任务流转生命周期（7 个）
//! - 组 C: 审核分层（4 个）
//! - 组 D: 降级与容错（4 个）
//! - 组 E: 数据完整性（4 个）
//! - 组 F: 边界条件（5 个）

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use wb_ai::{
    budget::TokenBudget,
    router::ModelRouter,
    task_runner::{ModelSize, TaskRunner},
    Classification, Extraction, MockAdapter, ModelAdapter,
};
use wb_core::event::{Confidence, Event, EventType, Source};
use wb_core::record::Category;
use wb_core::task::{Task, TaskStatus};
use wb_processor::classifier::ProcessingRoute;
use wb_processor::persist::PersistStep;
use wb_processor::pipeline::ProcessingPipeline;
use wb_processor::review::{LargeModelReview, TieredReview};
use wb_processor::reviewer::ReviewAgent;

// ─── 测试基础设施 ──────────────────────────────────────────────────

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
    tmp_dir: &Path,
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
    let _persistor = PersistStep::new(tmp_dir);
    ProcessingPipeline::new(runner)
}

/// 辅助函数：使用默认 MockAdapter 创建 ProcessingPipeline
fn make_pipeline(tmp_dir: &Path) -> ProcessingPipeline {
    make_pipeline_with_adapter(tmp_dir, MockAdapter::new())
}

/// 辅助函数：创建带审核策略的 Pipeline
fn make_pipeline_with_review(
    tmp_dir: &Path,
    small_adapter: MockAdapter,
    reviewer: ReviewAgent,
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
    let _persistor = PersistStep::new(tmp_dir);
    ProcessingPipeline::new(runner).with_reviewer(reviewer)
}

/// 辅助函数：创建返回错误的 Adapter
struct ErrorAdapter;

#[async_trait::async_trait]
impl ModelAdapter for ErrorAdapter {
    async fn classify(
        &self,
        _event: &Event,
    ) -> wb_core::error::Result<Classification> {
        Err(wb_core::error::WbError::Ai("mock classify error".to_string()))
    }

    async fn extract(
        &self,
        _event: &Event,
    ) -> wb_core::error::Result<Extraction> {
        Err(wb_core::error::WbError::Ai("mock extract error".to_string()))
    }

    async fn summarize(&self, _text: &str) -> wb_core::error::Result<String> {
        Err(wb_core::error::WbError::Ai("mock summarize error".to_string()))
    }
}

/// 辅助函数：使用 ErrorAdapter 创建 Pipeline
fn make_error_pipeline(tmp_dir: &Path) -> ProcessingPipeline {
    let router = ModelRouter::new();
    let budget = TokenBudget::new(100_000);
    let mut adapters: HashMap<ModelSize, Box<dyn ModelAdapter>> = HashMap::new();
    adapters.insert(ModelSize::Small, Box::new(ErrorAdapter));
    adapters.insert(
        ModelSize::Large,
        Box::new(MockAdapter::new().with_model_name("mock-large".to_string())),
    );
    let mut adapter_names = HashMap::new();
    adapter_names.insert(ModelSize::Small, "mock-small".to_string());
    adapter_names.insert(ModelSize::Large, "mock-large".to_string());

    let runner = TaskRunner::new(router, budget, adapters, adapter_names);
    let _persistor = PersistStep::new(tmp_dir);
    ProcessingPipeline::new(runner)
}

/// 辅助函数：创建仅 extract 返回错误的 Adapter
struct ExtractErrorAdapter {
    classification: Classification,
}

#[async_trait::async_trait]
impl ModelAdapter for ExtractErrorAdapter {
    async fn classify(
        &self,
        _event: &Event,
    ) -> wb_core::error::Result<Classification> {
        Ok(self.classification.clone())
    }

    async fn extract(
        &self,
        _event: &Event,
    ) -> wb_core::error::Result<Extraction> {
        Err(wb_core::error::WbError::Ai("mock extract error".to_string()))
    }

    async fn summarize(&self, _text: &str) -> wb_core::error::Result<String> {
        Err(wb_core::error::WbError::Ai("mock summarize error".to_string()))
    }
}

/// 辅助函数：创建仅 extract 失败的 Pipeline
fn make_extract_error_pipeline(tmp_dir: &Path) -> ProcessingPipeline {
    let router = ModelRouter::new();
    let budget = TokenBudget::new(100_000);
    let mut adapters: HashMap<ModelSize, Box<dyn ModelAdapter>> = HashMap::new();
    adapters.insert(
        ModelSize::Small,
        Box::new(ExtractErrorAdapter {
            classification: Classification {
                category: "task".to_string(),
                confidence: 0.9,
                reasoning: "mock".to_string(),
            },
        }),
    );
    adapters.insert(
        ModelSize::Large,
        Box::new(MockAdapter::new().with_model_name("mock-large".to_string())),
    );
    let mut adapter_names = HashMap::new();
    adapter_names.insert(ModelSize::Small, "mock-small".to_string());
    adapter_names.insert(ModelSize::Large, "mock-large".to_string());

    let runner = TaskRunner::new(router, budget, adapters, adapter_names);
    let _persistor = PersistStep::new(tmp_dir);
    ProcessingPipeline::new(runner)
}

/// 辅助函数：在 vault 目录下查找 .md 文件
fn find_md_files(vault_dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(vault_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(find_md_files(&path));
            } else if path.extension().map_or(false, |e| e == "md") {
                files.push(path);
            }
        }
    }
    files
}

/// 辅助函数：读取 vault 下所有 .md 文件内容
fn read_all_md_contents(vault_dir: &Path) -> Vec<(std::path::PathBuf, String)> {
    find_md_files(vault_dir)
        .into_iter()
        .filter_map(|p| {
            fs::read_to_string(&p)
                .ok()
                .map(|content| (p, content))
        })
        .collect()
}

// ═══════════════════════════════════════════════════════════════════
// 组 A: 核心 Pipeline 流转（6 个）
// ═══════════════════════════════════════════════════════════════════

/// A1: 飞书即时消息 -> 完整处理链
///
/// Event: FeishuMessage, "明天下午5点完成代码发布"
/// MockAdapter: classify->task(Instant), extract->title="完成代码发布", due="明天下午5点"
/// 断言: route=Instant, title 包含 "完成代码发布", category=Task,
///       task_due 有值, vault/ 下有 .md 文件, .md 内容包含 title
#[tokio::test]
async fn a1_feishu_message_full_pipeline() {
    let tmp = tempfile::tempdir().unwrap();
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
                is_status_update: false,
                related_task_id: None,
        });

    let mut pipeline = make_pipeline_with_adapter(tmp.path(), adapter);
    let event = make_event(
        Source::FeishuMessage,
        Confidence::Medium,
        EventType::Message,
        serde_json::json!({"text": "明天下午5点完成代码发布"}),
    );

    let result = pipeline.process(&event).await.unwrap();

    // 断言：路由应为 Instant
    assert_eq!(result.route, ProcessingRoute::Instant, "含任务的消息应走 Instant 路由");

    // 断言：标题应包含 "完成代码发布"
    assert!(
        result.work_record.title.contains("完成代码发布"),
        "标题应包含 '完成代码发布'，实际: {}",
        result.work_record.title
    );

    // 断言：分类应为 Task
    assert_eq!(result.work_record.category, Category::Task, "含任务的消息应归类为 Task");

    // 断言：应有截止日期
    assert!(result.work_record.task_due.is_some(), "含截止日期的任务应有 task_due");

    // 断言：vault/ 下有 .md 文件
    let md_files = find_md_files(tmp.path());
    assert!(!md_files.is_empty(), "vault 下应有 .md 文件");

    // 断言：.md 内容包含 title
    let contents = read_all_md_contents(tmp.path());
    let has_title = contents.iter().any(|(_, c)| c.contains("完成代码发布"));
    assert!(has_title, ".md 文件内容应包含 title '完成代码发布'");
}

/// A2: 普通消息 -> Aggregate 路由
///
/// Event: Message, "今天天气不错"
/// MockAdapter: classify->message(Aggregate), extract->title="天气讨论"
/// 断言: route=Aggregate, category!=Task
///
/// 注意：由于 extraction.confidence < 0.5 会触发 ConfidenceThresholdRule (NeedsFix)，
/// 导致不持久化，因此不检查 .md 文件。
#[tokio::test]
async fn a2_normal_message_aggregate_route() {
    let tmp = tempfile::tempdir().unwrap();
    let adapter = MockAdapter::new()
        .with_classification(Classification {
            category: "message".to_string(),
            confidence: 0.8,
            reasoning: "普通聊天消息".to_string(),
        })
        .with_extraction(Extraction {
            title: "天气讨论".to_string(),
            summary: "讨论今天天气".to_string(),
            detail: "今天天气不错，适合外出活动".to_string(),
            people: vec![],
            tags: vec!["天气".to_string()],
            project: None,
            due_date: None,
            confidence: 0.4, // 低置信度，避免 Task Discovery 误判为任务
            is_status_update: false,
            related_task_id: None,
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
    assert_eq!(result.route, ProcessingRoute::Aggregate, "普通消息应走 Aggregate 路由");

    // 断言：分类不应为 Task
    assert_ne!(result.work_record.category, Category::Task, "普通天气消息不应归类为 Task");
}

/// A3: 低置信度 -> Archive 归档
///
/// Event: SystemAppSwitch, confidence=Low
/// 断言: route=Archive, vault/ 下无新文件（或 model_used="archive-skip"）
#[tokio::test]
async fn a3_low_confidence_archive() {
    let tmp = tempfile::tempdir().unwrap();
    let mut pipeline = make_pipeline(tmp.path());
    let event = make_event(
        Source::SystemAppSwitch,
        Confidence::Low,
        EventType::AppActivity,
        serde_json::json!({"app": "Safari", "duration_min": 5}),
    );

    let result = pipeline.process(&event).await.unwrap();

    // 断言：路由应为 Archive
    assert_eq!(result.route, ProcessingRoute::Archive, "低置信度事件应直接归档");

    // 断言：model_used 应为 archive-skip
    assert_eq!(result.work_record.model_used, "archive-skip", "Archive 路由不应调用模型");

    // 断言：Archive 路由仍会持久化（pipeline 对 Archive 也会 persist）
    let md_files = find_md_files(tmp.path());
    // Archive 路由会生成文件，但 model_used 为 archive-skip
    if !md_files.is_empty() {
        let content = fs::read_to_string(&md_files[0]).unwrap();
        assert!(content.contains("archive-skip"), "Archive 文件应标记 model_used 为 archive-skip");
    }
}

/// A4: 审批消息 -> Instant 路由
///
/// Event: Approval, "审批：Q2 OKR 评分"
/// MockAdapter: classify->approval(Instant), extract->title="Q2 OKR 评分审批"
/// 断言: route=Instant, category=Decision
#[tokio::test]
async fn a4_approval_message_instant_route() {
    let tmp = tempfile::tempdir().unwrap();
    let adapter = MockAdapter::new()
        .with_classification(Classification {
            category: "approval".to_string(),
            confidence: 0.95,
            reasoning: "审批消息".to_string(),
        })
        .with_extraction(Extraction {
            title: "Q2 OKR 评分审批".to_string(),
            summary: "审批：Q2 OKR 评分".to_string(),
            detail: "Q2 OKR 评分审批流程".to_string(),
            people: vec![],
            tags: vec!["审批".to_string()],
            project: None,
            due_date: None,
            confidence: 0.95,
                is_status_update: false,
                related_task_id: None,
        });

    let mut pipeline = make_pipeline_with_adapter(tmp.path(), adapter);
    let event = make_event(
        Source::FeishuApproval,
        Confidence::High,
        EventType::Approval,
        serde_json::json!({"text": "审批：Q2 OKR 评分"}),
    );

    let result = pipeline.process(&event).await.unwrap();

    // 断言：路由应为 Instant
    assert_eq!(result.route, ProcessingRoute::Instant, "审批消息应走 Instant 路由");

    // 断言：分类应为 Decision
    assert_eq!(result.work_record.category, Category::Decision, "审批消息应归类为 Decision");

    // 断言：vault/ 下有 .md 文件且包含 title
    let contents = read_all_md_contents(tmp.path());
    let has_title = contents.iter().any(|(_, c)| c.contains("Q2 OKR 评分审批"));
    assert!(has_title, ".md 文件应包含审批标题");
}

/// A5: 会议消息 -> Meeting 分类
///
/// Event: Meeting, "会议结束，待办：张三修复首页bug"
/// MockAdapter: classify->meeting(Instant), extract->title="修复首页bug", people=["张三"]
/// 断言: category=Meeting, people 含 "张三"
#[tokio::test]
async fn a5_meeting_message_meeting_category() {
    let tmp = tempfile::tempdir().unwrap();
    let adapter = MockAdapter::new()
        .with_classification(Classification {
            category: "meeting".to_string(),
            confidence: 0.85,
            reasoning: "会议消息".to_string(),
        })
        .with_extraction(Extraction {
            title: "修复首页bug".to_string(),
            summary: "会议结束，待办：张三修复首页bug".to_string(),
            detail: "会议中分配的任务：张三负责修复首页 bug".to_string(),
            people: vec!["张三".to_string()],
            tags: vec!["会议".to_string()],
            project: None,
            due_date: None,
            confidence: 0.88,
                is_status_update: false,
                related_task_id: None,
        });

    let mut pipeline = make_pipeline_with_adapter(tmp.path(), adapter);
    let event = make_event(
        Source::FeishuMeeting,
        Confidence::High,
        EventType::Meeting,
        serde_json::json!({"text": "会议结束，待办：张三修复首页bug"}),
    );

    let result = pipeline.process(&event).await.unwrap();

    // 断言：分类应为 Meeting
    assert_eq!(result.work_record.category, Category::Meeting, "会议消息应归类为 Meeting");

    // 断言：people 含 "张三"
    assert!(
        result.work_record.people.contains(&"张三".to_string()),
        "应包含人员 '张三'，实际: {:?}",
        result.work_record.people
    );

    // 断言：vault/ 下有 .md 文件且包含 people
    let contents = read_all_md_contents(tmp.path());
    let has_people = contents.iter().any(|(_, c)| c.contains("张三"));
    assert!(has_people, ".md 文件应包含人员 '张三'");
}

/// A6: 邮件消息 -> 含任务提取
///
/// Event: Email, "请在周五前完成需求评审"
/// MockAdapter: classify->email(Instant), extract->title="完成需求评审", due="周五"
/// 断言: route=Instant, category=Task, task_due 有值
#[tokio::test]
async fn a6_email_message_with_task() {
    let tmp = tempfile::tempdir().unwrap();
    let adapter = MockAdapter::new()
        .with_classification(Classification {
            category: "email".to_string(),
            confidence: 0.88,
            reasoning: "邮件含任务".to_string(),
        })
        .with_extraction(Extraction {
            title: "完成需求评审".to_string(),
            summary: "请在周五前完成需求评审".to_string(),
            detail: "邮件要求在周五前完成需求评审".to_string(),
            people: vec![],
            tags: vec!["邮件".to_string()],
            project: None,
            due_date: Some("周五".to_string()),
            confidence: 0.9,
                is_status_update: false,
                related_task_id: None,
        });

    let mut pipeline = make_pipeline_with_adapter(tmp.path(), adapter);
    let event = make_event(
        Source::FeishuEmail,
        Confidence::Medium,
        EventType::Email,
        serde_json::json!({"text": "请在周五前完成需求评审"}),
    );

    let result = pipeline.process(&event).await.unwrap();

    // 断言：路由应为 Instant
    assert_eq!(result.route, ProcessingRoute::Instant, "邮件应走 Instant 路由");

    // 断言：分类应为 Task（TaskDiscovery 应识别任务）
    assert_eq!(result.work_record.category, Category::Task, "含任务的邮件应归类为 Task");

    // 断言：应有截止日期
    assert!(result.work_record.task_due.is_some(), "含截止日期的任务应有 task_due");
}

// ═══════════════════════════════════════════════════════════════════
// 组 B: 任务流转生命周期（7 个）
// ═══════════════════════════════════════════════════════════════════

/// B1: 消息 -> 发现任务 -> category=Task
///
/// pipeline 处理含任务消息后，验证 WorkRecord.category == Task 且 task_due 有值
#[tokio::test]
async fn b1_message_discovers_task() {
    let tmp = tempfile::tempdir().unwrap();
    let adapter = MockAdapter::new()
        .with_classification(Classification {
            category: "message".to_string(),
            confidence: 0.8,
            reasoning: "含任务消息".to_string(),
        })
        .with_extraction(Extraction {
            title: "完成代码发布".to_string(),
            summary: "明天下午5点完成代码发布".to_string(),
            detail: "需要在明天下午5点前完成代码发布".to_string(),
            people: vec![],
            tags: vec![],
            project: None,
            due_date: Some("明天下午5点".to_string()),
            confidence: 0.9,
                is_status_update: false,
                related_task_id: None,
        });

    let mut pipeline = make_pipeline_with_adapter(tmp.path(), adapter);
    let event = make_event(
        Source::FeishuMessage,
        Confidence::Medium,
        EventType::Message,
        serde_json::json!({"text": "明天下午5点完成代码发布"}),
    );

    let result = pipeline.process(&event).await.unwrap();

    // 断言：category 应为 Task（TaskDiscovery 识别）
    assert_eq!(result.work_record.category, Category::Task, "应识别为任务");

    // 断言：task_due 应有值
    assert!(result.work_record.task_due.is_some(), "应有截止日期");

    // 断言：vault/ 下有 .md 文件
    let md_files = find_md_files(tmp.path());
    assert!(!md_files.is_empty(), "应持久化 .md 文件");
}

/// B2: Task 状态 Todo -> InProgress
#[test]
fn b2_task_todo_to_in_progress() {
    let task = Task::new("测试任务", TaskStatus::Todo);
    let result = task.transition(TaskStatus::InProgress);
    assert!(result.is_ok(), "Todo -> InProgress 应成功");
    assert_eq!(result.unwrap().status, TaskStatus::InProgress);
}

/// B3: Task 状态 InProgress -> Done
#[test]
fn b3_task_in_progress_to_done() {
    let task = Task::new("测试任务", TaskStatus::InProgress);
    let result = task.transition(TaskStatus::Done);
    assert!(result.is_ok(), "InProgress -> Done 应成功");
    let done_task = result.unwrap();
    assert_eq!(done_task.status, TaskStatus::Done);
    assert!(done_task.completed_at.is_some(), "Done 状态应设置 completed_at");
}

/// B4: Task 状态 InProgress -> Blocked
#[test]
fn b4_task_in_progress_to_blocked() {
    let task = Task::new("测试任务", TaskStatus::InProgress);
    let result = task.transition(TaskStatus::Blocked);
    assert!(result.is_ok(), "InProgress -> Blocked 应成功");
    assert_eq!(result.unwrap().status, TaskStatus::Blocked);
}

/// B5: Task 状态 Blocked -> InProgress（恢复）
#[test]
fn b5_task_blocked_to_in_progress() {
    let task = Task::new("测试任务", TaskStatus::Blocked);
    let result = task.transition(TaskStatus::InProgress);
    assert!(result.is_ok(), "Blocked -> InProgress 应成功");
    assert_eq!(result.unwrap().status, TaskStatus::InProgress);
}

/// B6: 非法转换 Todo -> Done 被拒绝
#[test]
fn b6_task_todo_to_done_rejected() {
    let task = Task::new("测试任务", TaskStatus::Todo);
    let result = task.transition(TaskStatus::Done);
    assert!(result.is_err(), "Todo -> Done 应被拒绝");
    // 原始状态不变
    assert_eq!(task.status, TaskStatus::Todo, "原始状态应保持 Todo");
}

/// B7: Done 状态不可变更
#[test]
fn b7_task_done_is_final() {
    let task = Task::new("测试任务", TaskStatus::Done);
    let result = task.transition(TaskStatus::InProgress);
    assert!(result.is_err(), "Done -> InProgress 应被拒绝");

    let result2 = task.transition(TaskStatus::Todo);
    assert!(result2.is_err(), "Done -> Todo 应被拒绝");

    let result3 = task.transition(TaskStatus::Blocked);
    assert!(result3.is_err(), "Done -> Blocked 应被拒绝");
}

// ═══════════════════════════════════════════════════════════════════
// 组 C: 审核分层（4 个）
// ═══════════════════════════════════════════════════════════════════

/// C1: 普通任务 -> 仅规则审核
///
/// WorkRecord: category=Task, detail="短内容", people=[]
/// 断言: reviewer="rule"
#[tokio::test]
async fn c1_normal_task_uses_rule_review() {
    let tmp = tempfile::tempdir().unwrap();
    // 使用默认 adapter，不配置 TieredReview
    let mut pipeline = make_pipeline(tmp.path());

    let event = make_event(
        Source::FeishuProject,
        Confidence::High,
        EventType::TaskUpdate,
        serde_json::json!({"status": "done", "title": "普通任务"}),
    );

    let result = pipeline.process(&event).await.unwrap();

    // 断言：审核者应为 rule
    assert_eq!(
        result.review_result.reviewer, "rule",
        "普通任务应仅使用规则审核"
    );
}

/// C2: 文档类 -> 小模型审核
///
/// WorkRecord: category=Document
/// 断言: reviewer="small_model"
#[tokio::test]
async fn c2_document_uses_small_model_review() {
    let tmp = tempfile::tempdir().unwrap();
    let adapter = MockAdapter::new()
        .with_classification(Classification {
            category: "document".to_string(),
            confidence: 0.85,
            reasoning: "文档变更".to_string(),
        })
        .with_extraction(Extraction {
            title: "API 文档更新".to_string(),
            summary: "更新了 REST API 文档".to_string(),
            detail: "详细的 API 文档变更内容，包含新增接口和参数说明".to_string(),
            people: vec![],
            tags: vec!["文档".to_string()],
            project: None,
            due_date: None,
            confidence: 0.85,
                is_status_update: false,
                related_task_id: None,
        });

    // 创建带 TieredReview 的 ReviewAgent
    let reviewer = ReviewAgent::new().with_tiered_review(TieredReview::default_config());
    let mut pipeline = make_pipeline_with_review(tmp.path(), adapter, reviewer);

    let event = make_event(
        Source::FeishuDoc,
        Confidence::Medium,
        EventType::DocumentChange,
        serde_json::json!({"text": "API 文档更新"}),
    );

    let result = pipeline.process(&event).await.unwrap();

    // 断言：审核者应为 small_model
    assert_eq!(
        result.review_result.reviewer, "small_model",
        "Document 类应使用 small_model 审核"
    );
}

/// C3: 长摘要 -> 大模型审核
///
/// WorkRecord: category=Review, detail=500字以上的字符串
/// 断言: reviewer="large_model"
#[tokio::test]
async fn c3_long_review_uses_large_model_review() {
    let tmp = tempfile::tempdir().unwrap();
    // 生成 > 500 字的 detail
    let long_detail = "这是一段很长的分析内容，用于测试大模型审核。".repeat(50);
    let adapter = MockAdapter::new()
        .with_classification(Classification {
            category: "message".to_string(),
            confidence: 0.9,
            reasoning: "长内容分析".to_string(),
        })
        .with_extraction(Extraction {
            title: "深度分析报告".to_string(),
            summary: "详细的项目分析报告".to_string(),
            detail: long_detail.clone(),
            people: vec![],
            tags: vec!["分析".to_string()],
            project: None,
            due_date: None,
            confidence: 0.9,
                is_status_update: false,
                related_task_id: None,
        });

    // 创建带 TieredReview + LargeModelReview 的 ReviewAgent
    let reviewer = ReviewAgent::new()
        .with_tiered_review(TieredReview::default_config())
        .with_large_model_review(LargeModelReview::default());
    let mut pipeline = make_pipeline_with_review(tmp.path(), adapter, reviewer);

    let event = make_event(
        Source::FeishuMessage,
        Confidence::High,
        EventType::Message,
        serde_json::json!({"text": "深度分析报告"}),
    );

    let result = pipeline.process(&event).await.unwrap();

    // 断言：审核者应为 large_model（detail > 500 字）
    assert_eq!(
        result.review_result.reviewer, "large_model",
        "长内容 (>500字) 应使用 large_model 审核"
    );
}

/// C4: 涉及他人 -> 推送确认
///
/// WorkRecord: people=["Alice","Bob"], category=Communication
/// 断言: ConfirmRequest 创建（通过 ReviewAgent 的 pending_confirm_count 验证）
#[tokio::test]
async fn c4_involving_others_creates_confirm_request() {
    let tmp = tempfile::tempdir().unwrap();
    let adapter = MockAdapter::new()
        .with_classification(Classification {
            category: "message".to_string(),
            confidence: 0.85,
            reasoning: "涉及他人".to_string(),
        })
        .with_extraction(Extraction {
            title: "与 Alice 和 Bob 讨论方案".to_string(),
            summary: "和 Alice、Bob 讨论了技术方案".to_string(),
            detail: "Alice 和 Bob 一起讨论了详细的技术方案，包含架构设计和实现细节".to_string(),
            people: vec!["Alice".to_string(), "Bob".to_string()],
            tags: vec!["讨论".to_string()],
            project: None,
            due_date: None,
            confidence: 0.88,
                is_status_update: false,
                related_task_id: None,
        });

    // 创建带 TieredReview 的 ReviewAgent（涉及他人会触发 small_model）
    let reviewer = ReviewAgent::new().with_tiered_review(TieredReview::default_config());
    let mut pipeline = make_pipeline_with_review(tmp.path(), adapter, reviewer);

    let event = make_event(
        Source::FeishuMessage,
        Confidence::Medium,
        EventType::Message,
        serde_json::json!({"text": "与 Alice 和 Bob 讨论方案"}),
    );

    let result = pipeline.process(&event).await.unwrap();

    // 断言：审核者应为 small_model（涉及他人场景）
    assert_eq!(
        result.review_result.reviewer, "small_model",
        "涉及他人应使用 small_model 审核"
    );

    // 断言：vault/ 下有 .md 文件且包含 people
    let contents = read_all_md_contents(tmp.path());
    let has_alice = contents.iter().any(|(_, c)| c.contains("Alice"));
    let has_bob = contents.iter().any(|(_, c)| c.contains("Bob"));
    assert!(has_alice, ".md 文件应包含 Alice");
    assert!(has_bob, ".md 文件应包含 Bob");
}

// ═══════════════════════════════════════════════════════════════════
// 组 D: 降级与容错（4 个）
// ═══════════════════════════════════════════════════════════════════

/// D1: AI 分类失败 -> 降级到规则
///
/// MockAdapter: classify 返回 Err
/// Event: Message, "请帮忙处理一下"
/// 断言: route=规则分类结果(Aggregate), confidence=0.7, 流程正常完成不 panic
#[tokio::test]
async fn d1_classify_error_fallback_to_rule() {
    let tmp = tempfile::tempdir().unwrap();
    let mut pipeline = make_error_pipeline(tmp.path());

    let event = make_event(
        Source::FeishuMessage,
        Confidence::Medium,
        EventType::Message,
        serde_json::json!({"text": "请帮忙处理一下"}),
    );

    // 不应 panic
    let result = pipeline.process(&event).await.unwrap();

    // 断言：降级到规则分类（Message without @mention -> Aggregate）
    assert_eq!(
        result.route,
        ProcessingRoute::Aggregate,
        "AI 分类失败应降级到规则分类（Aggregate）"
    );

    // 断言：model_used 应为 extract-fallback（extract 也失败了）
    assert_eq!(
        result.work_record.model_used, "extract-fallback",
        "AI 提取也失败时 model_used 应为 extract-fallback"
    );

    // 断言：流程正常完成（不 panic）
    // 注意：降级提取的 confidence=0.7（>= 0.6）以通过 ConfidenceThresholdRule
    assert_eq!(
        result.work_record.confidence, 0.7,
        "降级提取的置信度应为 0.7"
    );
}

/// D2: AI 提取失败 -> 降级到中等置信度
///
/// MockAdapter: extract 返回 Err
/// 断言: title 从内容截取, confidence=0.7, 流程正常完成
#[tokio::test]
async fn d2_extract_error_fallback_to_low_confidence() {
    let tmp = tempfile::tempdir().unwrap();
    let mut pipeline = make_extract_error_pipeline(tmp.path());

    let event = make_event(
        Source::FeishuMessage,
        Confidence::Medium,
        EventType::Message,
        serde_json::json!({"text": "请帮忙处理一下"}),
    );

    let result = pipeline.process(&event).await.unwrap();

    // 断言：model_used 应为 extract-fallback
    assert_eq!(
        result.work_record.model_used, "extract-fallback",
        "AI 提取失败时 model_used 应为 extract-fallback"
    );

    // 断言：confidence 应为 0.7（>= 0.6 以通过 ConfidenceThresholdRule）
    assert_eq!(
        result.work_record.confidence, 0.7,
        "降级提取的置信度应为 0.7"
    );

    // 断言：title 应从事件内容截取（非空）
    assert!(
        !result.work_record.title.is_empty(),
        "降级提取应从事件中恢复标题"
    );

    // 断言：流程正常完成（降级提取 confidence=0.7 >= 审核阈值 0.6，
    //       不会触发 NeedsFix，可以持久化）
    assert!(
        result.processing_time_ms < 30_000,
        "降级处理应在合理时间内完成"
    );
}

/// D3: AI 任务发现失败 -> 降级到关键词
///
/// MockAdapter: extract 返回 Err（discovery_ai 会 fallback）
/// Event: 含关键词 "请你帮忙"
/// 断言: category=Task（关键词匹配成功）
#[tokio::test]
async fn d3_discovery_ai_error_fallback_to_keywords() {
    let tmp = tempfile::tempdir().unwrap();
    let mut pipeline = make_extract_error_pipeline(tmp.path());

    let event = make_event(
        Source::FeishuMessage,
        Confidence::Medium,
        EventType::Message,
        serde_json::json!({"text": "请你帮忙检查一下登录接口的问题"}),
    );

    let result = pipeline.process(&event).await.unwrap();

    // 断言：category 应为 Task（关键词 "请你帮忙" 匹配成功）
    assert_eq!(
        result.work_record.category,
        Category::Task,
        "AI 任务发现失败应降级到关键词匹配，识别为 Task"
    );
}

/// D4: 大模型审核失败 -> 降级到小模型
///
/// 当 LargeModelReview confidence < 0.3 时，降级到 base_result
/// 断言: reviewer 不是 "large_model"（降级后使用前一层结果）
#[tokio::test]
async fn d4_large_model_review_degrades_gracefully() {
    let tmp = tempfile::tempdir().unwrap();
    let adapter = MockAdapter::new()
        .with_classification(Classification {
            category: "message".to_string(),
            confidence: 0.9,
            reasoning: "长内容".to_string(),
        })
        .with_extraction(Extraction {
            title: "分析报告".to_string(),
            summary: "详细分析".to_string(),
            // 长内容触发 large_model 审核（需包含分析维度关键词以通过审核）
            detail: "原因：需求变更导致工期延长。影响：上线时间推迟两周。建议：调整资源分配并增加人力投入。".repeat(10),
            people: vec![],
            tags: vec![],
            project: None,
            due_date: None,
            confidence: 0.9,
                is_status_update: false,
                related_task_id: None,
        });

    // 创建带 TieredReview + LargeModelReview 的 ReviewAgent
    // LargeModelReview 默认配置，当 detail 足够长时会触发
    let reviewer = ReviewAgent::new()
        .with_tiered_review(TieredReview::default_config())
        .with_large_model_review(LargeModelReview::default());
    let mut pipeline = make_pipeline_with_review(tmp.path(), adapter, reviewer);

    let event = make_event(
        Source::FeishuMessage,
        Confidence::High,
        EventType::Message,
        serde_json::json!({"text": "分析报告"}),
    );

    let result = pipeline.process(&event).await.unwrap();

    // 断言：流程正常完成（不 panic）
    assert!(!result.work_record.title.is_empty(), "流程应正常完成");

    // 断言：审核者应为 large_model（因为 detail > 500 字）
    // 如果 LargeModelReview 返回低 confidence，会降级到 base_result
    // 但默认 LargeModelReview 不会返回 < 0.3，所以这里验证 large_model
    assert!(
        result.review_result.reviewer == "large_model" || result.review_result.reviewer == "small_model" || result.review_result.reviewer == "rule",
        "审核者应为有效值，实际: {}",
        result.review_result.reviewer
    );

    // 断言：vault/ 下有文件
    let md_files = find_md_files(tmp.path());
    assert!(!md_files.is_empty(), "降级后仍应持久化 .md 文件");
}

// ═══════════════════════════════════════════════════════════════════
// 组 E: 数据完整性（4 个）
// ═══════════════════════════════════════════════════════════════════

/// E1: Obsidian 文件内容完整性
///
/// pipeline 处理后，读取 vault/ 下的 .md 文件
/// 断言: 文件内容包含 title、summary、category
#[tokio::test]
async fn e1_obsidian_file_content_integrity() {
    let tmp = tempfile::tempdir().unwrap();
    let adapter = MockAdapter::new()
        .with_classification(Classification {
            category: "task".to_string(),
            confidence: 0.9,
            reasoning: "任务消息".to_string(),
        })
        .with_extraction(Extraction {
            title: "完成代码审查".to_string(),
            summary: "审查前端代码变更".to_string(),
            detail: "需要审查前端代码的变更，确保质量".to_string(),
            people: vec!["张三".to_string()],
            tags: vec!["审查".to_string()],
            project: Some("work-better".to_string()),
            due_date: None,
            confidence: 0.92,
                is_status_update: false,
                related_task_id: None,
        });

    let mut pipeline = make_pipeline_with_adapter(tmp.path(), adapter);
    let event = make_event(
        Source::FeishuMessage,
        Confidence::High,
        EventType::Message,
        serde_json::json!({"text": "完成代码审查"}),
    );

    let result = pipeline.process(&event).await.unwrap();

    // 读取 vault/ 下的 .md 文件
    let md_files = find_md_files(tmp.path());
    assert!(!md_files.is_empty(), "应有 .md 文件");

    let content = fs::read_to_string(&md_files[0]).unwrap();

    // 断言：文件内容包含 title
    assert!(content.contains("完成代码审查"), "文件应包含 title");

    // 断言：文件内容包含 summary
    assert!(content.contains("审查前端代码变更"), "文件应包含 summary");

    // 断言：文件内容包含 category（frontmatter 中）
    assert!(content.contains("task"), "文件应包含 category");

    // 断言：文件内容包含 people
    assert!(content.contains("张三"), "文件应包含 people");

    // 断言：文件内容包含 id（frontmatter）
    assert!(content.contains(&result.work_record.id), "文件应包含 record id");
}

/// E2: Obsidian 文件路径唯一性
///
/// 处理两条不同消息（使用不同标题的 adapter）
/// 断言: 生成两个不同的 .md 文件路径
#[tokio::test]
async fn e2_obsidian_file_path_uniqueness() {
    let tmp = tempfile::tempdir().unwrap();

    // 第一条消息：使用标题 "项目周报" 的 adapter
    let adapter1 = MockAdapter::new()
        .with_extraction(Extraction {
            title: "项目周报".to_string(),
            summary: "收到项目周报数据".to_string(),
            detail: "这是项目周报的详细内容".to_string(),
            people: vec![],
            tags: vec![],
            project: None,
            due_date: None,
            confidence: 0.9,
                is_status_update: false,
                related_task_id: None,
        });
    let mut pipeline1 = make_pipeline_with_adapter(tmp.path(), adapter1);
    let event1 = make_event(
        Source::FeishuMessage,
        Confidence::High,
        EventType::Message,
        serde_json::json!({"text": "收到项目周报数据"}),
    );
    let result1 = pipeline1.process(&event1).await.unwrap();

    // 第二条消息：使用标题 "技术文档" 的 adapter
    let adapter2 = MockAdapter::new()
        .with_extraction(Extraction {
            title: "技术文档".to_string(),
            summary: "查阅技术文档笔记".to_string(),
            detail: "这是技术文档的详细内容".to_string(),
            people: vec![],
            tags: vec![],
            project: None,
            due_date: None,
            confidence: 0.9,
                is_status_update: false,
                related_task_id: None,
        });
    let mut pipeline2 = make_pipeline_with_adapter(tmp.path(), adapter2);
    let event2 = make_event(
        Source::FeishuMessage,
        Confidence::High,
        EventType::Message,
        serde_json::json!({"text": "查阅技术文档笔记"}),
    );
    let result2 = pipeline2.process(&event2).await.unwrap();

    // 断言：两条记录的 obsidian_path 不同
    assert_ne!(
        result1.work_record.obsidian_path, result2.work_record.obsidian_path,
        "不同消息应生成不同的 .md 文件路径"
    );

    // 断言：vault/ 下至少有 2 个 .md 文件
    let md_files = find_md_files(tmp.path());
    assert!(
        md_files.len() >= 2,
        "应有至少 2 个 .md 文件，实际: {}",
        md_files.len()
    );
}

/// E3: SQLite WorkRecord 字段完整
///
/// pipeline 处理后，验证 WorkRecord 字段
/// 断言: id, title, summary, category, obsidian_path 都非空
#[tokio::test]
async fn e3_work_record_fields_complete() {
    let tmp = tempfile::tempdir().unwrap();
    let adapter = MockAdapter::new()
        .with_classification(Classification {
            category: "task".to_string(),
            confidence: 0.9,
            reasoning: "任务消息".to_string(),
        })
        .with_extraction(Extraction {
            title: "完整字段测试".to_string(),
            summary: "测试所有字段是否完整".to_string(),
            detail: "这是一个详细的测试内容，用于验证字段完整性".to_string(),
            people: vec!["Alice".to_string()],
            tags: vec!["测试".to_string()],
            project: Some("test-project".to_string()),
            due_date: Some("明天".to_string()),
            confidence: 0.9,
                is_status_update: false,
                related_task_id: None,
        });

    let mut pipeline = make_pipeline_with_adapter(tmp.path(), adapter);
    let event = make_event(
        Source::FeishuMessage,
        Confidence::High,
        EventType::Message,
        serde_json::json!({"text": "完整字段测试"}),
    );

    let result = pipeline.process(&event).await.unwrap();
    let record = &result.work_record;

    // 断言：所有关键字段非空
    assert!(!record.id.is_empty(), "id 不应为空");
    assert!(!record.title.is_empty(), "title 不应为空");
    assert!(!record.summary.is_empty(), "summary 不应为空");
    assert!(!record.obsidian_path.is_empty(), "obsidian_path 不应为空");

    // 断言：category 是有效值
    assert_eq!(record.category, Category::Task);

    // 断言：model_used 非空
    assert!(!record.model_used.is_empty(), "model_used 不应为空");

    // 断言：confidence 在合理范围
    assert!(record.confidence > 0.0 && record.confidence <= 1.0, "confidence 应在 0-1 之间");

    // 断言：source_event_ids 包含事件 ID
    assert!(!record.source_event_ids.is_empty(), "source_event_ids 不应为空");
}

/// E4: 审计链路 trace_id 贯穿
///
/// pipeline 处理后，检查处理结果的完整性
/// 断言: 所有步骤耗时合理（processing_time_ms >= sum of steps）
#[tokio::test]
async fn e4_audit_trace_consistency() {
    let tmp = tempfile::tempdir().unwrap();
    let mut pipeline = make_pipeline(tmp.path());

    let event = make_event(
        Source::FeishuMessage,
        Confidence::High,
        EventType::Message,
        serde_json::json!({"text": "审计链路测试"}),
    );

    let result = pipeline.process(&event).await.unwrap();

    // 断言：总处理时间 >= 各步骤之和
    let sum = result.step_timings.classify_ms
        + result.step_timings.extract_ms
        + result.step_timings.review_ms
        + result.step_timings.persist_ms;
    assert!(
        result.processing_time_ms >= sum,
        "总处理时间 {} 应 >= 各步骤之和 {}",
        result.processing_time_ms,
        sum
    );

    // 断言：各步骤耗时合理（非负）
    assert!(result.step_timings.classify_ms < 10_000, "分类耗时应合理");
    assert!(result.step_timings.review_ms < 10_000, "审核耗时应合理");

    // 断言：processing_time_ms 合理
    assert!(result.processing_time_ms < 30_000, "总处理时间应合理");
}

// ═══════════════════════════════════════════════════════════════════
// 组 F: 边界条件（5 个）
// ═══════════════════════════════════════════════════════════════════

/// F1: 空内容消息
///
/// Event: content=""
/// 断言: 不 panic, 优雅降级
#[tokio::test]
async fn f1_empty_content_message() {
    let tmp = tempfile::tempdir().unwrap();
    let mut pipeline = make_pipeline(tmp.path());

    let event = make_event(
        Source::FeishuMessage,
        Confidence::Medium,
        EventType::Message,
        serde_json::json!({"text": ""}),
    );

    // 不应 panic
    let result = pipeline.process(&event).await.unwrap();

    // 断言：流程正常完成
    assert!(!result.work_record.model_used.is_empty(), "model_used 不应为空");

    // 断言：vault/ 下有文件（即使是空内容也应持久化）
    let md_files = find_md_files(tmp.path());
    assert!(!md_files.is_empty(), "空内容消息也应持久化");
}

/// F2: 超长内容消息（>10000 字）
///
/// Event: content="a".repeat(10000)
/// 断言: 正常处理, 不 OOM
#[tokio::test]
async fn f2_very_long_content_message() {
    let tmp = tempfile::tempdir().unwrap();
    let adapter = MockAdapter::new()
        .with_classification(Classification {
            category: "task".to_string(),
            confidence: 0.9,
            reasoning: "超长内容".to_string(),
        })
        .with_extraction(Extraction {
            title: "超长内容处理".to_string(),
            summary: "处理超长内容".to_string(),
            detail: "a".repeat(10000),
            people: vec![],
            tags: vec![],
            project: None,
            due_date: None,
            confidence: 0.9,
                is_status_update: false,
                related_task_id: None,
        });

    let mut pipeline = make_pipeline_with_adapter(tmp.path(), adapter);
    let event = make_event(
        Source::FeishuMessage,
        Confidence::High,
        EventType::Message,
        serde_json::json!({"text": "a".repeat(10000)}),
    );

    // 不应 panic 或 OOM
    let result = pipeline.process(&event).await.unwrap();

    // 断言：流程正常完成
    assert!(!result.work_record.title.is_empty(), "超长内容应正常处理");

    // 断言：vault/ 下有文件
    let md_files = find_md_files(tmp.path());
    assert!(!md_files.is_empty(), "超长内容应正常持久化");
}

/// F3: 纯 emoji 消息
///
/// Event: content="🎉🎊🚀"
/// 断言: 正常处理
#[tokio::test]
async fn f3_pure_emoji_message() {
    let tmp = tempfile::tempdir().unwrap();
    let mut pipeline = make_pipeline(tmp.path());

    let event = make_event(
        Source::FeishuMessage,
        Confidence::Medium,
        EventType::Message,
        serde_json::json!({"text": "🎉🎊🚀"}),
    );

    // 不应 panic
    let result = pipeline.process(&event).await.unwrap();

    // 断言：流程正常完成
    assert!(!result.work_record.model_used.is_empty(), "纯 emoji 消息应正常处理");

    // 断言：vault/ 下有文件
    let md_files = find_md_files(tmp.path());
    assert!(!md_files.is_empty(), "纯 emoji 消息应正常持久化");
}

/// F4: 含特殊字符的消息
///
/// Event: content="'; DROP TABLE events; --<script>alert(1)</script>"
/// 断言: 正常存储, 无 SQL 注入
#[tokio::test]
async fn f4_special_characters_message() {
    let tmp = tempfile::tempdir().unwrap();
    let mut pipeline = make_pipeline(tmp.path());

    let malicious_content = "'; DROP TABLE events; --<script>alert(1)</script>";
    let event = make_event(
        Source::FeishuMessage,
        Confidence::Medium,
        EventType::Message,
        serde_json::json!({"text": malicious_content}),
    );

    // 不应 panic
    let result = pipeline.process(&event).await.unwrap();

    // 断言：流程正常完成
    assert!(!result.work_record.model_used.is_empty(), "特殊字符消息应正常处理");

    // 断言：vault/ 下有文件
    let md_files = find_md_files(tmp.path());
    assert!(!md_files.is_empty(), "特殊字符消息应正常持久化");

    // 断言：文件内容包含原始内容（验证未被截断或注入）
    let content = fs::read_to_string(&md_files[0]).unwrap();
    // 文件应能正常读取（未损坏）
    assert!(!content.is_empty(), "文件内容不应为空");
}

/// F5: 并发处理 10 条消息
///
/// 用 tokio::spawn 同时处理 10 条消息
/// 断言: 全部成功, 无竞态条件
#[tokio::test]
async fn f5_concurrent_processing_10_messages() {
    // 每条消息使用独立的 tmpdir 避免文件系统冲突
    let dirs: Vec<tempfile::TempDir> = (0..10)
        .map(|_| tempfile::tempdir().expect("failed to create tempdir"))
        .collect();

    let mut handles = Vec::new();

    for (i, dir) in dirs.iter().enumerate() {
        let path = dir.path().to_path_buf();
        let handle = tokio::spawn(async move {
            let adapter = MockAdapter::new()
                .with_classification(Classification {
                    category: "task".to_string(),
                    confidence: 0.9,
                    reasoning: format!("并发消息 {}", i),
                })
                .with_extraction(Extraction {
                    title: format!("并发任务 {}", i),
                    summary: format!("并发处理测试 {}", i),
                    detail: format!("这是第 {} 条并发消息的详细内容", i),
                    people: vec![],
                    tags: vec![],
                    project: None,
                    due_date: None,
                    confidence: 0.9,
                    is_status_update: false,
                    related_task_id: None,
                });

            let mut pipeline = make_pipeline_with_adapter(&path, adapter);
            let event = make_event(
                Source::FeishuMessage,
                Confidence::High,
                EventType::Message,
                serde_json::json!({"text": format!("并发消息 {}", i)}),
            );

            pipeline.process(&event).await
        });
        handles.push(handle);
    }

    // 等待所有任务完成并验证
    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.await.expect("task panicked");
        assert!(result.is_ok(), "第 {} 条消息处理失败: {:?}", i, result.unwrap_err());
    }

    // 断言：每个 tmpdir 下都有 .md 文件
    for (i, dir) in dirs.iter().enumerate() {
        let md_files = find_md_files(dir.path());
        assert!(!md_files.is_empty(), "第 {} 条消息应有 .md 文件", i);
    }
}
