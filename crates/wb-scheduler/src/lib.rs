//! wb-scheduler: 定时任务调度框架

pub mod cron;
pub mod dependency;
pub mod log;
pub mod resource;
pub mod scheduler;
pub mod task;

// Re-export callback type for convenience
pub use scheduler::OnTaskComplete;
