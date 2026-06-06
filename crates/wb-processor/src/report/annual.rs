//! Annual —— 年报生成

use chrono::{Datelike, NaiveDate};

use wb_core::record::{Category, WorkRecord};

use super::count_by_status;
use super::Report;

/// 成长轨迹点
#[derive(Debug, Clone, PartialEq)]
pub struct GrowthPoint {
    pub period: String,
    pub metric: String,
    pub value: f64,
}

/// 年报结构体
#[derive(Debug, Clone, PartialEq)]
pub struct AnnualReport {
    pub year: i32,
    pub panoramic_summary: String,
    pub growth_trajectory: Vec<GrowthPoint>,
    pub next_year_plan: Vec<String>,
    pub highlights: Vec<String>,
}

/// 生成年报
///
/// 汇总整年的 WorkRecord 数据，生成全景式年度报告。
pub fn generate_annual(year: i32, records: &[WorkRecord]) -> Report {
    let period_start = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
    let period_end = NaiveDate::from_ymd_opt(year, 12, 31).unwrap();

    // 按状态统计
    let done_count = count_by_status(records, &["done", "completed", "完成", "已完成"]);
    let blocked_count = count_by_status(records, &["blocked", "阻塞"]);
    let total = records.len();

    // 按 category 统计
    let mut category_counts: std::collections::HashMap<&str, usize> =
        std::collections::HashMap::new();
    for r in records {
        let key = match r.category {
            Category::Task => "任务",
            Category::Meeting => "会议",
            Category::Communication => "沟通",
            Category::Research => "调研",
            Category::Review => "评审",
            Category::Planning => "规划",
            Category::Document => "文档",
            Category::Decision => "决策",
        };
        *category_counts.entry(key).or_insert(0) += 1;
    }

    // 全景总结
    let panoramic_summary = build_panoramic_summary(
        year, total, done_count, blocked_count, &category_counts,
    );

    // 成长轨迹（按季度统计）
    let growth_trajectory = build_growth_trajectory(records, year);

    // 亮点
    let highlights = build_highlights(records, &category_counts);

    // 次年计划（基于年末进行中记录推断）
    let next_year_plan = build_next_year_plan(records);

    let annual_report = AnnualReport {
        year,
        panoramic_summary: panoramic_summary.clone(),
        growth_trajectory,
        next_year_plan,
        highlights,
    };

    let content = render_annual_markdown(&annual_report, &category_counts);

    Report::new(
        super::ReportType::Annual,
        format!("{}年 年报", year),
        content,
        period_start,
        period_end,
    )
}

/// 构建全景总结
fn build_panoramic_summary(
    year: i32,
    total: usize,
    done_count: usize,
    blocked_count: usize,
    category_counts: &std::collections::HashMap<&str, usize>,
) -> String {
    if total == 0 {
        return format!("{}年暂无工作记录。", year);
    }

    let completion_rate = (done_count as f64 / total as f64 * 100.0).round() as u32;
    let mut summary = format!(
        "{}年共处理 {} 项工作，完成 {} 项（{}%）。",
        year, total, done_count, completion_rate
    );

    if blocked_count > 0 {
        summary.push_str(&format!(" 全年 {} 项受阻。", blocked_count));
    }

    // Top 2 categories
    let mut sorted: Vec<_> = category_counts.iter().collect();
    sorted.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
    if let Some((cat, count)) = sorted.first() {
        summary.push_str(&format!(" 主要领域「{}」（{} 项）", cat, count));
    }
    if let Some((cat, count)) = sorted.get(1) {
        summary.push_str(&format!("、「{}」（{} 项）", cat, count));
    }
    summary.push('。');

    summary
}

/// 构建成长轨迹（按季度统计每月完成数）
fn build_growth_trajectory(records: &[WorkRecord], year: i32) -> Vec<GrowthPoint> {
    let mut points = Vec::new();

    for quarter in 1..=4u32 {
        let start_month = (quarter - 1) * 3 + 1;
        let mut quarter_done = 0usize;
        let mut quarter_total = 0usize;

        for offset in 0..3u32 {
            let month = start_month + offset;
            let month_records: Vec<&WorkRecord> = records
                .iter()
                .filter(|r| {
                    let d = r.created_at.date_naive();
                    d.year() == year && d.month() == month
                })
                .collect();

            quarter_total += month_records.len();
            quarter_done += month_records
                .iter()
                .filter(|r| {
                    r.task_status
                        .as_ref()
                        .map(|s| {
                            let lower = s.to_lowercase();
                            lower.contains("done")
                                || lower.contains("completed")
                                || lower.contains("完成")
                                || lower.contains("已完成")
                        })
                        .unwrap_or(false)
                })
                .count();
        }

        let rate = if quarter_total > 0 {
            (quarter_done as f64 / quarter_total as f64 * 100.0).round()
        } else {
            0.0
        };

        points.push(GrowthPoint {
            period: format!("Q{}", quarter),
            metric: "完成率".to_string(),
            value: rate,
        });

        points.push(GrowthPoint {
            period: format!("Q{}", quarter),
            metric: "总产出".to_string(),
            value: quarter_total as f64,
        });
    }

    points
}

