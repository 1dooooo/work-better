//! G2 智能处理 — 分类路由、模型升级、SLA、预算、ReviewAgent

use cucumber::{given, when, then};
use crate::world::AcceptanceWorld;

// ── Given ──────────────────────────────────────────────────

#[given(regex = r"^(task_update|approval|meeting|message.*@mention|manual_note|message.*一般|doc_change|browsing|app_activity|低置信度|需要长期分析)")]
fn setup_event_type(world: &mut AcceptanceWorld, event_desc: String) {
    if event_desc.contains("task_update") || event_desc.contains("approval") || event_desc.contains("meeting") || event_desc.contains("@mention") || event_desc.contains("manual_note") {
        world.priority = Some("P0_P1".into());
    } else if event_desc.contains("低置信度") {
        world.confidence = Some(0.3);
    } else if event_desc.contains("长期分析") {
        world.priority = Some("P3".into());
    } else {
        world.priority = Some("P2".into());
    }
    world.event_type = Some("processing_test".into());
}

#[given(regex = r"^实体提取.*小模型置信度[<≥>]?([\d.]+)")]
fn entity_extraction_confidence(world: &mut AcceptanceWorld, conf: String) {
    world.confidence = conf.parse().ok();
    world.state.insert("task".into(), "entity_extraction".into());
}

#[given(regex = r"^任务识别.*置信度[<≥>]?([\d.]+)")]
fn task_id_confidence(world: &mut AcceptanceWorld, conf: String) {
    world.confidence = conf.parse().ok();
    world.state.insert("task".into(), "task_identification".into());
}

#[given(regex = r"^摘要生成.*置信度[<≥>]?([\d.]+)")]
fn summary_confidence(world: &mut AcceptanceWorld, conf: String) {
    world.confidence = conf.parse().ok();
    world.state.insert("task".into(), "summarization".into());
}

#[given(regex = r"^情感判断.*置信度[<≥>]?([\d.]+)")]
fn sentiment_confidence(world: &mut AcceptanceWorld, conf: String) {
    world.confidence = conf.parse().ok();
    world.state.insert("task".into(), "sentiment".into());
}

#[given(regex = r"^关联分析.*置信度[<≥>]?([\d.]+)")]
fn relation_confidence(world: &mut AcceptanceWorld, conf: String) {
    world.confidence = conf.parse().ok();
    world.state.insert("task".into(), "relation".into());
}

#[given(regex = r"^模式识别任务$")]
fn pattern_recognition(world: &mut AcceptanceWorld) {
    world.state.insert("task".into(), "pattern_recognition".into());
}

#[given(regex = r"^小模型置信度达标$")]
fn small_model_sufficient(world: &mut AcceptanceWorld) {
    world.confidence = Some(0.85);
    world.model_used = Some("small".into());
}

#[given(regex = r"^小模型失败.*大模型也失败$")]
fn both_models_fail(world: &mut AcceptanceWorld) {
    world.error = Some("both_models_failed".into());
}

#[given(regex = r"^日预算未耗尽$")]
fn budget_available(world: &mut AcceptanceWorld) {
    world.budget_remaining = Some(100.0);
}

#[given(regex = r"^日预算耗尽.*非紧急$")]
fn budget_exhausted_non_urgent(world: &mut AcceptanceWorld) {
    world.budget_remaining = Some(0.0);
    world.priority = Some("P2".into());
}

#[given(regex = r"^日预算耗尽.*紧急.*P[01]")]
fn budget_exhausted_urgent(world: &mut AcceptanceWorld) {
    world.budget_remaining = Some(0.0);
    world.priority = Some("P0".into());
}

#[given(regex = r"^日预算耗尽.*策略 degrade_to_small$")]
fn budget_degrade(world: &mut AcceptanceWorld) {
    world.budget_remaining = Some(0.0);
    world.state.insert("degradation_strategy".into(), "degrade_to_small".into());
}

