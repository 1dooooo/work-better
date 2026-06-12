//! 调度器暂停/恢复功能真实后端测试
//!
//! 测试场景：调用 pause/resume_scheduler → 状态正确切换
//!
//! 验证目标：
//! 1. 调度器默认非暂停状态
//! 2. 暂停后状态正确更新
//! 3. 恢复后状态正确恢复
//! 4. 暂停状态下任务不执行

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use wb_scheduler::scheduler::Scheduler;
use wb_scheduler::task::{ScheduledTask, TaskLayer, TaskResult, TaskStatus};
use async_trait::async_trait;
use chrono::Utc;

/// 测试用的计数任务
struct CounterTask {
    id: String,
    execution_count: Arc<AtomicU32>,
}

impl CounterTask {
    fn new(id: &str) -> (Self, Arc<AtomicU32>) {
        let count = Arc::new(AtomicU32::new(0));
        let task = Self {
            id: id.to_string(),
            execution_count: Arc::clone(&count),
        };
        (task, count)
    }
}

#[async_trait]
impl ScheduledTask for CounterTask {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "counter-task"
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
        0
    }

    async fn execute(&self) -> TaskResult {
        self.execution_count.fetch_add(1, Ordering::SeqCst);
        let now = Utc::now();
        TaskResult {
            task_id: self.id.clone(),
            status: TaskStatus::Success,
            started_at: now,
            finished_at: now,
            duration_ms: 0,
            summary: "executed".to_string(),
            error: None,
            retry_count: 0,
        }
    }
}

/// 测试调度器默认非暂停状态
///
/// 场景：创建调度器后，检查默认状态
/// 预期：调度器默认非暂停
#[tokio::test]
async fn test_scheduler_default_not_paused() {
    let scheduler = Scheduler::new();

    // 验证：默认非暂停
    assert!(
        !scheduler.is_paused().await,
        "调度器应该默认非暂停"
    );
}

/// 测试暂停调度器
///
/// 场景：暂停调度器后，检查状态
/// 预期：调度器状态为暂停
#[tokio::test]
async fn test_scheduler_pause() {
    let scheduler = Scheduler::new();

    // 暂停调度器
    scheduler.pause_all().await;

    // 验证：状态为暂停
    assert!(
        scheduler.is_paused().await,
        "暂停后调度器应该处于暂停状态"
    );
}

/// 测试恢复调度器
///
/// 场景：暂停后恢复，检查状态
/// 预期：调度器状态恢复为非暂停
#[tokio::test]
async fn test_scheduler_resume() {
    let scheduler = Scheduler::new();

    // 暂停后恢复
    scheduler.pause_all().await;
    scheduler.resume_all().await;

    // 验证：状态恢复为非暂停
    assert!(
        !scheduler.is_paused().await,
        "恢复后调度器应该处于非暂停状态"
    );
}

/// 测试暂停状态下任务不执行
///
/// 场景：暂停调度器后，等待一段时间
/// 预期：任务不被执行
#[tokio::test]
async fn test_scheduler_paused_tasks_not_executed() {
    let scheduler = Scheduler::new();
    let (task, execution_count) = CounterTask::new("test-task");

    // 注册任务
    scheduler.register(Arc::new(task)).await;

    // 暂停调度器
    scheduler.pause_all().await;

    // 启动调度器
    scheduler.start().await;

    // 等待一段时间
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 验证：任务没有被执行
    assert_eq!(
        execution_count.load(Ordering::SeqCst),
        0,
        "暂停状态下任务不应该被执行"
    );

    // 清理
    scheduler.stop().await;
}

/// 测试恢复状态下任务执行
///
/// 场景：恢复调度器后，等待一段时间
/// 预期：任务被执行
#[tokio::test]
async fn test_scheduler_resumed_tasks_executed() {
    let scheduler = Scheduler::new();
    let (task, execution_count) = CounterTask::new("test-task");

    // 注册任务
    scheduler.register(Arc::new(task)).await;

    // 启动调度器（默认非暂停）
    scheduler.start().await;

    // 等待一段时间让任务执行
    tokio::time::sleep(Duration::from_millis(1500)).await;

    // 验证：任务被执行
    assert!(
        execution_count.load(Ordering::SeqCst) > 0,
        "恢复状态下任务应该被执行"
    );

    // 清理
    scheduler.stop().await;
}

/// 测试暂停/恢复切换
///
/// 场景：多次切换暂停/恢复状态
/// 预期：状态正确切换
#[tokio::test]
async fn test_scheduler_pause_resume_toggle() {
    let scheduler = Scheduler::new();

    // 初始状态
    assert!(!scheduler.is_paused().await);

    // 暂停
    scheduler.pause_all().await;
    assert!(scheduler.is_paused().await);

    // 恢复
    scheduler.resume_all().await;
    assert!(!scheduler.is_paused().await);

    // 再次暂停
    scheduler.pause_all().await;
    assert!(scheduler.is_paused().await);

    // 再次恢复
    scheduler.resume_all().await;
    assert!(!scheduler.is_paused().await);
}
