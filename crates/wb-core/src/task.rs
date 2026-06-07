use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

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

impl Task {
    /// 创建新任务，自动分配 id、created_at、updated_at
    pub fn new(title: impl Into<String>, status: TaskStatus) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            created_at: now,
            updated_at: now,
            title: title.into(),
            description: String::new(),
            status,
            priority: Priority::P2,
            due_date: None,
            completed_at: None,
            project: None,
            parent_task: None,
            assignee: String::new(),
            collaborators: Vec::new(),
            source_event_ids: Vec::new(),
            source_platform: String::new(),
            feishu_task_id: None,
            ai_summary: None,
            ai_progress: None,
            ai_risk: None,
            tags: Vec::new(),
            confidence: 0.0,
            needs_review: false,
            obsidian_path: String::new(),
        }
    }

    /// 转换到目标状态，返回新 Task（不可变）。失败返回错误描述。
    pub fn transition(&self, target: TaskStatus) -> std::result::Result<Self, String> {
        if !self.status.can_transition_to(&target) {
            return Err(format!(
                "Invalid transition: {:?} -> {:?}",
                self.status, target
            ));
        }
        let mut new_task = self.clone();
        new_task.status = target.clone();
        new_task.updated_at = Utc::now();
        if target == TaskStatus::Done {
            new_task.completed_at = Some(new_task.updated_at);
        }
        Ok(new_task)
    }

    /// 归档任务（仅允许从 Done 状态归档），返回新 Task
    pub fn archive(&self) -> std::result::Result<Self, String> {
        if self.status != TaskStatus::Done {
            return Err(format!(
                "Can only archive tasks in Done status, current: {:?}",
                self.status
            ));
        }
        // 归档通过设置 obsidian_path 前缀表示（实际业务中可能移动文件）
        let mut new_task = self.clone();
        new_task.obsidian_path = format!("archive/{}", self.obsidian_path);
        new_task.updated_at = Utc::now();
        Ok(new_task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    // ============================================================
    // A3-01~07: Valid transitions
    // ============================================================

    #[rstest]
    #[case(TaskStatus::Todo, TaskStatus::InProgress)] // A3-01
    #[case(TaskStatus::Todo, TaskStatus::Cancelled)] // A3-02
    #[case(TaskStatus::InProgress, TaskStatus::Done)] // A3-03
    #[case(TaskStatus::InProgress, TaskStatus::Blocked)] // A3-04
    #[case(TaskStatus::InProgress, TaskStatus::Cancelled)] // A3-05
    #[case(TaskStatus::Blocked, TaskStatus::InProgress)] // A3-06
    #[case(TaskStatus::Blocked, TaskStatus::Cancelled)] // A3-07
    fn a3_valid_transitions(#[case] from: TaskStatus, #[case] to: TaskStatus) {
        let task = Task::new("test task", from);
        let result = task.transition(to.clone());
        assert!(result.is_ok(), "Transition should succeed");
        assert_eq!(result.unwrap().status, to);
    }

    // ============================================================
    // A3-08~15: Invalid transitions
    // ============================================================

    #[rstest]
    #[case(TaskStatus::Todo, TaskStatus::Done)] // A3-08
    #[case(TaskStatus::Done, TaskStatus::InProgress)] // A3-09
    #[case(TaskStatus::Blocked, TaskStatus::Done)] // A3-10
    #[case(TaskStatus::Cancelled, TaskStatus::Todo)] // A3-11
    #[case(TaskStatus::Cancelled, TaskStatus::InProgress)] // A3-12
    #[case(TaskStatus::Done, TaskStatus::Todo)] // A3-13
    #[case(TaskStatus::Done, TaskStatus::Blocked)] // A3-14
    #[case(TaskStatus::Todo, TaskStatus::Blocked)] // A3-15
    fn a3_invalid_transitions(#[case] from: TaskStatus, #[case] to: TaskStatus) {
        let task = Task::new("test task", from);
        let result = task.transition(to);
        assert!(result.is_err(), "Transition should fail");
    }

    // ============================================================
    // A3-08b: same -> same is also invalid
    // ============================================================

    #[rstest]
    #[case(TaskStatus::Todo)] // A3-15b
    #[case(TaskStatus::InProgress)]
    #[case(TaskStatus::Blocked)]
    #[case(TaskStatus::Done)]
    #[case(TaskStatus::Cancelled)]
    fn a3_same_to_same_is_invalid(#[case] status: TaskStatus) {
        let task = Task::new("test task", status.clone());
        let result = task.transition(status);
        assert!(result.is_err(), "Same->same transition should fail");
    }

    // ============================================================
    // A3-16: Transition preserves immutability (original unchanged)
    // ============================================================

    #[test]
    fn a3_16_transition_preserves_immutability() {
        let original = Task::new("immutable task", TaskStatus::Todo);
        let transitioned = original.transition(TaskStatus::InProgress).unwrap();

        // Original must be unchanged
        assert_eq!(original.status, TaskStatus::Todo);
        // New task has the new status
        assert_eq!(transitioned.status, TaskStatus::InProgress);
        // Id is preserved (clone, not new task)
        assert_eq!(original.id, transitioned.id);
        assert_eq!(original.title, transitioned.title);
    }

    // ============================================================
    // A3-17: completed_at only set on Done
    // ============================================================

    #[test]
    fn a3_17_completed_at_only_set_on_done() {
        // Transition to InProgress — completed_at should remain None
        let todo = Task::new("task", TaskStatus::Todo);
        let in_progress = todo.transition(TaskStatus::InProgress).unwrap();
        assert!(
            in_progress.completed_at.is_none(),
            "completed_at should be None for InProgress"
        );

        // Transition to Done — completed_at should be set
        let done = in_progress.transition(TaskStatus::Done).unwrap();
        assert!(
            done.completed_at.is_some(),
            "completed_at should be set for Done"
        );

        // Transition to Cancelled — completed_at should remain None
        let todo2 = Task::new("task2", TaskStatus::Todo);
        let cancelled = todo2.transition(TaskStatus::Cancelled).unwrap();
        assert!(
            cancelled.completed_at.is_none(),
            "completed_at should be None for Cancelled"
        );
    }

    // ============================================================
    // A3-18: updated_at changes on every transition
    // ============================================================

    #[test]
    fn a3_18_updated_at_changes_on_transition() {
        let task = Task::new("task", TaskStatus::Todo);
        let original_updated = task.updated_at;

        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        let transitioned = task.transition(TaskStatus::InProgress).unwrap();
        assert!(
            transitioned.updated_at > original_updated,
            "updated_at should increase after transition"
        );

        let original_updated2 = transitioned.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));

        let done = transitioned.transition(TaskStatus::Done).unwrap();
        assert!(
            done.updated_at > original_updated2,
            "updated_at should increase again"
        );
    }

    // ============================================================
    // A3-19: Full lifecycle verification
    // ============================================================

    #[test]
    fn a3_19_full_lifecycle() {
        let task = Task::new("full lifecycle task", TaskStatus::Todo);
        assert_eq!(task.status, TaskStatus::Todo);
        assert!(task.completed_at.is_none());

        // Todo -> InProgress
        let task = task.transition(TaskStatus::InProgress).unwrap();
        assert_eq!(task.status, TaskStatus::InProgress);
        assert!(task.completed_at.is_none());

        // InProgress -> Blocked
        let task = task.transition(TaskStatus::Blocked).unwrap();
        assert_eq!(task.status, TaskStatus::Blocked);
        assert!(task.completed_at.is_none());

        // Blocked -> InProgress
        let task = task.transition(TaskStatus::InProgress).unwrap();
        assert_eq!(task.status, TaskStatus::InProgress);

        // InProgress -> Done
        let task = task.transition(TaskStatus::Done).unwrap();
        assert_eq!(task.status, TaskStatus::Done);
        assert!(task.completed_at.is_some());

        // Verify id is preserved throughout
        assert!(!task.id.is_empty());
    }

    // ============================================================
    // A3-20: Archive only available from Done
    // ============================================================

    #[test]
    fn a3_20_archive_only_from_done() {
        // Archive from Done — should succeed
        let done_task = Task::new("done task", TaskStatus::Todo)
            .transition(TaskStatus::InProgress)
            .unwrap()
            .transition(TaskStatus::Done)
            .unwrap();
        let archived = done_task.archive();
        assert!(archived.is_ok(), "Should be able to archive Done task");

        // Archive from Todo — should fail
        let todo_task = Task::new("todo task", TaskStatus::Todo);
        assert!(
            todo_task.archive().is_err(),
            "Should not be able to archive Todo task"
        );

        // Archive from InProgress — should fail
        let in_progress = todo_task.transition(TaskStatus::InProgress).unwrap();
        assert!(
            in_progress.archive().is_err(),
            "Should not be able to archive InProgress task"
        );

        // Archive from Blocked — should fail
        let blocked = in_progress.transition(TaskStatus::Blocked).unwrap();
        assert!(
            blocked.archive().is_err(),
            "Should not be able to archive Blocked task"
        );

        // Archive from Cancelled — should fail
        let todo_task2 = Task::new("cancelled task", TaskStatus::Todo);
        let cancelled = todo_task2.transition(TaskStatus::Cancelled).unwrap();
        assert!(
            cancelled.archive().is_err(),
            "Should not be able to archive Cancelled task"
        );
    }
}
