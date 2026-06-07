//! B5: Tauri Scheduler Commands Integration Tests
//!
//! Tests the Scheduler logic that the Tauri scheduler commands use.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use wb_scheduler::scheduler::Scheduler;
use wb_scheduler::task::{ScheduledTask, TaskLayer, TaskResult, TaskStatus};

// ---------------------------------------------------------------------------
// Mock tasks
// ---------------------------------------------------------------------------

struct MockSuccessTask {
    id: String,
}

impl MockSuccessTask {
    fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

#[async_trait]
impl ScheduledTask for MockSuccessTask {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> &str {
        "MockSuccessTask"
    }
    fn layer(&self) -> TaskLayer {
        TaskLayer::Processing
    }
    fn cron_expression(&self) -> &str {
        "0 * * * * *"
    }
    fn sla_ms(&self) -> u64 {
        5000
    }
    fn retry_limit(&self) -> u32 {
        1
    }

    async fn execute(&self) -> TaskResult {
        let now = Utc::now();
        TaskResult {
            task_id: self.id.clone(),
            status: TaskStatus::Success,
            started_at: now,
            finished_at: now,
            duration_ms: 0,
            summary: "ok".to_string(),
            error: None,
            retry_count: 0,
        }
    }
}

struct MockFailTask {
    id: String,
}

impl MockFailTask {
    fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

#[async_trait]
impl ScheduledTask for MockFailTask {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> &str {
        "MockFailTask"
    }
    fn layer(&self) -> TaskLayer {
        TaskLayer::Collection
    }
    fn cron_expression(&self) -> &str {
        "0 * * * * *"
    }
    fn sla_ms(&self) -> u64 {
        5000
    }
    fn retry_limit(&self) -> u32 {
        0
    }

    async fn execute(&self) -> TaskResult {
        let now = Utc::now();
        TaskResult {
            task_id: self.id.clone(),
            status: TaskStatus::Failed,
            started_at: now,
            finished_at: now,
            duration_ms: 0,
            summary: "fail".to_string(),
            error: Some("intentional".to_string()),
            retry_count: 0,
        }
    }
}

struct MockSlowTask {
    id: String,
}

impl MockSlowTask {
    fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

#[async_trait]
impl ScheduledTask for MockSlowTask {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> &str {
        "MockSlowTask"
    }
    fn layer(&self) -> TaskLayer {
        TaskLayer::Storage
    }
    fn cron_expression(&self) -> &str {
        "0 * * * * *"
    }
    fn sla_ms(&self) -> u64 {
        50
    }
    fn retry_limit(&self) -> u32 {
        0
    }

