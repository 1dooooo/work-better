//! 父子任务层级 —— 子任务拆分与层级管理

use wb_core::error::{Result, WbError};

use super::create::create_subtask;
use super::model::Task;

/// 添加子任务到父任务，返回 (更新后的父任务, 新子任务)
///
/// 规则：
/// - 父任务必须是叶子节点或已是父节点均可
/// - 父任务不能是 Archived 状态
pub fn add_subtask(parent: &Task, title: &str) -> Result<(Task, Task)> {
    if parent.status == super::model::TaskStatus::Archived {
        return Err(WbError::Ai(
            "不能为已归档的任务添加子任务".to_string(),
        ));
    }

    let child = create_subtask(parent, title);

    let mut new_children = parent.children_ids.clone();
    new_children.push(child.id.clone());

    let updated_parent = Task {
        id: parent.id.clone(),
        title: parent.title.clone(),
        description: parent.description.clone(),
        status: parent.status.clone(),
        priority: parent.priority.clone(),
        source: parent.source.clone(),
        parent_id: parent.parent_id.clone(),
        children_ids: new_children,
        due_date: parent.due_date.clone(),
        tags: parent.tags.clone(),
        created_at: parent.created_at.clone(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        completed_at: parent.completed_at.clone(),
        feishu_task_id: parent.feishu_task_id.clone(),
        obsidian_path: parent.obsidian_path.clone(),
    };

    Ok((updated_parent, child))
}

/// 获取任务的所有后代 ID（广度优先遍历）
pub fn collect_descendant_ids(task_id: &str, all_tasks: &std::collections::HashMap<String, Task>) -> Vec<String> {
    let mut result = Vec::new();
    let mut queue = std::collections::VecDeque::new();

    if let Some(task) = all_tasks.get(task_id) {
        for child_id in &task.children_ids {
            queue.push_back(child_id.clone());
        }
    }

    while let Some(current_id) = queue.pop_front() {
        result.push(current_id.clone());
        if let Some(task) = all_tasks.get(&current_id) {
            for child_id in &task.children_ids {
                queue.push_back(child_id.clone());
            }
        }
    }

    result
}

/// 判断一个任务是否是另一个任务的祖先
pub fn is_ancestor(
    potential_ancestor_id: &str,
    task_id: &str,
    all_tasks: &std::collections::HashMap<String, Task>,
) -> bool {
    let descendants = collect_descendant_ids(potential_ancestor_id, all_tasks);
    descendants.contains(&task_id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::create::create_task;
    use crate::task::model::{TaskPriority, TaskSource, TaskStatus};

    #[test]
    fn test_add_subtask_to_leaf() {
        let parent = create_task("Parent", TaskPriority::P1, TaskSource::Manual);
        let (updated_parent, child) = add_subtask(&parent, "Child 1").unwrap();

        assert_eq!(updated_parent.children_ids.len(), 1);
        assert_eq!(updated_parent.children_ids[0], child.id);
        assert_eq!(child.parent_id, Some(parent.id));
    }

    #[test]
    fn test_add_multiple_subtasks() {
        let parent = create_task("Parent", TaskPriority::P1, TaskSource::Manual);
        let (parent, _c1) = add_subtask(&parent, "Child 1").unwrap();
        let (parent, c2) = add_subtask(&parent, "Child 2").unwrap();

        assert_eq!(parent.children_ids.len(), 2);
        assert_eq!(parent.children_ids[1], c2.id);
    }

    #[test]
    fn test_add_subtask_fails_on_archived() {
        let mut parent = create_task("Parent", TaskPriority::P1, TaskSource::Manual);
        parent.status = TaskStatus::Archived;
        assert!(add_subtask(&parent, "Child").is_err());
    }

    #[test]
    fn test_add_subtask_preserves_immutability() {
        let parent = create_task("Parent", TaskPriority::P1, TaskSource::Manual);
        let (updated, _child) = add_subtask(&parent, "Child").unwrap();

        assert!(parent.children_ids.is_empty());
        assert_eq!(updated.children_ids.len(), 1);
    }

    #[test]
    fn test_add_subtask_inherits_priority() {
        let parent = create_task("Parent", TaskPriority::P0, TaskSource::Feishu);
        let (_parent, child) = add_subtask(&parent, "Child").unwrap();

        assert_eq!(child.priority, TaskPriority::P0);
        assert_eq!(child.source, TaskSource::Feishu);
    }

    #[test]
    fn test_collect_descendant_ids_empty() {
        let tasks = std::collections::HashMap::new();
        let result = collect_descendant_ids("nonexistent", &tasks);
        assert!(result.is_empty());
    }

    #[test]
    fn test_collect_descendant_ids_single_level() {
        let parent = create_task("Parent", TaskPriority::P2, TaskSource::Manual);
        let (parent, c1) = add_subtask(&parent, "C1").unwrap();
        let (parent, c2) = add_subtask(&parent, "C2").unwrap();

        let mut tasks = std::collections::HashMap::new();
        tasks.insert(parent.id.clone(), parent.clone());
        tasks.insert(c1.id.clone(), c1.clone());
        tasks.insert(c2.id.clone(), c2.clone());

        let descendants = collect_descendant_ids(&parent.id, &tasks);
        assert_eq!(descendants.len(), 2);
        assert!(descendants.contains(&c1.id));
        assert!(descendants.contains(&c2.id));
    }

    #[test]
    fn test_collect_descendant_ids_multi_level() {
        let grandparent = create_task("GP", TaskPriority::P2, TaskSource::Manual);
        let (grandparent, parent) = add_subtask(&grandparent, "P").unwrap();
        let (parent, child) = add_subtask(&parent, "C").unwrap();

        let mut tasks = std::collections::HashMap::new();
        tasks.insert(grandparent.id.clone(), grandparent.clone());
        tasks.insert(parent.id.clone(), parent.clone());
        tasks.insert(child.id.clone(), child.clone());

        let descendants = collect_descendant_ids(&grandparent.id, &tasks);
        assert_eq!(descendants.len(), 2);
        assert!(descendants.contains(&parent.id));
        assert!(descendants.contains(&child.id));
    }

    #[test]
    fn test_is_ancestor() {
        let gp = create_task("GP", TaskPriority::P2, TaskSource::Manual);
        let (gp, p) = add_subtask(&gp, "P").unwrap();
        let (p, c) = add_subtask(&p, "C").unwrap();

        let mut tasks = std::collections::HashMap::new();
        tasks.insert(gp.id.clone(), gp.clone());
        tasks.insert(p.id.clone(), p.clone());
        tasks.insert(c.id.clone(), c.clone());

        assert!(is_ancestor(&gp.id, &p.id, &tasks));
        assert!(is_ancestor(&gp.id, &c.id, &tasks));
        assert!(is_ancestor(&p.id, &c.id, &tasks));
        assert!(!is_ancestor(&c.id, &gp.id, &tasks));
        assert!(!is_ancestor(&p.id, &gp.id, &tasks));
    }
}
