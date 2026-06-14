//! 任务管理 Tauri 命令
//!
//! 提供任务的 CRUD、任务发现、确认/拒绝流程。

use std::sync::OnceLock;
use tokio::sync::Mutex;
use wb_processor::task::discovery::{PendingTask, TaskDiscovery};
use wb_processor::task::model::{Task, TaskFilter, TaskPriority, TaskSource, TaskStatus};
use wb_processor::task::TaskManager;
use super::CommandError;

/// 全局 TaskManager 实例
static TASK_MANAGER: OnceLock<Mutex<TaskManager>> = OnceLock::new();

/// 全局 TaskDiscovery 实例
static TASK_DISCOVERY: OnceLock<Mutex<TaskDiscovery>> = OnceLock::new();

/// 初始化任务管理器（在 Tauri setup 阶段调用）
pub fn init_task_manager() {
    let _ = TASK_MANAGER.set(Mutex::new(TaskManager::new()));
    let _ = TASK_DISCOVERY.set(Mutex::new(TaskDiscovery::new()));
}

/// 获取全局 TaskManager
fn get_task_manager() -> &'static Mutex<TaskManager> {
    TASK_MANAGER.get().expect("TaskManager not initialized")
}

/// 获取全局 TaskDiscovery（供 collect 模块在采集后自动发现任务）
pub(crate) fn get_task_discovery() -> &'static Mutex<TaskDiscovery> {
    TASK_DISCOVERY.get().expect("TaskDiscovery not initialized")
}

/// 任务 DTO（前端显示用）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskDto {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub source: String,
    pub due_date: Option<String>,
    pub created_at: String,
    pub tags: Vec<String>,
}

/// 待确认任务 DTO
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PendingTaskDto {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub source: String,
    pub priority: String,
    pub origin_text: String,
    pub created_at: String,
}

impl From<&Task> for TaskDto {
    fn from(t: &Task) -> Self {
        Self {
            id: t.id.clone(),
            title: t.title.clone(),
            description: t.description.clone(),
            status: format!("{:?}", t.status),
            priority: format!("{:?}", t.priority),
            source: format!("{:?}", t.source),
            due_date: t.due_date.clone(),
            created_at: t.created_at.clone(),
            tags: t.tags.clone(),
        }
    }
}

impl From<&PendingTask> for PendingTaskDto {
    fn from(p: &PendingTask) -> Self {
        Self {
            id: p.id.clone(),
            title: p.title.clone(),
            description: p.description.clone(),
            source: format!("{:?}", p.source),
            priority: format!("{:?}", p.priority),
            origin_text: p.origin_text.clone(),
            created_at: p.created_at.clone(),
        }
    }
}

/// 从文本中发现任务
///
/// 用于采集消息后自动发现任务。
/// 返回新发现的待确认任务列表。
#[tauri::command]
pub async fn discover_tasks_from_text(
    text: String,
    source: String,
) -> Result<Vec<PendingTaskDto>, CommandError> {
    let mut discovery = get_task_discovery().lock().await;

    let tasks = match source.as_str() {
        "message" => discovery.discover_from_message(&text),
        "meeting" => discovery.discover_from_meeting(&text),
        "email" => discovery.discover_from_email(&text),
        _ => discovery.discover_from_message(&text),
    };

    Ok(tasks.iter().map(PendingTaskDto::from).collect())
}

/// 获取所有待确认任务
#[tauri::command]
pub async fn get_pending_tasks() -> Result<Vec<PendingTaskDto>, CommandError> {
    let discovery = get_task_discovery().lock().await;
    let pending = discovery.pending();
    Ok(pending.iter().map(|p| PendingTaskDto::from(*p)).collect())
}

/// 确认待确认任务（创建为正式任务）
#[tauri::command]
pub async fn confirm_pending_task(pending_id: String) -> Result<TaskDto, CommandError> {
    // 先从 discovery 中确认
    let confirmed = {
        let mut discovery = get_task_discovery().lock().await;
        discovery.confirm(&pending_id)?
    };

    // 创建正式任务（确认后直接设为 Open，用户已确认无需再 Pending）
    let manager = get_task_manager().lock().await;
    let task = manager
        .create(&confirmed.title, confirmed.priority.clone(), confirmed.source.clone())
        .await
        ?;
    let task = manager
        .transition(&task.id, TaskStatus::Open)
        .await
        ?;

    Ok(TaskDto::from(&task))
}

/// 拒绝待确认任务
#[tauri::command]
pub async fn reject_pending_task(pending_id: String) -> Result<(), CommandError> {
    let mut discovery = get_task_discovery().lock().await;
    discovery.reject(&pending_id)?;
    Ok(())
}

/// 获取所有任务列表
#[tauri::command]
pub async fn list_tasks(
    status: Option<String>,
    priority: Option<String>,
) -> Result<Vec<TaskDto>, CommandError> {
    let manager = get_task_manager().lock().await;
    let filter = TaskFilter {
        status: status.and_then(|s| match s.as_str() {
            "Pending" => Some(TaskStatus::Pending),
            "Open" => Some(TaskStatus::Open),
            "InProgress" => Some(TaskStatus::InProgress),
            "Done" => Some(TaskStatus::Done),
            "Archived" => Some(TaskStatus::Archived),
            _ => None,
        }),
        priority: priority.and_then(|p| match p.as_str() {
            "P0" => Some(TaskPriority::P0),
            "P1" => Some(TaskPriority::P1),
            "P2" => Some(TaskPriority::P2),
            "P3" => Some(TaskPriority::P3),
            _ => None,
        }),
        ..TaskFilter::default()
    };

    let tasks = manager.list(filter).await?;
    Ok(tasks.iter().map(TaskDto::from).collect())
}

/// 手动创建任务
#[tauri::command]
pub async fn create_task(
    title: String,
    priority: Option<String>,
) -> Result<TaskDto, CommandError> {
    let p = match priority.as_deref() {
        Some("P0") => TaskPriority::P0,
        Some("P1") => TaskPriority::P1,
        Some("P2") => TaskPriority::P2,
        Some("P3") => TaskPriority::P3,
        _ => TaskPriority::P2,
    };

    let manager = get_task_manager().lock().await;
    let task = manager
        .create(&title, p, TaskSource::Manual)
        .await
        ?;

    Ok(TaskDto::from(&task))
}

/// 更新任务状态
#[tauri::command]
pub async fn update_task_status(task_id: String, new_status: String) -> Result<TaskDto, CommandError> {
    let status = match new_status.as_str() {
        "Pending" => TaskStatus::Pending,
        "Open" => TaskStatus::Open,
        "InProgress" => TaskStatus::InProgress,
        "Done" => TaskStatus::Done,
        "Archived" => TaskStatus::Archived,
        _ => return Err(CommandError::BadRequest(format!("无效的状态: {}", new_status))),
    };

    let manager = get_task_manager().lock().await;
    let task = manager
        .transition(&task_id, status)
        .await
        ?;

    Ok(TaskDto::from(&task))
}
