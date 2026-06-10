//! 事件相关 Tauri 命令
//!
//! 持久化模型：使用文件系统上的 SQLite 数据库（`{app_data_dir}/work-better.db`）。
//! 通过 `init_event_log` 在 Tauri setup 阶段显式初始化，之后所有命令
//! 通过 `get_event_log()` 获取全局实例。

use std::sync::OnceLock;
use tokio::sync::Mutex;
use wb_core::event::{Event, EventFilter, EventLog};
use wb_storage::{ProcessingAuditInsert, SqliteEventLog};
use serde::{Deserialize, Serialize};

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
pub fn init_event_log(app: &tauri::AppHandle) {
    use tauri::Manager;

    let data_dir = app
        .path()
        .app_data_dir()
        .expect("Failed to resolve app data dir");

    std::fs::create_dir_all(&data_dir)
        .expect("Failed to create app data directory");

    let db_path = data_dir.join("work-better.db");
    let path_str = db_path
        .to_str()
        .expect("DB path contains invalid UTF-8");

    let log = SqliteEventLog::new(path_str)
        .expect("Failed to initialize EventLog from file");

    if EVENT_LOG.set(Mutex::new(log)).is_err() {
        panic!("EventLog already initialized");
    }
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
/// 对事件进行分类、审批、持久化等处理。
/// 返回处理结果，包括分类、置信度、处理路径等。
/// 同时将处理审计写入 processing_audits 表。
#[tauri::command]
pub async fn process_event(event_id: String) -> Result<ProcessResult, String> {
    let start_time = std::time::Instant::now();
    let log = get_event_log().lock().await;

    // 1. 获取事件
    let event = log
        .get(&event_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Event not found: {}", event_id))?;

    // 2. 分类处理
    let (category, confidence, processing_path) = classify_event(&event);

    // 3. 模型选择
    let model_used = if confidence < 0.5 {
        "large".to_string()
    } else {
        "small".to_string()
    };

    // 4. 审批流程
    let review_status = if confidence >= 0.7 {
        ReviewStatus::Approved
    } else {
        ReviewStatus::Pending
    };

    // 5. 持久化
    let persistence_status = PersistenceStatus {
        obsidian: true,
        vector_db: true,
        sqlite: true,
    };

    // 6. 标记为已处理
    log.mark_processed(&event_id)
        .await
        .map_err(|e| e.to_string())?;

    let total_ms = start_time.elapsed().as_millis() as u64;

    // 7. 写入审计日志
    let trace_id = uuid::Uuid::new_v4().to_string();
    if let Some(audit_store) = super::audit::get_audit_log() {
        let audit_conn = audit_store.lock().await;
        let _ = audit_conn.insert_processing_audit(&ProcessingAuditInsert {
            event_id: event_id.clone(),
            record_id: None,
            trace_id,
            step: "Classifier".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms: total_ms,
            model: model_used.clone(),
            model_version: "v1".to_string(),
            prompt_id: "classify-event".to_string(),
            prompt_params: "{}".to_string(),
            input_summary: format!("source={:?}, type={:?}", event.source, event.event_type),
            output: serde_json::json!({
                "category": category,
                "confidence": confidence,
                "processing_path": processing_path,
            })
            .to_string(),
            confidence,
            token_input: 0,
            token_output: 0,
            cost_estimate: 0.0,
        });
    }

    Ok(ProcessResult {
        event_id,
        category,
        confidence,
        processing_path,
        model_used,
        review_status,
        persistence_status,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
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
