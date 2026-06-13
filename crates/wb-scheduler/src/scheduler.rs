use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use wb_core::task::Priority;

use crate::cron;
use crate::dependency::DependencyGraph;
use crate::resource;
use crate::task::{ScheduledTask, TaskResult, TaskStatus};

/// 任务完成回调类型
///
/// 每次任务执行完成后（含重试），调度器会调用此回调。
/// 回调在调度器的异步上下文中执行，不应阻塞太久。
pub type OnTaskComplete = Arc<dyn Fn(&TaskResult) + Send + Sync>;

/// Runtime state for a registered task.
struct TaskState {
    task: Arc<dyn ScheduledTask>,
    last_run: Option<DateTime<Utc>>,
    last_status: Option<TaskStatus>,
    last_result: Option<TaskResult>,
    interval_secs: u64,
    priority: Priority,
}

/// Information about a registered task (returned by `get_task_info`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub layer: String,
    pub cron: String,
    pub sla_ms: u64,
    pub interval_secs: u64,
    pub last_run: Option<DateTime<Utc>>,
    pub last_status: Option<TaskStatus>,
}

/// Shared state between the Scheduler and the background loop.
struct SharedState {
    tasks: RwLock<HashMap<String, TaskState>>,
    paused: RwLock<bool>,
    dependency_graph: RwLock<DependencyGraph>,
    budget: RwLock<u32>,
    on_task_complete: RwLock<Option<OnTaskComplete>>,
}

/// The central scheduler that manages and executes scheduled tasks.
pub struct Scheduler {
    state: Arc<SharedState>,
    loop_handle: RwLock<Option<JoinHandle<()>>>,
}

impl Scheduler {
    /// Create a new empty scheduler.
    pub fn new() -> Self {
        Self {
            state: Arc::new(SharedState {
                tasks: RwLock::new(HashMap::new()),
                paused: RwLock::new(false),
                dependency_graph: RwLock::new(DependencyGraph::new()),
                budget: RwLock::new(100),
                on_task_complete: RwLock::new(None),
            }),
            loop_handle: RwLock::new(None),
        }
    }

    /// Set a callback that fires after each task execution (including retries).
    ///
    /// The callback receives the final `TaskResult` and is called from the
    /// scheduler's async context. Keep the callback lightweight to avoid
    /// blocking subsequent task executions.
    pub async fn set_on_complete(&self, callback: OnTaskComplete) {
        let mut guard = self.state.on_task_complete.write().await;
        *guard = Some(callback);
    }

    /// Register a task with a default 60-second execution interval.
    pub async fn register(&self, task: Arc<dyn ScheduledTask>) {
        self.register_with_interval(task, 60).await;
    }

    /// Register a task with a custom execution interval in seconds.
    pub async fn register_with_interval(&self, task: Arc<dyn ScheduledTask>, interval_secs: u64) {
        let id = task.id().to_string();
        let state = TaskState {
            task,
            last_run: None,
            last_status: None,
            last_result: None,
            interval_secs,
            priority: Priority::P2,
        };
        let mut tasks = self.state.tasks.write().await;
        tasks.insert(id, state);
    }

    /// Register a task with a custom interval and priority.
    pub async fn register_with_priority(
        &self,
        task: Arc<dyn ScheduledTask>,
        interval_secs: u64,
        priority: Priority,
    ) {
        let id = task.id().to_string();
        let state = TaskState {
            task,
            last_run: None,
            last_status: None,
            last_result: None,
            interval_secs,
            priority,
        };
        let mut tasks = self.state.tasks.write().await;
        tasks.insert(id, state);
    }

    /// Register a task with dependencies and an optional priority.
    pub async fn register_with_deps(
        &self,
        task: Arc<dyn ScheduledTask>,
        interval_secs: u64,
        depends_on: Vec<&str>,
        priority: Priority,
    ) {
        let id = task.id().to_string();
        {
            let mut graph = self.state.dependency_graph.write().await;
            graph.add_task(&id, depends_on);
        }
        self.register_with_priority(task, interval_secs, priority)
            .await;
    }

    /// Set the resource budget (controls whether low-priority tasks are deferred).
    pub async fn set_budget(&self, budget: u32) {
        let mut b = self.state.budget.write().await;
        *b = budget;
    }

    /// Get the current resource budget.
    pub async fn budget(&self) -> u32 {
        let b = self.state.budget.read().await;
        *b
    }

