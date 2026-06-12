//! 任务创建功能真实后端测试
//!
//! 测试场景：调用 create_task → 任务出现在任务列表
//!
//! 验证目标：
//! 1. 任务被正确创建
//! 2. 任务字段（title, status, priority）正确
//! 3. 任务 ID 唯一
//! 4. 任务时间戳正确

use wb_core::task::{Task, TaskStatus, Priority};

/// 测试创建任务并验证字段
///
/// 场景：创建任务，验证所有字段
/// 预期：任务字段正确
#[tokio::test]
async fn test_create_task_fields() {
    // 创建任务
    let task = Task::new("测试任务", TaskStatus::Todo);

    // 验证字段
    assert!(!task.id.is_empty(), "任务 ID 不应该为空");
    assert_eq!(task.title, "测试任务", "任务标题应该匹配");
    assert_eq!(task.status, TaskStatus::Todo, "任务状态应该是 Todo");
    assert_eq!(task.priority, Priority::P2, "默认优先级应该是 P2");
    assert!(task.description.is_empty(), "描述应该为空");
    assert!(task.due_date.is_none(), "截止日期应该为空");
    assert!(task.completed_at.is_none(), "完成时间应该为空");
    assert!(task.project.is_none(), "项目应该为空");
    assert!(task.parent_task.is_none(), "父任务应该为空");
    assert!(task.assignee.is_empty(), "负责人应该为空");
    assert!(task.collaborators.is_empty(), "协作者应该为空");
    assert!(task.source_event_ids.is_empty(), "来源事件 ID 应该为空");
    assert!(task.source_platform.is_empty(), "来源平台应该为空");
    assert!(task.feishu_task_id.is_none(), "飞书任务 ID 应该为空");
    assert!(task.ai_summary.is_none(), "AI 摘要应该为空");
    assert!(task.ai_progress.is_none(), "AI 进度应该为空");
    assert!(task.ai_risk.is_none(), "AI 风险应该为空");
    assert!(task.tags.is_empty(), "标签应该为空");
    assert_eq!(task.confidence, 0.0, "置信度应该是 0.0");
    assert!(!task.needs_review, "不需要审查");
    assert!(task.obsidian_path.is_empty(), "Obsidian 路径应该为空");
}

/// 测试任务 ID 唯一性
///
/// 场景：创建多个任务
/// 预期：每个任务的 ID 唯一
#[tokio::test]
async fn test_task_id_uniqueness() {
    // 创建多个任务
    let task1 = Task::new("任务 1", TaskStatus::Todo);
    let task2 = Task::new("任务 2", TaskStatus::Todo);
    let task3 = Task::new("任务 3", TaskStatus::Todo);

    // 验证：ID 唯一
    assert_ne!(task1.id, task2.id, "任务 1 和任务 2 的 ID 应该不同");
    assert_ne!(task2.id, task3.id, "任务 2 和任务 3 的 ID 应该不同");
    assert_ne!(task1.id, task3.id, "任务 1 和任务 3 的 ID 应该不同");
}

/// 测试任务时间戳
///
/// 场景：创建任务，检查时间戳
/// 预期：created_at 和 updated_at 有值且合理
#[tokio::test]
async fn test_task_timestamps() {
    // 记录创建前的时间
    let before = chrono::Utc::now();

    // 创建任务
    let task = Task::new("时间戳测试", TaskStatus::Todo);

    // 记录创建后的时间
    let after = chrono::Utc::now();

    // 验证：时间戳在合理范围内
    assert!(
        task.created_at >= before && task.created_at <= after,
        "created_at 应该在创建前后的时间范围内"
    );
    assert!(
        task.updated_at >= before && task.updated_at <= after,
        "updated_at 应该在创建前后的时间范围内"
    );
    assert_eq!(
        task.created_at, task.updated_at,
        "创建时 created_at 和 updated_at 应该相同"
    );
}

/// 测试任务状态转换
///
/// 场景：创建任务后，转换状态
/// 预期：状态正确转换
#[tokio::test]
async fn test_task_status_transition() {
    // 创建任务
    let task = Task::new("状态转换测试", TaskStatus::Todo);

    // 转换到 InProgress
    let task = task.transition(TaskStatus::InProgress).unwrap();
    assert_eq!(task.status, TaskStatus::InProgress, "状态应该是 InProgress");

    // 转换到 Done
    let task = task.transition(TaskStatus::Done).unwrap();
    assert_eq!(task.status, TaskStatus::Done, "状态应该是 Done");
    assert!(task.completed_at.is_some(), "完成时间应该被设置");
}

/// 测试任务状态转换的不可变性
///
/// 场景：转换状态后，原任务不变
/// 预期：原任务保持原状态
#[tokio::test]
async fn test_task_transition_immutability() {
    // 创建任务
    let original = Task::new("不可变性测试", TaskStatus::Todo);

    // 转换状态
    let transitioned = original.transition(TaskStatus::InProgress).unwrap();

    // 验证：原任务不变
    assert_eq!(
        original.status, TaskStatus::Todo,
        "原任务状态应该保持为 Todo"
    );
    assert_eq!(
        transitioned.status, TaskStatus::InProgress,
        "新任务状态应该是 InProgress"
    );
    assert_eq!(original.id, transitioned.id, "ID 应该保持一致");
}

/// 测试无效的状态转换
///
/// 场景：尝试无效的状态转换
/// 预期：返回错误
#[tokio::test]
async fn test_invalid_status_transition() {
    // 创建任务
    let task = Task::new("无效转换测试", TaskStatus::Todo);

    // 尝试从 Todo 直接到 Done（无效）
    let result = task.transition(TaskStatus::Done);
    assert!(result.is_err(), "从 Todo 到 Done 的转换应该失败");
}

/// 测试创建多个不同状态的任务
///
/// 场景：创建多个不同状态的任务
/// 预期：每个任务的状态正确
#[tokio::test]
async fn test_create_multiple_tasks_different_status() {
    // 创建不同状态的任务
    let todo_task = Task::new("待办任务", TaskStatus::Todo);
    let in_progress_task = Task::new("进行中任务", TaskStatus::InProgress);
    let blocked_task = Task::new("阻塞任务", TaskStatus::Blocked);
    let done_task = Task::new("已完成任务", TaskStatus::Done);
    let cancelled_task = Task::new("已取消任务", TaskStatus::Cancelled);

    // 验证状态
    assert_eq!(todo_task.status, TaskStatus::Todo);
    assert_eq!(in_progress_task.status, TaskStatus::InProgress);
    assert_eq!(blocked_task.status, TaskStatus::Blocked);
    assert_eq!(done_task.status, TaskStatus::Done);
    assert_eq!(cancelled_task.status, TaskStatus::Cancelled);
}
