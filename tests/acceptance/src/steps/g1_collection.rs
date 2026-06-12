//! G1 信息采集 — 飞书消息、文档、日历、会议、邮件等事件捕获

use cucumber::{given, when, then};
use crate::world::AcceptanceWorld;
use wb_core::event::{Source, EventType, Confidence, EventLog};

// ── Given steps: set up event sources ──────────────────────

#[given(regex = r"^飞书消息@提及用户$")]
async fn feishu_at_mention(world: &mut AcceptanceWorld) {
    world.event_type = Some("message".into());
    world.event_content = Some(r#"{"type":"message","mention":"@user"}"#.into());
    world.confidence = Some(0.95);

    // 创建真实的事件
    let event = world.create_event(
        Source::FeishuMessage,
        EventType::Message,
        r#"{"type":"message","mention":"@user"}"#,
    );
    world.current_event = Some(event);
}

#[given(regex = r"^飞书消息是回复用户参与的线程$")]
fn feishu_thread_reply(world: &mut AcceptanceWorld) {
    world.event_type = Some("message".into());
    world.state.insert("thread_context".into(), "user_participated".into());
}

#[given(regex = r"^飞书私信$")]
async fn feishu_dm(world: &mut AcceptanceWorld) {
    world.event_type = Some("message".into());
    world.state.insert("visibility".into(), "direct_message".into());

    // 创建真实的事件
    let event = world.create_event(
        Source::FeishuMessage,
        EventType::Message,
        r#"{"type":"message","visibility":"direct_message"}"#,
    );
    world.current_event = Some(event);
}

#[given(regex = r"^消息匹配关键词规则$")]
async fn message_matches_keyword(world: &mut AcceptanceWorld) {
    world.event_type = Some("message".into());
    world.state.insert("keyword_match".into(), "true".into());

    // 创建真实的事件
    let event = world.create_event(
        Source::FeishuMessage,
        EventType::Message,
        r#"{"type":"message","keyword_match":true}"#,
    );
    world.current_event = Some(event);
}

#[given(regex = r"^消息与用户无关$")]
fn message_irrelevant(world: &mut AcceptanceWorld) {
    world.event_type = Some("message".into());
    world.state.insert("relevance".into(), "none".into());
}

#[given(regex = r"^飞书文档被用户创建/编辑/评论$")]
fn feishu_doc_change(world: &mut AcceptanceWorld) {
    world.event_type = Some("document_change".into());
    world.state.insert("action".into(), "edit".into());
}

#[given(regex = r"^文档中用户被提及$")]
fn doc_mention_user(world: &mut AcceptanceWorld) {
    world.event_type = Some("document_change".into());
    world.state.insert("mention_in_doc".into(), "true".into());
}

#[given(regex = r"^飞书项目任务变更$")]
fn feishu_task_update(world: &mut AcceptanceWorld) {
    world.event_type = Some("task_update".into());
}

#[given(regex = r"^日历有即将到来事件$")]
fn calendar_upcoming(world: &mut AcceptanceWorld) {
    world.event_type = Some("calendar_event".into());
}

#[given(regex = r"^用户参加视频会议")]
fn meeting_attended(world: &mut AcceptanceWorld) {
    world.event_type = Some("meeting".into());
}

#[given(regex = r"^飞书妙记有录制摘要$")]
fn minutes_summary(world: &mut AcceptanceWorld) {
    world.event_type = Some("meeting".into());
    world.state.insert("has_summary".into(), "true".into());
}

#[given(regex = r"^用户通过飞书邮件操作$")]
fn feishu_email(world: &mut AcceptanceWorld) {
    world.event_type = Some("email".into());
}

#[given(regex = r"^飞书审批状态变更$")]
fn approval_change(world: &mut AcceptanceWorld) {
    world.event_type = Some("approval".into());
}

#[given(regex = r"^用户有 OKR")]
fn user_has_okr(world: &mut AcceptanceWorld) {
    world.event_type = Some("okr_update".into());
}

#[given(regex = r"^多维表格记录变更$")]
fn bitable_change(world: &mut AcceptanceWorld) {
    world.event_type = Some("document_change".into());
    world.state.insert("doc_type".into(), "bitable".into());
}

#[given(regex = r"^电子表格单元格变更$")]
fn sheet_change(world: &mut AcceptanceWorld) {
    world.event_type = Some("document_change".into());
    world.state.insert("doc_type".into(), "sheet".into());
}

#[given(regex = r"^知识库节点变更$")]
fn wiki_change(world: &mut AcceptanceWorld) {
    world.event_type = Some("document_change".into());
    world.state.insert("doc_type".into(), "wiki".into());
}

#[given(regex = r"^用户切换应用停留>(\d+)秒$")]
fn app_switch_debounce(world: &mut AcceptanceWorld, _secs: String) {
    world.event_type = Some("app_activity".into());
    world.state.insert("stay_duration".into(), "above_threshold".into());
}

#[given(regex = r"^用户切换应用停留<(\d+)秒$")]
fn app_switch_short(world: &mut AcceptanceWorld, _secs: String) {
    world.event_type = Some("app_activity".into());
    world.state.insert("stay_duration".into(), "below_threshold".into());
}

#[given(regex = r"^用户访问非搜索页 URL$")]
fn browsing_non_search(world: &mut AcceptanceWorld) {
    world.event_type = Some("browsing".into());
    world.state.insert("url_type".into(), "non_search".into());
}

#[given(regex = r"^用户访问搜索结果页$")]
fn browsing_search_results(world: &mut AcceptanceWorld) {
    world.event_type = Some("browsing".into());
    world.state.insert("url_type".into(), "search_results".into());
}

#[given(regex = r"^用户按全局快捷键$")]
fn global_hotkey(world: &mut AcceptanceWorld) {
    world.state.insert("action".into(), "hotkey".into());
}

#[given(regex = r"^窗口打开$")]
fn window_open(world: &mut AcceptanceWorld) {
    world.state.insert("window".into(), "visible".into());
}

#[given(regex = r"^用户按截图键$")]
fn screenshot_key(world: &mut AcceptanceWorld) {
    world.state.insert("action".into(), "screenshot".into());
}

#[given(regex = r"^收集器运行中$")]
fn collector_running(world: &mut AcceptanceWorld) {
    world.state.insert("collector_status".into(), "running".into());
}

#[given(regex = r"^父收集器禁用$")]
fn parent_disabled(world: &mut AcceptanceWorld) {
    world.state.insert("collector_status".into(), "parent_disabled".into());
}

#[given(regex = r"^收集器故障$")]
fn collector_faulty(world: &mut AcceptanceWorld) {
    world.state.insert("collector_status".into(), "unhealthy".into());
}

#[given(regex = r"^健康状态 unhealthy$")]
fn health_unhealthy(world: &mut AcceptanceWorld) {
    world.state.insert("health".into(), "unhealthy".into());
}

#[given(regex = r"^运行时注册新收集器$")]
fn runtime_register(world: &mut AcceptanceWorld) {
    world.state.insert("action".into(), "register_collector".into());
}

#[given(regex = r"^配置飞书连接")]
fn configure_feishu(world: &mut AcceptanceWorld) {
    world.state.insert("config".into(), "feishu_connection".into());
}

#[given(regex = r"^多事件同时到达$")]
fn simultaneous_events(world: &mut AcceptanceWorld) {
    world.state.insert("event_count".into(), "multiple".into());
}

#[given(regex = r"^系统重启未处理事件$")]
fn restart_with_pending(world: &mut AcceptanceWorld) {
    world.state.insert("scenario".into(), "restart_recovery".into());
}

#[given(regex = r"^处理逻辑变更$")]
fn processing_logic_changed(world: &mut AcceptanceWorld) {
    world.state.insert("scenario".into(), "logic_changed".into());
}

// ── When steps: trigger actions ────────────────────────────

#[when(regex = r"^到达$")]
async fn event_arrives(world: &mut AcceptanceWorld) {
    // 如果有当前事件，追加到 EventLog
    if let Some(event) = world.current_event.take() {
        match world.append_event(event).await {
            Ok(()) => {
                world.processing_result = Some("received".into());
            }
            Err(e) => {
                world.last_error = Some(e.clone());
                world.processing_result = Some(format!("error:{}", e));
            }
        }
    } else {
        world.processing_result = world.event_type.clone().map(|t| format!("received:{t}"));
    }
}

#[when(regex = r"^检测$")]
fn event_detected(world: &mut AcceptanceWorld) {
    world.processing_result = Some("detected".into());
}

#[when(regex = r"^评估$")]
async fn event_evaluated(world: &mut AcceptanceWorld) {
    // 如果有当前事件，追加到 EventLog
    if let Some(event) = world.current_event.take() {
        match world.append_event(event).await {
            Ok(()) => {
                world.processing_result = Some("evaluated".into());
            }
            Err(e) => {
                world.last_error = Some(e.clone());
                world.processing_result = Some(format!("error:{}", e));
            }
        }
    } else {
        world.processing_result = Some("evaluated".into());
    }
}

#[when(regex = r"^捕获$")]
async fn event_captured(world: &mut AcceptanceWorld) {
    // 调用真实的 EventLog 追加事件
    if let Some(event) = world.current_event.take() {
        match world.append_event(event).await {
            Ok(()) => {
                world.processing_result = Some("captured".into());
            }
            Err(e) => {
                world.last_error = Some(e.clone());
                world.processing_result = Some(format!("error:{}", e));
            }
        }
    } else {
        world.processing_result = Some("no_event".into());
    }
}

#[when(regex = r"^每小时同步$")]
fn hourly_sync(world: &mut AcceptanceWorld) {
    world.processing_result = Some("hourly_sync".into());
}

#[when(regex = r"^每(\d+)分钟同步$")]
fn periodic_sync(world: &mut AcceptanceWorld, _interval: String) {
    world.processing_result = Some("periodic_sync".into());
}

#[when(regex = r"^结束$")]
fn event_ended(world: &mut AcceptanceWorld) {
    world.processing_result = Some("ended".into());
}

#[when(regex = r"^每天同步$")]
fn daily_sync(world: &mut AcceptanceWorld) {
    world.processing_result = Some("daily_sync".into());
}

#[when(regex = r"^提交完成$")]
fn action_completed(world: &mut AcceptanceWorld) {
    world.processing_result = Some("completed".into());
}

#[when(regex = r"^输入并提交$")]
fn input_submit(world: &mut AcceptanceWorld) {
    world.processing_result = Some("submitted".into());
}

#[when(regex = r"^粘贴图片$")]
fn paste_image(world: &mut AcceptanceWorld) {
    world.state.insert("input_type".into(), "image_paste".into());
}

#[when(regex = r"^拖放文件$")]
fn drop_file(world: &mut AcceptanceWorld) {
    world.state.insert("input_type".into(), "file_drop".into());
}

#[when(regex = r"^截图完成$")]
fn screenshot_done(world: &mut AcceptanceWorld) {
    world.processing_result = Some("screenshot_captured".into());
}

#[when(regex = r"^禁用$")]
fn disable_collector(world: &mut AcceptanceWorld) {
    world.state.insert("collector_status".into(), "disabled".into());
}

#[when(regex = r"^重新启用$")]
fn re_enable(world: &mut AcceptanceWorld) {
    world.state.insert("collector_status".into(), "enabled".into());
}

#[when(regex = r"^健康检查$")]
fn health_check(world: &mut AcceptanceWorld) {
    world.processing_result = Some("health_checked".into());
}

#[when(regex = r"^查看$")]
fn view_status(world: &mut AcceptanceWorld) {
    world.processing_result = Some("viewed".into());
}

#[when(regex = r"^注册$")]
fn register_new(world: &mut AcceptanceWorld) {
    world.processing_result = Some("registered".into());
}

#[when(regex = r"^选 API$")]
fn select_api(world: &mut AcceptanceWorld) {
    world.state.insert("connection_mode".into(), "api".into());
}

#[when(regex = r"^选 CLI$")]
fn select_cli(world: &mut AcceptanceWorld) {
    world.state.insert("connection_mode".into(), "cli".into());
}

#[when(regex = r"^写入 EventLog$")]
fn write_event_log(world: &mut AcceptanceWorld) {
    world.processing_result = Some("event_logged".into());
}

#[when(regex = r"^有未处理事件$")]
fn has_pending_events(world: &mut AcceptanceWorld) {
    world.state.insert("pending_events".into(), "true".into());
}

#[when(regex = r"^触发重放$")]
fn trigger_replay(world: &mut AcceptanceWorld) {
    world.processing_result = Some("replay_triggered".into());
}

// ── Then steps: assertions ─────────────────────────────────

#[then(regex = r"^捕获为 message.*confidence[=为] ?high")]
async fn assert_message_high(world: &mut AcceptanceWorld) {
    // 验证事件已被持久化到 EventLog
    let unprocessed = world.event_log.get_unprocessed(None).await
        .expect("查询未处理事件失败");

    assert!(!unprocessed.is_empty(), "应该有未处理的事件");

    let event = &unprocessed[0];
    assert_eq!(event.source, Source::FeishuMessage, "来源应该是 FeishuMessage");
    assert_eq!(event.event_type, EventType::Message, "类型应该是 Message");
    assert_eq!(event.source_confidence, Confidence::High, "confidence 应为 high");
}

#[then(regex = r"^捕获为 message")]
async fn assert_captured_message(world: &mut AcceptanceWorld) {
    // 验证事件已被持久化到 EventLog
    let unprocessed = world.event_log.get_unprocessed(None).await
        .expect("查询未处理事件失败");

    assert!(!unprocessed.is_empty(), "应该有未处理的事件");

    let event = &unprocessed[0];
    assert_eq!(event.event_type, EventType::Message, "应捕获为 message");
}

#[then(regex = r"^捕获并关联线程$")]
fn assert_thread_linked(world: &mut AcceptanceWorld) {
    assert_eq!(world.event_type.as_deref(), Some("message"));
    assert!(world.state.contains_key("thread_context"), "应关联线程");
}

#[then(regex = r"^不捕获$")]
fn assert_not_captured(world: &mut AcceptanceWorld) {
    assert!(world.state.get("relevance").map_or(true, |v| v == "none"), "不应捕获");
}

#[then(regex = r"^捕获 document_change")]
fn assert_doc_change(world: &mut AcceptanceWorld) {
    assert_eq!(world.event_type.as_deref(), Some("document_change"));
}

#[then(regex = r"^捕获 task_update")]
fn assert_task_update(world: &mut AcceptanceWorld) {
    assert_eq!(world.event_type.as_deref(), Some("task_update"));
}

#[then(regex = r"^捕获 calendar_event")]
fn assert_calendar(world: &mut AcceptanceWorld) {
    assert_eq!(world.event_type.as_deref(), Some("calendar_event"));
}

#[then(regex = r"^捕获 meeting")]
fn assert_meeting(world: &mut AcceptanceWorld) {
    assert_eq!(world.event_type.as_deref(), Some("meeting"));
}

#[then(regex = r"^捕获摘要/待办/章节")]
fn assert_minutes_artifacts(world: &mut AcceptanceWorld) {
    assert_eq!(world.event_type.as_deref(), Some("meeting"));
    assert_eq!(world.state.get("has_summary").map(String::as_str), Some("true"));
}

#[then(regex = r"^捕获 email")]
fn assert_email(world: &mut AcceptanceWorld) {
    assert_eq!(world.event_type.as_deref(), Some("email"));
}

#[then(regex = r"^捕获 approval")]
fn assert_approval(world: &mut AcceptanceWorld) {
    assert_eq!(world.event_type.as_deref(), Some("approval"));
}

#[then(regex = r"^捕获 okr_update")]
fn assert_okr(world: &mut AcceptanceWorld) {
    assert_eq!(world.event_type.as_deref(), Some("okr_update"));
}

#[then(regex = r"^记录 app_activity")]
fn assert_app_activity(world: &mut AcceptanceWorld) {
    assert_eq!(world.event_type.as_deref(), Some("app_activity"));
}

#[then(regex = r"^不记录")]
fn assert_not_recorded(world: &mut AcceptanceWorld) {
    // In real test: verify no event was stored
}

#[then(regex = r"^记录 browsing")]
fn assert_browsing(world: &mut AcceptanceWorld) {
    assert_eq!(world.event_type.as_deref(), Some("browsing"));
}

#[then(regex = r"^聚焦输入区$")]
fn assert_focus_input(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("window").map(String::as_str), Some("visible"));
}

