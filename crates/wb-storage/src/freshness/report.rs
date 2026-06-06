//! 保鲜报告 —— 记录保鲜任务的执行结果

/// 问题严重级别
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueSeverity {
    /// 低：不影响使用，建议修复
    Low,
    /// 中：影响文档质量，应尽快修复
    Medium,
    /// 高：导致链接断裂或数据不一致，必须修复
    High,
}

/// 发现的问题
#[derive(Debug, Clone)]
pub struct Issue {
    /// 问题所在的文件路径
    pub file_path: String,
    /// 问题描述
    pub description: String,
    /// 严重级别
    pub severity: IssueSeverity,
    /// 任务名称
    pub task_name: String,
}

/// 保鲜报告
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreshnessReport {
    /// 已执行的任务数
    pub tasks_run: u32,
    /// 发现的问题数
    pub issues_found: u32,
    /// 已修复的问题数
    pub issues_fixed: u32,
    /// 执行耗时（毫秒）
    pub duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_creation() {
        let issue = Issue {
            file_path: "Daily/2026-06-06.md".to_string(),
            description: "Broken link".to_string(),
            severity: IssueSeverity::High,
            task_name: "link_integrity_check".to_string(),
        };
        assert_eq!(issue.file_path, "Daily/2026-06-06.md");
        assert_eq!(issue.severity, IssueSeverity::High);
    }

    #[test]
    fn test_freshness_report_creation() {
        let report = FreshnessReport {
            tasks_run: 7,
            issues_found: 3,
            issues_fixed: 1,
            duration_ms: 150,
        };
        assert_eq!(report.tasks_run, 7);
        assert_eq!(report.issues_found, 3);
        assert_eq!(report.issues_fixed, 1);
        assert_eq!(report.duration_ms, 150);
    }

    #[test]
    fn test_issue_severity_equality() {
        assert_eq!(IssueSeverity::Low, IssueSeverity::Low);
        assert_ne!(IssueSeverity::Low, IssueSeverity::High);
    }
}
