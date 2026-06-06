//! 同步日志 —— 三层同步状态追踪

use serde::{Deserialize, Serialize};

/// 同步层
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncLayer {
    Collect,
    Process,
    Storage,
}

/// 同步状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    Pending,
    Success,
    Failed,
}

/// 同步日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncLogEntry {
    pub id: String,
    pub layer: SyncLayer,
    pub source: String,
    pub status: SyncStatus,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub error: Option<String>,
}

/// 同步日志
#[derive(Debug, Clone)]
pub struct SyncLog {
    entries: Vec<SyncLogEntry>,
}

impl SyncLog {
    /// 创建空的同步日志
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// 记录一条同步日志
    pub fn record(&mut self, entry: SyncLogEntry) {
        self.entries.push(entry);
    }

    /// 按同步层查询
    pub fn by_layer(&self, layer: SyncLayer) -> Vec<&SyncLogEntry> {
        self.entries.iter().filter(|e| e.layer == layer).collect()
    }

    /// 获取所有失败记录
    pub fn failures(&self) -> Vec<&SyncLogEntry> {
        self.entries
            .iter()
            .filter(|e| e.status == SyncStatus::Failed)
            .collect()
    }

    /// 获取最近 n 条记录
    pub fn recent(&self, n: usize) -> Vec<&SyncLogEntry> {
        let skip = self.entries.len().saturating_sub(n);
        self.entries.iter().skip(skip).collect()
    }

    /// 记录总数
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for SyncLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(id: &str, layer: SyncLayer, status: SyncStatus) -> SyncLogEntry {
        SyncLogEntry {
            id: id.into(),
            layer,
            source: "test-source".into(),
            status,
            started_at: "2026-01-01T00:00:00Z".into(),
            finished_at: Some("2026-01-01T00:00:01Z".into()),
            error: None,
        }
    }

    #[test]
    fn test_new_is_empty() {
        let log = SyncLog::new();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
    }

    #[test]
    fn test_record_and_len() {
        let mut log = SyncLog::new();
        log.record(make_entry("1", SyncLayer::Collect, SyncStatus::Success));
        log.record(make_entry("2", SyncLayer::Process, SyncStatus::Pending));
        assert_eq!(log.len(), 2);
    }

    #[test]
    fn test_by_layer() {
        let mut log = SyncLog::new();
        log.record(make_entry("1", SyncLayer::Collect, SyncStatus::Success));
        log.record(make_entry("2", SyncLayer::Process, SyncStatus::Success));
        log.record(make_entry("3", SyncLayer::Collect, SyncStatus::Failed));
        log.record(make_entry("4", SyncLayer::Storage, SyncStatus::Success));

        let collect = log.by_layer(SyncLayer::Collect);
        assert_eq!(collect.len(), 2);
        assert!(collect.iter().all(|e| e.layer == SyncLayer::Collect));

        let process = log.by_layer(SyncLayer::Process);
        assert_eq!(process.len(), 1);

        let storage = log.by_layer(SyncLayer::Storage);
        assert_eq!(storage.len(), 1);
    }

    #[test]
    fn test_failures() {
        let mut log = SyncLog::new();
        log.record(make_entry("1", SyncLayer::Collect, SyncStatus::Success));
        log.record(make_entry("2", SyncLayer::Process, SyncStatus::Failed));
        log.record(make_entry("3", SyncLayer::Storage, SyncStatus::Failed));
        log.record(make_entry("4", SyncLayer::Collect, SyncStatus::Pending));

        let failures = log.failures();
        assert_eq!(failures.len(), 2);
        assert!(failures.iter().all(|e| e.status == SyncStatus::Failed));
    }

    #[test]
    fn test_failures_with_error_message() {
        let mut log = SyncLog::new();
        let mut entry = make_entry("1", SyncLayer::Collect, SyncStatus::Failed);
        entry.error = Some("connection timeout".into());
        log.record(entry);

        let failures = log.failures();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].error.as_deref(), Some("connection timeout"));
    }

    #[test]
    fn test_recent() {
        let mut log = SyncLog::new();
        log.record(make_entry("1", SyncLayer::Collect, SyncStatus::Success));
        log.record(make_entry("2", SyncLayer::Process, SyncStatus::Success));
        log.record(make_entry("3", SyncLayer::Storage, SyncStatus::Success));
        log.record(make_entry("4", SyncLayer::Collect, SyncStatus::Pending));
        log.record(make_entry("5", SyncLayer::Process, SyncStatus::Failed));

        let recent = log.recent(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].id, "3");
        assert_eq!(recent[1].id, "4");
        assert_eq!(recent[2].id, "5");
    }

    #[test]
    fn test_recent_more_than_available() {
        let mut log = SyncLog::new();
        log.record(make_entry("1", SyncLayer::Collect, SyncStatus::Success));

        let recent = log.recent(10);
        assert_eq!(recent.len(), 1);
    }

    #[test]
    fn test_recent_zero() {
        let mut log = SyncLog::new();
        log.record(make_entry("1", SyncLayer::Collect, SyncStatus::Success));

        let recent = log.recent(0);
        assert!(recent.is_empty());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut log = SyncLog::new();
        log.record(SyncLogEntry {
            id: "1".into(),
            layer: SyncLayer::Collect,
            source: "feishu-api".into(),
            status: SyncStatus::Success,
            started_at: "2026-01-01T00:00:00Z".into(),
            finished_at: Some("2026-01-01T00:00:05Z".into()),
            error: None,
        });

        // Test individual entry serialization
        let entry = &log.recent(1)[0];
        let json = serde_json::to_string(entry).unwrap();
        let deserialized: SyncLogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "1");
        assert_eq!(deserialized.layer, SyncLayer::Collect);
        assert_eq!(deserialized.status, SyncStatus::Success);
    }

    #[test]
    fn test_default() {
        let log = SyncLog::default();
        assert!(log.is_empty());
    }
}
