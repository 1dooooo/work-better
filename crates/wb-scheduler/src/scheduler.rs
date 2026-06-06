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
            }),
            loop_handle: RwLock::new(None),
        }
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
        self.register_with_priority(task, interval_secs, priority).await;
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
                                        cron::is_due(ts.task.cron_expression(), last).unwrap_or(false)
                                    }
                                }
                            };
                            if due {
                                Some((id.clone(), ts.task.sla_ms(), ts.task.retry_limit(), ts.priority.clone()))
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
                let completed_refs: HashSet<&str> = completed_ids.iter().map(|s| s.as_str()).collect();

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
                    let mut tasks_guard = state.tasks.write().await;
                    if let Some(ts) = tasks_guard.get_mut(&id) {
                        ts.last_run = Some(Utc::now());
                        ts.last_status = Some(result.status.clone());
                        ts.last_result = Some(result);
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

        let mut tasks = self.state.tasks.write().await;
        if let Some(ts) = tasks.get_mut(id) {
            ts.last_run = Some(Utc::now());
            ts.last_status = Some(result.status.clone());
            ts.last_result = Some(result.clone());
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
async fn execute_once(
    task: &Arc<dyn ScheduledTask>,
    sla_ms: u64,
    retry_count: u32,
) -> TaskResult {
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
