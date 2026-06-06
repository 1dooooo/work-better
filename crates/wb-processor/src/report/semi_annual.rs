//! Semi-Annual —— 半年报生成

use chrono::NaiveDate;

use wb_core::record::{Category, WorkRecord};

use super::{count_by_status, is_done_status, Report};

/// 目标调整
#[derive(Debug, Clone, PartialEq)]
pub struct GoalAdjustment {
    pub goal: String,
    pub reason: String,
    pub new_direction: String,
}

/// 半年报结构体
#[derive(Debug, Clone, PartialEq)]
pub struct SemiAnnualReport {
    pub period: String,
    pub stage_summary: String,
    pub goal_adjustments: Vec<GoalAdjustment>,
    pub key_achievements: Vec<String>,
    pub next_half_plan: Vec<String>,
}

/// 生成半年报
///
/// 汇总上半年（H1: 1-6月）或下半年（H2: 7-12月）的 WorkRecord 数据。
pub fn generate_semi_annual(year: i32, half: u32, records: &[WorkRecord]) -> Report {
    let (period_start, period_end) = semi_annual_date_range(year, half);
    let half_label = if half == 1 { "H1" } else { "H2" };
    let period = format!("{}-{}", year, half_label);

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

    // 构建阶段总结
    let stage_summary = build_stage_summary(&period, total, done_count, blocked_count, &category_counts);

    // 构建目标调整（基于阻塞和高频 category 推断）
    let goal_adjustments = build_goal_adjustments(records, &category_counts);

    // 关键成就
    let key_achievements = build_key_achievements(records);

    // 下半年计划（基于进行中记录推断）
    let next_half_plan = build_next_half_plan(records);

    let semi_report = SemiAnnualReport {
        period: period.clone(),
        stage_summary: stage_summary.clone(),
        goal_adjustments,
        key_achievements,
        next_half_plan,
    };

    let content = render_semi_annual_markdown(&semi_report, &category_counts);

    Report::new(
        super::ReportType::SemiAnnual,
        format!("半年报：{}", period),
        content,
        period_start,
        period_end,
    )
}

/// 获取半年的日期范围
fn semi_annual_date_range(year: i32, half: u32) -> (NaiveDate, NaiveDate) {
    if half == 1 {
        (
            NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(year, 6, 30).unwrap(),
        )
    } else {
        (
            NaiveDate::from_ymd_opt(year, 7, 1).unwrap(),
            NaiveDate::from_ymd_opt(year, 12, 31).unwrap(),
        )
    }
}

/// 构建阶段总结
fn build_stage_summary(
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
        "{} 期间共处理 {} 项工作，完成 {} 项（{}%）。",
        period, total, done_count, completion_rate
    );

    if blocked_count > 0 {
        summary.push_str(&format!(" {} 项受阻。", blocked_count));
    }

    if let Some((top_cat, top_count)) = category_counts.iter().max_by_key(|(_, c)| *c) {
        summary.push_str(&format!(" 重点投入「{}」（{} 项）。", top_cat, top_count));
    }

    summary
}

/// 构建目标调整建议
fn build_goal_adjustments(
    records: &[WorkRecord],
    category_counts: &std::collections::HashMap<&str, usize>,
) -> Vec<GoalAdjustment> {
    let mut adjustments = Vec::new();

    let blocked_count = count_by_status(records, &["blocked", "阻塞"]);
    let total = records.len();

    // 如果阻塞比例高，建议调整
    if total > 0 && blocked_count as f64 / total as f64 > 0.15 {
        adjustments.push(GoalAdjustment {
            goal: "降低阻塞率".to_string(),
            reason: format!("当前阻塞率 {:.0}%", blocked_count as f64 / total as f64 * 100.0),
            new_direction: "加强风险预判，提前识别依赖关系".to_string(),
        });
    }

    // 如果沟通类记录占比高，建议优化
    let comm_count = category_counts.get("沟通").copied().unwrap_or(0)
        + category_counts.get("会议").copied().unwrap_or(0);
    if total > 0 && comm_count as f64 / total as f64 > 0.4 {
        adjustments.push(GoalAdjustment {
            goal: "优化沟通效率".to_string(),
            reason: format!("沟通/会议类占比 {:.0}%", comm_count as f64 / total as f64 * 100.0),
            new_direction: "减少低效会议，提升异步沟通比例".to_string(),
        });
    }

    adjustments
}

/// 构建关键成就列表
fn build_key_achievements(records: &[WorkRecord]) -> Vec<String> {
    let done_records: Vec<&WorkRecord> = records
        .iter()
        .filter(|r| r.task_status.as_ref().map(|s| is_done_status(s)).unwrap_or(false))
        .collect();

    if done_records.is_empty() {
        return vec!["暂无已完成记录".to_string()];
    }

    // 取前 5 条已完成记录作为关键成就
    done_records
        .iter()
        .take(5)
        .map(|r| format!("{}：{}", r.title, r.summary))
        .collect()
}

/// 构建下阶段计划
fn build_next_half_plan(records: &[WorkRecord]) -> Vec<String> {
    let in_progress: Vec<&WorkRecord> = records
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

    if in_progress.is_empty() {
        return vec!["待定".to_string()];
    }

    in_progress
        .iter()
        .take(5)
        .map(|r| format!("继续推进：{}", r.title))
        .collect()
}

