//! Daily —— 日报生成

use chrono::NaiveDate;

use wb_core::record::WorkRecord;

use super::Report;

/// 生成日报
///
/// 从指定日期的 WorkRecord 中聚合数据，生成 Markdown 格式日报。
/// 内容包含：完成事项、进行中、明日计划、阻塞项。
pub fn generate_daily(date: NaiveDate, records: &[WorkRecord]) -> Report {
    let completed = filter_by_status(records, &["done", "completed", "已完成"]);
    let in_progress = filter_by_status(records, &["in_progress", "进行中", "doing"]);
    let blocked = filter_by_status(records, &["blocked", "阻塞", "blocked_by"]);

    let content = render_daily_markdown(date, &completed, &in_progress, &blocked);

    Report::new(
        super::ReportType::Daily,
        format!("日报：{}", date.format("%Y-%m-%d")),
        content,
        date,
        date,
    )
}

/// 按 task_status 过滤记录
fn filter_by_status<'a>(records: &'a [WorkRecord], statuses: &[&str]) -> Vec<&'a WorkRecord> {
    records
        .iter()
        .filter(|r| {
            r.task_status
                .as_ref()
                .map(|s| {
                    let lower = s.to_lowercase();
                    statuses.iter().any(|status| lower == *status)
                })
                .unwrap_or(false)
        })
        .collect()
}

/// 渲染日报 Markdown
fn render_daily_markdown(
    date: NaiveDate,
    completed: &[&WorkRecord],
    in_progress: &[&WorkRecord],
    blocked: &[&WorkRecord],
) -> String {
    let mut md = String::new();

    md.push_str(&format!("# 日报：{}\n\n", date.format("%Y-%m-%d")));

    // 完成事项
    md.push_str("## 完成事项\n\n");
    if completed.is_empty() {
        md.push_str("_无_\n\n");
    } else {
        for r in completed {
            md.push_str(&format!("- **{}**：{}\n", r.title, r.summary));
        }
        md.push('\n');
    }

    // 进行中
    md.push_str("## 进行中\n\n");
    if in_progress.is_empty() {
        md.push_str("_无_\n\n");
    } else {
        for r in in_progress {
            let progress = r.task_progress.as_deref().unwrap_or("");
            if progress.is_empty() {
                md.push_str(&format!("- **{}**：{}\n", r.title, r.summary));
            } else {
                md.push_str(&format!(
                    "- **{}**：{}（{}）\n",
                    r.title, r.summary, progress
                ));
            }
        }
        md.push('\n');
    }

    // 明日计划（基于进行中的项目推断）
    md.push_str("## 明日计划\n\n");
    if in_progress.is_empty() && completed.is_empty() {
        md.push_str("_待定_\n\n");
    } else {
        for r in in_progress {
            md.push_str(&format!("- 继续：{}\n", r.title));
        }
        // 可以补充其他计划项
        if in_progress.is_empty() {
            md.push_str("_待定_\n\n");
        } else {
            md.push('\n');
        }
    }

    // 阻塞项
    md.push_str("## 阻塞项\n\n");
    if blocked.is_empty() {
        md.push_str("_无_\n\n");
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
    use crate::report::confirm::ReportStatus;
    use chrono::NaiveDate;
    use wb_core::record::Category;

    fn make_record(title: &str, summary: &str, category: Category, status: Option<&str>) -> WorkRecord {
        let mut r = WorkRecord::new(
            title.to_string(),
            summary.to_string(),
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
    fn test_generate_daily_basic() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 6).unwrap();
        let records = vec![
            make_record("Task A", "完成登录", Category::Task, Some("done")),
            make_record("Task B", "修复样式", Category::Task, Some("in_progress")),
        ];
        let report = generate_daily(date, &records);
        assert_eq!(report.title, "日报：2026-06-06");
        assert!(report.content.contains("完成登录"));
        assert!(report.content.contains("修复样式"));
        assert_eq!(report.period_start, date);
        assert_eq!(report.period_end, date);
    }

    #[test]
    fn test_generate_daily_empty_records() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 6).unwrap();
        let report = generate_daily(date, &[]);
        assert_eq!(report.title, "日报：2026-06-06");
        assert!(report.content.contains("_无_"));
    }

    #[test]
    fn test_generate_daily_categorizes_correctly() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 6).unwrap();
        let records = vec![
            make_record("Done Task", "done", Category::Task, Some("completed")),
            make_record("Blocked Task", "blocked", Category::Task, Some("blocked")),
            make_record("Doing Task", "doing", Category::Task, Some("in_progress")),
        ];
        let report = generate_daily(date, &records);
        assert!(report.content.contains("Done Task"));
        assert!(report.content.contains("Blocked Task"));
        assert!(report.content.contains("Doing Task"));
    }

    #[test]
    fn test_generate_daily_report_status_is_draft() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 6).unwrap();
        let report = generate_daily(date, &[]);
        assert_eq!(report.status, ReportStatus::Draft);
    }

    #[test]
    fn test_generate_daily_with_chinese_status() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 6).unwrap();
        let records = vec![
            make_record("中文完成", "已完成任务", Category::Task, Some("已完成")),
            make_record("中文进行", "进行中任务", Category::Task, Some("进行中")),
            make_record("中文阻塞", "阻塞任务", Category::Task, Some("阻塞")),
        ];
        let report = generate_daily(date, &records);
        assert!(report.content.contains("中文完成"));
        assert!(report.content.contains("中文进行"));
        assert!(report.content.contains("中文阻塞"));
    }

    #[test]
    fn test_generate_daily_with_task_progress() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 6).unwrap();
        let mut record = make_record("Progress Task", "summary", Category::Task, Some("in_progress"));
        record.task_progress = Some("60%".to_string());
        let report = generate_daily(date, &[record]);
        assert!(report.content.contains("60%"));
    }
}
