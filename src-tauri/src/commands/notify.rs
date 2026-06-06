//! 系统通知命令

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

/// 发送系统通知
#[tauri::command]
pub async fn send_notification(request: NotifyRequest) -> Result<(), String> {
    println!("[通知] {} — {} ({:?})", request.title, request.body, request.kind);
    Ok(())
}

/// 获取待确认通知列表
#[tauri::command]
pub async fn get_pending_notifications() -> Result<Vec<NotifyRequest>, String> {
    // TODO: 从 UserConfirmPush 获取待确认列表
    Ok(vec![])
}