/// 构建年度亮点
fn build_highlights(
    records: &[WorkRecord],
    category_counts: &std::collections::HashMap<&str, usize>,
) -> Vec<String> {
    let mut highlights = Vec::new();

    let done_count = count_by_status(records, &["done", "completed", "完成", "已完成"]);
    let total = records.len();

    if total > 0 {
        let rate = (done_count as f64 / total as f64 * 100.0).round() as u32;
        highlights.push(format!("年度完成率 {}%（{}/{}）", rate, done_count, total));
    }

    // 项目数
    let project_count: usize = records
        .iter()
        .filter_map(|r| r.project.as_ref())
        .collect::<std::collections::HashSet<_>>()
        .len();
    if project_count > 0 {
        highlights.push(format!("参与 {} 个项目", project_count));
    }

    // 最活跃 category
    if let Some((cat, count)) = category_counts.iter().max_by_key(|(_, c)| *c) {
        highlights.push(format!("最活跃领域「{}」共 {} 项", cat, count));
    }

    // 每季度平均产出
    let avg_per_quarter = total as f64 / 4.0;
    highlights.push(format!("季度平均产出 {:.0} 项", avg_per_quarter));

    highlights
}

/// 构建次年计划（基于年末进行中记录）
fn build_next_year_plan(records: &[WorkRecord]) -> Vec<String> {
    // 取 Q4 进行中的记录作为延续计划
    let q4_in_progress: Vec<&WorkRecord> = records
        .iter()
        .filter(|r| {
            let d = r.created_at.date_naive();
            d.month() >= 10
                && d.month() <= 12
                && r.task_status
                    .as_ref()
                    .map(|s| {
                        let lower = s.to_lowercase();
                        lower.contains("in_progress")
                            || lower.contains("doing")
                            || lower.contains("进行中")
                    })
                    .unwrap_or(false)
        })
        .collect();

    if q4_in_progress.is_empty() {
        return vec!["待定".to_string()];
    }

    let mut plan: Vec<String> = q4_in_progress
        .iter()
        .take(5)
        .map(|r| format!("延续：{}", r.title))
        .collect();

    plan.push("根据年度复盘调整新目标".to_string());
    plan
}

