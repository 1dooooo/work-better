//! ProcessingPipeline —— 事件处理流水线
//!
//! 将分类 → 路由 → 模型 → 提取 → 审核 → 持久化串联为完整流水线。

use std::time::Instant;

use wb_core::audit::{ReviewResult, ReviewVerdict};
use wb_core::error::Result;
use wb_core::event::{Event, EventType};
use wb_core::record::WorkRecord;

use crate::classifier::{Classifier, ProcessingRoute};
use crate::extraction::{EntityExtractor, ExtractedData};
use crate::persist::{Deduplicator, PersistStep};
use crate::reviewer::ReviewAgent;
use crate::task::discovery::TaskDiscovery;

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
    deduplicator: Option<Deduplicator>,
    task_discovery: TaskDiscovery,
}

impl ProcessingPipeline {
    /// 创建新的处理流水线
    pub fn new(task_runner: TaskRunner, persistor: PersistStep) -> Self {
        Self {
            classifier: Classifier,
            task_runner,
            reviewer: ReviewAgent::new(),
            persistor,
            deduplicator: None,
            task_discovery: TaskDiscovery::new(),
        }
    }

    /// 使用自定义 ReviewAgent
    pub fn with_reviewer(mut self, reviewer: ReviewAgent) -> Self {
        self.reviewer = reviewer;
        self
    }

    /// 启用语义去重
    pub fn with_deduplicator(mut self, deduplicator: Deduplicator) -> Self {
        self.deduplicator = Some(deduplicator);
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

        // Step 1: 规则分类
        let classify_start = Instant::now();
        let rule_route = Classifier::classify(event);

        // Step 1.5: AI 二次分类（非 Archive 路由）
        let route = if rule_route == ProcessingRoute::Archive {
            rule_route
        } else {
            let initial_confidence = Self::event_initial_confidence(event);
            match self
                .task_runner
                .run_classify(event, initial_confidence)
                .await
            {
                Ok(ai_output) => {
                    if let Ok(classification) =
                        serde_json::from_str::<wb_ai::Classification>(&ai_output.content)
                    {
                        let ai_route = Self::category_to_route(&classification.category);
                        if ai_route != rule_route {
                            tracing::info!(
                                "classify_ai_override: rule={:?} ai={:?}",
                                rule_route,
                                ai_route
                            );
                        }
                        ai_route
                    } else {
                        tracing::info!("classify_ai_fallback");
                        rule_route
                    }
                }
                Err(_) => {
                    tracing::info!("classify_ai_fallback");
                    rule_route
                }
            }
        };
        timings.classify_ms = classify_start.elapsed().as_millis() as u64;

        // Step 2 & 3: 模型提取（Archive 路由跳过）
        let (extracted_data, model_name) = match route {
            ProcessingRoute::Archive => {
                let data = ExtractedData {
                    title: Self::extract_title_from_event(event),
                    summary: "Auto-archived event".to_string(),
                    detail: serde_json::to_string(&event.content).unwrap_or_default(),
                    category: Self::map_event_to_category(event),
                    project: None,
                    people: vec![],
                    tags: vec![],
                    task_status: None,
                    confidence: 0.4,
                    due_date: None,
                };
                (data, "archive-skip".to_string())
            }
            _ => {
                let extract_start = Instant::now();
                let category = Self::map_event_to_category(event);
                match self
                    .task_runner
                    .run_extract(event, Self::event_initial_confidence(event))
                    .await
                {
                    Ok(output) => {
                        timings.extract_ms = extract_start.elapsed().as_millis() as u64;
                        let data = EntityExtractor::extract(&output.content, &category);
                        (data, output.model_used)
                    }
                    Err(e) => {
                        tracing::info!("extract_ai_fallback: {:?}", e);
                        timings.extract_ms = extract_start.elapsed().as_millis() as u64;
                        // 降级：从事件内容直接构建提取数据
                        let data = Self::fallback_extract_from_event(event, &category);
                        (data, "extract-fallback".to_string())
                    }
                }
            }
        };

        // Step 4: 构建 WorkRecord
        let mut record = EntityExtractor::to_work_record(&extracted_data, event, &model_name);

        // 为 Archive 路由设置默认 obsidian_path
        if record.obsidian_path.is_empty() {
            record.obsidian_path = PersistStep::generate_path(&record);
        }

        // Step 4.5: Task Discovery（AI 驱动，检查是否包含任务）
        // 仅对文本类事件运行任务发现，避免对 Approval、Browsing 等产生误报
        // 通过 TaskRunner 的 ModelRouter 决定使用小模型还是大模型
        if Self::is_text_rich_event(event) {
            let event_text = Self::extract_text_from_event(event);
            let discovery_tasks = self.task_discovery
                .discover_with_ai(&event_text, &mut self.task_runner, event.source.clone())
                .await;
            if let Some(candidate) = discovery_tasks.first() {
                record.category = wb_core::record::Category::Task;
                if let Some(ref due) = candidate.due_date {
                    let sanitized_due = sanitize_text_input(due, 100);
                    record.task_due = crate::extraction::parse_due_date_from_text(&sanitized_due);
                }
                // TaskPriority 是 Rust 枚举，format!("{:?}", ...) 始终产生合法值
                record.task_priority = Some(format!("{:?}", candidate.priority));
                tracing::info!("task_discovered: title={}", candidate.title);
            }
        }

        // Step 4.6: 语义去重（如果启用了 Deduplicator）
        if let Some(ref dedup) = self.deduplicator {
            if let Some(existing_id) = dedup.find_similar(&record).await {
                // 将当前事件的 source_event_ids 合并到已有记录的路径
                // 通过修改 obsidian_path 指向已有文件，触发 PersistStep 的 LCS 去重
                record.obsidian_path = Self::find_existing_path_by_id(
                    &self.persistor, &existing_id, &record.obsidian_path,
                ).unwrap_or(record.obsidian_path);
            }
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

            // 持久化成功后索引到向量库（用于后续去重）
            if let Some(ref dedup) = self.deduplicator {
                dedup.index_record(&record).await;
            }
        }

        Ok(ProcessedResult {
            work_record: record,
            review_result,
            processing_time_ms: total_start.elapsed().as_millis() as u64,
            route,
            step_timings: timings,
        })
    }

