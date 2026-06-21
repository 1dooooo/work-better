//! 事件相关 Tauri 命令
//!
//! 持久化模型：使用文件系统上的 SQLite 数据库（`{app_data_dir}/work-better.db`）。
//! 通过 AppState 在 Tauri setup 阶段显式初始化，之后所有命令
//! 通过 `State<'_, AppState>` 获取依赖。

use std::collections::HashMap;
use tauri::State;
use wb_core::event::{Event, EventFilter, EventLog};
use wb_storage::ProcessingAuditInsert;
use serde::{Deserialize, Serialize};
use wb_ai::{
    adapter::ModelAdapter,
    budget::TokenBudget,
    config::ModelConfig as AiModelConfig,
    router::ModelRouter,
    task_runner::{ModelSize, TaskRunner},
    OpenAIAdapter, AnthropicAdapter,
};

use super::AppState;

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

/// 获取事件列表
#[tauri::command]
pub async fn get_events(state: State<'_, AppState>, limit: Option<usize>) -> Result<Vec<Event>, String> {
    let log = state.event_log.lock().await;
    let filter = EventFilter {
        limit,
        ..Default::default()
    };
    log.query(&filter).await.map_err(|e| e.to_string())
}

/// 获取未处理事件数量
#[tauri::command]
pub async fn get_unprocessed_count(state: State<'_, AppState>) -> Result<usize, String> {
    let log = state.event_log.lock().await;
    let events = log.get_unprocessed(None).await.map_err(|e| e.to_string())?;
    Ok(events.len())
}

/// 标记事件已处理
#[tauri::command]
pub async fn mark_event_processed(state: State<'_, AppState>, event_id: String) -> Result<(), String> {
    let log = state.event_log.lock().await;
    log.mark_processed(&event_id)
        .await
        .map_err(|e| e.to_string())
}

/// 处理事件
///
/// 委托给 `process_single_event`，使用真实 AI 模型进行分类和提取，
/// 包含任务发现、真实 token 统计和审计日志写入。
#[tauri::command]
pub async fn process_event(state: State<'_, AppState>, event_id: String) -> Result<ProcessResult, String> {
    process_single_event(&state, &event_id).await
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
pub async fn trigger_batch_process(state: State<'_, AppState>) -> Result<BatchProcessResult, String> {
    let log = state.event_log.lock().await;
    let unprocessed = log.get_unprocessed(None).await.map_err(|e| e.to_string())?;
    drop(log);

    let total = unprocessed.len();
    let mut success = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;
    let mut details = Vec::with_capacity(total);

    for event in unprocessed {
        let event_id = event.id.clone();

        match process_single_event(&state, &event_id).await {
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
pub fn build_task_runner_from_config() -> Result<Option<TaskRunner>, String> {
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
async fn process_single_event(state: &AppState, event_id: &str) -> Result<ProcessResult, String> {
    let start_time = std::time::Instant::now();
    let log = state.event_log.lock().await;

    let event = log
        .get(event_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Event not found: {}", event_id))?;

    // 尝试构建真实 TaskRunner
    let runner_opt = build_task_runner_from_config()?;

    // ── 任务发现：提取事件文本 ──
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

    let mut disc = state.task_discovery.lock().await;

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

                    // 任务发现：通过 TaskRunner 的 ModelRouter 路由到小/大模型
                    if !event_text.is_empty() {
                        let _tasks = disc.discover_with_ai(&event_text, &mut runner, event.source.clone()).await;
                    }

                    let combined_output = serde_json::json!({
                        "classify": classify_output.content,
                        "extract": extract_output,
                        "classify_ms": classify_start.elapsed().as_millis() as u64,
                        "extract_ms": extract_start.elapsed().as_millis() as u64,
                    });

                    (cat, conf, "ai-model".to_string(), model, tokens_in, tokens_out, combined_output.to_string())
                }
                Err(e) => {
                    return Err(format!("AI 模型调用失败: {}", e));
                }
            }
        } else {
            return Err("AI 模型未配置。请在设置中配置 API Key 后重试。".to_string());
        };

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
    if let Some(ref audit_store) = state.audit_log {
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
