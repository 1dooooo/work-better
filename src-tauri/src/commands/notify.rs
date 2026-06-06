//! 系统通知命令
//!
//! TODO: `send_notification` 当前为 stub，仅记录调试日志。
//!      后续需接入 Tauri notification plugin 或系统通知 API 实现真正的通知发送。

use serde::{Deserialize, Serialize};

/// 通知类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotifyKind {
    /// 需要用户确认
    Confirm,
    /// 轻提醒
    Reminder,
    /// 任务完成
    TaskDone,
}

/// 通知请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyRequest {
    pub title: String,
    pub body: String,
    pub kind: NotifyKind,
    pub action_url: Option<String>,
}

/// 发送系统通知（stub 实现）
///
/// TODO: 当前仅记录调试日志到 stderr，不会发送真正的系统通知。
///      后续需接入 Tauri notification plugin 或各平台系统通知 API。
#[tauri::command]
pub async fn send_notification(request: NotifyRequest) -> Result<(), String> {
    eprintln!("[notify-stub] kind={:?} title={}", request.kind, request.title);
    Ok(())
}

/// 获取待确认通知列表
#[tauri::command]
pub async fn get_pending_notifications() -> Result<Vec<NotifyRequest>, String> {
    // TODO: 从 UserConfirmPush 获取待确认列表
    Ok(vec![])
}
