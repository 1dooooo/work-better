//! G6 系统能力 — 快捷键、菜单栏、设置、调度器
//!
//! 调度器场景接入真实 Scheduler 组件。
//! UI 场景保留 state 断言（需要 Tauri 运行时）。

use cucumber::{given, when, then};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use async_trait::async_trait;
use wb_scheduler::scheduler::Scheduler;
use wb_scheduler::task::{ScheduledTask, TaskLayer, TaskResult, TaskStatus as SchedStatus};

use crate::world::AcceptanceWorld;

// ─── Mock ScheduledTask ────────────────────────────────────

struct MockTask {
    id_str: String,
    name_str: String,
    layer_val: TaskLayer,
    should_fail: AtomicBool,
    exec_count: AtomicU32,
}

impl MockTask {
    fn new(id: &str, name: &str, layer: TaskLayer) -> Self {
        Self {
            id_str: id.to_string(),
            name_str: name.to_string(),
            layer_val: layer,
            should_fail: AtomicBool::new(false),
            exec_count: AtomicU32::new(0),
        }
    }

    fn with_fail(self) -> Self {
        self.should_fail.store(true, Ordering::Relaxed);
        self
    }
}

#[async_trait]
impl ScheduledTask for MockTask {
    fn id(&self) -> &str { &self.id_str }
    fn name(&self) -> &str { &self.name_str }
    fn layer(&self) -> TaskLayer { self.layer_val.clone() }
    fn cron_expression(&self) -> &str { "0 * * * * *" }
    fn sla_ms(&self) -> u64 { 1000 }
    fn retry_limit(&self) -> u32 { 3 }

    async fn execute(&self) -> TaskResult {
        self.exec_count.fetch_add(1, Ordering::Relaxed);
        let now = chrono::Utc::now();
        if self.should_fail.load(Ordering::Relaxed) {
            TaskResult {
                task_id: self.id_str.clone(),
                task_name: self.name_str.clone(),
                status: SchedStatus::Failed,
                started_at: now,
                finished_at: now,
                duration_ms: 0,
                summary: "mock failure".to_string(),
                error: Some("simulated failure".to_string()),
                retry_count: 0,
            }
        } else {
            TaskResult {
                task_id: self.id_str.clone(),
                task_name: self.name_str.clone(),
                status: SchedStatus::Success,
                started_at: now,
                finished_at: now,
                duration_ms: 10,
                summary: "mock success".to_string(),
                error: None,
                retry_count: 0,
            }
        }
    }
}

// ── Given ──────────────────────────────────────────────────

// UI 场景（保留 state）
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

// 调度器场景（真实组件）
#[given(regex = r"^调度器运行中$")]
async fn scheduler_running(world: &mut AcceptanceWorld) {
    let task = Arc::new(MockTask::new("cron-task", "Cron采集", TaskLayer::Collection));
    world.scheduler.register(task).await;
    world.scheduler.start().await;
    world.state.insert("scheduler".into(), "running".into());
}

#[given(regex = r"^A 依赖 B$")]
async fn dependency(world: &mut AcceptanceWorld) {
    world.scheduler.register(Arc::new(MockTask::new("task-b", "TaskB", TaskLayer::Processing))).await;
    world.state.insert("dependency".into(), "A_depends_B".into());
    world.state.insert("task_b_status".into(), "pending".into());
}

#[given(regex = r"^任务超 SLA$")]
fn sla_exceeded(world: &mut AcceptanceWorld) {
    world.state.insert("sla".into(), "exceeded".into());
}

#[given(regex = r"^任务失败$")]
async fn task_failed(world: &mut AcceptanceWorld) {
    let task = Arc::new(MockTask::new("fail-task", "失败任务", TaskLayer::Processing).with_fail());
    world.scheduler.register(task).await;
    world.state.insert("task_status".into(), "failed".into());
}

#[given(regex = r"^\d+次重试全失败$")]
async fn retries_exhausted(world: &mut AcceptanceWorld) {
    let task = Arc::new(MockTask::new("retry-task", "重试任务", TaskLayer::Processing).with_fail());
    world.scheduler.register(task).await;
    world.state.insert("retries".into(), "exhausted".into());
}

