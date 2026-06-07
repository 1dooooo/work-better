//! Quarterly —— 季报生成

use chrono::{Datelike, NaiveDate};

use wb_core::record::{Category, WorkRecord};

use super::{count_by_status, is_done_status, Report};

/// OKR 条目
#[derive(Debug, Clone, PartialEq)]
pub struct OkrItem {
    pub objective: String,
    pub key_results: Vec<String>,
    pub progress_pct: u32,
}

/// 项目里程碑
#[derive(Debug, Clone, PartialEq)]
pub struct Milestone {
    pub project: String,
    pub milestone: String,
    pub achieved: bool,
}

/// 能力成长指标
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityMetric {
    pub name: String,
    pub baseline: f64,
    pub current: f64,
}

/// 季报结构体
#[derive(Debug, Clone, PartialEq)]
pub struct QuarterlyReport {
    pub period: String,
    pub okr_progress: Vec<OkrItem>,
    pub project_milestones: Vec<Milestone>,
    pub capability_growth: Vec<CapabilityMetric>,
    pub summary: String,
}

/// 生成季报
///
/// 汇总指定季度（quarter 1-4）的 WorkRecord 数据。
/// 内容包含：OKR 进度、项目里程碑、能力成长、总结。
pub fn generate_quarter(year: i32, quarter: u32, records: &[WorkRecord]) -> Report {
    let (period_start, period_end) = quarter_date_range(year, quarter);
    let period = format!("{}-Q{}", year, quarter);

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

    // 提取项目列表
    let projects: Vec<String> = records
        .iter()
        .filter_map(|r| r.project.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    // 构建 OKR 条目（基于 category 分类）
    let okr_progress = build_okr_from_records(records);

    // 构建里程碑（基于已完成记录）
    let project_milestones = build_milestones(records, &projects);

    // 能力成长（基于每月完成趋势）
    let capability_growth = build_capability_metrics(records, year, quarter);

    let summary =
        build_quarterly_summary(&period, total, done_count, blocked_count, &category_counts);

    let quarterly_report = QuarterlyReport {
        period: period.clone(),
        okr_progress,
        project_milestones,
        capability_growth,
        summary: summary.clone(),
    };

    let content = render_quarterly_markdown(&quarterly_report, &category_counts);

    Report::new(
        super::ReportType::Quarterly,
        format!("季报：{}", period),
        content,
        period_start,
        period_end,
    )
}

/// 获取季度的日期范围
fn quarter_date_range(year: i32, quarter: u32) -> (NaiveDate, NaiveDate) {
    let start_month = (quarter - 1) * 3 + 1;
    let end_month = quarter * 3;
    let period_start = NaiveDate::from_ymd_opt(year, start_month, 1).unwrap();
    let period_end = if end_month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - chrono::Days::new(1)
    } else {
        NaiveDate::from_ymd_opt(year, end_month + 1, 1).unwrap() - chrono::Days::new(1)
    };
    (period_start, period_end)
}

/// 从记录构建 OKR 条目
fn build_okr_from_records(records: &[WorkRecord]) -> Vec<OkrItem> {
    let mut objectives: std::collections::HashMap<String, (usize, usize)> =
        std::collections::HashMap::new();

    for r in records {
        let key = match r.category {
            Category::Task => "任务执行",
            Category::Meeting => "沟通协作",
            Category::Communication => "沟通协作",
            Category::Research => "调研探索",
            Category::Review => "质量保障",
            Category::Planning => "战略规划",
            Category::Document => "知识沉淀",
            Category::Decision => "决策管理",
        };

        let entry = objectives.entry(key.to_string()).or_insert((0, 0));
        entry.1 += 1; // total
        if r.task_status
            .as_ref()
            .map(|s| is_done_status(s))
            .unwrap_or(false)
        {
            entry.0 += 1; // done
        }
    }

    objectives
        .into_iter()
        .map(|(obj, (done, total))| {
            let progress_pct = if total > 0 {
                (done as f64 / total as f64 * 100.0).round() as u32
            } else {
                0
            };
            OkrItem {
                objective: obj,
                key_results: vec![format!("完成 {}/{} 项", done, total)],
                progress_pct,
            }
        })
        .collect()
}

