//! 飞书任务同步 —— 飞书 ↔ Obsidian 双向同步 + 冲突处理

use serde::{Deserialize, Serialize};

use super::model::Task;

/// 飞书外部任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuTask {
    pub id: String,
    pub title: String,
    pub status: String,
    pub updated_at: String,
}

/// 同步方向
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SyncDirection {
    FeishuToObsidian,
    ObsidianToFeishu,
}

/// 同步动作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SyncAction {
    Created,
    Updated,
    ConflictResolved,
    Skipped,
}

/// 冲突类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConflictType {
    BothModified,
    DeletedOnOneSide,
    StatusMismatch,
}

/// 冲突解决策略
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Resolution {
    /// 保留本地版本
    KeepLocal,
    /// 保留远程版本
    KeepRemote,
    /// 时间戳优先（自动）
    TimestampPriority,
}

/// 同步日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncLogEntry {
    pub task_id: String,
    pub direction: SyncDirection,
    pub action: SyncAction,
    pub timestamp: String,
    pub conflict: Option<ConflictType>,
}

/// 同步结果
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub task_id: String,
    pub direction: SyncDirection,
    pub action: SyncAction,
    pub conflict: Option<ConflictType>,
}

/// 任务同步编排器
///
/// 管理飞书 ↔ Obsidian 双向同步，处理冲突并记录同步日志。
pub struct TaskSync {
    log: Vec<SyncLogEntry>,
}

impl TaskSync {
    /// 创建新的同步编排器
    pub fn new() -> Self {
        Self { log: Vec::new() }
    }

    /// 飞书 → Obsidian 同步（定时拉取变更，自动更新）
    ///
    /// 遍历飞书任务列表，根据 `feishu_task_id` 查找本地对应任务：
    /// - 若本地不存在 → 创建新任务
    /// - 若本地存在且有冲突 → 检测并记录冲突
    /// - 若本地存在且无冲突 → 更新
    pub fn sync_from_feishu(
        &mut self,
        feishu_tasks: &[FeishuTask],
        local_tasks: &[Task],
    ) -> Vec<SyncResult> {
        let now = chrono::Utc::now().to_rfc3339();
        let mut results = Vec::new();

        for ft in feishu_tasks {
            let local = local_tasks
                .iter()
                .find(|t| t.feishu_task_id.as_deref() == Some(&ft.id));

            match local {
                None => {
                    // 本地不存在，创建
                    let result = SyncResult {
                        task_id: ft.id.clone(),
                        direction: SyncDirection::FeishuToObsidian,
                        action: SyncAction::Created,
                        conflict: None,
                    };
                    self.log.push(SyncLogEntry {
                        task_id: ft.id.clone(),
                        direction: SyncDirection::FeishuToObsidian,
                        action: SyncAction::Created,
                        timestamp: now.clone(),
                        conflict: None,
                    });
                    results.push(result);
                }
                Some(local_task) => {
                    // 本地存在，检测冲突
                    let conflict = Self::detect_conflict_static(local_task, ft);
                    match conflict {
                        Some(c) => {
                            let result = SyncResult {
                                task_id: local_task.id.clone(),
                                direction: SyncDirection::FeishuToObsidian,
                                action: SyncAction::ConflictResolved,
                                conflict: Some(c.clone()),
                            };
                            self.log.push(SyncLogEntry {
                                task_id: local_task.id.clone(),
                                direction: SyncDirection::FeishuToObsidian,
                                action: SyncAction::ConflictResolved,
                                timestamp: now.clone(),
                                conflict: Some(c),
                            });
                            results.push(result);
                        }
                        None => {
                            // 无冲突，检查是否需要更新
                            let needs_update =
                                local_task.title != ft.title || local_task.updated_at < ft.updated_at;
                            if needs_update {
                                let result = SyncResult {
                                    task_id: local_task.id.clone(),
                                    direction: SyncDirection::FeishuToObsidian,
                                    action: SyncAction::Updated,
                                    conflict: None,
                                };
                                self.log.push(SyncLogEntry {
                                    task_id: local_task.id.clone(),
                                    direction: SyncDirection::FeishuToObsidian,
                                    action: SyncAction::Updated,
                                    timestamp: now.clone(),
                                    conflict: None,
                                });
                                results.push(result);
                            } else {
                                let result = SyncResult {
                                    task_id: local_task.id.clone(),
                                    direction: SyncDirection::FeishuToObsidian,
                                    action: SyncAction::Skipped,
                                    conflict: None,
                                };
                                self.log.push(SyncLogEntry {
                                    task_id: local_task.id.clone(),
                                    direction: SyncDirection::FeishuToObsidian,
                                    action: SyncAction::Skipped,
                                    timestamp: now.clone(),
                                    conflict: None,
                                });
                                results.push(result);
                            }
                        }
                    }
                }
            }
        }

        results
    }

