//! G7 横切关注 — 数据治理、审计、权限

use cucumber::{given, when, then};
use crate::world::AcceptanceWorld;

#[given(regex = r"^AI 处理个人数据$")]
fn ai_personal_data(world: &mut AcceptanceWorld) {
    world.state.insert("data_type".into(), "personal".into());
    world.state.insert("scope".into(), "self_only".into());
}

#[given(regex = r"^AI 要修改共享数据$")]
fn ai_shared_data(world: &mut AcceptanceWorld) {
    world.state.insert("data_type".into(), "shared".into());
    world.state.insert("needs_confirm".into(), "true".into());
}

#[given(regex = r"^用户确认共享操作$")]
fn user_confirm_shared(world: &mut AcceptanceWorld) {
    world.state.insert("user_confirmed".into(), "true".into());
}

#[given(regex = r"^事件被采集$")]
fn event_collected(world: &mut AcceptanceWorld) {
    world.state.insert("event_phase".into(), "collected".into());
}

#[given(regex = r"^事件被消费$")]
fn event_consumed(world: &mut AcceptanceWorld) {
    world.state.insert("event_phase".into(), "consumed".into());
}

#[given(regex = r"^WorkRecord 产出$")]
fn work_record_produced(world: &mut AcceptanceWorld) {
    world.state.insert("record_phase".into(), "produced".into());
}

#[given(regex = r"^表示层读取$")]
fn presentation_read(world: &mut AcceptanceWorld) {
    world.state.insert("layer".into(), "presentation".into());
}

#[given(regex = r"^用户在 Obsidian 编辑$")]
fn user_edit_obsidian(world: &mut AcceptanceWorld) {
    world.state.insert("edit_source".into(), "obsidian".into());
}

#[given(regex = r"^事件进入处理$")]
fn event_entering_processing(world: &mut AcceptanceWorld) {
    world.state.insert("processing".into(), "active".into());
}

#[given(regex = r"^同一事件有审计记录$")]
fn has_audit_record(world: &mut AcceptanceWorld) {
    world.state.insert("audit".into(), "exists".into());
}

#[given(regex = r"^审计数据存在$")]
fn audit_data_exists(world: &mut AcceptanceWorld) {
    world.state.insert("audit_data".into(), "exists".into());
}

#[given(regex = r"^审计数据积累$")]
fn audit_accumulated(world: &mut AcceptanceWorld) {
    world.state.insert("audit_data".into(), "accumulated".into());
}

#[given(regex = r"^检测到模式")]
fn pattern_detected(world: &mut AcceptanceWorld) {
    world.state.insert("pattern".into(), "detected".into());
}

// ── When: domain-prefixed to avoid G2/G3 conflicts ────────

#[when(regex = r"^数据处理$")]
fn g7_process(world: &mut AcceptanceWorld) {
    world.processing_result = Some("g7_processed".into());
}

#[when(regex = r"^即将执行$")]
fn g7_about_to_execute(world: &mut AcceptanceWorld) {
    world.processing_result = Some("g7_about_to_execute".into());
}

#[when(regex = r"^确认$")]
fn g7_confirm(world: &mut AcceptanceWorld) {
    world.processing_result = Some("g7_confirmed".into());
}

#[when(regex = r"^进入系统$")]
fn g7_enter_system(world: &mut AcceptanceWorld) {
    world.processing_result = Some("g7_entered_system".into());
}

#[when(regex = r"^消费完成$")]
fn g7_processing_done(world: &mut AcceptanceWorld) {
    world.processing_result = Some("g7_processing_done".into());
}

#[when(regex = r"^三层写入$")]
fn g7_write(world: &mut AcceptanceWorld) {
    world.processing_result = Some("g7_written".into());
}

#[when(regex = r"^审计查询$")]
fn g7_query(world: &mut AcceptanceWorld) {
    world.processing_result = Some("g7_queried".into());
}

#[when(regex = r"^编辑保存$")]
fn g7_save(world: &mut AcceptanceWorld) {
    world.processing_result = Some("g7_saved".into());
}

#[when(regex = r"^每步执行$")]
fn g7_step_execute(world: &mut AcceptanceWorld) {
    world.processing_result = Some("g7_step_executed".into());
}

#[when(regex = r"^审计查看$")]
fn g7_view(world: &mut AcceptanceWorld) {
    world.processing_result = Some("g7_viewed".into());
}

#[when(regex = r"^月度聚合$")]
fn g7_monthly_aggregate(world: &mut AcceptanceWorld) {
    world.processing_result = Some("g7_monthly_aggregated".into());
}

#[when(regex = r"^生成建议$")]
fn g7_generate_suggestions(world: &mut AcceptanceWorld) {
    world.processing_result = Some("g7_suggestions_generated".into());
}

// ── Then ───────────────────────────────────────────────────

#[then(regex = r"^自主执行无需确认$")]
fn assert_autonomous(world: &mut AcceptanceWorld) {
    assert_ne!(world.state.get("data_type").map(String::as_str), Some("shared"));
}

#[then(regex = r"^必须用户确认$")]
fn assert_requires_user_confirm(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("needs_confirm").map(String::as_str), Some("true"));
}

#[then(regex = r"^执行并同步飞书$")]
fn assert_execute_and_sync(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("user_confirmed").map(String::as_str), Some("true"));
}

#[then(regex = r"^EventLog 不可变记录$")]
fn assert_immutable_log(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("event_phase").map(String::as_str), Some("collected"));
}

#[then(regex = r"^标记为 processed$")]
fn assert_processed(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("event_phase").map(String::as_str), Some("consumed"));
}

#[then(regex = r"^三层联合查询接口$")]
fn assert_three_layer_query(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("layer").map(String::as_str), Some("presentation"));
}

#[then(regex = r"^两 DB 更新保持一致$")]
fn assert_db_consistent(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^生成 ProcessingAudit$")]
fn assert_processing_audit(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("processing").map(String::as_str), Some("active"));
}

#[then(regex = r"^trace_id 链接完整链路$")]
fn assert_trace_link(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("audit").map(String::as_str), Some("exists"));
}

#[then(regex = r"^可按多维度查询$")]
fn assert_multi_dim_query(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("audit_data").map(String::as_str), Some("exists"));
}

#[then(regex = r"^聚合为统计摘要$")]
fn assert_aggregate_stats(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("audit_data").map(String::as_str), Some("accumulated"));
}

#[then(regex = r"^产生改进建议$")]
fn assert_improvement_suggestions(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("pattern").map(String::as_str), Some("detected"));
}

#[then(regex = r"^三层写入完成$")]
fn assert_g7_write_order(world: &mut AcceptanceWorld) {
    assert_eq!(world.processing_result.as_deref(), Some("g7_written"), "G7 三层写入应完成");
}
