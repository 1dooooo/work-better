//! 真实后端集成测试基础设施
//!
//! 提供 `TestHarness` 结构体，用内存 SQLite 初始化所有子系统，
//! 不依赖 Tauri，直接测试 crate 层业务逻辑。

use std::sync::Arc;
use wb_collector::manager::CollectorManager;
use wb_processor::task::TaskManager;
use wb_scheduler::scheduler::Scheduler;
use wb_storage::SqliteEventLog;

pub struct TestHarness {
    pub event_log: Arc<SqliteEventLog>,
    pub task_manager: TaskManager,
    pub collector_manager: CollectorManager,
    pub scheduler: Scheduler,
}

impl TestHarness {
    pub fn new() -> Self {
        Self {
            event_log: Arc::new(
                SqliteEventLog::new_in_memory().expect("failed to create in-memory DB"),
            ),
            task_manager: TaskManager::new(),
            collector_manager: CollectorManager::new(),
            scheduler: Scheduler::new(),
        }
    }
}

impl Default for TestHarness {
    fn default() -> Self {
        Self::new()
    }
}
