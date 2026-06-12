//! 采集器定时任务
//!
//! 将 CollectorManager 中的采集器包装为 ScheduledTask，由 Scheduler 定期调用。

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use wb_scheduler::task::{ScheduledTask, TaskLayer, TaskResult, TaskStatus};

use crate::manager::CollectorManager;

/// 采集器定时任务
///
/// 包装 CollectorManager，定期执行所有已启用的采集器。
/// 每次执行会收集所有采集器的结果并汇总为 TaskResult。
pub struct CollectorTask {
    manager: Arc<CollectorManager>,
    collector_id: String,
    task_name: String,
    interval_secs: u64,
}

impl CollectorTask {
    /// 创建新的采集器定时任务
    pub fn new(
        manager: Arc<CollectorManager>,
        collector_id: String,
        task_name: String,
        interval_secs: u64,
    ) -> Self {
        Self {
            manager,
            collector_id,
            task_name,
            interval_secs,
        }
    }

    /// 获取间隔秒数
    pub fn interval_secs(&self) -> u64 {
        self.interval_secs
    }
}

/// 执行日志记录器
///
/// 用于将采集器执行结果写入执行日志。
/// 这是一个简化版本，实际应该通过依赖注入获取 AuditLogStore。
struct ExecutionLogger;

impl ExecutionLogger {
    /// 记录执行日志到 stderr（临时方案）
    ///
    /// TODO: 后续应该写入 execution_logs 表
    fn log_execution(task_id: &str, result: &TaskResult) {
        let status = match &result.status {
            TaskStatus::Success => "✅",
            TaskStatus::Failed => "❌",
            TaskStatus::Timeout => "⏰",
            TaskStatus::Aborted => "🚫",
        };

        eprintln!(
            "[execution-log] {} {} | {} | {}ms | {}",
            status,
            task_id,
            result.finished_at.format("%H:%M:%S"),
            result.duration_ms,
            result.summary
        );

        if let Some(error) = &result.error {
            eprintln!("[execution-log]   └─ error: {}", error);
        }
    }
}

#[async_trait]
impl ScheduledTask for CollectorTask {
    fn id(&self) -> &str {
        &self.collector_id
    }

    fn name(&self) -> &str {
        &self.task_name
    }

    fn layer(&self) -> TaskLayer {
        TaskLayer::Collection
    }

    fn cron_expression(&self) -> &str {
        // 使用间隔调度，cron 表达式设为每分钟
        "* * * * *"
    }

    fn sla_ms(&self) -> u64 {
        // 采集器超时时间：30秒
        30_000
    }

    fn retry_limit(&self) -> u32 {
        // 采集器失败后重试2次
        2
    }

    async fn execute(&self) -> TaskResult {
        let started_at = Utc::now();
        let task_id = self.collector_id.clone();

        // 检查采集器是否启用
        if !self.manager.is_enabled(&self.collector_id).await {
            let result = TaskResult {
                task_id: task_id.clone(),
                status: TaskStatus::Success,
                started_at,
                finished_at: Utc::now(),
                duration_ms: 0,
                summary: format!("Collector '{}' is disabled, skipping", self.collector_id),
                error: None,
                retry_count: 0,
            };
            ExecutionLogger::log_execution(&task_id, &result);
            return result;
        }

        // 执行采集
        let result = self.manager.collect_one(&self.collector_id).await;
        let finished_at = Utc::now();
        let duration_ms = (finished_at - started_at).num_milliseconds().max(0) as u64;

        let task_result = match result {
            Some(Ok(events)) => {
                let count = events.len();
                TaskResult {
                    task_id: task_id.clone(),
                    status: TaskStatus::Success,
                    started_at,
                    finished_at,
                    duration_ms,
                    summary: format!("Collected {} events from '{}'", count, self.collector_id),
                    error: None,
                    retry_count: 0,
                }
            }
            Some(Err(e)) => TaskResult {
                task_id: task_id.clone(),
                status: TaskStatus::Failed,
                started_at,
                finished_at,
                duration_ms,
                summary: format!("Collector '{}' failed", self.collector_id),
                error: Some(e.to_string()),
                retry_count: 0,
            },
            None => TaskResult {
                task_id: task_id.clone(),
                status: TaskStatus::Failed,
                started_at,
                finished_at,
                duration_ms,
                summary: format!("Collector '{}' not found", self.collector_id),
                error: Some("Collector not registered".to_string()),
                retry_count: 0,
            },
        };

        // 记录执行日志
        ExecutionLogger::log_execution(&task_id, &task_result);

        task_result
    }
}