    /// Obsidian → 飞书同步（检测本地变更，需确认后同步）
    ///
    /// 遍历本地任务列表，若有关联的飞书任务 ID，则检测变更：
    /// - 若有冲突 → 记录冲突，不自动同步
    /// - 若无冲突且本地更新 → 标记为需要同步
    pub fn sync_to_feishu(
        &mut self,
        local_tasks: &[Task],
        feishu_tasks: &[FeishuTask],
    ) -> Vec<SyncResult> {
        let now = chrono::Utc::now().to_rfc3339();
        let mut results = Vec::new();

        for task in local_tasks {
            let feishu_id = match &task.feishu_task_id {
                Some(id) => id.clone(),
                None => continue, // 无飞书关联，跳过
            };

            let remote = feishu_tasks.iter().find(|ft| ft.id == feishu_id);

            match remote {
                None => {
                    // 飞书端已删除
                    let result = SyncResult {
                        task_id: task.id.clone(),
                        direction: SyncDirection::ObsidianToFeishu,
                        action: SyncAction::ConflictResolved,
                        conflict: Some(ConflictType::DeletedOnOneSide),
                    };
                    self.log.push(SyncLogEntry {
                        task_id: task.id.clone(),
                        direction: SyncDirection::ObsidianToFeishu,
                        action: SyncAction::ConflictResolved,
                        timestamp: now.clone(),
                        conflict: Some(ConflictType::DeletedOnOneSide),
                    });
                    results.push(result);
                }
                Some(remote_task) => {
                    let conflict = Self::detect_conflict_static(task, remote_task);
                    match conflict {
                        Some(c) => {
                            let result = SyncResult {
                                task_id: task.id.clone(),
                                direction: SyncDirection::ObsidianToFeishu,
                                action: SyncAction::ConflictResolved,
                                conflict: Some(c.clone()),
                            };
                            self.log.push(SyncLogEntry {
                                task_id: task.id.clone(),
                                direction: SyncDirection::ObsidianToFeishu,
                                action: SyncAction::ConflictResolved,
                                timestamp: now.clone(),
                                conflict: Some(c),
                            });
                            results.push(result);
                        }
                        None => {
                            if task.updated_at > remote_task.updated_at {
                                let result = SyncResult {
                                    task_id: task.id.clone(),
                                    direction: SyncDirection::ObsidianToFeishu,
                                    action: SyncAction::Updated,
                                    conflict: None,
                                };
                                self.log.push(SyncLogEntry {
                                    task_id: task.id.clone(),
                                    direction: SyncDirection::ObsidianToFeishu,
                                    action: SyncAction::Updated,
                                    timestamp: now.clone(),
                                    conflict: None,
                                });
                                results.push(result);
                            } else {
                                let result = SyncResult {
                                    task_id: task.id.clone(),
                                    direction: SyncDirection::ObsidianToFeishu,
                                    action: SyncAction::Skipped,
                                    conflict: None,
                                };
                                self.log.push(SyncLogEntry {
                                    task_id: task.id.clone(),
                                    direction: SyncDirection::ObsidianToFeishu,
                                    action: SyncAction::Skipped,
                                    timestamp: now.clone(),
                                    conflict: None,
                                });
                                results.push(result);
                            }
                        }
                    }
                }
            }
        }

        results
    }

    /// 冲突检测（实例方法，委托到静态方法）
    pub fn detect_conflicts(&self, local: &Task, remote: &FeishuTask) -> Option<ConflictType> {
        Self::detect_conflict_static(local, remote)
    }

    /// 冲突检测（静态实现）
    fn detect_conflict_static(local: &Task, remote: &FeishuTask) -> Option<ConflictType> {
        // 状态映射：将飞书状态映射到本地状态进行比较
        let remote_status = map_feishu_status(&remote.status);
        let local_status_str = map_task_status(&local.status);

        let title_changed = local.title != remote.title;
        let status_mismatch = local_status_str != remote_status;
        let both_modified = local.updated_at != remote.updated_at
            && (title_changed || status_mismatch);

        if both_modified {
            Some(ConflictType::BothModified)
        } else if status_mismatch {
            Some(ConflictType::StatusMismatch)
        } else {
            None
        }
    }

