//! 自动发现确认流 —— 管理待确认任务的生命周期

use std::collections::HashMap;

use wb_core::error::{Result, WbError};

use super::discovery::PendingTask;

/// 确认流管理器
///
/// 维护一个 pending → confirmed / rejected 的状态机。
pub struct ConfirmationFlow {
    /// 待确认任务表（id → PendingTask）
    pending: HashMap<String, PendingTask>,
    /// 已拒绝的任务 id（用于去重和审计）
    rejected_ids: Vec<String>,
}

impl ConfirmationFlow {
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
            rejected_ids: Vec::new(),
        }
    }

    /// 添加待确认任务，返回其 id
    pub fn add(&mut self, task: PendingTask) -> String {
        let id = task.id.clone();
        self.pending.insert(id.clone(), task);
        id
    }

    /// 批量添加待确认任务
    pub fn add_batch(&mut self, tasks: Vec<PendingTask>) -> Vec<String> {
        tasks.into_iter().map(|t| self.add(t)).collect()
    }

    /// 确认任务：从 pending 中移除并返回
    pub fn confirm(&mut self, pending_id: &str) -> Result<PendingTask> {
        self.pending
            .remove(pending_id)
            .ok_or_else(|| WbError::NotFound(format!("待确认任务不存在: {}", pending_id)))
    }

    /// 拒绝任务：从 pending 中移除
    pub fn reject(&mut self, pending_id: &str) -> Result<()> {
        if self.pending.remove(pending_id).is_some() {
            self.rejected_ids.push(pending_id.to_string());
            Ok(())
        } else {
            Err(WbError::NotFound(format!("待确认任务不存在: {}", pending_id)))
        }
    }

    /// 获取所有待确认任务
    pub fn pending(&self) -> Vec<&PendingTask> {
        self.pending.values().collect()
    }

    /// 待确认任务数量
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// 已拒绝任务数量
    pub fn rejected_count(&self) -> usize {
        self.rejected_ids.len()
    }

    /// 获取指定待确认任务
    pub fn get(&self, pending_id: &str) -> Option<&PendingTask> {
        self.pending.get(pending_id)
    }
}

impl Default for ConfirmationFlow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::model::{TaskPriority, TaskSource};

    fn make_pending(title: &str) -> PendingTask {
        PendingTask::new(
            title,
            None,
            TaskSource::Meeting,
            TaskPriority::P2,
            None,
            "origin text",
        )
    }

    #[test]
    fn test_add_and_pending() {
        let mut flow = ConfirmationFlow::new();
        let id = flow.add(make_pending("Task 1"));
        assert_eq!(flow.pending_count(), 1);
        assert!(flow.get(&id).is_some());
    }

    #[test]
    fn test_add_batch() {
        let mut flow = ConfirmationFlow::new();
        let tasks = vec![
            make_pending("A"),
            make_pending("B"),
            make_pending("C"),
        ];
        let ids = flow.add_batch(tasks);
        assert_eq!(ids.len(), 3);
        assert_eq!(flow.pending_count(), 3);
    }

    #[test]
    fn test_confirm_removes_from_pending() {
        let mut flow = ConfirmationFlow::new();
        let id = flow.add(make_pending("Confirm me"));
        let confirmed = flow.confirm(&id).unwrap();
        assert_eq!(confirmed.title, "Confirm me");
        assert_eq!(flow.pending_count(), 0);
    }

    #[test]
    fn test_confirm_nonexistent() {
        let mut flow = ConfirmationFlow::new();
        let result = flow.confirm("no-such-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_removes_from_pending() {
        let mut flow = ConfirmationFlow::new();
        let id = flow.add(make_pending("Reject me"));
        flow.reject(&id).unwrap();
        assert_eq!(flow.pending_count(), 0);
        assert_eq!(flow.rejected_count(), 1);
    }

    #[test]
    fn test_reject_nonexistent() {
        let mut flow = ConfirmationFlow::new();
        let result = flow.reject("no-such-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_pending_returns_all() {
        let mut flow = ConfirmationFlow::new();
        flow.add(make_pending("A"));
        flow.add(make_pending("B"));
        let pending = flow.pending();
        assert_eq!(pending.len(), 2);
    }

    #[test]
    fn test_full_lifecycle() {
        let mut flow = ConfirmationFlow::new();
        let id1 = flow.add(make_pending("Will confirm"));
        let id2 = flow.add(make_pending("Will reject"));
        let _id3 = flow.add(make_pending("Still pending"));

        flow.confirm(&id1).unwrap();
        flow.reject(&id2).unwrap();

        assert_eq!(flow.pending_count(), 1);
        assert_eq!(flow.rejected_count(), 1);
        assert!(flow.get(&_id3).is_some());
    }
}
