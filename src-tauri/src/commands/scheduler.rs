//! 调度器管理 Tauri 命令

use tauri::State;
use super::AppState;

/// 已调度任务信息（序列化返回给前端）
#[derive(serde::Serialize)]
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub layer: String,
    pub cron: String,
    pub sla_ms: u64,
}

/// 列出所有已注册的定时任务
#[tauri::command]
pub async fn list_scheduled_tasks(state: State<'_, AppState>) -> Result<Vec<TaskInfo>, String> {
    let scheduler = &state.scheduler;
    let ids = scheduler.list_tasks().await;

    let mut tasks = Vec::with_capacity(ids.len());
    for id in &ids {
        if let Some(info) = scheduler.get_task_info(id).await {
            tasks.push(TaskInfo {
                id: info.id,
                name: info.name,
                layer: info.layer,
                cron: info.cron,
                sla_ms: info.sla_ms,
            });
        }
    }

    Ok(tasks)
}

/// 暂停调度器
#[tauri::command]
pub async fn pause_scheduler(state: State<'_, AppState>) -> Result<(), String> {
    state.scheduler.pause_all().await;
    Ok(())
}

/// 恢复调度器
#[tauri::command]
pub async fn resume_scheduler(state: State<'_, AppState>) -> Result<(), String> {
    state.scheduler.resume_all().await;
    Ok(())
}

/// 查询调度器是否处于暂停状态
#[tauri::command]
pub async fn is_scheduler_paused(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.scheduler.is_paused().await)
}
