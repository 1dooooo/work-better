//! 任务管理核心 —— CRUD + 父子层级 + 归档

pub mod create;
pub mod discovery;
pub mod discovery_confirm;
pub mod discovery_email;
pub mod discovery_meeting;
pub mod discovery_message;
pub mod hierarchy;
pub mod lifecycle;
pub mod model;
pub mod sync;

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use wb_core::error::{Result, WbError};

use create::create_task;
use model::{Task, TaskFilter, TaskPriority, TaskSource, TaskStatus, TaskUpdate};

/// 任务管理器
///
/// 内部使用 `Arc<RwLock<HashMap>>` 存储，支持并发访问。
pub struct TaskManager {
    tasks: Arc<RwLock<HashMap<String, Task>>>,
}

impl TaskManager {
    /// 创建新的任务管理器
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建任务
    pub async fn create(
        &self,
        title: &str,
        priority: TaskPriority,
        source: TaskSource,
    ) -> Result<Task> {
        let task = create_task(title, priority, source);
        let mut tasks = self.tasks.write().await;
        tasks.insert(task.id.clone(), task.clone());
        Ok(task)
    }

    /// 更新任务字段
    pub async fn update(&self, id: &str, update: TaskUpdate) -> Result<Task> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get(id)
            .ok_or_else(|| WbError::NotFound(format!("任务不存在: {}", id)))?
            .clone();

        let updated = Task {
            title: update.title.unwrap_or(task.title),
            description: update
                .description
                .unwrap_or(task.description),
            priority: update.priority.unwrap_or(task.priority),
            due_date: update.due_date.unwrap_or(task.due_date),
            tags: update.tags.unwrap_or(task.tags),
            feishu_task_id: update
                .feishu_task_id
                .unwrap_or(task.feishu_task_id),
            obsidian_path: update
                .obsidian_path
                .unwrap_or(task.obsidian_path),
            updated_at: chrono::Utc::now().to_rfc3339(),
            ..task
        };

