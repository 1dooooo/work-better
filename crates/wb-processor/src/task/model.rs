//! 任务数据模型 —— Task、状态、优先级、来源

use serde::{Deserialize, Serialize};

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    /// 待确认（自动发现的）
    Pending,
    /// 已确认，待处理
    Open,
    /// 进行中
    InProgress,
    /// 已完成
    Done,
    /// 已归档
    Archived,
}

/// 任务优先级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskPriority {
    /// 紧急
    P0,
    /// 高
    P1,
    /// 中
    P2,
    /// 低
    P3,
}

/// 任务来源
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskSource {
    Manual,
    Meeting,
    Message,
    Email,
    Document,
    Feishu,
}

/// 任务实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub source: TaskSource,
    pub parent_id: Option<String>,
    pub children_ids: Vec<String>,
    pub due_date: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub feishu_task_id: Option<String>,
    pub obsidian_path: Option<String>,
}

/// 任务更新请求（所有字段可选）
#[derive(Debug, Clone, Default)]
pub struct TaskUpdate {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub priority: Option<TaskPriority>,
    pub due_date: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
    pub feishu_task_id: Option<Option<String>>,
    pub obsidian_path: Option<Option<String>>,
}

/// 任务查询过滤器
#[derive(Debug, Clone, Default)]
pub struct TaskFilter {
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub source: Option<TaskSource>,
    pub parent_id: Option<Option<String>>,
}

impl Task {
    /// 判断是否为根任务（无父任务）
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }

    /// 判断是否为叶子任务（无子任务）
    pub fn is_leaf(&self) -> bool {
        self.children_ids.is_empty()
    }

    /// 判断是否已完成（Done 或 Archived）
    pub fn is_completed(&self) -> bool {
        self.status == TaskStatus::Done || self.status == TaskStatus::Archived
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_is_root() {
        let task = Task {
            id: "1".into(),
            title: "Test".into(),
            description: None,
            status: TaskStatus::Open,
            priority: TaskPriority::P2,
            source: TaskSource::Manual,
            parent_id: None,
            children_ids: vec![],
            due_date: None,
            tags: vec![],
            created_at: "2026-01-01".into(),
            updated_at: "2026-01-01".into(),
            completed_at: None,
            feishu_task_id: None,
            obsidian_path: None,
        };
        assert!(task.is_root());
        assert!(task.is_leaf());
    }

    #[test]
    fn test_task_with_parent_is_not_root() {
        let task = Task {
            id: "2".into(),
            title: "Child".into(),
            description: None,
            status: TaskStatus::Open,
            priority: TaskPriority::P2,
            source: TaskSource::Manual,
            parent_id: Some("1".into()),
            children_ids: vec![],
            due_date: None,
            tags: vec![],
            created_at: "2026-01-01".into(),
            updated_at: "2026-01-01".into(),
            completed_at: None,
            feishu_task_id: None,
            obsidian_path: None,
        };
        assert!(!task.is_root());
        assert!(task.is_leaf());
    }

    #[test]
    fn test_task_with_children_is_not_leaf() {
        let task = Task {
            id: "1".into(),
            title: "Parent".into(),
            description: None,
            status: TaskStatus::Open,
            priority: TaskPriority::P2,
            source: TaskSource::Manual,
            parent_id: None,
            children_ids: vec!["2".into(), "3".into()],
            due_date: None,
            tags: vec![],
            created_at: "2026-01-01".into(),
            updated_at: "2026-01-01".into(),
            completed_at: None,
            feishu_task_id: None,
            obsidian_path: None,
        };
        assert!(task.is_root());
        assert!(!task.is_leaf());
    }

    #[test]
    fn test_is_completed() {
        let mut task = Task {
            id: "1".into(),
            title: "Test".into(),
            description: None,
            status: TaskStatus::Open,
            priority: TaskPriority::P2,
            source: TaskSource::Manual,
            parent_id: None,
            children_ids: vec![],
            due_date: None,
            tags: vec![],
            created_at: "2026-01-01".into(),
            updated_at: "2026-01-01".into(),
            completed_at: None,
            feishu_task_id: None,
            obsidian_path: None,
        };
        assert!(!task.is_completed());

        task.status = TaskStatus::Done;
        assert!(task.is_completed());

        task.status = TaskStatus::Archived;
        assert!(task.is_completed());

        task.status = TaskStatus::InProgress;
        assert!(!task.is_completed());
    }
}
