//! 任务自动发现 —— 统一发现入口
//!
//! 从会议、消息、邮件、文档中提取候选任务，经用户确认后创建正式任务。

use chrono::Utc;
use uuid::Uuid;
use wb_core::error::Result;
use wb_core::event::Source;
use wb_ai::{ModelAdapter, TaskContext};

use super::discovery_ai;
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

    /// 从聊天消息中发现任务（AI 驱动版本）
    ///
    /// AI 优先：调用 adapter.extract() 分析消息内容，识别潜在任务。
    /// AI 返回有效候选则使用 AI 结果，否则降级到关键词匹配。
    ///
    /// 自动将当前 pending 任务列表作为上下文传给 AI，让 AI 能判断
    /// 新消息是"新任务"还是"已有任务的状态更新"。
    ///
    /// # Arguments
    /// - `text`: 待分析文本
    /// - `adapter`: AI 模型适配器
    /// - `source`: 事件来源类型（M5: 参数化替代硬编码）
    pub async fn discover_with_ai(
        &mut self,
        text: &str,
        adapter: &dyn ModelAdapter,
        source: Source,
    ) -> Vec<PendingTask> {
        // 从当前 pending 列表构建任务上下文
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

        let tasks = discovery_ai::discover_with_ai(text, adapter, source, &existing_tasks).await;
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

    /// 直接添加一个待确认任务（供 AI extract 结果使用）
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

    // ─── AI 驱动的 Task Discovery 测试 ─────────────────────────────

    #[tokio::test]
    async fn test_discover_with_ai_returns_ai_result() {
        use wb_ai::{Extraction, MockAdapter};
        use wb_core::event::Source;

        let mut discovery = TaskDiscovery::new();
        let adapter = MockAdapter::new().with_extraction(Extraction {
            title: "完成报告".to_string(),
            summary: "明天完成报告".to_string(),
            detail: String::new(),
            people: vec![],
            tags: vec![],
            project: None,
            due_date: Some("明天".to_string()),
            confidence: 0.9,
            is_status_update: false,
            related_task_id: None,
        });

        let tasks = discovery
            .discover_with_ai("请帮忙明天完成报告", &adapter, Source::FeishuMessage)
            .await;
        assert_eq!(tasks.len(), 1, "AI 应发现 1 个任务");
        assert_eq!(tasks[0].title, "完成报告");
        assert_eq!(discovery.pending_count(), 1, "应添加到 pending 列表");
    }

    #[tokio::test]
    async fn test_discover_with_ai_fallback_to_keywords() {
        use wb_ai::MockAdapter;
        use wb_core::event::Source;

        // MockAdapter 默认返回高置信度结果，但标题是 "Mock Title"
        // 我们使用一个会返回低置信度的 adapter 来触发 fallback
        let adapter = MockAdapter::new().with_extraction(wb_ai::Extraction {
            title: String::new(), // 空标题 → AI 认为不是任务
            summary: String::new(),
            detail: String::new(),
            people: vec![],
            tags: vec![],
            project: None,
            due_date: None,
            confidence: 0.2, // 低置信度
            is_status_update: false,
            related_task_id: None,
        });

        let mut discovery = TaskDiscovery::new();
        let tasks = discovery
            .discover_with_ai("请你帮忙检查一下登录接口", &adapter, Source::FeishuMessage)
            .await;
        // AI 返回空 → 降级到关键词匹配 → 应发现任务
        assert_eq!(tasks.len(), 1, "应降级到关键词匹配发现任务");
        assert_eq!(discovery.pending_count(), 1);
    }

    #[tokio::test]
    async fn test_discover_with_ai_no_match() {
        use wb_ai::{Extraction, MockAdapter};
        use wb_core::event::Source;

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

        let mut discovery = TaskDiscovery::new();
        let tasks = discovery
            .discover_with_ai("今天天气真好", &adapter, Source::FeishuMessage)
            .await;
        // AI 无结果 + 关键词无匹配 → 空
        assert!(tasks.is_empty(), "非任务文本应返回空");
        assert_eq!(discovery.pending_count(), 0);
    }

    // ─── 状态更新检测测试（端到端） ─────────────────────────────

    #[tokio::test]
    async fn test_discover_with_ai_status_update_no_duplicate() {
        use wb_ai::{Extraction, MockAdapter};
        use wb_core::event::Source;

        // 模拟第一条消息：发现任务
        let adapter1 = MockAdapter::new().with_extraction(Extraction {
            title: "发邮件给lily".to_string(),
            summary: "给lily发邮件".to_string(),
            detail: String::new(),
            people: vec!["lily".to_string()],
            tags: vec![],
            project: None,
            due_date: None,
            confidence: 0.9,
            is_status_update: false,
            related_task_id: None,
        });

        let mut discovery = TaskDiscovery::new();
        let tasks1 = discovery
            .discover_with_ai("我今天要发邮件给lily", &adapter1, Source::FeishuMessage)
            .await;
        assert_eq!(tasks1.len(), 1, "第一条消息应发现任务");
        assert_eq!(discovery.pending_count(), 1);

        // 模拟第二条消息：AI 判断为状态更新
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
        assert_eq!(discovery.pending_count(), 1, "pending 数量不应增加");
    }
}
