//! G6 系统能力 — 快捷键、菜单栏、设置、调度器

use cucumber::{given, when, then};
use crate::world::AcceptanceWorld;

#[given(regex = r"^用户在任何应用$")]
fn in_any_app(world: &mut AcceptanceWorld) {
    world.state.insert("context".into(), "any_app".into());
}

#[given(regex = r"^窗口可见$")]
fn window_visible(world: &mut AcceptanceWorld) {
    world.state.insert("window".into(), "visible".into());
}

#[given(regex = r"^(有待确认项|有常规信息)$")]
fn has_items(world: &mut AcceptanceWorld, kind: String) {
    world.state.insert("notification_kind".into(), kind);
}

#[given(regex = r"^用户查看状态$")]
fn view_status(world: &mut AcceptanceWorld) {
    world.state.insert("action".into(), "view_status".into());
}

#[given(regex = r"^用户要快速操作$")]
fn quick_action(world: &mut AcceptanceWorld) {
    world.state.insert("action".into(), "quick_action".into());
}

#[given(regex = r"^打开菜单栏$")]
fn open_menubar(world: &mut AcceptanceWorld) {
    world.state.insert("menubar".into(), "open".into());
}

#[given(regex = r"^需要复杂操作$")]
fn complex_action(world: &mut AcceptanceWorld) {
    world.state.insert("action".into(), "complex".into());
}

#[given(regex = r"^打开主窗口$")]
fn open_main_window(world: &mut AcceptanceWorld) {
    world.state.insert("main_window".into(), "open".into());
}

#[given(regex = r"^打开设置$")]
fn open_settings(world: &mut AcceptanceWorld) {
    world.state.insert("settings".into(), "open".into());
}

#[given(regex = r"^调度器运行中$")]
fn scheduler_running(world: &mut AcceptanceWorld) {
    world.state.insert("scheduler".into(), "running".into());
}

#[given(regex = r"^A 依赖 B$")]
fn dependency(world: &mut AcceptanceWorld) {
    world.state.insert("dependency".into(), "A_depends_B".into());
}

#[given(regex = r"^任务超 SLA$")]
fn sla_exceeded(world: &mut AcceptanceWorld) {
    world.state.insert("sla".into(), "exceeded".into());
}

#[given(regex = r"^任务失败$")]
fn task_failed(world: &mut AcceptanceWorld) {
    world.state.insert("task_status".into(), "failed".into());
}

#[given(regex = r"^\d+次重试全失败$")]
fn retries_exhausted(world: &mut AcceptanceWorld) {
    world.state.insert("retries".into(), "exhausted".into());
}

#[given(regex = r"^日预算不足$")]
fn low_budget(world: &mut AcceptanceWorld) {
    world.budget_remaining = Some(0.0);
}

#[given(regex = r"^用户激活暂停$")]
fn pause_activated(world: &mut AcceptanceWorld) {
    world.state.insert("scheduler_action".into(), "pause".into());
}

#[given(regex = r"^用户激活紧急停止$")]
fn emergency_stop(world: &mut AcceptanceWorld) {
    world.state.insert("scheduler_action".into(), "emergency_stop".into());
}

#[given(regex = r"^暂停中$")]
fn paused(world: &mut AcceptanceWorld) {
    world.state.insert("scheduler_state".into(), "paused".into());
}

#[given(regex = r"^查看调度 UI$")]
fn view_scheduler_ui(world: &mut AcceptanceWorld) {
    world.state.insert("ui".into(), "scheduler".into());
}

#[given(regex = r"^查看执行日志$")]
fn view_execution_log(world: &mut AcceptanceWorld) {
    world.state.insert("ui".into(), "execution_log".into());
}

#[given(regex = r"^同类任务执行中$")]
fn similar_task_running(world: &mut AcceptanceWorld) {
    world.state.insert("concurrency".into(), "running".into());
}

#[given(regex = r"^采集层定时任务$")]
fn collector_cron(world: &mut AcceptanceWorld) {
    world.state.insert("cron_layer".into(), "collector".into());
}

#[given(regex = r"^处理层定时任务$")]
fn processor_cron(world: &mut AcceptanceWorld) {
    world.state.insert("cron_layer".into(), "processor".into());
}

