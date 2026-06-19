//! 任务自动发现 —— 统一发现入口（AI 驱动）
//!
//! 所有内容理解统一走 AI 模型，不使用关键词匹配。
//! AI 不可用时返回空（不创建任务），由上层决定如何处理。

use chrono::Utc;
use uuid::Uuid;
use wb_core::error::Result;
use wb_core::event::Source;
use wb_ai::{ModelAdapter, TaskContext};

use super::discovery_ai;
use super::discovery_confirm::ConfirmationFlow;
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
/// 统一入口：所有内容理解走 AI 模型，管理确认流。
pub struct TaskDiscovery {
    confirm_flow: ConfirmationFlow,
}

impl TaskDiscovery {
    pub fn new() -> Self {
        Self {
            confirm_flow: ConfirmationFlow::new(),
        }
    }

    /// AI 驱动的任务发现（统一入口）
    ///
    /// 所有来源（消息、邮件、会议、手动捕获）统一走此路径。
    /// 自动将当前 pending 任务列表作为上下文传给 AI，让 AI 能判断
    /// 新消息是"新任务"还是"已有任务的状态更新"。
    ///
    /// AI 不可用时返回空，不降级到关键词匹配。
    pub async fn discover_with_ai(
        &mut self,
        text: &str,
        adapter: &dyn ModelAdapter,
        source: Source,
    ) -> Vec<PendingTask> {
        let existing_tasks: Vec<TaskContext> = self
            .confirm_flow
            .pending()
            .iter()
            .map(|p| TaskContext {
                id: p.id.clone(),
                title: p.title.clone(),
                status: "Pending".to_string(),
            })
            .collect();

        eprintln!(
            "[discovery] discover_with_ai: text={}, pending_count={}, existing_tasks={:?}",
            text,
            existing_tasks.len(),
            existing_tasks.iter().map(|t| t.title.as_str()).collect::<Vec<_>>()
        );

        let tasks = discovery_ai::discover_with_ai(text, adapter, source, &existing_tasks).await;

        eprintln!(
            "[discovery] discover_with_ai result: discovered={}, is_status_update_check_done",
            tasks.len()
        );

        self.confirm_flow.add_batch(tasks.clone());

        eprintln!(
            "[discovery] after add_batch: pending_count={}",
            self.confirm_flow.pending_count()
        );

        tasks
    }

    /// 确认任务
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

    /// 直接添加一个待确认任务
    pub fn add_pending(&mut self, task: PendingTask) {
        self.confirm_flow.add(task);
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
    use wb_ai::{Extraction, MockAdapter};

    fn make_adapter(title: &str, confidence: f64) -> MockAdapter {
        MockAdapter::new().with_extraction(Extraction {
            title: title.to_string(),
            summary: String::new(),
            detail: String::new(),
            people: vec![],
            tags: vec![],
            project: None,
            due_date: None,
            confidence,
            is_status_update: false,
            related_task_id: None,
        })
    }

    #[tokio::test]
    async fn test_discover_with_ai_returns_ai_result() {
        let mut discovery = TaskDiscovery::new();
        let adapter = make_adapter("完成报告", 0.9);

        let tasks = discovery
            .discover_with_ai("请帮忙明天完成报告", &adapter, Source::FeishuMessage)
            .await;
        assert_eq!(tasks.len(), 1, "AI 应发现 1 个任务");
        assert_eq!(tasks[0].title, "完成报告");
        assert_eq!(discovery.pending_count(), 1);
    }

    #[tokio::test]
    async fn test_discover_with_ai_no_match() {
        let mut discovery = TaskDiscovery::new();
        let adapter = MockAdapter::new().with_extraction(Extraction {
            title: String::new(),
            summary: String::new(),
            detail: String::new(),
            people: vec![],
            tags: vec![],
            project: None,
            due_date: None,
            confidence: 0.2,
            is_status_update: false,
            related_task_id: None,
        });

        let tasks = discovery
            .discover_with_ai("今天天气真好", &adapter, Source::FeishuMessage)
            .await;
        assert!(tasks.is_empty(), "非任务文本应返回空");
        assert_eq!(discovery.pending_count(), 0);
    }

    #[tokio::test]
    async fn test_discover_with_ai_low_confidence_returns_empty() {
        let mut discovery = TaskDiscovery::new();
        let adapter = make_adapter("可能的任务", 0.2);

        let tasks = discovery
            .discover_with_ai("请你帮忙检查 API", &adapter, Source::FeishuMessage)
            .await;
        assert!(tasks.is_empty(), "低置信度应返回空，不降级");
    }

    #[tokio::test]
    async fn test_discover_with_ai_status_update_no_duplicate() {
        let adapter1 = make_adapter("发邮件给lily", 0.9);

        let mut discovery = TaskDiscovery::new();
        let tasks1 = discovery
            .discover_with_ai("我今天要发邮件给lily", &adapter1, Source::FeishuMessage)
            .await;
        assert_eq!(tasks1.len(), 1, "第一条消息应发现任务");
        assert_eq!(discovery.pending_count(), 1);

        let adapter2 = MockAdapter::new().with_extraction(Extraction {
            title: String::new(),
            summary: String::new(),
            detail: String::new(),
            people: vec![],
            tags: vec![],
            project: None,
            due_date: None,
            confidence: 0.9,
            is_status_update: true,
            related_task_id: Some(tasks1[0].id.clone()),
        });

        let tasks2 = discovery
            .discover_with_ai("给Lily的邮件已经发送了", &adapter2, Source::FeishuMessage)
            .await;
        assert!(tasks2.is_empty(), "状态更新不应创建新任务");
        assert_eq!(discovery.pending_count(), 1);
    }

    #[tokio::test]
    async fn test_confirm_flow() {
        let adapter = make_adapter("修复 bug", 0.9);
        let mut discovery = TaskDiscovery::new();
        let tasks = discovery
            .discover_with_ai("TODO: 修复 bug", &adapter, Source::FeishuMessage)
            .await;
        let id = tasks[0].id.clone();

        let confirmed = discovery.confirm(&id).unwrap();
        assert_eq!(confirmed.title, "修复 bug");
        assert_eq!(discovery.pending_count(), 0);
    }

    #[tokio::test]
    async fn test_reject_flow() {
        let adapter = make_adapter("误报的任务", 0.9);
        let mut discovery = TaskDiscovery::new();
        let tasks = discovery
            .discover_with_ai("TODO: 误报的任务", &adapter, Source::FeishuMessage)
            .await;
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
}