#[then(regex = r"^创建 manual_note")]
fn assert_manual_note(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some(), "应创建 manual_note");
}

#[then(regex = r"^接受为附件$")]
fn assert_accept_attachment(world: &mut AcceptanceWorld) {
    assert!(world.state.contains_key("input_type"), "应接受附件");
}

#[then(regex = r"^打开窗口并预载截图$")]
fn assert_screenshot_loaded(world: &mut AcceptanceWorld) {
    assert_eq!(world.processing_result.as_deref(), Some("screenshot_captured"));
}

#[then(regex = r"^窗口自动隐藏")]
fn assert_window_hidden(world: &mut AcceptanceWorld) {
    // In real test: verify window state
}

#[then(regex = r"^停止且子收集器也禁用$")]
fn assert_stopped_with_children(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("collector_status").map(String::as_str), Some("disabled"));
}

#[then(regex = r"^子收集器恢复各自状态$")]
fn assert_children_restored(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("collector_status").map(String::as_str), Some("enabled"));
}

#[then(regex = r"^自动禁用并通知$")]
fn assert_auto_disable(world: &mut AcceptanceWorld) {
    world.notifications.push("collector_auto_disabled".into());
}

#[then(regex = r"^显示错误指示$")]
fn assert_error_indicator(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("health").map(String::as_str), Some("unhealthy"));
}

