//! 前端日志命令
//!
//! 将前端日志输出到Tauri终端，实现日志统一观测

use serde::Deserialize;

/// 前端日志级别
#[derive(Debug, Deserialize)]
pub enum LogLevel {
    #[serde(rename = "debug")]
    Debug,
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "error")]
    Error,
}

/// 前端日志命令
///
/// 将前端日志输出到Tauri终端，实现日志统一观测
#[tauri::command]
pub fn log_message(level: LogLevel, message: String, tag: Option<String>) {
    let tag = tag.unwrap_or_else(|| "Frontend".to_string());
    match level {
        LogLevel::Debug => eprintln!("[DEBUG] [{}] {}", tag, message),
        LogLevel::Info => eprintln!("[INFO] [{}] {}", tag, message),
        LogLevel::Warn => eprintln!("[WARN] [{}] {}", tag, message),
        LogLevel::Error => eprintln!("[ERROR] [{}] {}", tag, message),
    }
}
