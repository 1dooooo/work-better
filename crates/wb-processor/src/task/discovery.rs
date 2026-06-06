//! 任务自动发现 —— 统一发现入口
//!
//! 从会议、消息、邮件、文档中提取候选任务，经用户确认后创建正式任务。

use chrono::Utc;
use uuid::Uuid;
use wb_core::error::Result;

use super::discovery_confirm::ConfirmationFlow;
use super::discovery_email;
use super::discovery_meeting;
use super::discovery_message;
use super::model::{TaskPriority, TaskSource};

/// 待确认任务
#[derive(Debug, Clone)]
pub struct PendingTask {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub source: TaskSource,
    pub priority: TaskPriority,
    pub due_date: Option<String>,
    /// 触发发现的原始文本
    pub origin_text: String,
    pub created_at: String,
}

impl PendingTask {
    pub fn new(
        title: &str,
        description: Option<&str>,
        source: TaskSource,
        priority: TaskPriority,
        due_date: Option<String>,
        origin_text: &str,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            source,
            priority,
            due_date,
            origin_text: origin_text.to_string(),
            created_at: Utc::now().to_rfc3339(),
        }
    }
}

/// 任务发现器
///
/// 统一入口：从不同来源提取候选任务，管理确认流。
pub struct TaskDiscovery {
    confirm_flow: ConfirmationFlow,
}

impl TaskDiscovery {
    pub fn new() -> Self {
        Self {
            confirm_flow: ConfirmationFlow::new(),
        }
    }

    /// 从会议纪要中发现任务
    pub fn discover_from_meeting(&mut self, text: &str) -> Vec<PendingTask> {
        let tasks = discovery_meeting::discover_from_meeting(text);
        self.confirm_flow.add_batch(tasks.clone());
        tasks
    }

    /// 从聊天消息中发现任务
    pub fn discover_from_message(&mut self, text: &str) -> Vec<PendingTask> {
        let tasks = discovery_message::discover_from_message(text);
        self.confirm_flow.add_batch(tasks.clone());
        tasks
    }

    /// 从邮件中发现任务
    pub fn discover_from_email(&mut self, text: &str) -> Vec<PendingTask> {
        let tasks = discovery_email::discover_from_email(text);
        self.confirm_flow.add_batch(tasks.clone());
        tasks
    }

    /// 确认任务（用户确认后返回，供 TaskManager 创建正式任务）
    pub fn confirm(&mut self, pending_id: &str) -> Result<PendingTask> {
        self.confirm_flow.confirm(pending_id)
    }

    /// 拒绝任务
    pub fn reject(&mut self, pending_id: &str) -> Result<()> {
        self.confirm_flow.reject(pending_id)
    }

    /// 获取所有待确认任务
    pub fn pending(&self) -> Vec<&PendingTask> {
        self.confirm_flow.pending()
    }

    /// 待确认任务数量
    pub fn pending_count(&self) -> usize {
        self.confirm_flow.pending_count()
    }
}

impl Default for TaskDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_from_meeting() {
        let mut discovery = TaskDiscovery::new();
        let text = "TODO: 完成接口设计\n需要：部署测试环境";
        let tasks = discovery.discover_from_meeting(text);
        assert_eq!(tasks.len(), 2);
        assert_eq!(discovery.pending_count(), 2);
    }

    #[test]
    fn test_discover_from_message() {
        let mut discovery = TaskDiscovery::new();
        let text = "请你帮忙检查一下登录接口";
        let tasks = discovery.discover_from_message(text);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].source, TaskSource::Message);
    }

    #[test]
    fn test_discover_from_email() {
        let mut discovery = TaskDiscovery::new();
        let text = "请确认：API 文档是否完整";
        let tasks = discovery.discover_from_email(text);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].source, TaskSource::Email);
    }

    #[test]
    fn test_confirm_flow() {
        let mut discovery = TaskDiscovery::new();
        let text = "TODO: 修复 bug";
        let tasks = discovery.discover_from_meeting(text);
        let id = tasks[0].id.clone();

        // 确认
        let confirmed = discovery.confirm(&id).unwrap();
        assert_eq!(confirmed.title, "修复 bug");
        assert_eq!(discovery.pending_count(), 0);
    }

    #[test]
    fn test_reject_flow() {
        let mut discovery = TaskDiscovery::new();
        let text = "TODO: 误报的任务";
        let tasks = discovery.discover_from_meeting(text);
        let id = tasks[0].id.clone();

        discovery.reject(&id).unwrap();
        assert_eq!(discovery.pending_count(), 0);
    }

    #[test]
    fn test_confirm_nonexistent() {
        let mut discovery = TaskDiscovery::new();
        assert!(discovery.confirm("bad-id").is_err());
    }

    #[test]
    fn test_reject_nonexistent() {
        let mut discovery = TaskDiscovery::new();
        assert!(discovery.reject("bad-id").is_err());
    }

    #[test]
    fn test_pending_list() {
        let mut discovery = TaskDiscovery::new();
        discovery.discover_from_meeting("TODO: A\nTODO: B");
        discovery.discover_from_message("请你帮忙完成C模块的开发");

        let pending = discovery.pending();
        // 会议 2 个 + 消息 1 个 = 3
        assert_eq!(pending.len(), 3);
    }

    #[test]
    fn test_mixed_sources() {
        let mut discovery = TaskDiscovery::new();
        discovery.discover_from_meeting("待办：会议任务");
        discovery.discover_from_message("请你帮忙聊天任务");
        discovery.discover_from_email("请确认：邮件任务");

        assert_eq!(discovery.pending_count(), 3);

        // 确认一个，拒绝一个，保留一个
        let pending = discovery.pending();
        let ids: Vec<String> = pending.iter().map(|p| p.id.clone()).collect();

        discovery.confirm(&ids[0]).unwrap();
        discovery.reject(&ids[1]).unwrap();

        assert_eq!(discovery.pending_count(), 1);
    }

    #[test]
    fn test_discover_no_match() {
        let mut discovery = TaskDiscovery::new();
        let tasks = discovery.discover_from_meeting("普通文本没有关键词");
        assert!(tasks.is_empty());
        assert_eq!(discovery.pending_count(), 0);
    }
}
