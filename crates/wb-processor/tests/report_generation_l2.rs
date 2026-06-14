//! 报告生成 L2 集成测试
//!
//! 测试场景：验证完整的报告生成链路
//! - 获取 WorkRecord → 聚合数据 → 生成 Markdown

use chrono::NaiveDate;
use wb_core::record::{WorkRecord, Category};
use wb_processor::report::{ReportGenerator, ReportType};

/// 创建测试 WorkRecord
fn create_test_record(title: &str, status: &str) -> WorkRecord {
    WorkRecord {
        id: uuid::Uuid::new_v4().to_string(),
        created_at: chrono::Utc::now(),
        source_event_ids: vec![uuid::Uuid::new_v4().to_string()],
        title: title.to_string(),
        summary: format!("{}的摘要", title),
        detail: format!("{}的详情", title),
        category: Category::Task,
        project: Some("测试项目".to_string()),
        people: vec!["user-001".to_string()],
        tags: vec!["test".to_string()],
        task_status: Some(status.to_string()),
        task_due: None,
        task_priority: None,
        task_progress: None,
        model_used: "test-model".to_string(),
        confidence: 0.9,
        needs_review: false,
        obsidian_path: "test.md".to_string(),
    }
}

/// 测试日报生成
#[test]
fn test_daily_report_generation() {
    // 准备测试数据
    let records = vec![
        create_test_record("完成项目设计", "done"),
        create_test_record("代码审查", "completed"),
        create_test_record("待办任务", "todo"),
    ];

    // 生成报告
    let today = NaiveDate::from_ymd_opt(2024, 6, 6).unwrap();
    let report = ReportGenerator::generate_daily(today, &records);

    // 验证报告
    assert_eq!(report.report_type, ReportType::Daily);
    assert!(!report.content.is_empty());
    assert!(report.content.contains("完成项目设计") || report.content.contains("日报"));
}

/// 测试周报生成
#[test]
fn test_weekly_report_generation() {
    let records = vec![
        create_test_record("sprint 目标 1", "done"),
        create_test_record("sprint 目标 2", "completed"),
    ];

    let week_start = NaiveDate::from_ymd_opt(2024, 6, 3).unwrap();
    let report = ReportGenerator::generate_week(week_start, &records);

    assert_eq!(report.report_type, ReportType::Weekly);
    assert!(!report.content.is_empty());
    assert!(report.content.contains("本周进展") || report.content.contains("周报"));
}

/// 测试月报生成
#[test]
fn test_monthly_report_generation() {
    let records = vec![
        create_test_record("月度目标 1", "done"),
        create_test_record("月度目标 2", "in_progress"),
    ];

    let report = ReportGenerator::generate_month(2024, 6, &records);

    assert_eq!(report.report_type, ReportType::Monthly);
    assert!(!report.content.is_empty());
    assert!(report.content.contains("目标进展") || report.content.contains("月报"));
}

/// 测试季报生成
#[test]
fn test_quarterly_report_generation() {
    let records = vec![
        create_test_record("OKR 1", "done"),
        create_test_record("OKR 2", "in_progress"),
    ];

    let report = ReportGenerator::generate_quarter(2024, 2, &records);

    assert_eq!(report.report_type, ReportType::Quarterly);
    assert!(!report.content.is_empty());
    assert!(report.content.contains("OKR 进度") || report.content.contains("季报"));
}

/// 测试半年报生成
#[test]
fn test_semi_annual_report_generation() {
    let records = vec![
        create_test_record("半年目标 1", "done"),
        create_test_record("半年目标 2", "completed"),
    ];

    let report = ReportGenerator::generate_semi_annual(2024, 1, &records);

    assert_eq!(report.report_type, ReportType::SemiAnnual);
    assert!(!report.content.is_empty());
    assert!(report.content.contains("阶段性总结") || report.content.contains("半年报"));
}

/// 测试年报生成
#[test]
fn test_annual_report_generation() {
    let records = vec![
        create_test_record("年度目标 1", "done"),
        create_test_record("年度目标 2", "completed"),
    ];

    let report = ReportGenerator::generate_annual(2024, &records);

    assert_eq!(report.report_type, ReportType::Annual);
    assert!(!report.content.is_empty());
    assert!(report.content.contains("年度全景") || report.content.contains("年报"));
}

/// 测试空数据报告生成
#[test]
fn test_empty_data_report_generation() {
    let records = vec![];

    let today = NaiveDate::from_ymd_opt(2024, 6, 6).unwrap();
    let report = ReportGenerator::generate_daily(today, &records);

    // 验证报告仍然可以生成
    assert!(!report.content.is_empty());
    assert_eq!(report.report_type, ReportType::Daily);
}

/// 测试报告格式
#[test]
fn test_report_format() {
    let records = vec![
        create_test_record("测试任务", "done"),
    ];

    let today = NaiveDate::from_ymd_opt(2024, 6, 6).unwrap();
    let report = ReportGenerator::generate_daily(today, &records);

    // 验证报告格式（Markdown）
    assert!(report.content.contains("#") || report.content.contains("##")); // 标题
    assert!(report.content.contains("-") || report.content.contains("*")); // 列表
}

/// 测试报告 ID 唯一性
#[test]
fn test_report_id_uniqueness() {
    let records = vec![
        create_test_record("任务 1", "done"),
    ];

    let today = NaiveDate::from_ymd_opt(2024, 6, 6).unwrap();
    let report1 = ReportGenerator::generate_daily(today, &records);
    let report2 = ReportGenerator::generate_daily(today, &records);

    assert_ne!(report1.id, report2.id);
}

/// 测试报告时间戳
#[test]
fn test_report_timestamp() {
    let records = vec![
        create_test_record("任务 1", "done"),
    ];

    let today = NaiveDate::from_ymd_opt(2024, 6, 6).unwrap();
    let report = ReportGenerator::generate_daily(today, &records);

    assert!(report.generated_at.timestamp() > 0);
}