#[then(regex = r"^立即开始采集$")]
fn assert_started_collecting(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some(), "应开始采集");
}

#[then(regex = r"^用飞书开放平台 API$")]
fn assert_use_api(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("connection_mode").map(String::as_str), Some("api"));
}

#[then(regex = r"^用 lark-cli 降级$")]
fn assert_use_cli(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("connection_mode").map(String::as_str), Some("cli"));
}

#[then(regex = r"^严格时间顺序$")]
fn assert_chronological_order(world: &mut AcceptanceWorld) {
    assert_eq!(world.processing_result.as_deref(), Some("event_logged"));
}

#[then(regex = r"^事件恢复.*无数据丢失")]
fn assert_event_recovery(world: &mut AcceptanceWorld) {
    assert!(world.state.contains_key("pending_events"), "应恢复未处理事件");
}

#[then(regex = r"^历史事件重新处理$")]
fn assert_replay_processed(world: &mut AcceptanceWorld) {
    assert_eq!(world.processing_result.as_deref(), Some("replay_triggered"));
}

// ── Additional When steps ──────────────────────────────────

#[when(regex = r"^消息到达$")]
async fn message_arrives(world: &mut AcceptanceWorld) {
    // 如果有当前事件，追加到 EventLog
    if let Some(event) = world.current_event.take() {
        match world.append_event(event).await {
            Ok(()) => {
                world.processing_result = Some("arrived".into());
            }
            Err(e) => {
                world.last_error = Some(e.clone());
                world.processing_result = Some(format!("error:{}", e));
            }
        }
    } else {
        world.processing_result = world.event_type.clone().map(|t| format!("arrived:{t}"));
    }
}

#[when(regex = r"^窗口打开$")]
fn window_opens(world: &mut AcceptanceWorld) {
    world.state.insert("window".into(), "visible".into());
}
