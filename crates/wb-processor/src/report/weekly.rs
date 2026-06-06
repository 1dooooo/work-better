//! Weekly —— 周报生成

use chrono::NaiveDate;

use wb_core::record::{Category, WorkRecord};

use super::{is_done_status, Report};

/// 生成周报
///
/// 从 week_start（周一）到 week_start + 6 天（周日）的 WorkRecord 中聚合数据。
/// 内容包含：周进度、关键成果、下周计划、风险。
pub fn generate_week(week_start: NaiveDate, records: &[WorkRecord]) -> Report {
    let week_end = week_start + chrono::Days::new(6);

    // 分类统计
    let done_records: Vec<&WorkRecord> = records
        .iter()
        .filter(|r| r.task_status.as_ref().map(|s| is_done_status(s)).unwrap_or(false))
        .collect();

    let blocked_records: Vec<&WorkRecord> = records
        .iter()
        .filter(|r| {
            r.task_status
                .as_ref()
                .map(|s| {
                    let lower = s.to_lowercase();
                    lower.contains("blocked") || lower.contains("阻塞")
                })
                .unwrap_or(false)
        })
        .collect();

    let in_progress_records: Vec<&WorkRecord> = records
        .iter()
        .filter(|r| {
            r.task_status
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

    let content = render_weekly_markdown(
        week_start,
        week_end,
        records,
        &done_records,
        &in_progress_records,
        &blocked_records,
        &category_counts,
    );

    Report::new(
        super::ReportType::Weekly,
        format!(
            "周报：{} ~ {}",
            week_start.format("%Y-%m-%d"),
            week_end.format("%Y-%m-%d")
        ),
        content,
        week_start,
        week_end,
    )
}

fn render_weekly_markdown(
    week_start: NaiveDate,
    week_end: NaiveDate,
    all: &[WorkRecord],
    done: &[&WorkRecord],
    in_progress: &[&WorkRecord],
    blocked: &[&WorkRecord],
    category_counts: &std::collections::HashMap<&str, usize>,
) -> String {
    let mut md = String::new();

    md.push_str(&format!(
        "# 周报：{} ~ {}\n\n",
        week_start.format("%Y-%m-%d"),
        week_end.format("%Y-%m-%d")
    ));

    // 本周进度
    md.push_str("## 本周进度\n\n");
    md.push_str(&format!(
        "共处理 **{}** 条记录，完成 **{}** 条，进行中 **{}** 条。\n\n",
        all.len(),
        done.len(),
        in_progress.len()
    ));

    // 分类统计
    if !category_counts.is_empty() {
        md.push_str("### 分类统计\n\n");
        md.push_str("| 分类 | 数量 |\n|------|------|\n");
        let mut sorted: Vec<_> = category_counts.iter().collect();
        sorted.sort_by_key(|(k, _)| *k);
        for (cat, count) in sorted {
            md.push_str(&format!("| {} | {} |\n", cat, count));
        }
        md.push('\n');
    }

    // 关键成果
    md.push_str("## 关键成果\n\n");
    if done.is_empty() {
        md.push_str("_本周暂无完成事项_\n\n");
    } else {
        for r in done {
            md.push_str(&format!("- **{}**：{}\n", r.title, r.summary));
        }
        md.push('\n');
    }

    // 下周计划
    md.push_str("## 下周计划\n\n");
    if in_progress.is_empty() {
        md.push_str("_待定_\n\n");
    } else {
        for r in in_progress {
            md.push_str(&format!("- 继续推进：{}\n", r.title));
        }
        md.push('\n');
    }

    // 风险与阻塞
    md.push_str("## 风险与阻塞\n\n");
    if blocked.is_empty() {
        md.push_str("_无风险_\n\n");
    } else {
        for r in blocked {
            md.push_str(&format!("- **{}**：{}\n", r.title, r.summary));
        }
        md.push('\n');
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

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

    #[test]
    fn test_generate_week_basic() {
        let week_start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(); // Monday
        let records = vec![
            make_record("Task A", Some("done"), Category::Task),
            make_record("Meeting B", Some("completed"), Category::Meeting),
        ];
        let report = generate_week(week_start, &records);
        assert!(report.title.starts_with("周报"));
        assert_eq!(report.period_start, week_start);
        assert_eq!(report.period_end, week_start + chrono::Days::new(6));
        assert!(report.content.contains("关键成果"));
        assert!(report.content.contains("Task A"));
    }

    #[test]
    fn test_generate_week_empty() {
        let week_start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let report = generate_week(week_start, &[]);
        assert!(report.content.contains("共处理 **0** 条"));
        assert!(report.content.contains("_本周暂无完成事项_"));
    }

    #[test]
    fn test_generate_week_category_stats() {
        let week_start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let records = vec![
            make_record("T1", None, Category::Task),
            make_record("T2", None, Category::Task),
            make_record("M1", None, Category::Meeting),
        ];
        let report = generate_week(week_start, &records);
        assert!(report.content.contains("任务"));
        assert!(report.content.contains("会议"));
    }

    #[test]
    fn test_generate_week_blocked_section() {
        let week_start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let records = vec![make_record("Blocked", Some("blocked"), Category::Task)];
        let report = generate_week(week_start, &records);
        assert!(report.content.contains("风险与阻塞"));
        assert!(report.content.contains("Blocked"));
    }

    #[test]
    fn test_generate_week_report_type() {
        let week_start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let report = generate_week(week_start, &[]);
        assert_eq!(report.report_type, super::super::ReportType::Weekly);
    }
}