    /// 冲突解决（时间戳优先）
    pub fn resolve_conflict(
        &mut self,
        local: &Task,
        remote: &FeishuTask,
        resolution: Resolution,
    ) -> SyncResult {
        let now = chrono::Utc::now().to_rfc3339();
        let conflict = Self::detect_conflict_static(local, remote);

        let action = match resolution {
            Resolution::KeepLocal => SyncAction::ConflictResolved,
            Resolution::KeepRemote => SyncAction::ConflictResolved,
            Resolution::TimestampPriority => {
                // 时间戳优先：保留较新的一方
                if local.updated_at >= remote.updated_at {
                    SyncAction::Updated // 保留本地
                } else {
                    SyncAction::ConflictResolved // 保留远端
                }
            }
        };

        let result = SyncResult {
            task_id: local.id.clone(),
            direction: SyncDirection::FeishuToObsidian,
            action: action.clone(),
            conflict: conflict.clone(),
        };

        self.log.push(SyncLogEntry {
            task_id: local.id.clone(),
            direction: SyncDirection::FeishuToObsidian,
            action,
            timestamp: now,
            conflict,
        });

        result
    }

    /// 获取同步日志
    pub fn log(&self) -> &[SyncLogEntry] {
        &self.log
    }
}

impl Default for TaskSync {
    fn default() -> Self {
        Self::new()
    }
}

/// 将飞书状态映射到通用状态字符串
fn map_feishu_status(status: &str) -> String {
    match status.to_lowercase().as_str() {
        "todo" | "open" | "pending" => "Open".to_string(),
        "in_progress" | "in progress" | "doing" => "InProgress".to_string(),
        "done" | "completed" | "closed" => "Done".to_string(),
        "archived" => "Archived".to_string(),
        _ => status.to_string(),
    }
}

