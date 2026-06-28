//! 采集相关 Tauri 命令

use serde_json::json;
use tauri::{Emitter, State};
use wb_core::event::{Confidence, Event, EventLog, EventType, Source};
use wb_collector::feishu::messages::FeishuMessageCollector;

use super::events::build_task_runner_from_config;
use super::settings::load_config_for_collect;
use super::test_mode::TestModeState;
use super::AppState;

/// 飞书采集器 ID
const FEISHU_COLLECTOR_ID: &str = "feishu";

/// 触发飞书消息采集
#[tauri::command]
pub async fn trigger_feishu_collect(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    chat_id: Option<String>,
    limit: Option<u32>,
) -> Result<usize, String> {
    let manager = &state.collector_manager;

    // 检查采集器是否启用
    if !manager.is_enabled(FEISHU_COLLECTOR_ID).await {
        return Err("飞书采集器未启用，请先在设置中启用".to_string());
    }

    let limit = limit.unwrap_or(50);

    // 判断 chat_id 来源：优先使用参数，其次从配置读取
    let explicit_chat_id = match chat_id {
        Some(id) if !id.is_empty() => Some(id),
        _ => None,
    };

    let events = if let Some(ref cid) = explicit_chat_id {
        FeishuMessageCollector::collect(cid, limit)
            .map_err(|e| format!("飞书采集失败: {e}"))?
    } else {
        match manager.collect_one(FEISHU_COLLECTOR_ID).await {
            Some(Ok(events)) => events,
            Some(Err(e)) => return Err(format!("飞书采集失败: {e}")),
            None => {
                let config = load_config_for_collect()?;
                let cid = config
                    .collectors
                    .feishu_chat_id
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| {
                        "未配置飞书会话 ID，请在设置中配置或传入 chat_id 参数".to_string()
                    })?;
                FeishuMessageCollector::collect(&cid, limit)
                    .map_err(|e| format!("飞书采集失败: {e}"))?
            }
        }
    };

    let count = events.len();

    let log = state.event_log.lock().await;
    for event in &events {
        log.append(event).await.map_err(|e| e.to_string())?;
    }
    drop(log);

    eprintln!("[collect] 飞书采集完成: {} 条事件", count);

    app.emit("feishu:collect-complete", count)
        .map_err(|e| format!("发射事件失败: {e}"))?;

    Ok(count)
}