    async fn execute(&self) -> TaskResult {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let now = Utc::now();
        TaskResult {
            task_id: self.id.clone(),
            status: TaskStatus::Success,
            started_at: now,
            finished_at: now,
            duration_ms: 200,
            summary: "should not reach".to_string(),
            error: None,
            retry_count: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// B5-01: List tasks
// ---------------------------------------------------------------------------

/// Mirrors Tauri `list_scheduled_tasks` command
#[tokio::test]
async fn b5_01_list_tasks_empty() {
    let scheduler = Scheduler::new();
    let ids = scheduler.list_tasks().await;
    assert!(ids.is_empty());
}

#[tokio::test]
async fn b5_01_list_tasks_after_register() {
    let scheduler = Scheduler::new();
    scheduler
        .register(Arc::new(MockSuccessTask::new("t1")))
        .await;
    scheduler
        .register(Arc::new(MockSuccessTask::new("t2")))
        .await;

    let ids = scheduler.list_tasks().await;
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"t1".to_string()));
    assert!(ids.contains(&"t2".to_string()));
}

#[tokio::test]
async fn b5_01_get_task_info() {
    let scheduler = Scheduler::new();
    scheduler
        .register(Arc::new(MockSuccessTask::new("info-task")))
        .await;

    let info = scheduler.get_task_info("info-task").await;
    assert!(info.is_some());

    let info = info.unwrap();
    assert_eq!(info.id, "info-task");
    assert_eq!(info.name, "MockSuccessTask");
    assert_eq!(info.layer, "Processing");
    assert_eq!(info.sla_ms, 5000);
    assert!(info.last_run.is_none());
    assert!(info.last_status.is_none());
}

// ---------------------------------------------------------------------------
// B5-02: Pause/resume scheduler
// ---------------------------------------------------------------------------

/// Mirrors Tauri `pause_scheduler`/`resume_scheduler` commands
#[tokio::test]
async fn b5_02_pause_resume() {
    let scheduler = Scheduler::new();

    assert!(!scheduler.is_paused().await);

    scheduler.pause_all().await;
    assert!(scheduler.is_paused().await);

    scheduler.resume_all().await;
    assert!(!scheduler.is_paused().await);
}

#[tokio::test]
async fn b5_02_is_scheduler_paused_default() {
    let scheduler = Scheduler::new();
    assert!(
        !scheduler.is_paused().await,
        "Scheduler should not be paused by default"
    );
}

// ---------------------------------------------------------------------------
// B5-03: Task execution
// ---------------------------------------------------------------------------

#[tokio::test]
async fn b5_03_run_now_success() {
    let scheduler = Scheduler::new();
    scheduler
        .register(Arc::new(MockSuccessTask::new("run-now")))
        .await;

    let result = scheduler.run_now("run-now").await;
    assert!(result.is_some());
    assert_eq!(result.unwrap().status, TaskStatus::Success);
}

#[tokio::test]
async fn b5_03_run_now_failure() {
    let scheduler = Scheduler::new();
    scheduler
        .register(Arc::new(MockFailTask::new("fail-task")))
        .await;

    let result = scheduler.run_now("fail-task").await;
    assert!(result.is_some());
    assert_eq!(result.unwrap().status, TaskStatus::Failed);
}

#[tokio::test]
async fn b5_03_run_now_timeout() {
    let scheduler = Scheduler::new();
    scheduler
        .register(Arc::new(MockSlowTask::new("slow-task")))
        .await;

    let result = scheduler.run_now("slow-task").await;
    assert!(result.is_some());
    assert_eq!(result.unwrap().status, TaskStatus::Timeout);
}

#[tokio::test]
async fn b5_03_run_now_nonexistent() {
    let scheduler = Scheduler::new();
    assert!(scheduler.run_now("nope").await.is_none());
}

#[tokio::test]
async fn b5_03_run_now_stores_last_result() {
    let scheduler = Scheduler::new();
    scheduler
        .register(Arc::new(MockSuccessTask::new("stored")))
        .await;

    scheduler.run_now("stored").await;

    let stored = scheduler.get_last_result("stored").await;
    assert!(stored.is_some());
    assert_eq!(stored.unwrap().status, TaskStatus::Success);
}

// ---------------------------------------------------------------------------
// B5-04: Task registration with custom config
// ---------------------------------------------------------------------------

#[tokio::test]
async fn b5_04_register_with_custom_interval() {
    let scheduler = Scheduler::new();
    scheduler
        .register_with_interval(Arc::new(MockSuccessTask::new("custom")), 120)
        .await;

    let info = scheduler.get_task_info("custom").await.unwrap();
    assert_eq!(info.interval_secs, 120);
}

#[tokio::test]
async fn b5_04_unregister_task() {
    let scheduler = Scheduler::new();
    scheduler
        .register(Arc::new(MockSuccessTask::new("temp")))
        .await;

    assert!(scheduler.unregister("temp").await);
    assert!(scheduler.list_tasks().await.is_empty());
    assert!(!scheduler.unregister("temp").await);
}

#[tokio::test]
async fn b5_04_start_stop_lifecycle() {
    let scheduler = Scheduler::new();

    // Start should not panic
    scheduler.start().await;
    // Starting again should be no-op
    scheduler.start().await;
    // Stop should not panic
    scheduler.stop().await;
    // Stopping again should be no-op
    scheduler.stop().await;
}

#[tokio::test]
async fn b5_04_budget_management() {
    let scheduler = Scheduler::new();
    assert_eq!(scheduler.budget().await, 100);

    scheduler.set_budget(42).await;
    assert_eq!(scheduler.budget().await, 42);
}
