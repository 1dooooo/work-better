//! 审计日志 Tauri 命令
//!
//! 提供处理审计日志和执行日志的查询接口，以及开发者模式配置。

use std::sync::OnceLock;
use tokio::sync::Mutex;
use wb_storage::{AuditLogStore, AuditQueryFilter, ExecutionLogFilter};

/// 全局 AuditLogStore 实例
static AUDIT_LOG: OnceLock<Mutex<AuditLogStore>> = OnceLock::new();

/// 在 Tauri setup 阶段初始化 AuditLogStore。
///
/// 注意：此函数会打开独立的数据库连接（与 SqliteEventLog 分开）。
/// SQLite 支持多连接并发读取，写操作通过各自的 Mutex 串行化。
pub fn init_audit_log(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::Manager;

    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;

    // 确保目录存在（避免隐式依赖 init_event_log 的调用顺序）
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;

    let db_path = data_dir.join("work-better.db");
    let path_str = db_path
        .to_str()
        .ok_or("DB path contains invalid UTF-8")?;

    let conn = rusqlite::Connection::open(path_str)
        .map_err(|e| format!("Failed to open database for audit log: {}", e))?;

    // 初始化完整 schema（包括 processing_audits 和 execution_logs 表）
    wb_storage::sqlite::schema::initialize_schema(&conn)
        .map_err(|e| format!("Failed to initialize audit schema: {}", e))?;

    let store = AuditLogStore::new(conn);

    if AUDIT_LOG.set(Mutex::new(store)).is_err() {
        return Err("AuditLogStore already initialized".into());
    }

    Ok(())
}

/// 获取全局 AuditLogStore 实例的引用。
///
/// 返回 `None` 如果尚未初始化。供 `events` 模块写入审计日志使用。
pub(crate) fn get_audit_log() -> Option<&'static Mutex<AuditLogStore>> {
    AUDIT_LOG.get()
}

/// 检查开发者模式是否启用
fn is_developer_mode_enabled() -> Result<bool, String> {
    super::settings::load_config().map(|config| config.developer_mode)
}

/// 查询处理审计日志
#[tauri::command]
pub async fn get_processing_audits(
    step: Option<String>,
    trace_id: Option<String>,
    since: Option<String>,
    until: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<wb_storage::sqlite::audit_log::ProcessingAuditRow>, String> {
    // 服务端权限检查
    if !is_developer_mode_enabled()? {
        return Err("开发者模式未启用".to_string());
    }

    let store = get_audit_log()
        .ok_or("AuditLogStore not initialized")?
        .lock()
        .await;
    let filter = AuditQueryFilter {
        step,
        trace_id,
        since,
        until,
        limit,
    };
    store.query_processing_audits(&filter)
}

/// 查询执行日志
#[tauri::command]
pub async fn get_execution_logs(
    task_id: Option<String>,
    status: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<wb_storage::sqlite::audit_log::ExecutionLogRow>, String> {
    // 服务端权限检查
    if !is_developer_mode_enabled()? {
        return Err("开发者模式未启用".to_string());
    }

    let store = get_audit_log()
        .ok_or("AuditLogStore not initialized")?
        .lock()
        .await;
    let filter = ExecutionLogFilter {
        task_id,
        status,
        limit,
    };
    store.query_execution_logs(&filter)
}

/// 获取审计日志统计摘要
#[tauri::command]
pub async fn get_audit_summary() -> Result<wb_storage::sqlite::audit_log::AuditSummary, String> {
    // 服务端权限检查
    if !is_developer_mode_enabled()? {
        return Err("开发者模式未启用".to_string());
    }

    let store = get_audit_log()
        .ok_or("AuditLogStore not initialized")?
        .lock()
        .await;
    store.get_summary()
}
