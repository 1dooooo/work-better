//! Monthly —— 月报生成

use chrono::NaiveDate;

use wb_core::record::{Category, WorkRecord};

use super::count_by_status;
use super::Report;

/// 生成月报
///
/// 从指定年月的 WorkRecord 中聚合数据。
/// 内容包含：目标进度、时间分配、效率趋势。
pub fn generate_month(year: i32, month: u32, records: &[WorkRecord]) -> Report {
    let period_start = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let period_end = last_day_of_month(year, month);

    // 按 category 统计时间分配
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

    // 状态统计
    let done_count = count_by_status(records, &["done", "completed", "完成", "已完成"]);
    let blocked_count = count_by_status(records, &["blocked", "阻塞"]);
    let total = records.len();

    // 按周统计效率（每周完成数）
    let weekly_done = group_by_week(records, true);
    let weekly_total = group_by_week(records, false);

    let content = render_monthly_markdown(
        year,
        month,
        total,
        done_count,
        blocked_count,
        &category_counts,
        &weekly_done,
        &weekly_total,
    );

    Report::new(
        super::ReportType::Monthly,
        format!("月报：{}年{}月", year, month),
        content,
        period_start,
        period_end,
    )
}

/// 获取指定年月的最后一天
fn last_day_of_month(year: i32, month: u32) -> NaiveDate {
    if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - chrono::Days::new(1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap() - chrono::Days::new(1)
    }
}

/// 按 ISO 周分组，返回每周的记录数
fn group_by_week(records: &[WorkRecord], done_only: bool) -> Vec<(String, usize)> {
    let mut map: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for r in records {
        if done_only {
            let is_done = r
                .task_status
                .as_ref()
                .map(|s| {
                    let lower = s.to_lowercase();
                    lower.contains("done")
                        || lower.contains("completed")
                        || lower.contains("完成")
                        || lower.contains("已完成")
                })
                .unwrap_or(false);
            if !is_done {
                continue;
            }
        }
        let week_key = format!("W{}", r.created_at.format("%V"));
        *map.entry(week_key).or_insert(0) += 1;
    }
    map.into_iter().collect()
}

