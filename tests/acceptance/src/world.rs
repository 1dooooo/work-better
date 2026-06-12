use std::collections::HashMap;
use std::path::PathBuf;

use tempfile::TempDir;
use wb_core::event::{Event, EventLog, Source, Confidence, EventType};
use wb_storage::SqliteEventLog;
use wb_collector::manager::CollectorManager;
use wb_scheduler::scheduler::Scheduler;
use wb_core::task::Task;

/// Acceptance test world — holds all mutable state across Given/When/Then steps.
///
/// 增强版本：持有真实的系统实例，而非仅持有字符串状态。
#[derive(cucumber::World)]
pub struct AcceptanceWorld {
    // 真实系统实例
    pub event_log: SqliteEventLog,
    pub collector_manager: CollectorManager,
    pub scheduler: Scheduler,
    pub temp_dir: TempDir,

    // 测试状态
    pub current_event: Option<Event>,
    pub current_task: Option<Task>,
    pub processing_result: Option<String>,
    pub last_error: Option<String>,
    pub model_used: Option<String>,

    // 兼容旧接口
    pub event_type: Option<String>,
    pub event_content: Option<String>,
    pub confidence: Option<f64>,
    pub priority: Option<String>,
    pub task_status: Option<String>,
    pub task_source: Option<String>,
    pub completed_at: Option<String>,
    pub storage_path: Option<PathBuf>,
    pub vault_path: Option<PathBuf>,
    pub stored_records: Vec<String>,
    pub budget_remaining: Option<f64>,
    pub error: Option<String>,
    pub notifications: Vec<String>,
    pub state: HashMap<String, String>,
}

impl Default for AcceptanceWorld {
    fn default() -> Self {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let event_log = SqliteEventLog::new_in_memory().expect("failed to create event log");
        let collector_manager = CollectorManager::new();
        let scheduler = Scheduler::new();

        Self {
            // 真实系统实例
            event_log,
            collector_manager,
            scheduler,
            temp_dir,

            // 测试状态
            current_event: None,
            current_task: None,
            processing_result: None,
            last_error: None,
            model_used: None,

            // 兼容旧接口
            event_type: None,
            event_content: None,
            confidence: None,
            priority: None,
            task_status: None,
            task_source: None,
            completed_at: None,
            storage_path: None,
            vault_path: None,
            stored_records: Vec::new(),
            budget_remaining: None,
            error: None,
            notifications: Vec::new(),
            state: HashMap::new(),
        }
    }
}

impl std::fmt::Debug for AcceptanceWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AcceptanceWorld")
            .field("current_event", &self.current_event)
            .field("current_task", &self.current_task)
            .field("processing_result", &self.processing_result)
            .field("last_error", &self.last_error)
            .field("model_used", &self.model_used)
            .field("event_type", &self.event_type)
            .field("state", &self.state)
            .finish()
    }
}

impl AcceptanceWorld {
    /// 创建测试事件
    pub fn create_event(&self, source: Source, event_type: EventType, content: &str) -> Event {
        Event::new(
            source,
            Confidence::High,
            event_type,
            serde_json::json!({"text": content}),
            format!(r#"{{"raw": "{}"}}"#, content),
        )
    }

    /// 追加事件到 EventLog
    pub async fn append_event(&mut self, event: Event) -> Result<(), String> {
        self.event_log.append(&event).await.map_err(|e| e.to_string())?;
        self.current_event = Some(event);
        Ok(())
    }

    /// 获取未处理事件数量
    pub async fn get_unprocessed_count(&self) -> Result<usize, String> {
        let events = self.event_log.get_unprocessed(None).await.map_err(|e| e.to_string())?;
        Ok(events.len())
    }

    /// 标记事件为已处理
    pub async fn mark_event_processed(&mut self, event_id: &str) -> Result<(), String> {
        self.event_log.mark_processed(event_id).await.map_err(|e| e.to_string())?;
        Ok(())
    }
}
