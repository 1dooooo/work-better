//! 系统通知命令
//!
//! 接入 Tauri notification plugin 实现真实系统通知。
//! 通知数据持久化到内存列表，支持待确认列表和已读标记。
//! TODO: 后续迁移到 SQLite notifications 表。

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::AppHandle;
use uuid::Uuid;

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

/// 通知记录（包含持久化字段）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRecord {
    pub id: String,
    pub title: String,
    pub body: String,
    pub kind: NotifyKind,
    pub action_url: Option<String>,
    pub read: bool,
    pub created_at: String,
}

// 内存中的待通知列表
//
// TODO: 后续迁移到 SQLite notifications 表
lazy_static::lazy_static! {
    static ref PENDING_NOTIFICATIONS: Mutex<Vec<NotificationRecord>> = Mutex::new(Vec::new());
}

/// 获取锁（处理 poisoned mutex，避免级联 panic）
fn lock_notifications() -> std::sync::MutexGuard<'static, Vec<NotificationRecord>> {
    PENDING_NOTIFICATIONS.lock().unwrap_or_else(|e| e.into_inner())
}

/// 发送系统通知
///
/// 1. 通过 Tauri notification plugin 发送 macOS 系统通知
/// 2. 同时持久化到内存列表
#[tauri::command]
pub async fn send_notification(
    app: AppHandle,
    request: NotifyRequest,
) -> Result<(), String> {
    // 发送系统通知
    use tauri_plugin_notification::NotificationExt;
    let result = app
        .notification()
        .builder()
        .title(&request.title)
        .body(&request.body)
        .show();

    if let Err(e) = result {
        eprintln!("[notify] 系统通知发送失败: {}，降级为日志输出", e);
        eprintln!(
            "[notify-fallback] kind={:?} title={} body={}",
            request.kind, request.title, request.body
        );
    }

    // 持久化到待通知列表
    let record = NotificationRecord {
        id: Uuid::new_v4().to_string(),
        title: request.title,
        body: request.body,
        kind: request.kind,
        action_url: request.action_url,
        read: false,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    lock_notifications().push(record);

    Ok(())
}

/// 获取待确认通知列表
#[tauri::command]
pub async fn get_pending_notifications() -> Result<Vec<NotificationRecord>, String> {
    let notifications = lock_notifications();
    Ok(notifications.iter().filter(|n| !n.read).cloned().collect())
}

/// 标记通知为已读
#[tauri::command]
pub async fn mark_notification_read(id: String) -> Result<(), String> {
    let mut notifications = lock_notifications();
    if let Some(notification) = notifications.iter_mut().find(|n| n.id == id) {
        notification.read = true;
        Ok(())
    } else {
        Err(format!("通知 {} 未找到", id))
    }
}

/// 清除所有已读通知
#[tauri::command]
pub async fn clear_read_notifications() -> Result<(), String> {
    let mut notifications = lock_notifications();
    notifications.retain(|n| !n.read);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notify_request_serialization() {
        let request = NotifyRequest {
            title: "测试通知".to_string(),
            body: "这是一条测试通知".to_string(),
            kind: NotifyKind::Confirm,
            action_url: Some("task://123".to_string()),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("测试通知"));
        assert!(json.contains("Confirm"));
    }

    #[test]
    fn test_notification_record_defaults() {
        let record = NotificationRecord {
            id: "test-id".to_string(),
            title: "标题".to_string(),
            body: "内容".to_string(),
            kind: NotifyKind::Reminder,
            action_url: None,
            read: false,
            created_at: "2026-01-01T00:00:00Z".to_string(),
        };
        assert!(!record.read);
        assert!(record.action_url.is_none());
    }

    #[test]
    fn test_notify_kind_serialization_roundtrip() {
        let kinds = vec![NotifyKind::Confirm, NotifyKind::Reminder, NotifyKind::TaskDone];
        for kind in kinds {
            let json = serde_json::to_string(&kind).unwrap();
            let deserialized: NotifyKind = serde_json::from_str(&json).unwrap();
            let json2 = serde_json::to_string(&deserialized).unwrap();
            assert_eq!(json, json2);
        }
    }
}