/// 从已完成记录构建里程碑
fn build_milestones(records: &[WorkRecord], projects: &[String]) -> Vec<Milestone> {
    projects
        .iter()
        .filter_map(|proj| {
            let proj_records: Vec<&WorkRecord> = records
                .iter()
                .filter(|r| r.project.as_ref() == Some(proj))
                .collect();

            let done_count = proj_records
                .iter()
                .filter(|r| {
                    r.task_status
                        .as_ref()
                        .map(|s| is_done_status(s))
                        .unwrap_or(false)
                })
                .count();

            let total = proj_records.len();
            if total == 0 {
                return None;
            }

            Some(Milestone {
                project: proj.clone(),
                milestone: format!("完成 {}/{} 项工作", done_count, total),
                achieved: done_count == total,
            })
        })
        .collect()
}

/// 构建能力成长指标（按月拆分季度）
fn build_capability_metrics(
    records: &[WorkRecord],
    year: i32,
    quarter: u32,
) -> Vec<CapabilityMetric> {
    let start_month = (quarter - 1) * 3 + 1;
    let mut monthly_done: Vec<usize> = Vec::new();

    for offset in 0..3u32 {
        let month = start_month + offset;
        let month_records: Vec<&WorkRecord> = records
            .iter()
            .filter(|r| {
                let d = r.created_at.date_naive();
                d.year() == year && d.month() == month
            })
            .collect();
        let done = month_records
            .iter()
            .filter(|r| {
                r.task_status
                    .as_ref()
                    .map(|s| is_done_status(s))
                    .unwrap_or(false)
            })
            .count();
        monthly_done.push(done);
    }

    let mut metrics = Vec::new();

    // 产出效率
    let first = *monthly_done.first().unwrap_or(&0) as f64;
    let last = *monthly_done.last().unwrap_or(&0) as f64;
    metrics.push(CapabilityMetric {
        name: "月度产出".to_string(),
        baseline: first,
        current: last,
    });

    // 总产出
    let total_done: usize = monthly_done.iter().sum();
    metrics.push(CapabilityMetric {
        name: "季度总产出".to_string(),
        baseline: 0.0,
        current: total_done as f64,
    });

    metrics
}

/// 构建季度总结文本
fn build_quarterly_summary(
    period: &str,
    total: usize,
    done_count: usize,
    blocked_count: usize,
    category_counts: &std::collections::HashMap<&str, usize>,
) -> String {
    if total == 0 {
        return format!("{} 暂无工作记录。", period);
    }

    let completion_rate = (done_count as f64 / total as f64 * 100.0).round() as u32;
    let mut summary = format!(
        "{} 共处理 {} 项工作，完成 {} 项（{}%）。",
        period, total, done_count, completion_rate
    );

    if blocked_count > 0 {
        summary.push_str(&format!(" {} 项受阻需关注。", blocked_count));
    }

    // 主要投入领域
    if let Some((top_cat, top_count)) = category_counts.iter().max_by_key(|(_, c)| *c) {
        summary.push_str(&format!(
            " 主要投入领域为「{}」（{} 项）。",
            top_cat, top_count
        ));
    }

    summary
}

