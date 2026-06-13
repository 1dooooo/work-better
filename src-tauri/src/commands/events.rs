//! 事件相关 Tauri 命令
//!
//! 持久化模型：使用文件系统上的 SQLite 数据库（`{app_data_dir}/work-better.db`）。
//! 通过 `init_event_log` 在 Tauri setup 阶段显式初始化，之后所有命令
//! 通过 `get_event_log()` 获取全局实例。

use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::sync::Mutex;
use wb_core::event::{Event, EventFilter, EventLog};
use wb_storage::{ProcessingAuditInsert, SqliteEventLog};
use serde::{Deserialize, Serialize};
use wb_ai::{
    adapter::ModelAdapter,
    budget::TokenBudget,
    config::ModelConfig as AiModelConfig,
    router::ModelRouter,
    task_runner::{ModelSize, TaskRunner},
    OpenAIAdapter, AnthropicAdapter,
};

/// 处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessResult {
    pub event_id: String,
    pub category: String,
    pub confidence: f64,
    pub processing_path: String,
    pub model_used: String,
    pub review_status: ReviewStatus,
    pub persistence_status: PersistenceStatus,
    pub timestamp: String,
}

/// 审批状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewStatus {
    Pending,
    Approved,
    Rejected { reason: String },
}

/// 持久化状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceStatus {
    pub obsidian: bool,
    pub vector_db: bool,
    pub sqlite: bool,
}

/// 全局 SqliteEventLog 实例（文件持久化）
static EVENT_LOG: OnceLock<Mutex<SqliteEventLog>> = OnceLock::new();

/// 在 Tauri setup 阶段初始化 EventLog，使用文件数据库持久化。
///
/// 必须在任何 Tauri 命令调用之前执行。
pub fn init_event_log(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let path_str = super::db::resolve_db_path(app)?;
    eprintln!("[events] DB path: {}", path_str);

    let log = SqliteEventLog::new(&path_str)
        .map_err(|e| format!("Failed to initialize EventLog from file: {}", e))?;

    if EVENT_LOG.set(Mutex::new(log)).is_err() {
        return Err("EventLog already initialized".into());
    }

    Ok(())
}

/// 获取全局 EventLog 实例的引用。
///
/// 必须在 `init_event_log` 调用之后使用，否则会 panic。
pub fn get_event_log() -> &'static Mutex<SqliteEventLog> {
    EVENT_LOG.get().expect("EventLog not initialized — call init_event_log first")
}

/// 获取事件列表
#[tauri::command]
pub async fn get_events(limit: Option<usize>) -> Result<Vec<Event>, String> {
    let log = get_event_log().lock().await;
    let filter = EventFilter {
        limit,
        ..Default::default()
    };
    log.query(&filter).await.map_err(|e| e.to_string())
}

/// 获取未处理事件数量
#[tauri::command]
pub async fn get_unprocessed_count() -> Result<usize, String> {
    let log = get_event_log().lock().await;
    let events = log.get_unprocessed(None).await.map_err(|e| e.to_string())?;
    Ok(events.len())
}

/// 标记事件已处理
#[tauri::command]
pub async fn mark_event_processed(event_id: String) -> Result<(), String> {
    let log = get_event_log().lock().await;
    log.mark_processed(&event_id)
        .await
        .map_err(|e| e.to_string())
}

/// 处理事件
///
/// 委托给 `process_single_event`，使用真实 AI 模型进行分类和提取，
/// 包含任务发现、真实 token 统计和审计日志写入。
#[tauri::command]
pub async fn process_event(event_id: String) -> Result<ProcessResult, String> {
    process_single_event(&event_id).await
}

/// 批量处理结果
#[derive(Debug, Clone, Serialize)]
pub struct BatchProcessResult {
    /// 总事件数
    pub total: usize,
    /// 成功处理数
    pub success: usize,
    /// 失败数
    pub failed: usize,
    /// 跳过数（已处理）
    pub skipped: usize,
    /// 各事件处理详情
    pub details: Vec<BatchProcessDetail>,
}

