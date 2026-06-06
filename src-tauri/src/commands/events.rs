//! 事件相关 Tauri 命令
//!
//! 持久化模型：使用文件系统上的 SQLite 数据库（`{app_data_dir}/work-better.db`）。
//! 通过 `init_event_log` 在 Tauri setup 阶段显式初始化，之后所有命令
//! 通过 `get_event_log()` 获取全局实例。

use std::sync::OnceLock;
use tokio::sync::Mutex;
use wb_core::event::{Event, EventFilter, EventLog};
use wb_storage::SqliteEventLog;

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