    /// Unregister a task by id. Returns `true` if the task existed.
    pub async fn unregister(&self, id: &str) -> bool {
        let mut tasks = self.state.tasks.write().await;
        tasks.remove(id).is_some()
    }

    /// Start the scheduler loop. The loop checks every second whether any
    /// task is due for execution.
    pub async fn start(&self) {
        let mut handle_guard = self.loop_handle.write().await;
        if handle_guard.is_some() {
            return; // already running
        }

        let state = Arc::clone(&self.state);

        let handle = tokio::spawn(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(1));
            loop {
                tick.tick().await;

                // Clone the callback for use in this tick
                let on_complete: Option<OnTaskComplete> = {
                    let guard = state.on_task_complete.read().await;
                    guard.as_ref().map(Arc::clone)
                };

                // Skip if paused
                {
                    let p = state.paused.read().await;
                    if *p {
                        continue;
                    }
                }

                let now = Utc::now();

                // Collect ids of tasks that are due (interval + cron)
                let due_ids: Vec<(String, u64, u32, Priority)> = {
                    let tasks_guard = state.tasks.read().await;
                    tasks_guard
                        .iter()
                        .filter_map(|(id, ts)| {
                            let due = match ts.last_run {
                                None => true,
                                Some(last) => {
                                    // Interval-based check
                                    let elapsed = (now - last).num_seconds().max(0) as u64;
                                    if elapsed >= ts.interval_secs {
                                        true
                                    } else {
                                        // Cron-based check (falls back to false on parse error)
                                        cron::is_due(ts.task.cron_expression(), last)
                                            .unwrap_or(false)
                                    }
                                }
                            };
                            if due {
                                Some((
                                    id.clone(),
                                    ts.task.sla_ms(),
                                    ts.task.retry_limit(),
                                    ts.priority.clone(),
                                ))
                            } else {
                                None
                            }
                        })
                        .collect()
                };

                // Snapshot completed tasks for dependency check
                let completed_ids: HashSet<String> = {
                    let tasks_guard = state.tasks.read().await;
                    tasks_guard
                        .iter()
                        .filter_map(|(id, ts)| {
                            if ts.last_status.as_ref() == Some(&TaskStatus::Success) {
                                Some(id.clone())
                            } else {
                                None
                            }
                        })
                        .collect()
                };
                let completed_refs: HashSet<&str> =
                    completed_ids.iter().map(|s| s.as_str()).collect();

                // Snapshot dependency graph and budget (drop guards before loop body)
                let can_run_map: HashMap<String, bool> = {
                    let graph = state.dependency_graph.read().await;
                    due_ids
                        .iter()
                        .map(|(id, _, _, _)| (id.clone(), graph.can_run(id, &completed_refs)))
                        .collect()
                };
                let budget = *state.budget.read().await;

                // Execute due tasks (filtered by dependency and resource checks)
                for (id, sla_ms, retry_limit, priority) in due_ids {
                    // Dependency gate: skip if dependencies are not met
                    if !can_run_map.get(&id).copied().unwrap_or(true) {
                        continue;
                    }

                    // Resource gate: defer low-priority tasks when budget is low
                    if resource::should_defer(priority, budget) {
                        continue;
                    }

                    let task = {
                        let tasks_guard = state.tasks.read().await;
                        match tasks_guard.get(&id) {
                            Some(ts) => Arc::clone(&ts.task),
                            None => continue,
                        }
                    };

                    let result = execute_with_retry(&task, sla_ms, retry_limit).await;

                    // Update state
                    {
                        let mut tasks_guard = state.tasks.write().await;
                        if let Some(ts) = tasks_guard.get_mut(&id) {
                            ts.last_run = Some(Utc::now());
                            ts.last_status = Some(result.status.clone());
                            ts.last_result = Some(result.clone());
                        }
                    }

                    // Fire the completion callback (outside the lock)
                    if let Some(ref cb) = on_complete {
                        cb(&result);
                    }
                }
            }
        });

        *handle_guard = Some(handle);
    }

    /// Stop the scheduler loop.
    pub async fn stop(&self) {
        let mut handle_guard = self.loop_handle.write().await;
        if let Some(handle) = handle_guard.take() {
            handle.abort();
        }
    }

    /// Pause all task execution.
    pub async fn pause_all(&self) {
        let mut paused = self.state.paused.write().await;
        *paused = true;
    }

    /// Resume task execution.
    pub async fn resume_all(&self) {
        let mut paused = self.state.paused.write().await;
        *paused = false;
    }

    /// Check whether the scheduler is paused.
    pub async fn is_paused(&self) -> bool {
        let paused = self.state.paused.read().await;
        *paused
    }

    /// List all registered task ids.
    pub async fn list_tasks(&self) -> Vec<String> {
        let tasks = self.state.tasks.read().await;
        tasks.keys().cloned().collect()
    }

    /// Get info about a specific task.
    pub async fn get_task_info(&self, id: &str) -> Option<TaskInfo> {
        let tasks = self.state.tasks.read().await;
        tasks.get(id).map(|ts| TaskInfo {
            id: ts.task.id().to_string(),
            name: ts.task.name().to_string(),
            layer: ts.task.layer().to_string(),
            cron: ts.task.cron_expression().to_string(),
            sla_ms: ts.task.sla_ms(),
            interval_secs: ts.interval_secs,
            last_run: ts.last_run,
            last_status: ts.last_status.clone(),
        })
    }

    /// Get the last result for a specific task.
    pub async fn get_last_result(&self, id: &str) -> Option<TaskResult> {
        let tasks = self.state.tasks.read().await;
        tasks.get(id).and_then(|ts| ts.last_result.clone())
    }

    /// Run a specific task immediately (bypasses interval and paused state).
    pub async fn run_now(&self, id: &str) -> Option<TaskResult> {
        let task = {
            let tasks = self.state.tasks.read().await;
            tasks.get(id).map(|ts| {
                (
                    Arc::clone(&ts.task),
                    ts.task.sla_ms(),
                    ts.task.retry_limit(),
                )
            })
        };

        let (task, sla_ms, retry_limit) = task?;
        let result = execute_with_retry(&task, sla_ms, retry_limit).await;

        {
            let mut tasks = self.state.tasks.write().await;
            if let Some(ts) = tasks.get_mut(id) {
                ts.last_run = Some(Utc::now());
                ts.last_status = Some(result.status.clone());
                ts.last_result = Some(result.clone());
            }
        }

        // Fire the completion callback
        let on_complete: Option<OnTaskComplete> = {
            let guard = self.state.on_task_complete.read().await;
            guard.as_ref().map(Arc::clone)
        };
        if let Some(ref cb) = on_complete {
            cb(&result);
        }

        Some(result)
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Execute a task with retry and timeout logic.
async fn execute_with_retry(
    task: &Arc<dyn ScheduledTask>,
    sla_ms: u64,
    retry_limit: u32,
) -> TaskResult {
    let mut last_result = None;

    for attempt in 0..=retry_limit {
        let result = execute_once(task, sla_ms, attempt).await;

        if result.status == TaskStatus::Success {
            return result;
        }

        last_result = Some(result);

        // Don't retry after the last attempt
        if attempt < retry_limit {
            tokio::time::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
        }
    }

    last_result.expect("at least one attempt must have run")
}

/// Execute a single attempt with timeout.
async fn execute_once(task: &Arc<dyn ScheduledTask>, sla_ms: u64, retry_count: u32) -> TaskResult {
    let started_at = Utc::now();
    let timeout_duration = Duration::from_millis(sla_ms);

    match tokio::time::timeout(timeout_duration, task.execute()).await {
        Ok(result) => {
            let mut result = result;
            result.retry_count = retry_count;
            result
        }
        Err(_elapsed) => {
            let finished_at = Utc::now();
            let duration_ms = (finished_at - started_at).num_milliseconds().max(0) as u64;
            TaskResult {
                task_id: task.id().to_string(),
                task_name: task.name().to_string(),
                status: TaskStatus::Timeout,
                started_at,
                finished_at,
                duration_ms,
                summary: format!("Task timed out after {}ms", sla_ms),
                error: Some("execution exceeded SLA timeout".to_string()),
                retry_count,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{ScheduledTask, TaskLayer, TaskResult, TaskStatus};
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    /// A mock task that fails the first N attempts then succeeds.
    struct FlakeyTask {
        id: String,
        fail_count: AtomicU32,
        max_failures: u32,
    }

    impl FlakeyTask {
        fn new(id: &str, max_failures: u32) -> Self {
            Self {
                id: id.to_string(),
                fail_count: AtomicU32::new(0),
                max_failures,
            }
        }
    }

    #[async_trait]
    impl ScheduledTask for FlakeyTask {
        fn id(&self) -> &str {
            &self.id
        }
        fn name(&self) -> &str {
            "flakey"
        }
        fn layer(&self) -> TaskLayer {
            TaskLayer::Collection
        }
        fn cron_expression(&self) -> &str {
            "* * * * * *"
        }
        fn sla_ms(&self) -> u64 {
            5000
        }
        fn retry_limit(&self) -> u32 {
            3
        }

        async fn execute(&self) -> TaskResult {
            let attempt = self.fail_count.fetch_add(1, Ordering::SeqCst);
            let now = Utc::now();
            if attempt < self.max_failures {
                TaskResult {
                    task_id: self.id.clone(),
                    task_name: self.name().to_string(),
                    status: TaskStatus::Failed,
                    started_at: now,
                    finished_at: now,
                    duration_ms: 0,
                    summary: "deliberate failure".to_string(),
                    error: Some(format!("attempt {attempt} failed")),
                    retry_count: 0,
                }
            } else {
                TaskResult {
                    task_id: self.id.clone(),
                    task_name: self.name().to_string(),
                    status: TaskStatus::Success,
                    started_at: now,
                    finished_at: now,
                    duration_ms: 0,
                    summary: "success".to_string(),
                    error: None,
                    retry_count: 0,
                }
            }
        }
    }

    /// A mock task that always succeeds immediately.
    struct SuccessTask {
        id: String,
    }

    #[async_trait]
    impl ScheduledTask for SuccessTask {
        fn id(&self) -> &str {
            &self.id
        }
        fn name(&self) -> &str {
            "success"
        }
        fn layer(&self) -> TaskLayer {
            TaskLayer::Collection
        }
        fn cron_expression(&self) -> &str {
            "* * * * * *"
        }
        fn sla_ms(&self) -> u64 {
            5000
        }
        fn retry_limit(&self) -> u32 {
            3
        }

        async fn execute(&self) -> TaskResult {
            let now = Utc::now();
            TaskResult {
                task_id: self.id.clone(),
                task_name: self.name().to_string(),
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

    /// A mock task that always fails.
    struct AlwaysFailTask {
        id: String,
    }

    #[async_trait]
    impl ScheduledTask for AlwaysFailTask {
        fn id(&self) -> &str {
            &self.id
        }
        fn name(&self) -> &str {
            "always_fail"
        }
        fn layer(&self) -> TaskLayer {
            TaskLayer::Collection
        }
        fn cron_expression(&self) -> &str {
            "* * * * * *"
        }
        fn sla_ms(&self) -> u64 {
            5000
        }
        fn retry_limit(&self) -> u32 {
            3
        }

        async fn execute(&self) -> TaskResult {
            let now = Utc::now();
            TaskResult {
                task_id: self.id.clone(),
                task_name: self.name().to_string(),
                status: TaskStatus::Failed,
                started_at: now,
                finished_at: now,
                duration_ms: 0,
                summary: "always fails".to_string(),
                error: Some("permanent failure".to_string()),
                retry_count: 0,
            }
        }
    }

    /// A mock task that exceeds its SLA timeout.
    struct SlowTask {
        id: String,
    }

    #[async_trait]
    impl ScheduledTask for SlowTask {
        fn id(&self) -> &str {
            &self.id
        }
        fn name(&self) -> &str {
            "slow"
        }
        fn layer(&self) -> TaskLayer {
            TaskLayer::Collection
        }
        fn cron_expression(&self) -> &str {
            "* * * * * *"
        }
        fn sla_ms(&self) -> u64 {
            50 // very short SLA
        }
        fn retry_limit(&self) -> u32 {
            2
        }

        async fn execute(&self) -> TaskResult {
            // Sleep longer than SLA to trigger timeout
            tokio::time::sleep(Duration::from_millis(200)).await;
            let now = Utc::now();
            TaskResult {
                task_id: self.id.clone(),
                task_name: self.name().to_string(),
                status: TaskStatus::Success,
                started_at: now,
                finished_at: now,
                duration_ms: 200,
                summary: "should not reach here".to_string(),
                error: None,
                retry_count: 0,
            }
        }
    }

    // ── A9-01: Success on first try ───────────────────────────────────
    #[tokio::test]
    async fn a9_01_success_on_first_try() {
        let task: Arc<dyn ScheduledTask> = Arc::new(SuccessTask {
            id: "ok-task".to_string(),
        });
        let result = execute_with_retry(&task, 5000, 3).await;
        assert_eq!(result.status, TaskStatus::Success);
        assert_eq!(result.retry_count, 0);
    }

    // ── A9-02: Retry until limit ──────────────────────────────────────
    // Task fails 3 times, succeeds on 4th attempt (retry_limit=3 means 4 total attempts)
    #[tokio::test]
    async fn a9_02_retry_until_success() {
        let task: Arc<dyn ScheduledTask> = Arc::new(FlakeyTask::new("flakey", 3));
        let result = execute_with_retry(&task, 5000, 3).await;
        assert_eq!(result.status, TaskStatus::Success);
    }

    // Task always fails -- exhausts all retries
    #[tokio::test]
    async fn a9_02_retry_exhausted_still_fails() {
        let task: Arc<dyn ScheduledTask> = Arc::new(AlwaysFailTask {
            id: "always-fail".to_string(),
        });
        let result = execute_with_retry(&task, 5000, 3).await;
        assert_eq!(result.status, TaskStatus::Failed);
        // 4 attempts total (0, 1, 2, 3)
        assert_eq!(result.retry_count, 3);
    }

    // ── A9-03: Timeout ────────────────────────────────────────────────
    #[tokio::test]
    async fn a9_03_timeout_triggers() {
        let task: Arc<dyn ScheduledTask> = Arc::new(SlowTask {
            id: "slow-task".to_string(),
        });
        // SLA is 50ms, task sleeps 200ms -> timeout on every attempt
        let result = execute_with_retry(&task, 50, 2).await;
        assert_eq!(result.status, TaskStatus::Timeout);
        assert!(result.error.as_deref().unwrap().contains("exceeded SLA"));
    }

    // ── A9-04: Increasing backoff interval ────────────────────────────
    // Verify that retries take progressively longer (backoff).
    // We measure wall-clock time: retry_limit=3, backoff=100ms*(attempt+1)
    // Expected total backoff: 100+200+300 = 600ms minimum.
    #[tokio::test]
    async fn a9_04_increasing_backoff() {
        let task: Arc<dyn ScheduledTask> = Arc::new(AlwaysFailTask {
            id: "backoff-task".to_string(),
        });
        let start = std::time::Instant::now();
        let result = execute_with_retry(&task, 5000, 3).await;
        let elapsed = start.elapsed();

        assert_eq!(result.status, TaskStatus::Failed);

        // Backoff: 100ms*(0+1) + 100ms*(1+1) + 100ms*(2+1) = 100+200+300 = 600ms
        // Allow some tolerance for scheduling jitter
        assert!(
            elapsed >= Duration::from_millis(500),
            "Expected at least 500ms for backoff, got {:?}",
            elapsed
        );
    }

    // ── Additional: Scheduler basic operations ────────────────────────
    #[tokio::test]
    async fn scheduler_register_and_list() {
        let scheduler = Scheduler::new();
        let task: Arc<dyn ScheduledTask> = Arc::new(SuccessTask {
            id: "t1".to_string(),
        });
        scheduler.register(task).await;
        let list = scheduler.list_tasks().await;
        assert_eq!(list.len(), 1);
        assert!(list.contains(&"t1".to_string()));
    }

    #[tokio::test]
    async fn scheduler_run_now() {
        let scheduler = Scheduler::new();
        let task: Arc<dyn ScheduledTask> = Arc::new(SuccessTask {
            id: "t1".to_string(),
        });
        scheduler.register(task).await;
        let result = scheduler.run_now("t1").await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().status, TaskStatus::Success);
    }

    #[tokio::test]
    async fn scheduler_run_now_nonexistent() {
        let scheduler = Scheduler::new();
        let result = scheduler.run_now("nope").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn scheduler_pause_resume() {
        let scheduler = Scheduler::new();
        assert!(!scheduler.is_paused().await);

        scheduler.pause_all().await;
        assert!(scheduler.is_paused().await);

        scheduler.resume_all().await;
        assert!(!scheduler.is_paused().await);
    }

    #[tokio::test]
    async fn scheduler_budget() {
        let scheduler = Scheduler::new();
        assert_eq!(scheduler.budget().await, 100);

        scheduler.set_budget(42).await;
        assert_eq!(scheduler.budget().await, 42);
    }
}
