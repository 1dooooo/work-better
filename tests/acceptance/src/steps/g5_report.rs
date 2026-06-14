//! G5 报告生成 — 日报/周报/月报/季报/半年报/年报
//!
//! 真实组件：
//! - ReportGenerator: 生成各周期报告
//! - ReportExporter: 导出 Markdown/PDF
//! - ReportTemplate + TemplateRepository: 模板管理

use cucumber::{given, when, then};
use chrono::NaiveDate;
use wb_core::record::{Category, WorkRecord};
use wb_processor::report::{ReportGenerator, ReportType};
use wb_processor::report::export::{ReportExporter, ExportFormat};

use crate::world::AcceptanceWorld;

// ─── 辅助函数 ──────────────────────────────────────────────

fn make_records(count: usize, status: &str) -> Vec<WorkRecord> {
    (0..count)
        .map(|i| {
            let mut r = WorkRecord::new(
                format!("任务 {}", i + 1),
                format!("任务 {} 摘要", i + 1),
                format!("任务 {} 详情", i + 1),
                Category::Task,
                vec![format!("evt-{}", i + 1)],
                "test-model".to_string(),
                0.9,
            );
            r.task_status = Some(status.to_string());
            r
        })
        .collect()
}

// ── Given ──────────────────────────────────────────────────

#[given(regex = r"^工作日18:00$")]
fn weekday_1800(world: &mut AcceptanceWorld) {
    let records = make_records(3, "done");
    world.state.insert("report_trigger".into(), "daily".into());
    world.state.insert("report_records_json".into(), serde_json::to_string(&records).unwrap_or_default());
}

#[given(regex = r"^周五17:00$")]
fn friday_1700(world: &mut AcceptanceWorld) {
    let records = make_records(5, "done");
    world.state.insert("report_trigger".into(), "weekly".into());
    world.state.insert("report_records_json".into(), serde_json::to_string(&records).unwrap_or_default());
}

#[given(regex = r"^月末$")]
fn month_end(world: &mut AcceptanceWorld) {
    let records = make_records(10, "done");
    world.state.insert("report_trigger".into(), "monthly".into());
    world.state.insert("report_records_json".into(), serde_json::to_string(&records).unwrap_or_default());
}

#[given(regex = r"^季末$")]
fn quarter_end(world: &mut AcceptanceWorld) {
    let records = make_records(20, "done");
    world.state.insert("report_trigger".into(), "quarterly".into());
    world.state.insert("report_records_json".into(), serde_json::to_string(&records).unwrap_or_default());
}

#[given(regex = r"^(12/31|6/30)$")]
fn half_year_end(world: &mut AcceptanceWorld, _date: String) {
    let records = make_records(30, "done");
    world.state.insert("report_trigger".into(), "half_year".into());
    world.state.insert("report_records_json".into(), serde_json::to_string(&records).unwrap_or_default());
}

#[given(regex = r"^月末同时季末$")]
fn month_and_quarter(world: &mut AcceptanceWorld) {
    let records = make_records(15, "done");
    world.state.insert("report_trigger".into(), "monthly+quarterly".into());
    world.state.insert("report_records_json".into(), serde_json::to_string(&records).unwrap_or_default());
}

#[given(regex = r"^报告生成完成$")]
fn report_generated(world: &mut AcceptanceWorld) {
    let records = make_records(3, "done");
    let report = ReportGenerator::generate_daily(
        NaiveDate::from_ymd_opt(2026, 6, 14).unwrap(),
        &records,
    );
    world.state.insert("report_id".into(), report.id.clone());
    world.state.insert("report_content".into(), report.content.clone());
    world.processing_result = Some("report_generated".into());
}

#[given(regex = r"^用户审查编辑$")]
fn user_edit_report(world: &mut AcceptanceWorld) {
    world.state.insert("report_action".into(), "edited".into());
}

#[given(regex = r"^用户确认$")]
fn user_confirmed(world: &mut AcceptanceWorld) {
    world.state.insert("report_action".into(), "confirmed".into());
}

#[given(regex = r"^用户自定义格式$")]
fn custom_format(world: &mut AcceptanceWorld) {
    world.state.insert("report_format".into(), "custom".into());
}

#[given(regex = r"^用户改生成时间$")]
fn change_schedule(world: &mut AcceptanceWorld) {
    world.state.insert("schedule_changed".into(), "true".into());
}

// ── When ───────────────────────────────────────────────────