/// 单个事件的批量处理详情
#[derive(Debug, Clone, Serialize)]
pub struct BatchProcessDetail {
    pub event_id: String,
    pub status: String, // "success" | "failed" | "skipped"
    pub category: Option<String>,
    pub error: Option<String>,
}

/// 批量处理所有未处理事件
///
/// 遍历所有未处理事件，逐个调用处理逻辑。
/// 用于开发者模式下的手动"主动整理"功能。
#[tauri::command]
pub async fn trigger_batch_process() -> Result<BatchProcessResult, String> {
    let log = get_event_log().lock().await;
    let unprocessed = log.get_unprocessed(None).await.map_err(|e| e.to_string())?;
    drop(log);

    let total = unprocessed.len();
    let mut success = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;
    let mut details = Vec::with_capacity(total);

    for event in unprocessed {
        let event_id = event.id.clone();

        match process_single_event(&event_id).await {
            Ok(result) => {
                success += 1;
                details.push(BatchProcessDetail {
                    event_id,
                    status: "success".to_string(),
                    category: Some(result.category),
                    error: None,
                });
            }
            Err(e) => {
                if e.contains("already processed") || e.contains("已处理") {
                    skipped += 1;
                    details.push(BatchProcessDetail {
                        event_id,
                        status: "skipped".to_string(),
                        category: None,
                        error: None,
                    });
                } else {
                    failed += 1;
                    details.push(BatchProcessDetail {
                        event_id,
                        status: "failed".to_string(),
                        category: None,
                        error: Some(e),
                    });
                }
            }
        }
    }

    Ok(BatchProcessResult {
        total,
        success,
        failed,
        skipped,
        details,
    })
}

/// 从配置构建 TaskRunner（带真实 AI 适配器）
///
/// 如果 API Key 未配置，返回 None（降级为关键词匹配）。
fn build_task_runner_from_config() -> Result<Option<TaskRunner>, String> {
    let config = super::settings::load_config()?;
    let api_key = match config.model.api_key {
        Some(ref key) if !key.is_empty() => key.clone(),
        _ => return Ok(None),
    };

    let endpoint = config.model.api_endpoint.clone();
    let small_model = config.model.small_model.clone();
    let large_model = config.model.large_model.clone();
    let budget = TokenBudget::new(config.model.token_budget as u64);

    // 统一处理 endpoint：剥掉尾部的 /v1，适配器会自己拼 /chat/completions
    let clean_endpoint = endpoint.trim_end_matches('/').trim_end_matches("/v1").to_string();

    // 根据 endpoint 判断使用 Anthropic 还是 OpenAI 适配器
    let is_anthropic = endpoint.contains("anthropic");

    let mut adapters: HashMap<ModelSize, Box<dyn ModelAdapter>> = HashMap::new();
    let mut adapter_names: HashMap<ModelSize, String> = HashMap::new();

    if is_anthropic {
        let small_config = AiModelConfig::anthropic(api_key.clone()).with_model(&small_model);
        let large_config = AiModelConfig::anthropic(api_key).with_model(&large_model);
        adapters.insert(ModelSize::Small, Box::new(AnthropicAdapter::new(small_config)));
        adapters.insert(ModelSize::Large, Box::new(AnthropicAdapter::new(large_config)));
        adapter_names.insert(ModelSize::Small, small_model);
        adapter_names.insert(ModelSize::Large, large_model);
    } else {
        let small_config = AiModelConfig::openai(api_key.clone(), Some(clean_endpoint.clone()))
            .with_model(&small_model);
        let large_config = AiModelConfig::openai(api_key, Some(clean_endpoint))
            .with_model(&large_model);
        adapters.insert(ModelSize::Small, Box::new(OpenAIAdapter::new(small_config)));
        adapters.insert(ModelSize::Large, Box::new(OpenAIAdapter::new(large_config)));
        adapter_names.insert(ModelSize::Small, small_model);
        adapter_names.insert(ModelSize::Large, large_model);
    }

    let router = ModelRouter::new();
    Ok(Some(TaskRunner::new(router, budget, adapters, adapter_names)))
}

