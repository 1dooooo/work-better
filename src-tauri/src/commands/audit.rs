//! 审计日志 Tauri 命令
//!
//! 提供处理审计日志和执行日志的查询接口，以及开发者模式配置。

use tauri::State;
use wb_storage::{AuditQueryFilter, ExecutionLogFilter};
use super::AppState;

/// 检查开发者模式是否启用
fn is_developer_mode_enabled() -> Result<bool, String> {
    super::settings::load_config().map(|config| config.developer_mode)
}

/// 查询处理审计日志
#[tauri::command]
pub async fn get_processing_audits(
    state: State<'_, AppState>,
    step: Option<String>,
    trace_id: Option<String>,
    since: Option<String>,
    until: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<wb_storage::sqlite::audit_log::ProcessingAuditRow>, String> {
    if !is_developer_mode_enabled()? {
        return Err("开发者模式未启用".to_string());
    }

    let store = state.audit_log
        .as_ref()
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
    state: State<'_, AppState>,
    task_id: Option<String>,
    status: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<wb_storage::sqlite::audit_log::ExecutionLogRow>, String> {
    if !is_developer_mode_enabled()? {
        return Err("开发者模式未启用".to_string());
    }

    let store = state.audit_log
        .as_ref()
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
pub async fn get_audit_summary(state: State<'_, AppState>) -> Result<wb_storage::sqlite::audit_log::AuditSummary, String> {
    if !is_developer_mode_enabled()? {
        return Err("开发者模式未启用".to_string());
    }

    let store = state.audit_log
        .as_ref()
        .ok_or("AuditLogStore not initialized")?
        .lock()
        .await;
    store.get_summary()
}