        tasks.insert(id.to_string(), updated.clone());
        Ok(updated)
    }

    /// 状态流转
    pub async fn transition(&self, id: &str, new_status: TaskStatus) -> Result<Task> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get(id)
            .ok_or_else(|| WbError::NotFound(format!("任务不存在: {}", id)))?
            .clone();

        let updated = lifecycle::transition(&task, new_status)?;
        tasks.insert(id.to_string(), updated.clone());
        Ok(updated)
    }

    /// 归档任务（Done → Archived）
    pub async fn archive(&self, id: &str) -> Result<Task> {
        self.transition(id, TaskStatus::Archived).await
    }

    /// 添加子任务
    pub async fn add_subtask(&self, parent_id: &str, title: &str) -> Result<Task> {
        let mut tasks = self.tasks.write().await;
        let parent = tasks
            .get(parent_id)
            .ok_or_else(|| WbError::NotFound(format!("父任务不存在: {}", parent_id)))?
            .clone();

        let (updated_parent, child) = hierarchy::add_subtask(&parent, title)?;
        tasks.insert(parent_id.to_string(), updated_parent);
        tasks.insert(child.id.clone(), child.clone());
        Ok(child)
    }

    /// 获取单个任务
    pub async fn get(&self, id: &str) -> Result<Option<Task>> {
        let tasks = self.tasks.read().await;
        Ok(tasks.get(id).cloned())
    }

    /// 按过滤器查询任务
    pub async fn list(&self, filter: TaskFilter) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().await;
        let result: Vec<Task> = tasks
            .values()
            .filter(|t| {
                if let Some(ref status) = filter.status {
                    if t.status != *status {
                        return false;
                    }
                }
                if let Some(ref priority) = filter.priority {
                    if t.priority != *priority {
                        return false;
                    }
                }
                if let Some(ref source) = filter.source {
                    if t.source != *source {
                        return false;
                    }
                }
                if let Some(ref parent_id) = filter.parent_id {
                    if t.parent_id != *parent_id {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        Ok(result)
    }

    /// 按状态查询任务
    pub async fn list_by_status(&self, status: TaskStatus) -> Result<Vec<Task>> {
        self.list(TaskFilter {
            status: Some(status),
            ..TaskFilter::default()
        })
        .await
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mgr() -> TaskManager {
        TaskManager::new()
    }

    // --- 创建 ---

    #[tokio::test]
    async fn test_create_task() {
        let m = mgr();
        let task = m.create("Test", TaskPriority::P1, TaskSource::Manual).await.unwrap();
        assert_eq!(task.title, "Test");
        assert_eq!(task.status, TaskStatus::Open);
    }

    #[tokio::test]
    async fn test_create_auto_discovered_is_pending() {
        let m = mgr();
        let task = m.create("Auto", TaskPriority::P2, TaskSource::Meeting).await.unwrap();
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[tokio::test]
    async fn test_get_existing() {
        let m = mgr();
        let created = m.create("T", TaskPriority::P2, TaskSource::Manual).await.unwrap();
        let fetched = m.get(&created.id).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().title, "T");
    }

    #[tokio::test]
    async fn test_get_nonexistent() {
        let m = mgr();
        let result = m.get("no-such-id").await.unwrap();
        assert!(result.is_none());
    }

    // --- 更新 ---

    #[tokio::test]
    async fn test_update_title() {
        let m = mgr();
        let task = m.create("Old", TaskPriority::P2, TaskSource::Manual).await.unwrap();
        let updated = m
            .update(
                &task.id,
                TaskUpdate {
                    title: Some("New".into()),
                    ..TaskUpdate::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(updated.title, "New");
    }

    #[tokio::test]
    async fn test_update_nonexistent() {
        let m = mgr();
        let result = m
            .update("bad", TaskUpdate { title: Some("X".into()), ..TaskUpdate::default() })
            .await;
        assert!(result.is_err());
    }

    // --- 状态流转 ---

    #[tokio::test]
    async fn test_transition_open_to_in_progress() {
        let m = mgr();
        let task = m.create("T", TaskPriority::P2, TaskSource::Manual).await.unwrap();
        let t = m.transition(&task.id, TaskStatus::InProgress).await.unwrap();
        assert_eq!(t.status, TaskStatus::InProgress);
    }

    #[tokio::test]
    async fn test_transition_rejected() {
        let m = mgr();
        let task = m.create("T", TaskPriority::P2, TaskSource::Manual).await.unwrap();
        // Open → Done 是非法的
        let result = m.transition(&task.id, TaskStatus::Done).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_transition_nonexistent() {
        let m = mgr();
        let result = m.transition("bad", TaskStatus::InProgress).await;
        assert!(result.is_err());
    }

    // --- 归档 ---

    #[tokio::test]
    async fn test_archive_full_flow() {
        let m = mgr();
        let task = m.create("T", TaskPriority::P2, TaskSource::Manual).await.unwrap();
        let id = task.id.clone();
        m.transition(&id, TaskStatus::InProgress).await.unwrap();
        m.transition(&id, TaskStatus::Done).await.unwrap();
        let archived = m.archive(&id).await.unwrap();
        assert_eq!(archived.status, TaskStatus::Archived);
    }

    #[tokio::test]
    async fn test_archive_fails_from_open() {
        let m = mgr();
        let task = m.create("T", TaskPriority::P2, TaskSource::Manual).await.unwrap();
        assert!(m.archive(&task.id).await.is_err());
    }

    // --- 父子关系 ---

    #[tokio::test]
    async fn test_add_subtask() {
        let m = mgr();
        let parent = m.create("Parent", TaskPriority::P1, TaskSource::Manual).await.unwrap();
        let child = m.add_subtask(&parent.id, "Child").await.unwrap();

        assert_eq!(child.parent_id, Some(parent.id.clone()));
        assert_eq!(child.priority, TaskPriority::P1);

        let parent = m.get(&parent.id).await.unwrap().unwrap();
        assert_eq!(parent.children_ids.len(), 1);
        assert_eq!(parent.children_ids[0], child.id);
    }

    #[tokio::test]
    async fn test_add_subtask_to_nonexistent() {
        let m = mgr();
        assert!(m.add_subtask("bad", "Child").await.is_err());
    }

    // --- 列表过滤 ---

    #[tokio::test]
    async fn test_list_by_status() {
        let m = mgr();
        m.create("A", TaskPriority::P2, TaskSource::Manual).await.unwrap();
        m.create("B", TaskPriority::P2, TaskSource::Manual).await.unwrap();
        m.create("C", TaskPriority::P2, TaskSource::Meeting).await.unwrap(); // Pending

        let open = m.list_by_status(TaskStatus::Open).await.unwrap();
        assert_eq!(open.len(), 2);

        let pending = m.list_by_status(TaskStatus::Pending).await.unwrap();
        assert_eq!(pending.len(), 1);
    }

    #[tokio::test]
    async fn test_list_filter_by_priority() {
        let m = mgr();
        m.create("A", TaskPriority::P0, TaskSource::Manual).await.unwrap();
        m.create("B", TaskPriority::P1, TaskSource::Manual).await.unwrap();
        m.create("C", TaskPriority::P0, TaskSource::Manual).await.unwrap();

        let p0 = m
            .list(TaskFilter {
                priority: Some(TaskPriority::P0),
                ..TaskFilter::default()
            })
            .await
            .unwrap();
        assert_eq!(p0.len(), 2);
    }

    #[tokio::test]
    async fn test_list_filter_by_source() {
        let m = mgr();
        m.create("A", TaskPriority::P2, TaskSource::Manual).await.unwrap();
        m.create("B", TaskPriority::P2, TaskSource::Feishu).await.unwrap();

        let manual = m
            .list(TaskFilter {
                source: Some(TaskSource::Manual),
                ..TaskFilter::default()
            })
            .await
            .unwrap();
        assert_eq!(manual.len(), 1);
        assert_eq!(manual[0].source, TaskSource::Manual);
    }

    #[tokio::test]
    async fn test_list_filter_by_parent_id() {
        let m = mgr();
        let parent = m.create("Parent", TaskPriority::P2, TaskSource::Manual).await.unwrap();
        m.add_subtask(&parent.id, "Child").await.unwrap();
        m.create("Orphan", TaskPriority::P2, TaskSource::Manual).await.unwrap();

        let children = m
            .list(TaskFilter {
                parent_id: Some(Some(parent.id)),
                ..TaskFilter::default()
            })
            .await
            .unwrap();
        assert_eq!(children.len(), 1);

        let roots = m
            .list(TaskFilter {
                parent_id: Some(None),
                ..TaskFilter::default()
            })
            .await
            .unwrap();
        // 2 roots: parent + orphan
        assert_eq!(roots.len(), 2);
    }

    #[tokio::test]
    async fn test_list_combined_filter() {
        let m = mgr();
        m.create("A", TaskPriority::P0, TaskSource::Manual).await.unwrap();
        m.create("B", TaskPriority::P0, TaskSource::Manual).await.unwrap();
        m.create("C", TaskPriority::P1, TaskSource::Manual).await.unwrap();

        let result = m
            .list(TaskFilter {
                status: Some(TaskStatus::Open),
                priority: Some(TaskPriority::P0),
                ..TaskFilter::default()
            })
            .await
            .unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_list_empty() {
        let m = mgr();
        let all = m.list(TaskFilter::default()).await.unwrap();
        assert!(all.is_empty());
    }
}