/// 渲染年报 Markdown
fn render_annual_markdown(
    report: &AnnualReport,
    category_counts: &std::collections::HashMap<&str, usize>,
) -> String {
    let mut md = String::new();

    md.push_str(&format!("# {}年 年报\n\n", report.year));

    // 全景总结
    md.push_str("## 全景总结\n\n");
    md.push_str(&report.panoramic_summary);
    md.push_str("\n\n");

    // 成长轨迹
    md.push_str("## 成长轨迹\n\n");
    if report.growth_trajectory.is_empty() {
        md.push_str("_暂无数据_\n\n");
    } else {
        md.push_str("| 季度 | 指标 | 数值 |\n|------|------|------|\n");
        for point in &report.growth_trajectory {
            md.push_str(&format!(
                "| {} | {} | {} |\n",
                point.period, point.metric, point.value
            ));
        }
        md.push('\n');
    }

    // 年度亮点
    md.push_str("## 年度亮点\n\n");
    for highlight in &report.highlights {
        md.push_str(&format!("- {}\n", highlight));
    }
    md.push('\n');

    // 分类统计
    md.push_str("## 时间分配\n\n");
    if category_counts.is_empty() {
        md.push_str("_暂无记录_\n\n");
    } else {
        md.push_str("| 分类 | 数量 |\n|------|------|\n");
        let mut sorted: Vec<_> = category_counts.iter().collect();
        sorted.sort_by_key(|(k, _)| *k);
        for (cat, count) in sorted {
            md.push_str(&format!("| {} | {} |\n", cat, count));
        }
        md.push('\n');
    }

    // 次年计划
    md.push_str("## 次年计划\n\n");
    for plan in &report.next_year_plan {
        md.push_str(&format!("- {}\n", plan));
    }
    md.push('\n');

    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use wb_core::record::Category;

    fn make_record(title: &str, status: Option<&str>, category: Category) -> WorkRecord {
        let mut r = WorkRecord::new(
            title.to_string(),
            format!("{} summary", title),
            format!("{} detail", title),
            category,
            vec![],
            "test".to_string(),
            0.9,
        );
        r.task_status = status.map(|s| s.to_string());
        r
    }

    fn make_record_with_date(
        title: &str,
        status: Option<&str>,
        category: Category,
        year: i32,
        month: u32,
        day: u32,
    ) -> WorkRecord {
        let mut r = make_record(title, status, category);
        r.created_at = Utc.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap();
        r
    }

    fn make_record_with_project(
        title: &str,
        status: Option<&str>,
        category: Category,
        project: &str,
    ) -> WorkRecord {
        let mut r = make_record(title, status, category);
        r.project = Some(project.to_string());
        r
    }

    #[test]
    fn test_generate_annual_basic() {
        let records = vec![
            make_record("T1", Some("done"), Category::Task),
            make_record("T2", Some("in_progress"), Category::Task),
        ];
        let report = generate_annual(2026, &records);
        assert!(report.title.contains("2026年"));
        assert!(report.title.contains("年报"));
        assert_eq!(
            report.period_start,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
        );
        assert_eq!(
            report.period_end,
            NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()
        );
        assert!(report.content.contains("全景总结"));
        assert!(report.content.contains("成长轨迹"));
        assert!(report.content.contains("年度亮点"));
        assert!(report.content.contains("次年计划"));
    }

    #[test]
    fn test_generate_annual_empty() {
        let report = generate_annual(2026, &[]);
        assert!(report.content.contains("暂无工作记录"));
    }

    #[test]
    fn test_panoramic_summary_with_records() {
        let mut counts = std::collections::HashMap::new();
        counts.insert("任务", 50);
        counts.insert("会议", 20);
        let summary = build_panoramic_summary(2026, 100, 80, 5, &counts);
        assert!(summary.contains("2026年"));
        assert!(summary.contains("100 项"));
        assert!(summary.contains("80 项"));
        assert!(summary.contains("80%"));
        assert!(summary.contains("5 项受阻"));
        assert!(summary.contains("任务"));
    }

    #[test]
    fn test_panoramic_summary_empty() {
        let counts = std::collections::HashMap::new();
        let summary = build_panoramic_summary(2026, 0, 0, 0, &counts);
        assert!(summary.contains("暂无工作记录"));
    }

    #[test]
    fn test_growth_trajectory() {
        let records = vec![
            make_record_with_date("Q1T", Some("done"), Category::Task, 2026, 2, 10),
            make_record_with_date("Q2T", Some("done"), Category::Task, 2026, 5, 15),
            make_record_with_date("Q3T", Some("in_progress"), Category::Task, 2026, 8, 20),
            make_record_with_date("Q4T", Some("done"), Category::Task, 2026, 11, 5),
        ];
        let trajectory = build_growth_trajectory(&records, 2026);
        assert_eq!(trajectory.len(), 8); // 4 quarters * 2 metrics
        assert!(trajectory.iter().any(|p| p.period == "Q1" && p.metric == "完成率"));
        assert!(trajectory.iter().any(|p| p.period == "Q4" && p.value == 100.0));
    }

    #[test]
    fn test_highlights() {
        let records = vec![
            make_record_with_project("T1", Some("done"), Category::Task, "Alpha"),
            make_record_with_project("T2", Some("done"), Category::Task, "Alpha"),
            make_record_with_project("T3", Some("done"), Category::Meeting, "Beta"),
        ];
        let mut counts = std::collections::HashMap::new();
        counts.insert("任务", 2);
        counts.insert("会议", 1);
        let highlights = build_highlights(&records, &counts);
        assert!(highlights.iter().any(|h| h.contains("100%")));
        assert!(highlights.iter().any(|h| h.contains("2 个项目")));
        assert!(highlights.iter().any(|h| h.contains("任务")));
    }

    #[test]
    fn test_next_year_plan_with_q4_records() {
        let records = vec![
            make_record_with_date("Q4 Continue", Some("in_progress"), Category::Task, 2026, 10, 15),
            make_record_with_date("Q4 Done", Some("done"), Category::Task, 2026, 11, 1),
        ];
        let plan = build_next_year_plan(&records);
        assert!(plan.iter().any(|p| p.contains("Q4 Continue")));
        assert!(plan.iter().any(|p| p.contains("根据年度复盘")));
    }

    #[test]
    fn test_next_year_plan_empty() {
        let plan = build_next_year_plan(&[]);
        assert_eq!(plan, vec!["待定"]);
    }

    #[test]
    fn test_annual_report_struct_fields() {
        let report = AnnualReport {
            year: 2026,
            panoramic_summary: "test summary".to_string(),
            growth_trajectory: vec![],
            next_year_plan: vec![],
            highlights: vec!["h1".to_string()],
        };
        assert_eq!(report.year, 2026);
        assert_eq!(report.highlights.len(), 1);
    }
}