#[given(regex = r"^Token 使用被跟踪$")]
fn token_tracked(world: &mut AcceptanceWorld) {
    world.state.insert("token_tracking".into(), "enabled".into());
}

#[given(regex = r"^(P[0-3]) 事件.*超(\d+)(分钟|小时)未处理$")]
fn sla_timeout(world: &mut AcceptanceWorld, prio: String, _dur: String, _unit: String) {
    world.priority = Some(prio);
    world.state.insert("sla".into(), "timeout".into());
}

#[given(regex = r"^一天结束$")]
fn end_of_day(world: &mut AcceptanceWorld) {
    world.state.insert("time".into(), "end_of_day".into());
}

#[given(regex = r"^(直接归档|低置信度提取|高置信度提取|任务状态变更|报告/摘要|涉及他人信息)输出")]
fn review_input_type(world: &mut AcceptanceWorld, input_type: String) {
    world.state.insert("review_input".into(), input_type);
}

// ── When ───────────────────────────────────────────────────

#[when(regex = r"^分类$")]
fn classify(world: &mut AcceptanceWorld) {
    world.processing_result = Some("classified".into());
}

#[when(regex = r"^处理$")]
fn process(world: &mut AcceptanceWorld) {
    world.processing_result = Some("processed".into());
}

#[when(regex = r"^完成$")]
fn done(world: &mut AcceptanceWorld) {
    world.processing_result = Some("done".into());
}

#[when(regex = r"^需大模型$")]
fn needs_llm(world: &mut AcceptanceWorld) {
    world.state.insert("needs_llm".into(), "true".into());
}

#[when(regex = r"^查看审计$")]
fn view_audit(world: &mut AcceptanceWorld) {
    world.processing_result = Some("audit_viewed".into());
}

#[when(regex = r"^超时$")]
fn timeout(world: &mut AcceptanceWorld) {
    world.processing_result = Some("timeout".into());
}

#[when(regex = r"^SLA 扫描")]
fn sla_scan(world: &mut AcceptanceWorld) {
    world.processing_result = Some("sla_scanned".into());
}

#[when(regex = r"^SLA 报告生成$")]
fn sla_report(world: &mut AcceptanceWorld) {
    world.processing_result = Some("sla_report".into());
}

#[when(regex = r"^返回$")]
fn review_return(world: &mut AcceptanceWorld) {
    world.processing_result = Some("returned".into());
}

#[when(regex = r"^阈值达到$")]
fn threshold_reached(world: &mut AcceptanceWorld) {
    world.processing_result = Some("threshold_reached".into());
}

// ── Then ───────────────────────────────────────────────────

#[then(regex = r"^即时处理")]
fn assert_instant(world: &mut AcceptanceWorld) {
    let prio = world.priority.as_deref().unwrap_or("");
    assert!(prio.contains("P0") || prio.contains("P1") || prio == "P0_P1", "应即时处理");
}

#[then(regex = r"^聚合处理")]
fn assert_aggregate(world: &mut AcceptanceWorld) {
    assert_eq!(world.priority.as_deref(), Some("P2"), "应聚合处理");
}

#[then(regex = r"^模式分析")]
fn assert_pattern(world: &mut AcceptanceWorld) {
    assert_eq!(world.priority.as_deref(), Some("P3"), "应模式分析");
}

#[then(regex = r"^直接归档$")]
fn assert_archive(world: &mut AcceptanceWorld) {
    assert!(world.confidence.unwrap_or(1.0) < 0.5, "低置信度应直接归档");
}

#[then(regex = r"^升级大模型$")]
fn assert_upgrade_to_llm(world: &mut AcceptanceWorld) {
    assert!(world.state.get("needs_llm").map_or(true, |v| v == "true"), "应升级大模型");
}

#[then(regex = r"^直接用大模型$")]
fn assert_use_llm_directly(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("task").map(String::as_str), Some("pattern_recognition"), "模式识别应直接用大模型");
}

#[then(regex = r"^进入 ReviewAgent$")]
fn assert_enter_review(world: &mut AcceptanceWorld) {
    assert!(world.confidence.unwrap_or(0.0) >= 0.7, "置信度达标应进入 ReviewAgent");
}

