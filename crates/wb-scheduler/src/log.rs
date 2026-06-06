//! 定时任务执行日志

use serde::{Deserialize, Serialize};

/// 执行状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Success,
    Failed,
    Skipped,
    Timeout,
}

/// 单次执行记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub task_id: String,
    pub task_name: String,
    pub status: ExecutionStatus,
    pub started_at: String,
    pub finished_at: String,
    pub duration_ms: u64,
    pub output: Option<String>,
    pub error: Option<String>,
}

/// 执行日志管理器
pub struct ExecutionLog {
    records: Vec<ExecutionRecord>,
}

impl Default for ExecutionLog {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionLog {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// 记录一次执行
    pub fn record(&mut self, record: ExecutionRecord) {
        self.records.push(record);
    }

    /// 查询指定任务的执行记录
    pub fn by_task(&self, task_id: &str) -> Vec<&ExecutionRecord> {
        self.records
            .iter()
            .filter(|r| r.task_id == task_id)
            .collect()
    }

    /// 查询最近 N 条记录
    pub fn recent(&self, n: usize) -> Vec<&ExecutionRecord> {
        let start = self.records.len().saturating_sub(n);
        self.records[start..].iter().collect()
    }

    /// 查询失败的记录
    pub fn failures(&self) -> Vec<&ExecutionRecord> {
        self.records
            .iter()
            .filter(|r| r.status == ExecutionStatus::Failed)
            .collect()
    }

    /// 总记录数
    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// 清空日志
    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_record(task_id: &str, status: ExecutionStatus) -> ExecutionRecord {
        ExecutionRecord {
            task_id: task_id.to_string(),
            task_name: format!("task_{task_id}"),
            status,
            started_at: "2026-06-06T10:00:00Z".to_string(),
            finished_at: "2026-06-06T10:00:05Z".to_string(),
            duration_ms: 5000,
            output: Some("done".to_string()),
            error: None,
        }
    }

    #[test]
    fn test_record_and_len() {
        let mut log = ExecutionLog::new();
        assert!(log.is_empty());

        log.record(sample_record("t1", ExecutionStatus::Success));
        log.record(sample_record("t2", ExecutionStatus::Failed));
        assert_eq!(log.len(), 2);
    }

    #[test]
    fn test_by_task() {
        let mut log = ExecutionLog::new();
        log.record(sample_record("t1", ExecutionStatus::Success));
        log.record(sample_record("t2", ExecutionStatus::Failed));
        log.record(sample_record("t1", ExecutionStatus::Success));

        let t1_records = log.by_task("t1");
        assert_eq!(t1_records.len(), 2);
        assert!(t1_records.iter().all(|r| r.task_id == "t1"));
    }

    #[test]
    fn test_recent() {
        let mut log = ExecutionLog::new();
        for i in 0..10 {
            log.record(sample_record(&format!("t{i}"), ExecutionStatus::Success));
        }

        let recent = log.recent(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].task_id, "t7");
    }

    #[test]
    fn test_failures() {
        let mut log = ExecutionLog::new();
        log.record(sample_record("t1", ExecutionStatus::Success));
        log.record(sample_record("t2", ExecutionStatus::Failed));
        log.record(sample_record("t3", ExecutionStatus::Failed));
        log.record(sample_record("t4", ExecutionStatus::Success));

        let failures = log.failures();
        assert_eq!(failures.len(), 2);
        assert!(failures.iter().all(|r| r.status == ExecutionStatus::Failed));
    }

    #[test]
    fn test_clear() {
        let mut log = ExecutionLog::new();
        log.record(sample_record("t1", ExecutionStatus::Success));
        log.clear();
        assert!(log.is_empty());
    }
}
