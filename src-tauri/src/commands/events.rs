//! 事件相关 Tauri 命令

use std::sync::OnceLock;
use tokio::sync::Mutex;
use wb_core::event::{Event, EventFilter, EventLog};
use wb_storage::SqliteEventLog;

/// 全局 SqliteEventLog 实例
static EVENT_LOG: OnceLock<Mutex<SqliteEventLog>> = OnceLock::new();

/// 获取或初始化 EventLog
pub(crate) fn get_event_log() -> &'static Mutex<SqliteEventLog> {
    EVENT_LOG.get_or_init(|| {
        let log = SqliteEventLog::new_in_memory().expect("Failed to initialize EventLog");
        Mutex::new(log)
    })
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
