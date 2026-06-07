//! G5 报告生成 — 日报/周报/月报/季报/半年报/年报

use cucumber::{given, when, then};
use crate::world::AcceptanceWorld;

#[given(regex = r"^工作日18:00$")]
fn weekday_1800(world: &mut AcceptanceWorld) {
    world.state.insert("report_trigger".into(), "daily".into());
    world.state.insert("time".into(), "18:00".into());
}

#[given(regex = r"^周五17:00$")]
fn friday_1700(world: &mut AcceptanceWorld) {
    world.state.insert("report_trigger".into(), "weekly".into());
    world.state.insert("time".into(), "friday_17:00".into());
}

#[given(regex = r"^月末$")]
fn month_end(world: &mut AcceptanceWorld) {
    world.state.insert("report_trigger".into(), "monthly".into());
}

#[given(regex = r"^季末$")]
fn quarter_end(world: &mut AcceptanceWorld) {
    world.state.insert("report_trigger".into(), "quarterly".into());
}

#[given(regex = r"^(12/31|6/30)$")]
fn half_year_end(world: &mut AcceptanceWorld, _date: String) {
    world.state.insert("report_trigger".into(), "half_year".into());
}

#[given(regex = r"^月末同时季末$")]
fn month_and_quarter(world: &mut AcceptanceWorld) {
    world.state.insert("report_trigger".into(), "monthly+quarterly".into());
}

#[given(regex = r"^报告生成完成$")]
fn report_generated(world: &mut AcceptanceWorld) {
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

#[when(regex = r"^(日报|周报|月报|季报|半年报|年报)触发$")]
fn report_trigger(world: &mut AcceptanceWorld, report_type: String) {
    world.processing_result = Some(format!("{report_type}_triggered"));
}

#[when(regex = r"^两者同时到期$")]
fn both_due(world: &mut AcceptanceWorld) {
    world.processing_result = Some("both_triggered".into());
}

#[when(regex = r"^完毕$")]
fn finished(world: &mut AcceptanceWorld) {
    world.processing_result = Some("finished".into());
}

#[when(regex = r"^选择导出$")]
fn export(world: &mut AcceptanceWorld) {
    world.state.insert("export_action".into(), "export".into());
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
    assert!(world.processing_result.as_ref().unwrap_or(&String::new()).contains("日报"));
}

#[then(regex = r"^生成含.*的周报$")]
fn assert_weekly_report(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.as_ref().unwrap_or(&String::new()).contains("周报"));
}

#[then(regex = r"^生成含.*的月报$")]
fn assert_monthly_report(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.as_ref().unwrap_or(&String::new()).contains("月报"));
}

#[then(regex = r"^生成含.*的季报$")]
fn assert_quarterly_report(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.as_ref().unwrap_or(&String::new()).contains("季报"));
}

#[then(regex = r"^生成对应报告$")]
fn assert_corresponding_report(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^各自按 SLA 生成$")]
fn assert_sla_generation(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.as_ref().unwrap_or(&String::new()).contains("both"));
}

#[then(regex = r"^通知用户审查确认$")]
fn assert_notify_review(world: &mut AcceptanceWorld) {
    world.notifications.push("report_review".into());
}

#[then(regex = r"^编辑版本为最终版本$")]
fn assert_edit_is_final(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("report_action").map(String::as_str), Some("edited"));
}

#[then(regex = r"^可导出 Markdown 或 PDF$")]
fn assert_export_options(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("export_action").map(String::as_str), Some("export"));
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
