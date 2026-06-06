//! 信息保鲜引擎 —— 定期检查 Obsidian vault 中的文档质量与一致性

mod integrity;
mod quality;
mod report;
mod sync;

use std::path::PathBuf;

use wb_core::error::Result;

pub use integrity::IntegrityTask;
pub use quality::QualityTask;
pub use report::{FreshnessReport, Issue, IssueSeverity};
pub use sync::SyncTask;

/// 信息保鲜引擎
///
/// 负责对 Obsidian vault 执行三类任务：
/// - 同步任务：任务状态同步、文档变更检测
/// - 完整性任务：链接完整性、重复检测、标签规范化
/// - 质量任务：三层一致性检查、知识过期审查
#[derive(Debug, Clone)]
pub struct FreshnessEngine {
    vault_path: PathBuf,
}

impl FreshnessEngine {
    /// 创建保鲜引擎
    pub fn new(vault_path: PathBuf) -> Self {
        Self { vault_path }
    }

    /// 获取 vault 路径
    pub fn vault_path(&self) -> &PathBuf {
        &self.vault_path
    }

    /// 运行同步任务（任务状态同步 + 文档变更检测）
    pub fn run_sync_tasks(&self) -> Result<FreshnessReport> {
        let start = std::time::Instant::now();
        let mut all_issues: Vec<Issue> = Vec::new();
        let mut tasks_run: u32 = 0;

        let sync_task = SyncTask::new(&self.vault_path);

        all_issues.extend(sync_task.task_status_sync()?);
        tasks_run += 1;

        all_issues.extend(sync_task.document_change_detect()?);
        tasks_run += 1;

        let issues_found = all_issues.len() as u32;
        Ok(FreshnessReport {
            tasks_run,
            issues_found,
            issues_fixed: 0,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// 运行完整性任务（链接完整性 + 重复检测 + 标签规范化）
    pub fn run_integrity_tasks(&self) -> Result<FreshnessReport> {
        let start = std::time::Instant::now();
        let mut all_issues: Vec<Issue> = Vec::new();
        let mut tasks_run: u32 = 0;

        let integrity_task = IntegrityTask::new(&self.vault_path);

        all_issues.extend(integrity_task.link_integrity_check()?);
        tasks_run += 1;

        all_issues.extend(integrity_task.duplicate_detection()?);
        tasks_run += 1;

        all_issues.extend(integrity_task.tag_normalization()?);
        tasks_run += 1;

        let issues_found = all_issues.len() as u32;
        Ok(FreshnessReport {
            tasks_run,
            issues_found,
            issues_fixed: 0,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// 运行质量任务（三层一致性 + 知识过期审查）
    pub fn run_quality_tasks(&self) -> Result<FreshnessReport> {
        let start = std::time::Instant::now();
        let mut all_issues: Vec<Issue> = Vec::new();
        let mut tasks_run: u32 = 0;

        let quality_task = QualityTask::new(&self.vault_path);

        all_issues.extend(quality_task.consistency_check()?);
        tasks_run += 1;

        all_issues.extend(quality_task.staleness_review()?);
        tasks_run += 1;

        let issues_found = all_issues.len() as u32;
        Ok(FreshnessReport {
            tasks_run,
            issues_found,
            issues_fixed: 0,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// 运行所有保鲜任务，合并为一份报告
    pub fn run_all(&self) -> Result<FreshnessReport> {
        let start = std::time::Instant::now();
        let mut all_issues: Vec<Issue> = Vec::new();
        let mut tasks_run: u32 = 0;

        // 同步任务
        let sync_task = SyncTask::new(&self.vault_path);
        all_issues.extend(sync_task.task_status_sync()?);
        tasks_run += 1;
        all_issues.extend(sync_task.document_change_detect()?);
        tasks_run += 1;

        // 完整性任务
        let integrity_task = IntegrityTask::new(&self.vault_path);
        all_issues.extend(integrity_task.link_integrity_check()?);
        tasks_run += 1;
        all_issues.extend(integrity_task.duplicate_detection()?);
        tasks_run += 1;
        all_issues.extend(integrity_task.tag_normalization()?);
        tasks_run += 1;

        // 质量任务
        let quality_task = QualityTask::new(&self.vault_path);
        all_issues.extend(quality_task.consistency_check()?);
        tasks_run += 1;
        all_issues.extend(quality_task.staleness_review()?);
        tasks_run += 1;

        let issues_found = all_issues.len() as u32;
        Ok(FreshnessReport {
            tasks_run,
            issues_found,
            issues_fixed: 0,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// 生成保鲜报告
    pub fn generate_report(&self) -> Result<FreshnessReport> {
        self.run_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_freshness_engine_new() {
        let tmp = tempfile::tempdir().unwrap();
        let engine = FreshnessEngine::new(tmp.path().to_path_buf());
        assert_eq!(engine.vault_path(), &tmp.path().to_path_buf());
    }

    #[test]
    fn test_run_sync_tasks_on_empty_vault() {
        let tmp = tempfile::tempdir().unwrap();
        let engine = FreshnessEngine::new(tmp.path().to_path_buf());
        let report = engine.run_sync_tasks().unwrap();
        assert_eq!(report.tasks_run, 2);
        assert_eq!(report.issues_found, 0);
    }

    #[test]
    fn test_run_integrity_tasks_on_empty_vault() {
        let tmp = tempfile::tempdir().unwrap();
        let engine = FreshnessEngine::new(tmp.path().to_path_buf());
        let report = engine.run_integrity_tasks().unwrap();
        assert_eq!(report.tasks_run, 3);
    }

    #[test]
    fn test_run_quality_tasks_on_empty_vault() {
        let tmp = tempfile::tempdir().unwrap();
        let engine = FreshnessEngine::new(tmp.path().to_path_buf());
        let report = engine.run_quality_tasks().unwrap();
        assert_eq!(report.tasks_run, 2);
    }

    #[test]
    fn test_run_all_aggregates_tasks() {
        let tmp = tempfile::tempdir().unwrap();
        let engine = FreshnessEngine::new(tmp.path().to_path_buf());
        let report = engine.run_all().unwrap();
        assert_eq!(report.tasks_run, 7);
    }

    #[test]
    fn test_generate_report_equals_run_all() {
        let tmp = tempfile::tempdir().unwrap();
        let engine = FreshnessEngine::new(tmp.path().to_path_buf());
        let report = engine.generate_report().unwrap();
        assert_eq!(report.tasks_run, 7);
    }
}