/// 将本地任务状态映射到字符串
fn map_task_status(status: &super::model::TaskStatus) -> String {
    match status {
        super::model::TaskStatus::Pending => "Pending".to_string(),
        super::model::TaskStatus::Open => "Open".to_string(),
        super::model::TaskStatus::InProgress => "InProgress".to_string(),
        super::model::TaskStatus::Done => "Done".to_string(),
        super::model::TaskStatus::Archived => "Archived".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::model::{TaskPriority, TaskSource, TaskStatus};

    fn make_task(id: &str, title: &str, status: TaskStatus, updated_at: &str) -> Task {
        make_task_with_feishu(id, title, status, updated_at, Some(format!("feishu-{}", id)))
    }

    fn make_task_with_feishu(
        id: &str,
        title: &str,
        status: TaskStatus,
        updated_at: &str,
        feishu_task_id: Option<String>,
    ) -> Task {
        Task {
            id: id.into(),
            title: title.into(),
            description: None,
            status,
            priority: TaskPriority::P2,
            source: TaskSource::Feishu,
            parent_id: None,
            children_ids: vec![],
            due_date: None,
            tags: vec![],
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: updated_at.into(),
            completed_at: None,
            feishu_task_id,
            obsidian_path: None,
        }
    }

    fn make_feishu_task(id: &str, title: &str, status: &str, updated_at: &str) -> FeishuTask {
        FeishuTask {
            id: id.into(),
            title: title.into(),
            status: status.into(),
            updated_at: updated_at.into(),
        }
    }

    // --- new ---

    #[test]
    fn test_new_sync_is_empty() {
        let sync = TaskSync::new();
        assert!(sync.log().is_empty());
    }

    // --- sync_from_feishu: 创建新任务 ---

    #[test]
    fn test_sync_from_feishu_creates_new_task() {
        let mut sync = TaskSync::new();
        let feishu_tasks = vec![make_feishu_task("f1", "New Task", "todo", "2026-01-02T00:00:00Z")];
        let local_tasks: Vec<Task> = vec![];

        let results = sync.sync_from_feishu(&feishu_tasks, &local_tasks);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].action, SyncAction::Created);
        assert_eq!(results[0].direction, SyncDirection::FeishuToObsidian);
        assert!(results[0].conflict.is_none());
        assert_eq!(sync.log().len(), 1);
    }

    // --- sync_from_feishu: 跳过未变更 ---

    #[test]
    fn test_sync_from_feishu_skips_unchanged() {
        let mut sync = TaskSync::new();
        let feishu_tasks = vec![make_feishu_task("f1", "Same Task", "todo", "2026-01-01T00:00:00Z")];
        let local_tasks = vec![make_task_with_feishu("t1", "Same Task", TaskStatus::Open, "2026-01-01T00:00:00Z", Some("f1".into()))];

        let results = sync.sync_from_feishu(&feishu_tasks, &local_tasks);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].action, SyncAction::Skipped);
    }

    // --- sync_from_feishu: 更新已变更任务 ---

    #[test]
    fn test_sync_from_feishu_updates_changed_task() {
        let mut sync = TaskSync::new();
        // Same title & status, only updated_at differs → no conflict, just update
        let feishu_tasks = vec![make_feishu_task("f1", "Same Title", "todo", "2026-01-02T00:00:00Z")];
        let local_tasks = vec![make_task_with_feishu("t1", "Same Title", TaskStatus::Open, "2026-01-01T00:00:00Z", Some("f1".into()))];

        let results = sync.sync_from_feishu(&feishu_tasks, &local_tasks);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].action, SyncAction::Updated);
    }

    // --- sync_from_feishu: 检测冲突 ---

    #[test]
    fn test_sync_from_feishu_detects_conflict() {
        let mut sync = TaskSync::new();
        // 双方都修改了：标题不同 + 时间戳不同
        let feishu_tasks = vec![make_feishu_task("f1", "Remote Changed", "in_progress", "2026-01-03T00:00:00Z")];
        let local_tasks = vec![make_task_with_feishu("t1", "Local Changed", TaskStatus::Open, "2026-01-02T00:00:00Z", Some("f1".into()))];

        let results = sync.sync_from_feishu(&feishu_tasks, &local_tasks);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].action, SyncAction::ConflictResolved);
        assert!(results[0].conflict.is_some());
    }

    // --- sync_to_feishu: 跳过无飞书关联 ---

    #[test]
    fn test_sync_to_feishu_skips_no_feishu_id() {
        let mut sync = TaskSync::new();
        let local_tasks = vec![Task {
            feishu_task_id: None,
            ..make_task("t1", "No Feishu", TaskStatus::Open, "2026-01-01T00:00:00Z")
        }];
        let feishu_tasks = vec![];

        let results = sync.sync_to_feishu(&local_tasks, &feishu_tasks);

        assert!(results.is_empty());
    }

    // --- sync_to_feishu: 检测飞书端已删除 ---

    #[test]
    fn test_sync_to_feishu_detects_remote_deleted() {
        let mut sync = TaskSync::new();
        let local_tasks = vec![make_task("t1", "Task", TaskStatus::Open, "2026-01-01T00:00:00Z")];
        let feishu_tasks: Vec<FeishuTask> = vec![]; // 飞书端无此任务

        let results = sync.sync_to_feishu(&local_tasks, &feishu_tasks);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].conflict, Some(ConflictType::DeletedOnOneSide));
    }

    // --- sync_to_feishu: 标记需要同步 ---

    #[test]
    fn test_sync_to_feishu_marks_for_sync() {
        let mut sync = TaskSync::new();
        let local_tasks = vec![make_task("t1", "Task", TaskStatus::Open, "2026-01-02T00:00:00Z")];
        let feishu_tasks = vec![make_feishu_task("feishu-t1", "Task", "todo", "2026-01-01T00:00:00Z")];

        let results = sync.sync_to_feishu(&local_tasks, &feishu_tasks);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].action, SyncAction::Updated);
        assert_eq!(results[0].direction, SyncDirection::ObsidianToFeishu);
    }

    // --- sync_to_feishu: 跳过未变更 ---

    #[test]
    fn test_sync_to_feishu_skips_unchanged() {
        let mut sync = TaskSync::new();
        let local_tasks = vec![make_task("t1", "Task", TaskStatus::Open, "2026-01-01T00:00:00Z")];
        let feishu_tasks = vec![make_feishu_task("feishu-t1", "Task", "todo", "2026-01-01T00:00:00Z")];

        let results = sync.sync_to_feishu(&local_tasks, &feishu_tasks);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].action, SyncAction::Skipped);
    }

    // --- 冲突检测 ---

    #[test]
    fn test_detect_conflicts_both_modified() {
        let sync = TaskSync::new();
        let local = make_task("t1", "Local Title", TaskStatus::Open, "2026-01-02T00:00:00Z");
        let remote = make_feishu_task("f1", "Remote Title", "in_progress", "2026-01-03T00:00:00Z");

        let conflict = sync.detect_conflicts(&local, &remote);
        assert_eq!(conflict, Some(ConflictType::BothModified));
    }

    #[test]
    fn test_detect_conflicts_status_mismatch() {
        let sync = TaskSync::new();
        let local = make_task("t1", "Same Title", TaskStatus::Open, "2026-01-01T00:00:00Z");
        let remote = make_feishu_task("f1", "Same Title", "done", "2026-01-01T00:00:00Z");

        let conflict = sync.detect_conflicts(&local, &remote);
        // 标题相同但状态不同，且时间戳相同 → StatusMismatch
        assert_eq!(conflict, Some(ConflictType::StatusMismatch));
    }

    #[test]
    fn test_detect_conflicts_no_conflict() {
        let sync = TaskSync::new();
        let local = make_task("t1", "Same Title", TaskStatus::Open, "2026-01-01T00:00:00Z");
        let remote = make_feishu_task("f1", "Same Title", "todo", "2026-01-01T00:00:00Z");

        let conflict = sync.detect_conflicts(&local, &remote);
        assert!(conflict.is_none());
    }

    // --- 冲突解决 ---

    #[test]
    fn test_resolve_conflict_keep_local() {
        let mut sync = TaskSync::new();
        let local = make_task("t1", "Local", TaskStatus::Open, "2026-01-02T00:00:00Z");
        let remote = make_feishu_task("f1", "Remote", "in_progress", "2026-01-03T00:00:00Z");

        let result = sync.resolve_conflict(&local, &remote, Resolution::KeepLocal);

        assert_eq!(result.action, SyncAction::ConflictResolved);
        assert!(result.conflict.is_some());
    }

    #[test]
    fn test_resolve_conflict_keep_remote() {
        let mut sync = TaskSync::new();
        let local = make_task("t1", "Local", TaskStatus::Open, "2026-01-02T00:00:00Z");
        let remote = make_feishu_task("f1", "Remote", "in_progress", "2026-01-03T00:00:00Z");

        let result = sync.resolve_conflict(&local, &remote, Resolution::KeepRemote);

        assert_eq!(result.action, SyncAction::ConflictResolved);
        assert!(result.conflict.is_some());
    }

    #[test]
    fn test_resolve_conflict_timestamp_priority() {
        let mut sync = TaskSync::new();
        let local = make_task("t1", "Local", TaskStatus::Open, "2026-01-02T00:00:00Z");
        let remote = make_feishu_task("f1", "Remote", "in_progress", "2026-01-03T00:00:00Z");

        let result = sync.resolve_conflict(&local, &remote, Resolution::TimestampPriority);

        assert_eq!(result.action, SyncAction::ConflictResolved);
    }

    // --- 同步日志 ---

    #[test]
    fn test_sync_log_accumulates() {
        let mut sync = TaskSync::new();
        let feishu_tasks = vec![
            make_feishu_task("f1", "Task 1", "todo", "2026-01-01T00:00:00Z"),
            make_feishu_task("f2", "Task 2", "todo", "2026-01-01T00:00:00Z"),
        ];
        let local_tasks: Vec<Task> = vec![];

        sync.sync_from_feishu(&feishu_tasks, &local_tasks);

        assert_eq!(sync.log().len(), 2);
        assert_eq!(sync.log()[0].task_id, "f1");
        assert_eq!(sync.log()[1].task_id, "f2");
    }

    #[test]
    fn test_sync_log_has_timestamp() {
        let mut sync = TaskSync::new();
        let feishu_tasks = vec![make_feishu_task("f1", "Task", "todo", "2026-01-01T00:00:00Z")];
        let local_tasks: Vec<Task> = vec![];

        sync.sync_from_feishu(&feishu_tasks, &local_tasks);

        let entry = &sync.log()[0];
        assert!(!entry.timestamp.is_empty());
    }

    // --- 多任务批量同步 ---

    #[test]
    fn test_batch_sync_mixed_results() {
        let mut sync = TaskSync::new();
        let feishu_tasks = vec![
            make_feishu_task("f-new", "New Task", "todo", "2026-01-01T00:00:00Z"),
            make_feishu_task("feishu-t-same", "Same Task", "todo", "2026-01-01T00:00:00Z"),
            make_feishu_task("feishu-t-conflict", "Remote Changed", "in_progress", "2026-01-03T00:00:00Z"),
        ];
        let local_tasks = vec![
            make_task("t-same", "Same Task", TaskStatus::Open, "2026-01-01T00:00:00Z"),
            make_task("t-conflict", "Local Changed", TaskStatus::Open, "2026-01-02T00:00:00Z"),
        ];

        let results = sync.sync_from_feishu(&feishu_tasks, &local_tasks);

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].action, SyncAction::Created);
        assert_eq!(results[1].action, SyncAction::Skipped);
        assert_eq!(results[2].action, SyncAction::ConflictResolved);
    }
}
