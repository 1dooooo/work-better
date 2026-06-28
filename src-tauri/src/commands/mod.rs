//! Tauri 命令模块

pub mod audit;
pub mod capture;
pub mod collect;
pub mod collectors;
pub mod db;
pub mod events;
pub mod notify;
pub mod scheduler;
pub mod settings;
pub mod tasks;
pub mod test_mode;
pub mod window;

use std::sync::Arc;
use serde::Serialize;
use tokio::sync::Mutex;
use wb_collector::manager::CollectorManager;
use wb_processor::task::discovery::TaskDiscovery;
use wb_processor::task::TaskManager;
use wb_scheduler::scheduler::Scheduler;
use wb_storage::{AuditLogStore, SqliteEventLog};

/// 应用全局状态
///
/// 在 Tauri setup 阶段一次性构建，通过 `manage()` 注入。
/// 所有命令函数通过 `State<'_, AppState>` 提取依赖。
pub struct AppState {
    /// 事件日志（SQLite，Tauri setup 阶段初始化，不可为 None）
    pub event_log: Mutex<SqliteEventLog>,
    /// 采集器管理器（纯内存，Arc 用于跨线程共享）
    pub collector_manager: Arc<CollectorManager>,
    /// 任务管理器（纯内存）
    pub task_manager: Mutex<TaskManager>,
    /// 任务发现器（纯内存）
    pub task_discovery: Mutex<TaskDiscovery>,
    /// 定时任务调度器（纯内存）
    pub scheduler: Scheduler,
    /// 审计日志（SQLite，Option 因为可能未初始化，Arc 用于回调闭包克隆）
    pub audit_log: Option<Arc<Mutex<AuditLogStore>>>,
}

impl AppState {
    /// 在 Tauri setup 阶段构建
    pub fn new(
        event_log: SqliteEventLog,
        audit_log: Option<AuditLogStore>,
    ) -> Self {
        Self {
            event_log: Mutex::new(event_log),
            collector_manager: Arc::new(CollectorManager::new()),
            task_manager: Mutex::new(TaskManager::new()),
            task_discovery: Mutex::new(TaskDiscovery::new()),
            scheduler: Scheduler::new(),
            audit_log: audit_log.map(|a| Arc::new(Mutex::new(a))),
        }
    }

    /// 测试用构造器
    #[cfg(test)]
    pub fn for_testing() -> Self {
        Self {
            event_log: Mutex::new(SqliteEventLog::new(":memory:")
                .expect("Failed to create in-memory EventLog")),
            collector_manager: Arc::new(CollectorManager::new()),
            task_manager: Mutex::new(TaskManager::new()),
            task_discovery: Mutex::new(TaskDiscovery::new()),
            scheduler: Scheduler::new(),
            audit_log: None,
        }
    }
}

/// Tauri 命令结构化错误类型
#[derive(Debug, Serialize, thiserror::Error)]
pub enum CommandError {
    #[error("存储错误: {0}")]
    Storage(String),
    #[error("采集错误: {0}")]
    Collector(String),
    #[error("AI 错误: {0}")]
    Ai(String),
    #[error("未找到: {0}")]
    NotFound(String),
    #[error("序列化错误: {0}")]
    Serialization(String),
    #[error("IO 错误: {0}")]
    Io(String),
    #[error("参数错误: {0}")]
    BadRequest(String),
    #[error("内部错误: {0}")]
    Internal(String),
}

impl From<wb_core::error::WbError> for CommandError {
    fn from(err: wb_core::error::WbError) -> Self {
        match err {
            wb_core::error::WbError::Storage(msg) => CommandError::Storage(msg),
            wb_core::error::WbError::Collector(msg) => CommandError::Collector(msg),
            wb_core::error::WbError::Ai(msg) => CommandError::Ai(msg),
            wb_core::error::WbError::NotFound(msg) => CommandError::NotFound(msg),
            wb_core::error::WbError::Serialization(e) => CommandError::Serialization(e.to_string()),
            wb_core::error::WbError::Io(e) => CommandError::Io(e.to_string()),
        }
    }
}

impl From<serde_json::Error> for CommandError {
    fn from(e: serde_json::Error) -> Self {
        CommandError::Serialization(e.to_string())
    }
}

impl From<tauri::Error> for CommandError {
    fn from(e: tauri::Error) -> Self {
        CommandError::Internal(e.to_string())
    }
}

impl From<std::io::Error> for CommandError {
    fn from(e: std::io::Error) -> Self {
        CommandError::Io(e.to_string())
    }
}
