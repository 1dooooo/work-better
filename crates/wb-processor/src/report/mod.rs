//! Report —— 报告生成系统
//!
//! 从 WorkRecord 聚合数据，生成日报 / 周报 / 月报 / 季报 / 半年报 / 年报，
//! 支持模板管理、确认流程和报告导出。

pub mod annual;
pub mod confirm;
pub mod daily;
pub mod export;
pub mod monthly;
pub mod quarterly;
pub mod semi_annual;
pub mod template;
pub mod weekly;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use confirm::ReportStatus;
use wb_core::record::WorkRecord;

/// 报告类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportType {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    SemiAnnual,
    Annual,
}

/// 报告结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: String,
    pub report_type: ReportType,
    pub title: String,
    pub content: String, // Markdown
    pub status: ReportStatus,
    pub generated_at: DateTime<Utc>,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
}

impl Report {
    /// 创建新报告
    pub fn new(
        report_type: ReportType,
        title: String,
        content: String,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            report_type,
            title,
            content,
            status: ReportStatus::Draft,
            generated_at: Utc::now(),
            period_start,
            period_end,
        }
    }
}

/// 报告生成器
///
/// 汇总 WorkRecord，按类型生成日报 / 周报 / 月报 / 季报 / 半年报 / 年报。
pub struct ReportGenerator;

impl ReportGenerator {
    pub fn new() -> Self {
        Self
    }

    /// 生成日报
    pub fn generate_daily(date: NaiveDate, records: &[WorkRecord]) -> Report {
        daily::generate_daily(date, records)
    }

    /// 生成周报（week_start 为该周周一）
    pub fn generate_week(week_start: NaiveDate, records: &[WorkRecord]) -> Report {
        weekly::generate_week(week_start, records)
    }

    /// 生成月报
    pub fn generate_month(year: i32, month: u32, records: &[WorkRecord]) -> Report {
        monthly::generate_month(year, month, records)
    }

    /// 生成季报（quarter: 1-4）
    pub fn generate_quarter(year: i32, quarter: u32, records: &[WorkRecord]) -> Report {
        quarterly::generate_quarter(year, quarter, records)
    }

    /// 生成半年报（half: 1=上半年, 2=下半年）
    pub fn generate_semi_annual(year: i32, half: u32, records: &[WorkRecord]) -> Report {
        semi_annual::generate_semi_annual(year, half, records)
    }

    /// 生成年报
    pub fn generate_annual(year: i32, records: &[WorkRecord]) -> Report {
        annual::generate_annual(year, records)
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wb_core::record::Category;

    fn make_record(title: &str, category: Category, task_status: Option<&str>) -> WorkRecord {
        let mut r = WorkRecord::new(
            title.to_string(),
            format!("{} summary", title),
            format!("{} detail", title),
            category,
            vec![],
            "test".to_string(),
            0.9,
        );
        r.task_status = task_status.map(|s| s.to_string());
        r
    }

    #[test]
    fn test_report_new_has_id_and_draft_status() {
        let report = Report::new(
            ReportType::Daily,
            "Test".to_string(),
            "content".to_string(),
            NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        );
        assert!(!report.id.is_empty());
        assert_eq!(report.status, ReportStatus::Draft);
        assert_eq!(report.report_type, ReportType::Daily);
    }

    #[test]
    fn test_report_generator_default() {
        let _gen = ReportGenerator::default();
    }

    #[test]
    fn test_generate_daily_delegates() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 6).unwrap();
        let records = vec![make_record("Task A", Category::Task, Some("done"))];
        let report = ReportGenerator::generate_daily(date, &records);
        assert_eq!(report.report_type, ReportType::Daily);
        assert_eq!(report.period_start, date);
    }

    #[test]
    fn test_generate_week_delegates() {
        let week_start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let records = vec![make_record("Task B", Category::Task, None)];
        let report = ReportGenerator::generate_week(week_start, &records);
        assert_eq!(report.report_type, ReportType::Weekly);
    }

    #[test]
    fn test_generate_month_delegates() {
        let records = vec![make_record("Task C", Category::Meeting, None)];
        let report = ReportGenerator::generate_month(2026, 6, &records);
        assert_eq!(report.report_type, ReportType::Monthly);
    }
}
