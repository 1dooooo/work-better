//! 状态流转 —— 任务生命周期管理
//!
//! 合法流转：Pending → Open → InProgress → Done → Archived
//! 不可回退。

use chrono::Utc;
use wb_core::error::{Result, WbError};

use super::model::{Task, TaskStatus};

/// 校验状态流转是否合法
///
/// 规则：
/// - Pending → Open
/// - Open → InProgress
/// - InProgress → Done
/// - Done → Archived
/// - 其余均为非法
pub fn validate_transition(current: &TaskStatus, target: &TaskStatus) -> Result<()> {
    let valid = matches!(
        (current, target),
        (TaskStatus::Pending, TaskStatus::Open)
            | (TaskStatus::Open, TaskStatus::InProgress)
            | (TaskStatus::Open, TaskStatus::Done)
            | (TaskStatus::InProgress, TaskStatus::Done)
            | (TaskStatus::Done, TaskStatus::Archived)
    );

    if valid {
        Ok(())
    } else {
        Err(WbError::Ai(format!(
            "非法状态流转: {:?} → {:?}",
            current, target
        )))
    }
}

/// 执行状态流转，返回更新后的任务（不可变——返回新实例）
pub fn transition(task: &Task, new_status: TaskStatus) -> Result<Task> {
    validate_transition(&task.status, &new_status)?;

    let now = Utc::now().to_rfc3339();
    let completed_at = if new_status == TaskStatus::Done {
        Some(now.clone())
    } else {
        task.completed_at.clone()
    };

    Ok(Task {
        id: task.id.clone(),
        title: task.title.clone(),
        description: task.description.clone(),
        status: new_status,
        priority: task.priority.clone(),
        source: task.source.clone(),
        parent_id: task.parent_id.clone(),
        children_ids: task.children_ids.clone(),
        due_date: task.due_date.clone(),
        tags: task.tags.clone(),
        created_at: task.created_at.clone(),
        updated_at: now,
        completed_at,
        feishu_task_id: task.feishu_task_id.clone(),
        obsidian_path: task.obsidian_path.clone(),
    })
}

/// 归档任务（Done → Archived 的便捷方法）
pub fn archive(task: &Task) -> Result<Task> {
    transition(task, TaskStatus::Archived)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::create::create_task;
    use crate::task::model::{TaskPriority, TaskSource};

    fn make_task_with_status(status: TaskStatus) -> Task {
        let mut task = create_task("Test", TaskPriority::P2, TaskSource::Manual);
        task.status = status;
        task
    }

    #[test]
    fn test_valid_transition_pending_to_open() {
        let task = make_task_with_status(TaskStatus::Pending);
        let updated = transition(&task, TaskStatus::Open).unwrap();
        assert_eq!(updated.status, TaskStatus::Open);
    }

    #[test]
    fn test_valid_transition_open_to_in_progress() {
        let task = make_task_with_status(TaskStatus::Open);
        let updated = transition(&task, TaskStatus::InProgress).unwrap();
        assert_eq!(updated.status, TaskStatus::InProgress);
    }

    #[test]
    fn test_valid_transition_in_progress_to_done() {
        let task = make_task_with_status(TaskStatus::InProgress);
        let updated = transition(&task, TaskStatus::Done).unwrap();
        assert_eq!(updated.status, TaskStatus::Done);
        assert!(updated.completed_at.is_some());
    }

    #[test]
    fn test_valid_transition_done_to_archived() {
        let task = make_task_with_status(TaskStatus::Done);
        let updated = transition(&task, TaskStatus::Archived).unwrap();
        assert_eq!(updated.status, TaskStatus::Archived);
    }

    #[test]
    fn test_full_lifecycle() {
        let task = make_task_with_status(TaskStatus::Pending);
        let task = transition(&task, TaskStatus::Open).unwrap();
        let task = transition(&task, TaskStatus::InProgress).unwrap();
        let task = transition(&task, TaskStatus::Done).unwrap();
        let task = transition(&task, TaskStatus::Archived).unwrap();
        assert_eq!(task.status, TaskStatus::Archived);
    }

    // --- 非法流转测试 ---

    #[test]
    fn test_invalid_done_to_open() {
        let task = make_task_with_status(TaskStatus::Done);
        assert!(transition(&task, TaskStatus::Open).is_err());
    }

    #[test]
    fn test_invalid_open_to_done() {
        let task = make_task_with_status(TaskStatus::Open);
        assert!(transition(&task, TaskStatus::Done).is_err());
    }

    #[test]
    fn test_invalid_archived_to_anything() {
        let task = make_task_with_status(TaskStatus::Archived);
        assert!(transition(&task, TaskStatus::Open).is_err());
        assert!(transition(&task, TaskStatus::InProgress).is_err());
        assert!(transition(&task, TaskStatus::Done).is_err());
        assert!(transition(&task, TaskStatus::Pending).is_err());
    }

    #[test]
    fn test_invalid_in_progress_to_open() {
        let task = make_task_with_status(TaskStatus::InProgress);
        assert!(transition(&task, TaskStatus::Open).is_err());
    }

    #[test]
    fn test_invalid_pending_to_in_progress() {
        let task = make_task_with_status(TaskStatus::Pending);
        assert!(transition(&task, TaskStatus::InProgress).is_err());
    }

    #[test]
    fn test_invalid_same_status() {
        let task = make_task_with_status(TaskStatus::Open);
        assert!(transition(&task, TaskStatus::Open).is_err());
    }

    // --- 归档测试 ---

    #[test]
    fn test_archive_from_done() {
        let task = make_task_with_status(TaskStatus::Done);
        let archived = archive(&task).unwrap();
        assert_eq!(archived.status, TaskStatus::Archived);
    }

    #[test]
    fn test_archive_fails_from_non_done() {
        for status in [
            TaskStatus::Pending,
            TaskStatus::Open,
            TaskStatus::InProgress,
            TaskStatus::Archived,
        ] {
            let task = make_task_with_status(status.clone());
            assert!(
                archive(&task).is_err(),
                "archive should fail from {:?}",
                status
            );
        }
    }

    // --- 不可变性测试 ---

    #[test]
    fn test_transition_preserves_immutability() {
        let task = make_task_with_status(TaskStatus::Open);
        let updated = transition(&task, TaskStatus::InProgress).unwrap();
        // 原任务不变
        assert_eq!(task.status, TaskStatus::Open);
        // 新任务已更新
        assert_eq!(updated.status, TaskStatus::InProgress);
    }

    #[test]
    fn test_completed_at_only_set_on_done() {
        let task = make_task_with_status(TaskStatus::Open);
        assert!(task.completed_at.is_none());

        let task = transition(&task, TaskStatus::InProgress).unwrap();
        assert!(task.completed_at.is_none());

        let task = transition(&task, TaskStatus::Done).unwrap();
        assert!(task.completed_at.is_some());

        let task = transition(&task, TaskStatus::Archived).unwrap();
        // 归档保留 completed_at
        assert!(task.completed_at.is_some());
    }

    #[test]
    fn test_transition_updates_updated_at() {
        let task = make_task_with_status(TaskStatus::Open);
        let old_updated = task.updated_at.clone();
        let updated = transition(&task, TaskStatus::InProgress).unwrap();
        assert_ne!(updated.updated_at, old_updated);
    }
}