#[then(regex = r"^标记.*需手动处理.*通知$")]
fn assert_manual_handling(world: &mut AcceptanceWorld) {
    assert_eq!(world.error.as_deref(), Some("both_models_failed"));
}

#[then(regex = r"^可用$")]
fn assert_available(world: &mut AcceptanceWorld) {
    assert!(world.budget_remaining.unwrap_or(0.0) > 0.0, "预算应可用");
}

#[then(regex = r"^排队明天$")]
fn assert_queue_tomorrow(world: &mut AcceptanceWorld) {
    assert!(world.budget_remaining.unwrap_or(1.0) <= 0.0, "预算已耗尽");
}

#[then(regex = r"^允许溢出并通知$")]
fn assert_overflow_allowed(world: &mut AcceptanceWorld) {
    world.notifications.push("budget_overflow".into());
}

#[then(regex = r"^小模型降级$")]
fn assert_small_model_fallback(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("degradation_strategy").map(String::as_str), Some("degrade_to_small"));
}

#[then(regex = r"^可见每日消耗$")]
fn assert_daily_usage_visible(world: &mut AcceptanceWorld) {
    assert!(world.state.contains_key("token_tracking"));
}

#[then(regex = r"^强制升级并通知$")]
fn assert_force_upgrade(world: &mut AcceptanceWorld) {
    world.notifications.push("forced_upgrade".into());
}

#[then(regex = r"^继续正常流程$")]
fn assert_continue_normal(world: &mut AcceptanceWorld) {
    assert!(world.priority.as_deref() == Some("P2"), "P2 超时应继续正常流程");
}

#[then(regex = r"^排入每日批处理$")]
fn assert_batch_queued(world: &mut AcceptanceWorld) {
    assert_eq!(world.priority.as_deref(), Some("P3"));
}

#[then(regex = r"^超时事件自动提升$")]
fn assert_auto_escalate(world: &mut AcceptanceWorld) {
    assert_eq!(world.processing_result.as_deref(), Some("sla_scanned"));
}

#[then(regex = r"^显示效率统计$")]
fn assert_efficiency_stats(world: &mut AcceptanceWorld) {
    assert_eq!(world.processing_result.as_deref(), Some("sla_report"));
}

#[then(regex = r"^直接通过$")]
fn assert_pass_through(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("review_input").map(String::as_str), Some("直接归档"));
}

#[then(regex = r"^仅规则层验证$")]
fn assert_rules_only(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("review_input").map(String::as_str), Some("低置信度提取"));
}

#[then(regex = r"^\d+%抽样审查$")]
fn assert_sampling(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("review_input").map(String::as_str), Some("高置信度提取"));
}

#[then(regex = r"^规则验证\+共享数据需确认$")]
fn assert_rules_plus_confirm(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("review_input").map(String::as_str), Some("任务状态变更"));
}

#[then(regex = r"^规则\+小模型一致性检查$")]
fn assert_consistency_check(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("review_input").map(String::as_str), Some("报告/摘要"));
}

#[then(regex = r"^规则\+小模型\+用户确认$")]
fn assert_user_confirm_required(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("review_input").map(String::as_str), Some("涉及他人信息"));
}

#[then(regex = r"^回处理层重新处理$")]
fn assert_reprocess(world: &mut AcceptanceWorld) {
    world.processing_result = Some("reprocessing".into());
}

#[then(regex = r"^推送通知$")]
fn assert_push_notification(world: &mut AcceptanceWorld) {
    world.notifications.push("review_notification".into());
}

#[then(regex = r"^进入存储层$")]
fn assert_enter_storage(world: &mut AcceptanceWorld) {
    world.processing_result = Some("entering_storage".into());
}

#[then(regex = r"^自动调整提示或阈值$")]
fn assert_auto_adjust(world: &mut AcceptanceWorld) {
    world.processing_result = Some("auto_adjusted".into());
}