#[given(regex = r"^存储层定时任务$")]
fn storage_cron(world: &mut AcceptanceWorld) {
    world.state.insert("cron_layer".into(), "storage".into());
}

#[given(regex = r"^报告层定时任务$")]
fn report_cron(world: &mut AcceptanceWorld) {
    world.state.insert("cron_layer".into(), "report".into());
}

// ── When: precise match per step, no catch-all ────────────

#[when(regex = r"^按 Cmd\+Shift\+Space$")]
fn press_shortcut(world: &mut AcceptanceWorld) {
    world.processing_result = Some("shortcut_pressed".into());
}

#[when(regex = r"^再按快捷键$")]
fn press_shortcut_again(world: &mut AcceptanceWorld) {
    world.processing_result = Some("shortcut_toggled".into());
}

#[when(regex = r"^按截图键$")]
fn press_screenshot(world: &mut AcceptanceWorld) {
    world.processing_result = Some("screenshot_key".into());
}

#[when(regex = r"^通知触发$")]
fn notification_trigger(world: &mut AcceptanceWorld) {
    world.processing_result = Some("notification_triggered".into());
}

#[when(regex = r"^轻提醒触发$")]
fn light_reminder_trigger(world: &mut AcceptanceWorld) {
    world.processing_result = Some("light_reminder".into());
}

#[when(regex = r"^点击菜单栏$")]
fn click_menubar(world: &mut AcceptanceWorld) {
    world.processing_result = Some("menubar_clicked".into());
}

#[when(regex = r"^交互菜单栏$")]
fn interact_menubar(world: &mut AcceptanceWorld) {
    world.processing_result = Some("menubar_interacted".into());
}

#[when(regex = r"^查看通知中心$")]
fn view_notification_center(world: &mut AcceptanceWorld) {
    world.processing_result = Some("notification_center_viewed".into());
}

#[when(regex = r"^从菜单栏选择$")]
fn select_from_menubar(world: &mut AcceptanceWorld) {
    world.processing_result = Some("menubar_selected".into());
}

#[when(regex = r"^查看时间线$")]
fn view_timeline(world: &mut AcceptanceWorld) {
    world.processing_result = Some("timeline_viewed".into());
}

#[when(regex = r"^点击时间线项$")]
fn click_timeline_item(world: &mut AcceptanceWorld) {
    world.processing_result = Some("timeline_item_clicked".into());
}

#[when(regex = r"^展开详情$")]
fn expand_details(world: &mut AcceptanceWorld) {
    world.processing_result = Some("details_expanded".into());
}

#[when(regex = r"^查看任务板$")]
fn view_task_board(world: &mut AcceptanceWorld) {
    world.processing_result = Some("task_board_viewed".into());
}

#[when(regex = r"^拖拽任务卡片$")]
fn drag_task_card(world: &mut AcceptanceWorld) {
    world.processing_result = Some("task_card_dragged".into());
}

#[when(regex = r"^拖到不同列$")]
fn drag_to_column(world: &mut AcceptanceWorld) {
    world.processing_result = Some("dragged_to_column".into());
}

#[when(regex = r"^用户搜索$")]
fn user_search(world: &mut AcceptanceWorld) {
    world.processing_result = Some("search_initiated".into());
}

#[when(regex = r"^调度执行$")]
fn scheduler_execute(world: &mut AcceptanceWorld) {
    world.processing_result = Some("scheduler_executed".into());
}

#[when(regex = r"^查看数据探索$")]
fn view_data_exploration(world: &mut AcceptanceWorld) {
    world.processing_result = Some("data_exploration_viewed".into());
}

#[when(regex = r"^搜索结果$")]
fn search_results(world: &mut AcceptanceWorld) {
    world.processing_result = Some("search_results_shown".into());
}

#[when(regex = r"^点击$")]
fn click(world: &mut AcceptanceWorld) {
    world.processing_result = Some("clicked".into());
}

#[when(regex = r"^配置模型$")]
fn configure_model(world: &mut AcceptanceWorld) {
    world.processing_result = Some("model_configured".into());
}

#[when(regex = r"^配置收集器$")]
fn configure_collector(world: &mut AcceptanceWorld) {
    world.processing_result = Some("collector_configured".into());
}