/// 手动捕获事件（AI 驱动任务发现）
///
/// 测试模式下跳过 AI 处理，直接创建事件。
#[tauri::command]
pub async fn trigger_manual_capture(
    state: State<'_, AppState>,
    test_mode: State<'_, TestModeState>,
    app: tauri::AppHandle,
    text: String,
) -> Result<Event, String> {
    // 测试模式：跳过 AI 处理，直接创建事件
    if test_mode.is_enabled() {
        let event = Event::new(
            Source::UserCapture,
            Confidence::High,
            EventType::ManualNote,
            json!({"text": text}),
            json!({"text": text}).to_string(),
        );

        let log = state.event_log.lock().await;
        log.append(&event).await.map_err(|e| e.to_string())?;

        eprintln!("[capture] Test mode: saved event directly (no AI)");
        return Ok(event);
    }

    // 正常模式：需要 AI 处理
    let event = Event::new(
        Source::UserCapture,
        Confidence::High,
        EventType::ManualNote,
        json!({"text": text}),
        json!({"text": text}).to_string(),
    );

    let log = state.event_log.lock().await;
    log.append(&event).await.map_err(|e| e.to_string())?;

    // 从数据库查询最近的 UserCapture 事件作为上下文
    let filter = wb_core::event::EventFilter {
        source: Some(Source::UserCapture),
        limit: Some(20),
        ..Default::default()
    };
    let recent_events = log.query(&filter).await.unwrap_or_default();
    drop(log);

    // 构建任务上下文：从最近事件中提取文本
    let existing_tasks: Vec<wb_ai::TaskContext> = recent_events
        .iter()
        .filter(|e| e.id != event.id)
        .enumerate()
        .map(|(i, e)| {
            let text = match &e.content {
                serde_json::Value::Object(obj) => {
                    obj.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string()
                }
                serde_json::Value::String(s) => s.clone(),
                _ => String::new(),
            };
            wb_ai::TaskContext {
                id: format!("recent-{}", i),
                title: text,
                status: "Open".to_string(),
            }
        })
        .collect();

    eprintln!(
        "[capture] recent context: {} events, {} task contexts",
        recent_events.len(),
        existing_tasks.len()
    );

    // AI 驱动：通过 TaskRunner.run_extract() 调用 AI
    let mut runner = build_task_runner_from_config()
        .map_err(|e| format!("AI 模型配置错误: {}", e))?
        .ok_or("AI 模型未配置。请在设置中配置 API Key。")?;

    // 构建合成事件
    let ai_event = wb_core::event::Event::new(
        Source::UserCapture, wb_core::event::Confidence::Medium,
        wb_core::event::EventType::Message,
        serde_json::json!({"text": text, "task_context": existing_tasks.iter().map(|t| serde_json::json!({"id": t.id, "title": t.title, "status": t.status})).collect::<Vec<_>>()}),
        text.clone(),
    );

    let extraction_result = runner.run_extract(&ai_event, 0.5).await;
    let extraction: wb_ai::Extraction = match extraction_result {
        Ok(output) => match serde_json::from_str(&output.content) {
            Ok(e) => e,
            Err(e) => return Err(format!("AI 输出解析失败: {}", e)),
        },
        Err(e) => return Err(format!("AI 模型调用失败: {}", e)),
    };

    if extraction.is_status_update {
        eprintln!(
            "[capture] status_update: text='{}', related_task_id={:?}",
            text, extraction.related_task_id
        );

        let mut disc = state.task_discovery.lock().await;
        let pending_tasks: Vec<_> = disc.pending().iter().map(|p| (p.id.clone(), p.title.clone())).collect();

        let related_title = extraction.related_task_id.as_ref()
            .and_then(|id| {
                pending_tasks.iter().find(|(pid, _)| pid == id).map(|(_, title)| title.clone())
                    .or_else(|| {
                        id.strip_prefix("recent-")
                            .and_then(|idx_str| idx_str.parse::<usize>().ok())
                            .and_then(|idx| existing_tasks.get(idx))
                            .map(|ctx| ctx.title.clone())
                    })
            });

        if let Some(ref title) = related_title {
            eprintln!("[capture] found related task title: '{}'", title);

            if let Some((pid, _)) = pending_tasks.iter().find(|(_, t)| t == title) {
                if let Ok(confirmed) = disc.confirm(pid) {
                    drop(disc);
                    let manager = state.task_manager.lock().await;
                    if let Ok(task) = manager.create(&confirmed.title, confirmed.priority.clone(), confirmed.source.clone()).await {
                        if let Err(e) = manager.transition(&task.id, wb_processor::task::model::TaskStatus::Open).await {
                            eprintln!("[capture] WARN: transition to Open failed: {}", e);
                        }
                        if let Err(e) = manager.transition(&task.id, wb_processor::task::model::TaskStatus::InProgress).await {
                            eprintln!("[capture] WARN: transition to InProgress failed: {}", e);
                        }
                        if let Err(e) = manager.transition(&task.id, wb_processor::task::model::TaskStatus::Done).await {
                            eprintln!("[capture] WARN: transition to Done failed: {}", e);
                        }
                        eprintln!("[capture] task '{}' confirmed and transitioned to Done", task.title);
                        let _ = app.emit("tasks:updated", ());
                    }
                }
            } else {
                drop(disc);
                let manager = state.task_manager.lock().await;
                let all_tasks = manager.list(wb_processor::task::model::TaskFilter::default()).await.unwrap_or_default();
                if let Some(task) = all_tasks.iter().find(|t| {
                    t.status != wb_processor::task::model::TaskStatus::Done &&
                    t.title.contains(title.as_str())
                }) {
                    if let Err(e) = manager.transition(&task.id, wb_processor::task::model::TaskStatus::Done).await {
                        eprintln!("[capture] WARN: transition to Done failed: {}", e);
                    }
                    eprintln!("[capture] task '{}' transitioned to Done", task.title);
                    let _ = app.emit("tasks:updated", ());
                } else {
                    eprintln!("[capture] no matching task found for status update");
                }
            }
        } else {
            drop(disc);
            eprintln!("[capture] no matching task found for status update");
        }
    } else if !extraction.title.is_empty() && extraction.confidence >= 0.5 {
        let mut disc = state.task_discovery.lock().await;
        let pending = wb_processor::task::discovery::PendingTask::new(
            &extraction.title,
            extraction.summary.as_str().into(),
            wb_processor::task::model::TaskSource::Message,
            wb_processor::task::model::TaskPriority::P2,
            extraction.due_date.clone(),
            &text,
        );
        disc.add_pending(pending.clone());
        eprintln!("[capture] new task discovered: '{}'", pending.title);
    }

    Ok(event)
}
