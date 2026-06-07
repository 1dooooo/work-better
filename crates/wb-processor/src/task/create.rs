//! 任务创建 —— 手动创建与自动发现

use chrono::Utc;
use uuid::Uuid;

use super::model::{Task, TaskPriority, TaskSource, TaskStatus};

/// 创建新任务
pub fn create_task(title: &str, priority: TaskPriority, source: TaskSource) -> Task {
    let now = Utc::now().to_rfc3339();
    let status = match source {
        TaskSource::Manual => TaskStatus::Open,
        _ => TaskStatus::Pending,
    };

    Task {
        id: Uuid::new_v4().to_string(),
        title: title.to_string(),
        description: None,
        status,
        priority,
        source,
        parent_id: None,
        children_ids: vec![],
        due_date: None,
        tags: vec![],
        created_at: now.clone(),
        updated_at: now,
        completed_at: None,
        feishu_task_id: None,
        obsidian_path: None,
    }
}

/// 创建子任务
pub fn create_subtask(parent: &Task, title: &str) -> Task {
    let now = Utc::now().to_rfc3339();

    Task {
        id: Uuid::new_v4().to_string(),
        title: title.to_string(),
        description: None,
        status: TaskStatus::Open,
        priority: parent.priority.clone(),
        source: parent.source.clone(),
        parent_id: Some(parent.id.clone()),
        children_ids: vec![],
        due_date: None,
        tags: vec![],
        created_at: now.clone(),
        updated_at: now,
        completed_at: None,
        feishu_task_id: None,
        obsidian_path: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_manual_task_is_open() {
        let task = create_task("Test task", TaskPriority::P1, TaskSource::Manual);
        assert_eq!(task.status, TaskStatus::Open);
        assert_eq!(task.title, "Test task");
        assert_eq!(task.priority, TaskPriority::P1);
        assert_eq!(task.source, TaskSource::Manual);
        assert!(task.parent_id.is_none());
        assert!(task.children_ids.is_empty());
        assert!(!task.id.is_empty());
    }

    #[test]
    fn test_create_auto_discovered_task_is_pending() {
        for source in [
            TaskSource::Meeting,
            TaskSource::Message,
            TaskSource::Email,
            TaskSource::Document,
            TaskSource::Feishu,
        ] {
            let task = create_task("Auto task", TaskPriority::P2, source.clone());
            assert_eq!(
                task.status,
                TaskStatus::Pending,
                "source {:?} should yield Pending",
                source
            );
        }
    }

    #[test]
    fn test_create_subtask_inherits_parent_info() {
        let parent = create_task("Parent", TaskPriority::P0, TaskSource::Feishu);
        let child = create_subtask(&parent, "Child");

        assert_eq!(child.parent_id, Some(parent.id));
        assert_eq!(child.priority, TaskPriority::P0);
        assert_eq!(child.source, TaskSource::Feishu);
        assert_eq!(child.status, TaskStatus::Open);
        assert_eq!(child.title, "Child");
    }

    #[test]
    fn test_created_at_and_updated_at_are_set() {
        let task = create_task("T", TaskPriority::P3, TaskSource::Manual);
        assert!(!task.created_at.is_empty());
        assert!(!task.updated_at.is_empty());
        assert_eq!(task.created_at, task.updated_at);
    }
}
