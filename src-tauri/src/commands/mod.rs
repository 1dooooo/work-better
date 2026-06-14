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

use serde::Serialize;

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