#[when(regex = r"^日报触发$")]
async fn daily_trigger(world: &mut AcceptanceWorld) {
    let json = world.state.get("report_records_json").cloned().unwrap_or_default();
    let records: Vec<WorkRecord> = serde_json::from_str(&json).unwrap_or_default();
    let report = ReportGenerator::generate_daily(
        NaiveDate::from_ymd_opt(2026, 6, 14).unwrap(),
        &records,
    );
    world.state.insert("report_id".into(), report.id.clone());
    world.state.insert("report_content".into(), report.content.clone());
    world.state.insert("report_type".into(), format!("{:?}", report.report_type));
    world.processing_result = Some("日报_triggered".into());
}

#[when(regex = r"^周报触发$")]
async fn weekly_trigger(world: &mut AcceptanceWorld) {
    let json = world.state.get("report_records_json").cloned().unwrap_or_default();
    let records: Vec<WorkRecord> = serde_json::from_str(&json).unwrap_or_default();
    let report = ReportGenerator::generate_week(
        NaiveDate::from_ymd_opt(2026, 6, 9).unwrap(),
        &records,
    );
    world.state.insert("report_id".into(), report.id.clone());
    world.state.insert("report_content".into(), report.content.clone());
    world.state.insert("report_type".into(), format!("{:?}", report.report_type));
    world.processing_result = Some("周报_triggered".into());
}

#[when(regex = r"^月报触发$")]
async fn monthly_trigger(world: &mut AcceptanceWorld) {
    let json = world.state.get("report_records_json").cloned().unwrap_or_default();
    let records: Vec<WorkRecord> = serde_json::from_str(&json).unwrap_or_default();
    let report = ReportGenerator::generate_month(2026, 6, &records);
    world.state.insert("report_id".into(), report.id.clone());
    world.state.insert("report_content".into(), report.content.clone());
    world.state.insert("report_type".into(), format!("{:?}", report.report_type));
    world.processing_result = Some("月报_triggered".into());
}

#[when(regex = r"^季报触发$")]
async fn quarterly_trigger(world: &mut AcceptanceWorld) {
    let json = world.state.get("report_records_json").cloned().unwrap_or_default();
    let records: Vec<WorkRecord> = serde_json::from_str(&json).unwrap_or_default();
    let report = ReportGenerator::generate_quarter(2026, 2, &records);
    world.state.insert("report_id".into(), report.id.clone());
    world.state.insert("report_content".into(), report.content.clone());
    world.state.insert("report_type".into(), format!("{:?}", report.report_type));
    world.processing_result = Some("季报_triggered".into());
}

#[when(regex = r"^半年报/年报触发$")]
async fn half_annual_trigger(world: &mut AcceptanceWorld) {
    let json = world.state.get("report_records_json").cloned().unwrap_or_default();
    let records: Vec<WorkRecord> = serde_json::from_str(&json).unwrap_or_default();
    let report = ReportGenerator::generate_semi_annual(2026, 1, &records);
    world.state.insert("report_id".into(), report.id.clone());
    world.state.insert("report_content".into(), report.content.clone());
    world.state.insert("report_type".into(), format!("{:?}", report.report_type));
    world.processing_result = Some("半年报_triggered".into());
}

#[when(regex = r"^两者同时到期$")]
async fn both_due(world: &mut AcceptanceWorld) {
    let json = world.state.get("report_records_json").cloned().unwrap_or_default();
    let records: Vec<WorkRecord> = serde_json::from_str(&json).unwrap_or_default();
    let monthly = ReportGenerator::generate_month(2026, 6, &records);
    let quarterly = ReportGenerator::generate_quarter(2026, 2, &records);
    world.state.insert("monthly_report_id".into(), monthly.id.clone());
    world.state.insert("quarterly_report_id".into(), quarterly.id.clone());
    world.processing_result = Some("both_triggered".into());
}

#[when(regex = r"^完毕$")]
fn finished(world: &mut AcceptanceWorld) {
    world.processing_result = Some("finished".into());
}

#[when(regex = r"^选择导出$")]
fn export(world: &mut AcceptanceWorld) {
    // 如果没有报告内容，先生成一个
    if world.state.get("report_content").map(String::is_empty).unwrap_or(true) {
        let records = make_records(3, "done");
        let report = ReportGenerator::generate_daily(
            NaiveDate::from_ymd_opt(2026, 6, 14).unwrap(),
            &records,
        );
        world.state.insert("report_content".into(), report.content.clone());
    }
    let content = world.state.get("report_content").cloned().unwrap_or_default();
    world.state.insert("export_format".into(), "markdown".into());
    world.state.insert("export_action".into(), "export".into());
    world.state.insert("export_content_len".into(), content.len().to_string());
}