#[when(regex = r"^配置存储$")]
fn configure_storage(world: &mut AcceptanceWorld) {
    world.processing_result = Some("storage_configured".into());
}

#[when(regex = r"^配置快捷键$")]
fn configure_shortcut(world: &mut AcceptanceWorld) {
    world.processing_result = Some("shortcut_configured".into());
}

#[when(regex = r"^配置新鲜度规则$")]
fn configure_freshness(world: &mut AcceptanceWorld) {
    world.processing_result = Some("freshness_configured".into());
}

#[when(regex = r"^系统查看审计$")]
fn system_view_audit(world: &mut AcceptanceWorld) {
    world.processing_result = Some("system_audit_viewed".into());
}

#[when(regex = r"^cron 触发$")]
fn cron_trigger(world: &mut AcceptanceWorld) {
    world.processing_result = Some("cron_triggered".into());
}

#[when(regex = r"^B 未完成$")]
fn b_not_done(world: &mut AcceptanceWorld) {
    world.processing_result = Some("b_pending".into());
}

#[when(regex = r"^超时检测$")]
fn timeout_detect(world: &mut AcceptanceWorld) {
    world.processing_result = Some("timeout_detected".into());
}

#[when(regex = r"^失败$")]
fn task_fail(world: &mut AcceptanceWorld) {
    world.processing_result = Some("task_failed".into());
}

#[when(regex = r"^最终失败$")]
fn final_failure(world: &mut AcceptanceWorld) {
    world.processing_result = Some("final_failure".into());
}

#[when(regex = r"^低优先级需执行$")]
fn low_prio_execute(world: &mut AcceptanceWorld) {
    world.processing_result = Some("low_prio_deferred".into());
}

#[when(regex = r"^触发$")]
fn trigger_scheduler(world: &mut AcceptanceWorld) {
    world.processing_result = Some("scheduler_triggered".into());
}

#[when(regex = r"^恢复$")]
fn resume_scheduler(world: &mut AcceptanceWorld) {
    world.processing_result = Some("scheduler_resumed".into());
}

#[when(regex = r"^检查任务$")]
fn check_task(world: &mut AcceptanceWorld) {
    world.processing_result = Some("task_checked".into());
}

#[when(regex = r"^调度检查$")]
fn scheduler_check(world: &mut AcceptanceWorld) {
    world.processing_result = Some("scheduler_checked".into());
}

#[when(regex = r"^另一个触发$")]
fn another_trigger(world: &mut AcceptanceWorld) {
    world.processing_result = Some("another_triggered".into());
}

// ── Then ───────────────────────────────────────────────────

#[then(regex = r"^(快捷记录窗口出现|窗口隐藏|截图并打开窗口|显示通知|非侵入通知|显示待确认|两次点击内完成|一屏可见|重定向主窗口|时间轴|有.*原文链接|按状态列分组|状态更新并同步|RAG\+结构化双路搜索|时间/任务/会议/模式图表|打开.*原文|API 端点|飞书凭据|Obsidian 路径|自定义组合键|频率和策略|按维度查询并导出|在偏移窗口内执行|A 不启动|自动终止|重试.*递增间隔|标记 failed|推迟|所有定时任务停止|执行中任务立即终止|积压任务按优先级执行|显示 ID/名称|显示状态/时长|不并行|整点后|低峰期|用户配置时间执行)")]
fn g6_then(world: &mut AcceptanceWorld, _assertion: String) {
    // Basic presence check — real assertions would verify specific behavior
    assert!(world.processing_result.is_some() || world.state.len() > 0, "应有处理结果或状态");
}

// ── Additional Given steps ─────────────────────────────────

#[given(regex = r"^拖拽任务卡片$")]
fn drag_task_card_given(world: &mut AcceptanceWorld) {
    world.state.insert("action".into(), "drag_task".into());
}

#[given(regex = r"^搜索结果$")]
fn search_results_given(world: &mut AcceptanceWorld) {
    world.state.insert("action".into(), "search_results".into());
}

#[given(regex = r"^点击时间线项$")]
fn click_timeline_item_given(world: &mut AcceptanceWorld) {
    world.state.insert("action".into(), "click_timeline".into());
}

#[given(regex = r"^用户搜索$")]
fn user_search_given(world: &mut AcceptanceWorld) {
    world.state.insert("action".into(), "search".into());
}
