//! L4 Pipeline 验收测试 — 步骤定义
//!
//! 使用真实 ProcessingPipeline + MockAdapter 验证完整流水线行为。
//! 每个场景通过 cucumber feature 文件驱动。

use std::collections::HashMap;

use cucumber::{given, then, when};
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

use crate::world::AcceptanceWorld;

// ─── 辅助函数 ──────────────────────────────────────────────

/// 使用自定义 MockAdapter 创建 ProcessingPipeline
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
    ProcessingPipeline::new(runner)
}

/// 使用默认 MockAdapter 创建 ProcessingPipeline
fn make_pipeline(tmp_dir: &std::path::Path) -> ProcessingPipeline {
    make_pipeline_with_adapter(tmp_dir, MockAdapter::new())
}

/// 将 ProcessedResult 存入 world 的 state map
fn store_result(world: &mut AcceptanceWorld, result: &wb_processor::pipeline::ProcessedResult) {
    world.route = Some(result.route.clone());
    world.work_record = Some(result.work_record.clone());
    world.review_result = Some(result.review_result.clone());
    world.model_used = Some(result.work_record.model_used.clone());
    world
        .state
        .insert("route".to_string(), format!("{:?}", result.route));
}

// ── Given ──────────────────────────────────────────────────

#[given(regex = r#"^飞书消息 "(.*)""#)]
fn given_feishu_message(world: &mut AcceptanceWorld, text: String) {
    let event = Event::new(
        Source::FeishuMessage,
        Confidence::Medium,
        EventType::Message,
        serde_json::json!({"text": text}),
        "{}".to_string(),
    );
    world.pending_event = Some(event);
    world.event_content = Some(text);
}

#[given(regex = r#"^审批事件 "(.*)""#)]
fn given_approval_event(world: &mut AcceptanceWorld, text: String) {
    let event = Event::new(
        Source::FeishuApproval,
        Confidence::High,
        EventType::Approval,
        serde_json::json!({"text": text, "approved": true}),
        "{}".to_string(),
    );
    world.pending_event = Some(event);
    world.event_content = Some(text);
}

#[given(regex = r#"^会议事件 "(.*)""#)]
fn given_meeting_event(world: &mut AcceptanceWorld, text: String) {
    let event = Event::new(
        Source::FeishuMeeting,
        Confidence::High,
        EventType::Meeting,
        serde_json::json!({"text": text}),
        "{}".to_string(),
    );
    world.pending_event = Some(event);
    world.event_content = Some(text);
}

#[given(regex = r"^低置信度应用切换事件$")]
fn given_low_confidence_app_switch(world: &mut AcceptanceWorld) {
    let event = Event::new(
        Source::SystemAppSwitch,
        Confidence::Low,
        EventType::AppActivity,
        serde_json::json!({"app": "Safari", "duration_min": 5}),
        "{}".to_string(),
    );
    world.pending_event = Some(event);
    world.event_content = Some("Safari 切换".to_string());
}

// ── When ───────────────────────────────────────────────────

#[when(regex = r"^流水线处理$")]
async fn when_pipeline_process(world: &mut AcceptanceWorld) {
    let event = match world.pending_event.clone() {
        Some(e) => e,
        None => {
            world.error = Some("No pending event".to_string());
            return;
        }
    };

    let content_text = world.event_content.clone().unwrap_or_default();
    let tmp = tempdir().expect("Failed to create temp dir");

    // 低置信度事件：使用默认 pipeline（Classifier 会直接 Archive）
    if event.source_confidence == Confidence::Low {
        let mut pipeline = make_pipeline(tmp.path());
        match pipeline.process(&event).await {
            Ok(r) => store_result(world, &r),
            Err(e) => world.error = Some(e.to_string()),
        }
        return;
    }

    // 审批事件
    if event.event_type == EventType::Approval {
        let adapter = MockAdapter::new()
            .with_classification(Classification {
                category: "approval".to_string(),
                confidence: 0.9,
                reasoning: "审批事件".to_string(),
            })
            .with_extraction(Extraction {
                title: content_text.clone(),
                summary: content_text.clone(),
                detail: content_text.clone(),
                people: vec![],
                tags: vec!["审批".to_string()],
                project: None,
                due_date: None,
                confidence: 0.9,
                is_status_update: false,
                related_task_id: None,
            });
        let mut pipeline = make_pipeline_with_adapter(tmp.path(), adapter);
        match pipeline.process(&event).await {
            Ok(r) => store_result(world, &r),
            Err(e) => world.error = Some(e.to_string()),
        }
        return;
    }

    // 会议事件
    if event.event_type == EventType::Meeting {
        let adapter = MockAdapter::new()
            .with_classification(Classification {
                category: "meeting".to_string(),
                confidence: 0.85,
                reasoning: "会议事件".to_string(),
            })
            .with_extraction(Extraction {
                title: content_text.clone(),
                summary: content_text.clone(),
                detail: content_text.clone(),
                people: vec!["张三".to_string()],
                tags: vec!["会议".to_string()],
                project: None,
                due_date: None,
                confidence: 0.85,
                is_status_update: false,
                related_task_id: None,
            });
        let mut pipeline = make_pipeline_with_adapter(tmp.path(), adapter);
        match pipeline.process(&event).await {
            Ok(r) => store_result(world, &r),
            Err(e) => world.error = Some(e.to_string()),
        }
        return;
    }

    // 普通消息：根据内容判断分类
    let is_task_like = content_text.contains("完成")
        || content_text.contains("发布")
        || content_text.contains("排查")
        || content_text.contains("修复")
        || content_text.contains("需要");

    if is_task_like {
        let adapter = MockAdapter::new()
            .with_classification(Classification {
                category: "task".to_string(),
                confidence: 0.9,
                reasoning: "contains actionable task".to_string(),
            })
            .with_extraction(Extraction {
                title: content_text.clone(),
                summary: content_text.clone(),
                detail: content_text.clone(),
                people: vec![],
                tags: vec![],
                project: None,
                due_date: if content_text.contains("明天") {
                    Some("明天下午5点".to_string())
                } else {
                    None
                },
                confidence: 0.9,
                is_status_update: false,
                related_task_id: None,
            });
        let mut pipeline = make_pipeline_with_adapter(tmp.path(), adapter);
        match pipeline.process(&event).await {
            Ok(r) => store_result(world, &r),
            Err(e) => world.error = Some(e.to_string()),
        }
    } else {
        let adapter = MockAdapter::new()
            .with_classification(Classification {
                category: "message".to_string(),
                confidence: 0.8,
                reasoning: "普通聊天消息".to_string(),
            })
            .with_extraction(Extraction {
                title: content_text.clone(),
                summary: content_text.clone(),
                detail: content_text.clone(),
                people: vec![],
                tags: vec![],
                project: None,
                due_date: None,
                confidence: 0.0, // 低于 AI_CONFIDENCE_THRESHOLD → 不触发任务发现
                is_status_update: false,
                related_task_id: None,
            });
        let mut pipeline = make_pipeline_with_adapter(tmp.path(), adapter);
        match pipeline.process(&event).await {
            Ok(r) => store_result(world, &r),
            Err(e) => world.error = Some(e.to_string()),
        }
    }
}

