use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Blocked,
    Done,
    Cancelled,
}

impl TaskStatus {
    /// 检查是否可以转换到目标状态
    pub fn can_transition_to(&self, target: &TaskStatus) -> bool {
        matches!(
            (self, target),
            (TaskStatus::Todo, TaskStatus::InProgress)
                | (TaskStatus::Todo, TaskStatus::Cancelled)
                | (TaskStatus::InProgress, TaskStatus::Done)
                | (TaskStatus::InProgress, TaskStatus::Blocked)
                | (TaskStatus::InProgress, TaskStatus::Cancelled)
                | (TaskStatus::Blocked, TaskStatus::InProgress)
                | (TaskStatus::Blocked, TaskStatus::Cancelled)
        )
    }
}

/// 优先级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub enum Priority {
    P0,
    P1,
    P2,
    P3,
}

/// 任务
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct Task {
    pub id: String,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: Priority,
    #[ts(type = "string | null")]
    pub due_date: Option<DateTime<Utc>>,
    #[ts(type = "string | null")]
    pub completed_at: Option<DateTime<Utc>>,
    pub project: Option<String>,
    pub parent_task: Option<String>,
    pub assignee: String,
    pub collaborators: Vec<String>,
    pub source_event_ids: Vec<String>,
    pub source_platform: String,
    pub feishu_task_id: Option<String>,
    pub ai_summary: Option<String>,
    pub ai_progress: Option<String>,
    pub ai_risk: Option<String>,
    pub tags: Vec<String>,
    pub confidence: f64,
    pub needs_review: bool,
    pub obsidian_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        assert!(TaskStatus::Todo.can_transition_to(&TaskStatus::InProgress));
        assert!(TaskStatus::Todo.can_transition_to(&TaskStatus::Cancelled));
        assert!(TaskStatus::InProgress.can_transition_to(&TaskStatus::Done));
        assert!(TaskStatus::InProgress.can_transition_to(&TaskStatus::Blocked));
        assert!(TaskStatus::InProgress.can_transition_to(&TaskStatus::Cancelled));
        assert!(TaskStatus::Blocked.can_transition_to(&TaskStatus::InProgress));
        assert!(TaskStatus::Blocked.can_transition_to(&TaskStatus::Cancelled));
    }

    #[test]
    fn test_invalid_transitions() {
        assert!(!TaskStatus::Todo.can_transition_to(&TaskStatus::Done));
        assert!(!TaskStatus::Done.can_transition_to(&TaskStatus::InProgress));
        assert!(!TaskStatus::Blocked.can_transition_to(&TaskStatus::Done));
        assert!(!TaskStatus::Cancelled.can_transition_to(&TaskStatus::Todo));
        assert!(!TaskStatus::Cancelled.can_transition_to(&TaskStatus::InProgress));
        assert!(!TaskStatus::Done.can_transition_to(&TaskStatus::Todo));
        assert!(!TaskStatus::Done.can_transition_to(&TaskStatus::Blocked));
        assert!(!TaskStatus::Todo.can_transition_to(&TaskStatus::Blocked));
    }
}