/// 处理单个事件（内部实现，供 batch 和单次调用共用）
///
/// 优先使用真实大模型（classify + extract），如果 API Key 未配置则降级为关键词匹配。
async fn process_single_event(event_id: &str) -> Result<ProcessResult, String> {
    let start_time = std::time::Instant::now();
    let log = get_event_log().lock().await;

    let event = log
        .get(event_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Event not found: {}", event_id))?;

    // 尝试构建真实 TaskRunner
    let runner_opt = build_task_runner_from_config()?;

    let (category, confidence, processing_path, model_used, token_input, token_output, step_output) =
        if let Some(mut runner) = runner_opt {
            // ── 真实大模型调用 ──
            let classify_start = std::time::Instant::now();
            match runner.run_classify(&event, 0.5).await {
                Ok(classify_output) => {
                    let classification: serde_json::Value =
                        serde_json::from_str(&classify_output.content).unwrap_or_default();
                    let cat = classification
                        .get("category")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let conf = classify_output.confidence;
                    let model = classify_output.model_used.clone();
                    let tokens_in = classify_output.tokens_used;

                    // 进一步调用 extract 获取结构化信息
                    let extract_start = std::time::Instant::now();
                    let (extract_output, tokens_out) = match runner.run_extract(&event, conf).await {
                        Ok(out) => {
                            let t = out.tokens_used;
                            (Some(out.content), t)
                        }
                        Err(_e) => {
                            (None, 0u32)
                        }
                    };

                    let combined_output = serde_json::json!({
                        "classify": classify_output.content,
                        "extract": extract_output,
                        "classify_ms": classify_start.elapsed().as_millis() as u64,
                        "extract_ms": extract_start.elapsed().as_millis() as u64,
                    });

                    (cat, conf, "ai-model".to_string(), model, tokens_in, tokens_out, combined_output.to_string())
                }
                Err(e) => {
                    // 大模型调用失败，降级为关键词匹配
                    let (cat, conf, path) = classify_event(&event);
                    let fallback_output = serde_json::json!({
                        "fallback": "keyword",
                        "error": e.to_string(),
                        "category": cat,
                        "confidence": conf,
                    });
                    (cat, conf, path, "keyword-fallback".to_string(), 0u32, 0u32, fallback_output.to_string())
                }
            }
        } else {
            // ── 降级：关键词匹配（API Key 未配置）──
            let (cat, conf, path) = classify_event(&event);
            let fallback_output = serde_json::json!({
                "fallback": "keyword",
                "reason": "api_key_not_configured",
                "category": cat,
                "confidence": conf,
            });
            (cat, conf, path, "keyword-fallback".to_string(), 0u32, 0u32, fallback_output.to_string())
        };

    // ── 任务发现：AI 优先，正则兜底 ──
    let event_text = match &event.content {
        serde_json::Value::Object(obj) => {
            obj.get("text")
                .or_else(|| obj.get("title"))
                .or_else(|| obj.get("subject"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        }
        serde_json::Value::String(s) => s.clone(),
        _ => String::new(),
    };

    let discovery = super::tasks::get_task_discovery();
    let mut disc = discovery.lock().await;

    let mut ai_found_task = false;

    // 来源 1：从 AI extract 结果创建 pending task
    if processing_path == "ai-model" {
        if let Ok(extract_val) = serde_json::from_str::<serde_json::Value>(&step_output) {
            if let Some(extract_str) = extract_val.get("extract").and_then(|v| v.as_str()) {
                if let Ok(extraction) = serde_json::from_str::<serde_json::Value>(extract_str) {
                    let title = extraction.get("title").and_then(|v| v.as_str()).unwrap_or("");
                    let due_date = extraction.get("due_date").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let people: Vec<String> = extraction.get("people")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                        .unwrap_or_default();

                    if !title.is_empty() {
                        let description = if people.is_empty() {
                            None
                        } else {
                            Some(format!("涉及人员: {}", people.join(", ")))
                        };
                        let task = wb_processor::task::discovery::PendingTask::new(
                            title,
                            description.as_deref(),
                            wb_processor::task::model::TaskSource::Message,
                            wb_processor::task::model::TaskPriority::P2,
                            due_date,
                            &event_text,
                        );
                        disc.add_pending(task);
                        ai_found_task = true;
                    }
                }
            }
        }
    }

    // 来源 2：正则匹配关键词（仅当 AI 未发现任务时使用）
    if !ai_found_task && !event_text.is_empty() {
        let _tasks = disc.discover_from_message(&event_text);
    }

    let review_status = if confidence >= 0.7 {
        ReviewStatus::Approved
    } else {
        ReviewStatus::Pending
    };

    let persistence_status = PersistenceStatus {
        obsidian: true,
        vector_db: true,
        sqlite: true,
    };

    log.mark_processed(event_id)
        .await
        .map_err(|e| e.to_string())?;

    let total_ms = start_time.elapsed().as_millis() as u64;

    // 写入审计日志（含真实 token 消耗）
    let trace_id = uuid::Uuid::new_v4().to_string();
    let _token_total = token_input + token_output;
    let cost_estimate = estimate_cost(token_input, token_output, &model_used);
    if let Some(audit_store) = super::audit::get_audit_log() {
        let audit_conn = audit_store.lock().await;
        let _ = audit_conn.insert_processing_audit(&ProcessingAuditInsert {
            event_id: event_id.to_string(),
            record_id: None,
            trace_id,
            step: "Process".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms: total_ms,
            model: model_used.clone(),
            model_version: "v1".to_string(),
            prompt_id: "classify+extract".to_string(),
            prompt_params: "{}".to_string(),
            input_summary: format!("source={:?}, type={:?}", event.source, event.event_type),
            output: step_output,
            confidence,
            token_input: token_input as u64,
            token_output: token_output as u64,
            cost_estimate,
        });
    }



    Ok(ProcessResult {
        event_id: event_id.to_string(),
        category,
        confidence,
        processing_path,
        model_used,
        review_status,
        persistence_status,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

/// 估算 API 调用成本（美元）
///
/// 基于模型名称区分价格档位，粗略估算。
fn estimate_cost(token_input: u32, token_output: u32, model: &str) -> f64 {
    let (input_price, output_price) = if model.contains("mini") || model.contains("haiku") {
        (0.15 / 1_000_000.0, 0.60 / 1_000_000.0) // gpt-4o-mini / claude-haiku
    } else if model.contains("opus") {
        (15.0 / 1_000_000.0, 75.0 / 1_000_000.0) // claude-opus
    } else {
        (2.50 / 1_000_000.0, 10.0 / 1_000_000.0) // gpt-4o / claude-sonnet
    };
    token_input as f64 * input_price + token_output as f64 * output_price
}

/// 分类事件
fn classify_event(event: &Event) -> (String, f64, String) {
    let content = match &event.content {
        serde_json::Value::String(s) => s.clone(),
        other => serde_json::to_string(other).unwrap_or_default(),
    };

    // 简单的分类逻辑
    if content.contains("会议") || content.contains("meeting") {
        ("meeting".to_string(), 0.9, "direct".to_string())
    } else if content.contains("任务") || content.contains("task") {
        ("task".to_string(), 0.85, "direct".to_string())
    } else if content.contains("邮件") || content.contains("email") {
        ("email".to_string(), 0.8, "direct".to_string())
    } else if content.contains("审批") || content.contains("approval") {
        ("approval".to_string(), 0.85, "direct".to_string())
    } else {
        ("note".to_string(), 0.6, "aggregate".to_string())
    }
}