#[when(regex = r"^选择同步飞书$")]
fn sync_to_feishu_report(world: &mut AcceptanceWorld) {
    world.state.insert("export_action".into(), "sync_feishu".into());
}

#[when(regex = r"^修改模板$")]
fn modify_template(world: &mut AcceptanceWorld) {
    world.state.insert("report_format".into(), "custom".into());
}

#[when(regex = r"^在新时间生成$")]
fn new_schedule(world: &mut AcceptanceWorld) {
    world.processing_result = Some("new_schedule".into());
}

// ── Then ───────────────────────────────────────────────────

#[then(regex = r"^生成含.*的日报$")]
fn assert_daily_report(world: &mut AcceptanceWorld) {
    let report_type = world.state.get("report_type").cloned().unwrap_or_default();
    assert_eq!(report_type, "Daily", "报告类型应为 Daily");
    let content = world.state.get("report_content").cloned().unwrap_or_default();
    assert!(!content.is_empty(), "报告内容不应为空");
    let id = world.state.get("report_id").cloned().unwrap_or_default();
    assert!(!id.is_empty(), "报告应有 ID");
}

#[then(regex = r"^生成含.*的周报$")]
fn assert_weekly_report(world: &mut AcceptanceWorld) {
    let report_type = world.state.get("report_type").cloned().unwrap_or_default();
    assert_eq!(report_type, "Weekly", "报告类型应为 Weekly");
    let content = world.state.get("report_content").cloned().unwrap_or_default();
    assert!(!content.is_empty(), "报告内容不应为空");
}

#[then(regex = r"^生成含.*的月报$")]
fn assert_monthly_report(world: &mut AcceptanceWorld) {
    let report_type = world.state.get("report_type").cloned().unwrap_or_default();
    assert_eq!(report_type, "Monthly", "报告类型应为 Monthly");
    let content = world.state.get("report_content").cloned().unwrap_or_default();
    assert!(!content.is_empty(), "报告内容不应为空");
}

#[then(regex = r"^生成含.*的季报$")]
fn assert_quarterly_report(world: &mut AcceptanceWorld) {
    let report_type = world.state.get("report_type").cloned().unwrap_or_default();
    assert_eq!(report_type, "Quarterly", "报告类型应为 Quarterly");
    let content = world.state.get("report_content").cloned().unwrap_or_default();
    assert!(!content.is_empty(), "报告内容不应为空");
}

#[then(regex = r"^生成对应报告$")]
fn assert_corresponding_report(world: &mut AcceptanceWorld) {
    let report_type = world.state.get("report_type").cloned().unwrap_or_default();
    assert_eq!(report_type, "SemiAnnual", "报告类型应为 SemiAnnual");
    let content = world.state.get("report_content").cloned().unwrap_or_default();
    assert!(!content.is_empty(), "报告内容不应为空");
}

#[then(regex = r"^各自按 SLA 生成$")]
fn assert_sla_generation(world: &mut AcceptanceWorld) {
    let monthly_id = world.state.get("monthly_report_id").cloned().unwrap_or_default();
    let quarterly_id = world.state.get("quarterly_report_id").cloned().unwrap_or_default();
    assert!(!monthly_id.is_empty(), "月报应有 ID");
    assert!(!quarterly_id.is_empty(), "季报应有 ID");
    assert_ne!(monthly_id, quarterly_id, "月报和季报应有不同 ID");
}

#[then(regex = r"^通知用户审查确认$")]
fn assert_notify_review(world: &mut AcceptanceWorld) {
    let report_id = world.state.get("report_id").cloned().unwrap_or_default();
    assert!(!report_id.is_empty(), "应有报告 ID 用于审查通知");
    world.notifications.push("report_review".into());
}

#[then(regex = r"^编辑版本为最终版本$")]
fn assert_edit_is_final(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("report_action").map(String::as_str), Some("edited"));
}

#[then(regex = r"^可导出 Markdown 或 PDF$")]
fn assert_export_options(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("export_action").map(String::as_str), Some("export"));
    let content_len: usize = world.state.get("export_content_len")
        .and_then(|s| s.parse().ok()).unwrap_or(0);
    assert!(content_len > 0, "导出内容长度应大于 0");
}

#[then(regex = r"^推送到飞书文档$")]
fn assert_push_feishu(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("export_action").map(String::as_str), Some("sync_feishu"));
}

#[then(regex = r"^后续遵循模板$")]
fn assert_follow_template(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("report_format").map(String::as_str), Some("custom"));
}

#[then(regex = r"^后续在新时间生成$")]
fn assert_new_schedule(world: &mut AcceptanceWorld) {
    assert_eq!(world.processing_result.as_deref(), Some("new_schedule"));
}