// ── Then ───────────────────────────────────────────────────

#[then(regex = r#"^标题包含 "(.*)""#)]
fn then_title_contains(world: &mut AcceptanceWorld, expected: String) {
    let record = world
        .work_record
        .as_ref()
        .expect("work_record should be set after pipeline processing");
    assert!(
        record.title.contains(&expected),
        "标题应包含 '{}'，实际: '{}'",
        expected,
        record.title
    );
}

#[then(regex = r"^分类为 Task$")]
fn then_category_is_task(world: &mut AcceptanceWorld) {
    let record = world
        .work_record
        .as_ref()
        .expect("work_record should be set");
    assert_eq!(
        record.category,
        Category::Task,
        "分类应为 Task，实际: {:?}",
        record.category
    );
}

#[then(regex = r"^分类为 Meeting$")]
fn then_category_is_meeting(world: &mut AcceptanceWorld) {
    let record = world
        .work_record
        .as_ref()
        .expect("work_record should be set");
    assert_eq!(
        record.category,
        Category::Meeting,
        "分类应为 Meeting，实际: {:?}",
        record.category
    );
}

#[then(regex = r"^分类不为 Task$")]
fn then_category_not_task(world: &mut AcceptanceWorld) {
    let record = world
        .work_record
        .as_ref()
        .expect("work_record should be set");
    assert_ne!(
        record.category,
        Category::Task,
        "分类不应为 Task，实际: {:?}",
        record.category
    );
}

#[then(regex = r"^有截止日期$")]
fn then_has_due_date(world: &mut AcceptanceWorld) {
    let record = world
        .work_record
        .as_ref()
        .expect("work_record should be set");
    assert!(
        record.task_due.is_some(),
        "应有截止日期 (task_due)，实际: None"
    );
}

#[then(regex = r"^无截止日期$")]
fn then_no_due_date(world: &mut AcceptanceWorld) {
    let record = world
        .work_record
        .as_ref()
        .expect("work_record should be set");
    assert!(
        record.task_due.is_none(),
        "不应有截止日期，实际: {:?}",
        record.task_due
    );
}

#[then(regex = r"^提取步骤已执行$")]
fn then_extract_executed(world: &mut AcceptanceWorld) {
    let model_used = world.model_used.as_deref().unwrap_or("");
    assert_ne!(
        model_used, "archive-skip",
        "提取步骤应已执行（model_used 不应为 archive-skip），实际: {}",
        model_used
    );
}

#[then(regex = r"^提取步骤未执行$")]
fn then_extract_not_executed(world: &mut AcceptanceWorld) {
    let model_used = world.model_used.as_deref().unwrap_or("");
    assert_eq!(
        model_used, "archive-skip",
        "提取步骤不应执行（model_used 应为 archive-skip），实际: {}",
        model_used
    );
}

#[then(regex = r"^路由为 Aggregate$")]
fn then_route_aggregate(world: &mut AcceptanceWorld) {
    let route = world.route.as_ref().expect("route should be set");
    assert_eq!(
        *route,
        ProcessingRoute::Aggregate,
        "路由应为 Aggregate，实际: {:?}",
        route
    );
}

#[then(regex = r"^路由为 Instant$")]
fn then_route_instant(world: &mut AcceptanceWorld) {
    let route = world.route.as_ref().expect("route should be set");
    assert_eq!(
        *route,
        ProcessingRoute::Instant,
        "路由应为 Instant，实际: {:?}",
        route
    );
}

#[then(regex = r"^路由为 Archive$")]
fn then_route_archive(world: &mut AcceptanceWorld) {
    let route = world.route.as_ref().expect("route should be set");
    assert_eq!(
        *route,
        ProcessingRoute::Archive,
        "路由应为 Archive，实际: {:?}",
        route
    );
}

#[then(regex = r#"^涉及人员包含 "(.*)""#)]
fn then_people_contains(world: &mut AcceptanceWorld, name: String) {
    let record = world
        .work_record
        .as_ref()
        .expect("work_record should be set");
    assert!(
        record.people.contains(&name),
        "人员列表应包含 '{}'，实际: {:?}",
        name,
        record.people
    );
}