/// 渲染半年报 Markdown
fn render_semi_annual_markdown(
    report: &SemiAnnualReport,
    category_counts: &std::collections::HashMap<&str, usize>,
) -> String {
    let mut md = String::new();

    md.push_str(&format!("# 半年报：{}\n\n", report.period));

    // 阶段总结
    md.push_str("## 阶段总结\n\n");
    md.push_str(&report.stage_summary);
    md.push_str("\n\n");

    // 目标调整
    md.push_str("## 目标调整\n\n");
    if report.goal_adjustments.is_empty() {
        md.push_str("_无需调整_\n\n");
    } else {
        for adj in &report.goal_adjustments {
            md.push_str(&format!("### {}\n\n", adj.goal));
            md.push_str(&format!("- **原因**：{}\n", adj.reason));
            md.push_str(&format!("- **新方向**：{}\n\n", adj.new_direction));
        }
    }

    // 关键成就
    md.push_str("## 关键成就\n\n");
    for achievement in &report.key_achievements {
        md.push_str(&format!("- {}\n", achievement));
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

    // 下阶段计划
    md.push_str("## 下阶段计划\n\n");
    for plan in &report.next_half_plan {
        md.push_str(&format!("- {}\n", plan));
    }
    md.push('\n');

    md
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn test_generate_semi_annual_h1() {
        let records = vec![
            make_record("T1", Some("done"), Category::Task),
            make_record("T2", Some("in_progress"), Category::Task),
        ];
        let report = generate_semi_annual(2026, 1, &records);
        assert!(report.title.contains("半年报"));
        assert!(report.title.contains("2026-H1"));
        assert_eq!(
            report.period_start,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
        );
        assert_eq!(
            report.period_end,
            NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()
        );
        assert!(report.content.contains("阶段总结"));
        assert!(report.content.contains("目标调整"));
        assert!(report.content.contains("关键成就"));
        assert!(report.content.contains("下阶段计划"));
    }

    #[test]
    fn test_generate_semi_annual_h2() {
        let report = generate_semi_annual(2026, 2, &[]);
        assert!(report.title.contains("2026-H2"));
        assert_eq!(
            report.period_start,
            NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()
        );
        assert_eq!(
            report.period_end,
            NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()
        );
    }

    #[test]
    fn test_semi_annual_empty() {
        let report = generate_semi_annual(2026, 1, &[]);
        assert!(report.content.contains("暂无工作记录"));
        assert!(report.content.contains("暂无已完成记录"));
    }

    #[test]
    fn test_semi_annual_goal_adjustments_high_block() {
        let records = vec![
            make_record("B1", Some("blocked"), Category::Task),
            make_record("B2", Some("blocked"), Category::Task),
            make_record("T1", Some("done"), Category::Task),
            make_record("T2", Some("done"), Category::Task),
            make_record("T3", Some("done"), Category::Task),
            make_record("T4", Some("done"), Category::Task),
        ];
        // 2/6 = 33% blocked > 15% threshold
        let adjustments = build_goal_adjustments(&records, &std::collections::HashMap::new());
        assert!(!adjustments.is_empty());
        assert!(adjustments.iter().any(|a| a.goal == "降低阻塞率"));
    }

    #[test]
    fn test_semi_annual_goal_adjustments_high_comm() {
        let records = vec![
            make_record("M1", None, Category::Meeting),
            make_record("M2", None, Category::Meeting),
            make_record("C1", None, Category::Communication),
            make_record("T1", None, Category::Task),
        ];
        let mut counts = std::collections::HashMap::new();
        counts.insert("会议", 2);
        counts.insert("沟通", 1);
        counts.insert("任务", 1);
        // 3/4 = 75% comm > 40% threshold
        let adjustments = build_goal_adjustments(&records, &counts);
        assert!(adjustments.iter().any(|a| a.goal == "优化沟通效率"));
    }

    #[test]
    fn test_semi_annual_key_achievements() {
        let records = vec![
            make_record("Achievement 1", Some("done"), Category::Task),
            make_record("Achievement 2", Some("completed"), Category::Task),
            make_record("Ongoing", Some("in_progress"), Category::Task),
        ];
        let achievements = build_key_achievements(&records);
        assert_eq!(achievements.len(), 2);
        assert!(achievements[0].contains("Achievement 1"));
    }

    #[test]
    fn test_semi_annual_next_half_plan() {
        let records = vec![
            make_record("Continue A", Some("in_progress"), Category::Task),
            make_record("Continue B", Some("doing"), Category::Task),
        ];
        let plan = build_next_half_plan(&records);
        assert_eq!(plan.len(), 2);
        assert!(plan[0].contains("Continue A"));
    }

    #[test]
    fn test_semi_annual_next_half_plan_empty() {
        let plan = build_next_half_plan(&[]);
        assert_eq!(plan, vec!["待定"]);
    }

    #[test]
    fn test_semi_annual_date_range() {
        let (s1, e1) = semi_annual_date_range(2026, 1);
        assert_eq!(s1, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert_eq!(e1, NaiveDate::from_ymd_opt(2026, 6, 30).unwrap());

        let (s2, e2) = semi_annual_date_range(2026, 2);
        assert_eq!(s2, NaiveDate::from_ymd_opt(2026, 7, 1).unwrap());
        assert_eq!(e2, NaiveDate::from_ymd_opt(2026, 12, 31).unwrap());
    }
}
