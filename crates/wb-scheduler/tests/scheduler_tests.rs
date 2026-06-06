use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use wb_scheduler::scheduler::Scheduler;
use wb_scheduler::task::{ScheduledTask, TaskLayer, TaskResult, TaskStatus};

// ---------------------------------------------------------------------------
// Mock tasks
// ---------------------------------------------------------------------------

/// A task that always succeeds immediately.
struct MockSuccessTask {
    id: String,
    call_count: Arc<AtomicU32>,
}

impl MockSuccessTask {
    fn new(id: &str) -> (Self, Arc<AtomicU32>) {
        let count = Arc::new(AtomicU32::new(0));
        (
            Self {
                id: id.to_string(),
                call_count: Arc::clone(&count),
            },
            count,
        )
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

    async fn execute(&self) -> TaskResult {
        self.call_count.fetch_add(1, Ordering::SeqCst);
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

/// A task that always fails.
struct MockFailTask {
    id: String,
    call_count: Arc<AtomicU32>,
}

impl MockFailTask {
    fn new(id: &str) -> (Self, Arc<AtomicU32>) {
        let count = Arc::new(AtomicU32::new(0));
        (
            Self {
                id: id.to_string(),
                call_count: Arc::clone(&count),
            },
            count,
        )
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
        TaskLayer::Processing
    }
    fn cron_expression(&self) -> &str {
        "0 * * * * *"
    }
    fn retry_limit(&self) -> u32 {
        2
    }

    async fn execute(&self) -> TaskResult {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        let now = Utc::now();
        TaskResult {
            task_id: self.id.clone(),
            status: TaskStatus::Failed,
            started_at: now,
            finished_at: now,
            duration_ms: 0,
            summary: "intentional failure".to_string(),
            error: Some("boom".to_string()),
            retry_count: 0,
        }
    }
}

/// A task that sleeps longer than its SLA to trigger timeout.
struct MockSlowTask {
    id: String,
    sleep_ms: u64,
    call_count: Arc<AtomicU32>,
}

impl MockSlowTask {
    fn new(id: &str, sleep_ms: u64) -> (Self, Arc<AtomicU32>) {
        let count = Arc::new(AtomicU32::new(0));
        (
            Self {
                id: id.to_string(),
                sleep_ms,
                call_count: Arc::clone(&count),
            },
            count,
        )
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
        TaskLayer::Processing
    }
    fn cron_expression(&self) -> &str {
        "0 * * * * *"
    }
    fn sla_ms(&self) -> u64 {
        50
    }
    fn retry_limit(&self) -> u32 {
        0
    } // no retries for timeout tests

    async fn execute(&self) -> TaskResult {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(self.sleep_ms)).await;
        let now = Utc::now();
        TaskResult {
            task_id: self.id.clone(),
            status: TaskStatus::Success,
            started_at: now,
            finished_at: now,
            duration_ms: self.sleep_ms,
            summary: "done".to_string(),
            error: None,
            retry_count: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_register_and_list_tasks() {
    let scheduler = Scheduler::new();
    let (task, _) = MockSuccessTask::new("task-1");
    scheduler.register(Arc::new(task)).await;

    let ids = scheduler.list_tasks().await;
    assert_eq!(ids.len(), 1);
    assert!(ids.contains(&"task-1".to_string()));
}

#[tokio::test]
async fn test_unregister_task() {
    let scheduler = Scheduler::new();
    let (task, _) = MockSuccessTask::new("task-1");
    scheduler.register(Arc::new(task)).await;

    assert!(scheduler.unregister("task-1").await);
    assert!(scheduler.list_tasks().await.is_empty());

    // Unregistering non-existent task returns false
    assert!(!scheduler.unregister("task-1").await);
}

#[tokio::test]
async fn test_get_task_info() {
    let scheduler = Scheduler::new();
    let (task, _) = MockSuccessTask::new("info-task");
    scheduler.register(Arc::new(task)).await;

    let info = scheduler.get_task_info("info-task").await;
    assert!(info.is_some());

    let info = info.unwrap();
    assert_eq!(info.id, "info-task");
    assert_eq!(info.name, "MockSuccessTask");
    assert_eq!(info.layer, "Processing");
    assert_eq!(info.sla_ms, 300_000);
    assert_eq!(info.interval_secs, 60);
    assert!(info.last_run.is_none());
    assert!(info.last_status.is_none());
}

#[tokio::test]
async fn test_run_now_success() {
    let scheduler = Scheduler::new();
    let (task, call_count) = MockSuccessTask::new("run-now");
    scheduler.register(Arc::new(task)).await;

    let result = scheduler.run_now("run-now").await;
    assert!(result.is_some());

    let result = result.unwrap();
    assert_eq!(result.status, TaskStatus::Success);
    assert_eq!(result.task_id, "run-now");
    assert_eq!(call_count.load(Ordering::SeqCst), 1);

    // Verify last result is stored
    let stored = scheduler.get_last_result("run-now").await;
    assert!(stored.is_some());
    assert_eq!(stored.unwrap().status, TaskStatus::Success);
}

#[tokio::test]
async fn test_run_now_nonexistent() {
    let scheduler = Scheduler::new();
    let result = scheduler.run_now("nope").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_pause_and_resume() {
    let scheduler = Scheduler::new();

    assert!(!scheduler.is_paused().await);

    scheduler.pause_all().await;
    assert!(scheduler.is_paused().await);

    scheduler.resume_all().await;
    assert!(!scheduler.is_paused().await);
}

#[tokio::test]
async fn test_retry_on_failure() {
    let scheduler = Scheduler::new();
    let (task, call_count) = MockFailTask::new("retry-task");
    scheduler.register(Arc::new(task)).await;

    let result = scheduler.run_now("retry-task").await;
    assert!(result.is_some());

    let result = result.unwrap();
    // Should be Failed (all retries exhausted)
    assert_eq!(result.status, TaskStatus::Failed);
    // retry_limit is 2, so total attempts = 3 (initial + 2 retries)
    assert_eq!(call_count.load(Ordering::SeqCst), 3);
    // Last result should have retry_count == 2
    assert_eq!(result.retry_count, 2);
}

#[tokio::test]
async fn test_timeout_handling() {
    let scheduler = Scheduler::new();
    // Sleep 200ms but SLA is 50ms => should timeout
    let (task, call_count) = MockSlowTask::new("timeout-task", 200);
    scheduler.register(Arc::new(task)).await;

    let result = scheduler.run_now("timeout-task").await;
    assert!(result.is_some());

    let result = result.unwrap();
    assert_eq!(result.status, TaskStatus::Timeout);
    assert!(result.error.is_some());
    // retry_limit is 0, so only 1 attempt
    assert_eq!(call_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_register_with_custom_interval() {
    let scheduler = Scheduler::new();
    let (task, _) = MockSuccessTask::new("custom-interval");
    scheduler.register_with_interval(Arc::new(task), 120).await;

    let info = scheduler.get_task_info("custom-interval").await.unwrap();
    assert_eq!(info.interval_secs, 120);
}

#[tokio::test]
async fn test_scheduler_start_and_stop() {
    let scheduler = Scheduler::new();

    // Start should not panic
    scheduler.start().await;

    // Starting again should be a no-op
    scheduler.start().await;

    // Stop should not panic
    scheduler.stop().await;

    // Stopping again should be a no-op
    scheduler.stop().await;
}

#[tokio::test]
async fn test_task_layer_display() {
    assert_eq!(TaskLayer::Collection.to_string(), "Collection");
    assert_eq!(TaskLayer::Processing.to_string(), "Processing");
    assert_eq!(TaskLayer::Storage.to_string(), "Storage");
    assert_eq!(TaskLayer::Presentation.to_string(), "Presentation");
}
