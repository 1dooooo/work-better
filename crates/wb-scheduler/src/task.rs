use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Success,
    Failed,
    Timeout,
    Aborted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub task_name: String,
    pub status: TaskStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: chrono::DateTime<chrono::Utc>,
    pub duration_ms: u64,
    pub summary: String,
    pub error: Option<String>,
    pub retry_count: u32,
}

#[derive(Debug, Clone)]
pub enum TaskLayer {
    Collection,
    Processing,
    Storage,
    Presentation,
}

impl std::fmt::Display for TaskLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskLayer::Collection => write!(f, "Collection"),
            TaskLayer::Processing => write!(f, "Processing"),
            TaskLayer::Storage => write!(f, "Storage"),
            TaskLayer::Presentation => write!(f, "Presentation"),
        }
    }
}

#[async_trait]
pub trait ScheduledTask: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn layer(&self) -> TaskLayer;
    fn cron_expression(&self) -> &str;
    fn sla_ms(&self) -> u64 {
        300_000
    } // default 5 minutes
    fn retry_limit(&self) -> u32 {
        3
    }

    async fn execute(&self) -> TaskResult;
}