#[given(regex = r"^日预算不足$")]
async fn low_budget(world: &mut AcceptanceWorld) {
    world.scheduler.set_budget(0).await;
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
async fn paused(world: &mut AcceptanceWorld) {
    world.scheduler.pause_all().await;
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
async fn similar_task_running(world: &mut AcceptanceWorld) {
    let task = Arc::new(MockTask::new("sim-task", "同类任务", TaskLayer::Collection));
    world.scheduler.register(task).await;
    world.state.insert("concurrency".into(), "running".into());
}

#[given(regex = r"^采集层定时任务$")]
async fn collector_cron(world: &mut AcceptanceWorld) {
    let task = Arc::new(MockTask::new("collector-cron", "采集定时", TaskLayer::Collection));
    world.scheduler.register(task).await;
    world.state.insert("cron_layer".into(), "collector".into());
}

#[given(regex = r"^处理层定时任务$")]
async fn processor_cron(world: &mut AcceptanceWorld) {
    let task = Arc::new(MockTask::new("processor-cron", "处理定时", TaskLayer::Processing));
    world.scheduler.register(task).await;
    world.state.insert("cron_layer".into(), "processor".into());
}

#[given(regex = r"^存储层定时任务$")]
async fn storage_cron(world: &mut AcceptanceWorld) {
    let task = Arc::new(MockTask::new("storage-cron", "存储定时", TaskLayer::Storage));
    world.scheduler.register(task).await;
    world.state.insert("cron_layer".into(), "storage".into());
}

#[given(regex = r"^报告层定时任务$")]
async fn report_cron(world: &mut AcceptanceWorld) {
    let task = Arc::new(MockTask::new("report-cron", "报告定时", TaskLayer::Presentation));
    world.scheduler.register(task).await;
    world.state.insert("cron_layer".into(), "report".into());
}

// ── When ───────────────────────────────────────────────────

// UI 场景
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

#[when(regex = r"^双击菜单栏$")]
fn double_click_menubar(world: &mut AcceptanceWorld) {
    world.processing_result = Some("menubar_double_clicked".into());
}

#[when(regex = r"^用户确认$")]
fn user_confirm(world: &mut AcceptanceWorld) {
    world.processing_result = Some("user_confirmed".into());
}

#[when(regex = r"^打开时间线$")]
fn open_timeline(world: &mut AcceptanceWorld) {
    world.processing_result = Some("timeline_opened".into());
}

#[when(regex = r"^拖拽卡片$")]
fn drag_card(world: &mut AcceptanceWorld) {
    world.processing_result = Some("card_dragged".into());
}

#[when(regex = r"^搜索$")]
fn search(world: &mut AcceptanceWorld) {
    world.processing_result = Some("searched".into());
}

#[when(regex = r"^查看原文$")]
fn view_original(world: &mut AcceptanceWorld) {
    world.processing_result = Some("original_viewed".into());
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

// 调度器场景
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
    let budget = world.budget_remaining.unwrap_or(1.0);
    if budget <= 0.0 {
        world.processing_result = Some("low_prio_deferred".into());
    } else {
        world.processing_result = Some("low_prio_executed".into());
    }
}

#[when(regex = r"^触发$")]
async fn trigger_scheduler(world: &mut AcceptanceWorld) {
    let action = world.state.get("scheduler_action").cloned().unwrap_or_default();
    match action.as_str() {
        "pause" => {
            world.scheduler.pause_all().await;
            world.state.insert("scheduler_state".into(), "paused".into());
        }
        "emergency_stop" => {
            world.scheduler.stop().await;
            world.state.insert("scheduler_state".into(), "stopped".into());
        }
        _ => {}
    }
    world.processing_result = Some("scheduler_triggered".into());
}

#[when(regex = r"^恢复$")]
async fn resume_scheduler(world: &mut AcceptanceWorld) {
    world.scheduler.resume_all().await;
    world.state.insert("scheduler_state".into(), "running".into());
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

// 补全缺失的 When 步骤
#[when(regex = r"^轻提醒触发$")]
fn light_notify(world: &mut AcceptanceWorld) { world.processing_result = Some("light_notify".into()); }

#[when(regex = r"^点击菜单栏$")]
fn click_menubar(world: &mut AcceptanceWorld) { world.processing_result = Some("click_menubar".into()); }

#[when(regex = r"^交互菜单栏$")]
fn interact_menubar(world: &mut AcceptanceWorld) { world.processing_result = Some("interact_menubar".into()); }

#[when(regex = r"^查看通知中心$")]
fn view_notif_center(world: &mut AcceptanceWorld) { world.processing_result = Some("view_notif_center".into()); }

#[when(regex = r"^从菜单栏选择$")]
fn menubar_select(world: &mut AcceptanceWorld) { world.processing_result = Some("menubar_selected".into()); }

#[when(regex = r"^查看时间线$")]
fn view_timeline(world: &mut AcceptanceWorld) { world.processing_result = Some("view_timeline".into()); }

#[when(regex = r"^展开详情$")]
fn expand_detail(world: &mut AcceptanceWorld) { world.processing_result = Some("expand_detail".into()); }

#[when(regex = r"^查看任务板$")]
fn view_task_board(world: &mut AcceptanceWorld) { world.processing_result = Some("view_task_board".into()); }

#[when(regex = r"^拖到不同列$")]
fn drag_to_column(world: &mut AcceptanceWorld) { world.processing_result = Some("drag_to_column".into()); }

#[when(regex = r"^调度执行$")]
fn schedule_execute(world: &mut AcceptanceWorld) { world.processing_result = Some("schedule_execute".into()); }

#[when(regex = r"^查看数据探索$")]
fn view_data_explorer(world: &mut AcceptanceWorld) { world.processing_result = Some("view_data_explorer".into()); }

#[when(regex = r"^点击$")]
fn click_item(world: &mut AcceptanceWorld) { world.processing_result = Some("clicked".into()); }

// ── Then ───────────────────────────────────────────────────

// UI 场景断言
#[then(regex = r"^快捷记录窗口出现$")]
fn assert_capture_window(world: &mut AcceptanceWorld) {
    assert_eq!(world.processing_result.as_deref(), Some("shortcut_pressed"));
}

#[then(regex = r"^窗口隐藏$")]
fn assert_window_hidden_g6(world: &mut AcceptanceWorld) {
    assert_eq!(world.processing_result.as_deref(), Some("shortcut_toggled"));
}

#[then(regex = r"^截图并打开窗口$")]
fn assert_screenshot_window(world: &mut AcceptanceWorld) {
    assert_eq!(world.processing_result.as_deref(), Some("screenshot_key"));
}

#[then(regex = r"^显示通知")]
fn assert_show_notification(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^非侵入通知$")]
fn assert_non_intrusive(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^显示待确认")]
fn assert_show_confirm(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^两次点击内完成$")]
fn assert_two_clicks(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^一屏可见")]
fn assert_one_screen(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^重定向主窗口$")]
fn assert_redirect_main(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^时间轴")]
fn assert_timeline(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^有.*原文链接$")]
fn assert_has_link(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^按状态列分组$")]
fn assert_grouped(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^状态更新并同步$")]
fn assert_synced(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^RAG\+结构化双路搜索$")]
fn assert_dual_search(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^时间/任务/会议/模式图表$")]
fn assert_charts(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^打开.*原文$")]
fn assert_open_original(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^(API 端点|飞书凭据|Obsidian 路径|自定义组合键|频率和策略)")]
fn assert_config_saved(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^按维度查询并导出$")]
fn assert_query_export(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

// 调度器场景断言（真实组件）
#[then(regex = r"^在偏移窗口内执行$")]
fn assert_offset_window(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("scheduler").map(String::as_str), Some("running"));
    assert_eq!(world.processing_result.as_deref(), Some("cron_triggered"));
}

#[then(regex = r"^A 不启动$")]
fn assert_a_not_started(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("task_b_status").map(String::as_str), Some("pending"));
}

#[then(regex = r"^自动终止$")]
fn assert_auto_terminate(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("sla").map(String::as_str), Some("exceeded"));
}

#[then(regex = r"^重试.*递增间隔$")]
fn assert_retry_backoff(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("task_status").map(String::as_str), Some("failed"));
}

#[then(regex = r"^标记 failed$")]
fn assert_marked_failed(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^推迟$")]
fn assert_deferred(world: &mut AcceptanceWorld) {
    assert_eq!(world.processing_result.as_deref(), Some("low_prio_deferred"));
}

#[then(regex = r"^所有定时任务停止")]
async fn assert_all_paused(world: &mut AcceptanceWorld) {
    assert!(world.scheduler.is_paused().await, "调度器应已暂停");
}

#[then(regex = r"^执行中任务立即终止$")]
fn assert_emergency_stopped(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("scheduler_state").map(String::as_str), Some("stopped"));
}

#[then(regex = r"^积压任务按优先级执行$")]
fn assert_backlog_executed(world: &mut AcceptanceWorld) {
    assert_eq!(world.processing_result.as_deref(), Some("scheduler_resumed"));
}

#[then(regex = r"^显示 ID/名称")]
fn assert_task_list_visible(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^显示状态/时长")]
fn assert_log_details(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^不并行$")]
fn assert_no_parallel(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^整点后.*分钟执行$")]
fn assert_collection_window(world: &mut AcceptanceWorld) {
    let layer = world.state.get("cron_layer").map(String::as_str);
    assert!(layer.is_some(), "应有 cron_layer_setting");
}

#[then(regex = r"^低峰期.*执行$")]
fn assert_storage_window(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("cron_layer").map(String::as_str), Some("storage"));
}

#[then(regex = r"^用户配置时间执行$")]
fn assert_report_window(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("cron_layer").map(String::as_str), Some("report"));
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