    /// 降级提取：当模型提取失败时，从事件内容直接构建 ExtractedData
    fn fallback_extract_from_event(event: &Event, category: &wb_core::record::Category) -> ExtractedData {
        let title = Self::extract_title_from_event(event);
        let content_str = match &event.content {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Object(obj) => {
                obj.get("text")
                    .or_else(|| obj.get("title"))
                    .or_else(|| obj.get("subject"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            }
            _ => serde_json::to_string(&event.content).unwrap_or_default(),
        };
        ExtractedData {
            title,
            summary: content_str.clone(),
            detail: content_str,
            category: category.clone(),
            project: None,
            people: vec![],
            tags: vec![],
            task_status: None,
            confidence: 0.7,
            due_date: None,
        }
    }

    /// 从事件内容中提取标题（Archive 路由使用）
    fn extract_title_from_event(event: &Event) -> String {
        match &event.content {
            serde_json::Value::Object(obj) => obj
                .get("text")
                .or_else(|| obj.get("title"))
                .or_else(|| obj.get("subject"))
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled")
                .to_string(),
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
            EventType::DocumentChange | EventType::ManualNote => {
                wb_core::record::Category::Document
            }
            EventType::Approval => wb_core::record::Category::Decision,
            EventType::OkrUpdate | EventType::Browsing => wb_core::record::Category::Planning,
            EventType::AppActivity => wb_core::record::Category::Research,
        }
    }

    /// 将 AI 分类标签映射为 ProcessingRoute
    fn category_to_route(category: &str) -> ProcessingRoute {
        match category {
            "task" | "approval" | "manual_note" | "meeting" | "calendar" | "email" => {
                ProcessingRoute::Instant
            }
            "message" | "document" | "browsing" | "app_activity" => ProcessingRoute::Aggregate,
            "okr" => ProcessingRoute::Pattern,
            _ => ProcessingRoute::Aggregate,
        }
    }

    /// 判断事件是否为文本丰富的类型（适合运行任务发现）
    ///
    /// 仅对 Message、Email、ManualNote 运行任务发现，
    /// 避免对 Approval、Browsing、AppActivity 等产生误报。
    fn is_text_rich_event(event: &Event) -> bool {
        matches!(
            event.event_type,
            EventType::Message | EventType::Email | EventType::ManualNote
        )
    }

    /// 从事件内容中提取文本（用于 TaskDiscovery）
    fn extract_text_from_event(event: &Event) -> String {
        match &event.content {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Object(obj) => obj
                .get("text")
                .or_else(|| obj.get("title"))
                .or_else(|| obj.get("subject"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            _ => serde_json::to_string(&event.content).unwrap_or_default(),
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

    /// 在 Obsidian vault 中查找包含指定 record_id 的已有文件路径
    ///
    /// 扫描目标目录下的 markdown 文件，检查 frontmatter 中的 id 字段。
    fn find_existing_path_by_id(
        persistor: &PersistStep,
        record_id: &str,
        new_path: &str,
    ) -> Option<String> {
        let vault_path = persistor.writer().vault_path();
        let new_full = vault_path.join(new_path);
        let parent = new_full.parent()?;

        let entries = std::fs::read_dir(parent).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(true, |e| e != "md") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&path) {
                // 检查 frontmatter 中的 id 行
                if content.lines().any(|l| l.trim() == format!("id: {}", record_id)) {
                    // 返回相对于 vault 的路径
                    if let Ok(rel) = path.strip_prefix(vault_path) {
                        return Some(rel.to_string_lossy().to_string());
                    }
                }
            }
        }
        None
    }
}

/// 净化文本输入：按字符截断到指定最大长度
///
/// 用于防止 AI 生成的过长内容污染 WorkRecord。
/// 使用 `chars().take(max_len)` 确保按 Unicode 字符截断，不破坏字符边界。
fn sanitize_text_input(text: &str, max_chars: usize) -> String {
    text.chars().take(max_chars).collect()
}

#[cfg(test)]
#[path = "pipeline_tests.rs"]
mod tests;