/// 渲染季报 Markdown
fn render_quarterly_markdown(
    report: &QuarterlyReport,
    category_counts: &std::collections::HashMap<&str, usize>,
) -> String {
    let mut md = String::new();

    md.push_str(&format!("# 季报：{}\n\n", report.period));

    // OKR 进度
    md.push_str("## OKR 进度\n\n");
    if report.okr_progress.is_empty() {
        md.push_str("_暂无 OKR 数据_\n\n");
    } else {
        md.push_str("| 目标 | 关键结果 | 进度 |\n|------|----------|------|\n");
        for okr in &report.okr_progress {
            let kr_str = okr.key_results.join("; ");
            md.push_str(&format!(
                "| {} | {} | {}% |\n",
                okr.objective, kr_str, okr.progress_pct
            ));
        }
        md.push('\n');
    }

    // 项目里程碑
    md.push_str("## 项目里程碑\n\n");
    if report.project_milestones.is_empty() {
        md.push_str("_暂无项目里程碑_\n\n");
    } else {
        for m in &report.project_milestones {
            let status = if m.achieved { "已达成" } else { "进行中" };
            md.push_str(&format!(
                "- **{}**：{} [{}]\n",
                m.project, m.milestone, status
            ));
        }
        md.push('\n');
    }

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

    // 能力成长
    md.push_str("## 能力成长\n\n");
    if report.capability_growth.is_empty() {
        md.push_str("_暂无数据_\n\n");
    } else {
        md.push_str("| 指标 | 基线 | 当前 |\n|------|------|------|\n");
        for metric in &report.capability_growth {
            md.push_str(&format!(
                "| {} | {} | {} |\n",
                metric.name, metric.baseline, metric.current
            ));
        }
        md.push('\n');
    }

    // 总结
    md.push_str("## 总结\n\n");
    md.push_str(&report.summary);
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

    #[test]
    fn test_generate_quarter_basic() {
        let records = vec![
            make_record("T1", Some("done"), Category::Task),
            make_record("T2", Some("in_progress"), Category::Task),
            make_record("T3", Some("blocked"), Category::Task),
        ];
        let report = generate_quarter(2026, 2, &records);
        assert!(report.title.contains("季报"));
        assert!(report.title.contains("2026-Q2"));
        assert_eq!(
            report.period_start,
            NaiveDate::from_ymd_opt(2026, 4, 1).unwrap()
        );
        assert_eq!(
            report.period_end,
            NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()
        );
        assert!(report.content.contains("OKR 进度"));
        assert!(report.content.contains("项目里程碑"));
        assert!(report.content.contains("能力成长"));
    }

    #[test]
    fn test_generate_quarter_empty() {
        let report = generate_quarter(2026, 1, &[]);
        assert!(report.content.contains("暂无 OKR 数据"));
        assert!(report.content.contains("暂无项目里程碑"));
        assert!(report.content.contains("暂无工作记录"));
    }

    #[test]
    fn test_quarter_date_range_q1() {
        let (start, end) = quarter_date_range(2026, 1);
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 3, 31).unwrap());
    }

    #[test]
    fn test_quarter_date_range_q4() {
        let (start, end) = quarter_date_range(2026, 4);
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 10, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 12, 31).unwrap());
    }

    #[test]
    fn test_quarterly_okr_progress() {
        let records = vec![
            make_record("T1", Some("done"), Category::Task),
            make_record("T2", Some("done"), Category::Task),
            make_record("T3", Some("in_progress"), Category::Task),
        ];
        let okr = build_okr_from_records(&records);
        assert!(!okr.is_empty());
        let task_okr = okr.iter().find(|o| o.objective == "任务执行").unwrap();
        assert_eq!(task_okr.progress_pct, 67); // 2/3 = 67%
    }

    #[test]
    fn test_quarterly_milestones_with_projects() {
        let records = vec![
            make_record_with_project("T1", Some("done"), Category::Task, "Alpha"),
            make_record_with_project("T2", Some("in_progress"), Category::Task, "Alpha"),
            make_record_with_project("T3", Some("done"), Category::Task, "Beta"),
        ];
        let projects: Vec<String> = records
            .iter()
            .filter_map(|r| r.project.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        let milestones = build_milestones(&records, &projects);
        assert_eq!(milestones.len(), 2);
        let alpha = milestones.iter().find(|m| m.project == "Alpha").unwrap();
        assert!(!alpha.achieved);
        let beta = milestones.iter().find(|m| m.project == "Beta").unwrap();
        assert!(beta.achieved);
    }

    #[test]
    fn test_quarterly_capability_metrics() {
        let records = vec![
            make_record_with_date("T1", Some("done"), Category::Task, 2026, 4, 5),
            make_record_with_date("T2", Some("done"), Category::Task, 2026, 5, 10),
            make_record_with_date("T3", Some("done"), Category::Task, 2026, 6, 15),
        ];
        let metrics = build_capability_metrics(&records, 2026, 2);
        assert!(!metrics.is_empty());
        let output = metrics.iter().find(|m| m.name == "月度产出").unwrap();
        assert_eq!(output.baseline, 1.0);
        assert_eq!(output.current, 1.0);
    }

    #[test]
    fn test_quarterly_summary_with_records() {
        let mut counts = std::collections::HashMap::new();
        counts.insert("任务", 10);
        counts.insert("会议", 5);
        let summary = build_quarterly_summary("2026-Q2", 15, 10, 2, &counts);
        assert!(summary.contains("2026-Q2"));
        assert!(summary.contains("15 项"));
        assert!(summary.contains("10 项"));
        assert!(summary.contains("67%"));
        assert!(summary.contains("2 项受阻"));
        assert!(summary.contains("任务"));
    }

    #[test]
    fn test_quarterly_summary_empty() {
        let counts = std::collections::HashMap::new();
        let summary = build_quarterly_summary("2026-Q1", 0, 0, 0, &counts);
        assert!(summary.contains("暂无工作记录"));
    }
}