#[allow(clippy::too_many_arguments)]
fn render_monthly_markdown(
    year: i32,
    month: u32,
    total: usize,
    done_count: usize,
    blocked_count: usize,
    category_counts: &std::collections::HashMap<&str, usize>,
    weekly_done: &[(String, usize)],
    weekly_total: &[(String, usize)],
) -> String {
    let mut md = String::new();

    md.push_str(&format!("# 月报：{}年{}月\n\n", year, month));

    // 目标进度
    md.push_str("## 目标进度\n\n");
    let completion_rate = if total > 0 {
        (done_count as f64 / total as f64 * 100.0).round() as u32
    } else {
        0
    };
    md.push_str(&format!(
        "- 总记录数：**{}** 条\n- 已完成：**{}** 条（{}%）\n- 阻塞：**{}** 条\n\n",
        total, done_count, completion_rate, blocked_count
    ));

    // 时间分配
    md.push_str("## 时间分配\n\n");
    if category_counts.is_empty() {
        md.push_str("_本月暂无记录_\n\n");
    } else {
        md.push_str("| 分类 | 数量 | 占比 |\n|------|------|------|\n");
        let mut sorted: Vec<_> = category_counts.iter().collect();
        sorted.sort_by_key(|(k, _)| *k);
        for (cat, count) in &sorted {
            let pct = if total > 0 {
                (**count as f64 / total as f64 * 100.0).round() as u32
            } else {
                0
            };
            md.push_str(&format!("| {} | {} | {}% |\n", cat, count, pct));
        }
        md.push('\n');
    }

    // 效率趋势
    md.push_str("## 效率趋势\n\n");
    if weekly_total.is_empty() {
        md.push_str("_本月暂无数据_\n\n");
    } else {
        md.push_str("| 周次 | 总数 | 完成 | 完成率 |\n|------|------|------|--------|\n");
        for (week, total_w) in weekly_total {
            let done_w = weekly_done
                .iter()
                .find(|(w, _)| w == week)
                .map(|(_, d)| *d)
                .unwrap_or(0);
            let rate = if *total_w > 0 {
                (done_w as f64 / *total_w as f64 * 100.0).round() as u32
            } else {
                0
            };
            md.push_str(&format!(
                "| {} | {} | {} | {}% |\n",
                week, total_w, done_w, rate
            ));
        }
        md.push('\n');
    }

    // 总结
    md.push_str("## 总结\n\n");
    if total == 0 {
        md.push_str(&format!("{}年{}月暂无工作记录。\n", year, month));
    } else {
        md.push_str(&format!(
            "{}年{}月共处理 **{}** 条工作记录，完成率 {}%。",
            year, month, total, completion_rate
        ));
        if blocked_count > 0 {
            md.push_str(&format!(" 当前有 **{}** 条阻塞项需关注。", blocked_count));
        }
        md.push('\n');
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone, Utc};

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
        r.created_at = Utc
            .with_ymd_and_hms(year, month, day, 12, 0, 0)
            .unwrap();
        r
    }

    #[test]
    fn test_generate_month_basic() {
        let records = vec![
            make_record("T1", Some("done"), Category::Task),
            make_record("T2", Some("in_progress"), Category::Task),
        ];
        let report = generate_month(2026, 6, &records);
        assert_eq!(report.title, "月报：2026年6月");
        assert_eq!(report.period_start, NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
        assert_eq!(report.period_end, NaiveDate::from_ymd_opt(2026, 6, 30).unwrap());
        assert!(report.content.contains("目标进度"));
        assert!(report.content.contains("时间分配"));
        assert!(report.content.contains("效率趋势"));
    }

    #[test]
    fn test_generate_month_empty() {
        let report = generate_month(2026, 6, &[]);
        assert!(report.content.contains("暂无记录"));
        assert!(report.content.contains("暂无工作记录"));
    }

    #[test]
    fn test_generate_month_completion_rate() {
        let records = vec![
            make_record("T1", Some("done"), Category::Task),
            make_record("T2", Some("done"), Category::Task),
            make_record("T3", Some("in_progress"), Category::Task),
            make_record("T4", Some("blocked"), Category::Task),
        ];
        let report = generate_month(2026, 6, &records);
        // 2/4 = 50%
        assert!(report.content.contains("50%"));
        assert!(report.content.contains("阻塞：**1** 条"));
    }

    #[test]
    fn test_generate_month_december() {
        let report = generate_month(2026, 12, &[]);
        assert_eq!(report.period_end, NaiveDate::from_ymd_opt(2026, 12, 31).unwrap());
    }

    #[test]
    fn test_generate_month_report_type() {
        let report = generate_month(2026, 6, &[]);
        assert_eq!(report.report_type, super::super::ReportType::Monthly);
    }

    #[test]
    fn test_generate_month_weekly_trend() {
        let records = vec![
            make_record_with_date("T1", Some("done"), Category::Task, 2026, 6, 1),
            make_record_with_date("T2", Some("done"), Category::Task, 2026, 6, 8),
            make_record_with_date("T3", Some("in_progress"), Category::Task, 2026, 6, 15),
        ];
        let report = generate_month(2026, 6, &records);
        assert!(report.content.contains("效率趋势"));
        // 应该有 W23, W24, W25 等周次数据
        assert!(report.content.contains("| W"));
    }

    #[test]
    fn test_last_day_of_month_feb_leap() {
        // 2024 是闰年
        assert_eq!(
            last_day_of_month(2024, 2),
            NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()
        );
    }

    #[test]
    fn test_last_day_of_month_feb_non_leap() {
        assert_eq!(
            last_day_of_month(2026, 2),
            NaiveDate::from_ymd_opt(2026, 2, 28).unwrap()
        );
    }
}
