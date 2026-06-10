//! 采集相关 Tauri 命令

use serde_json::json;
use tauri::Emitter;
use wb_core::event::{Confidence, Event, EventLog, EventType, Source};
use wb_collector::feishu::messages::FeishuMessageCollector;

use super::collectors::get_collector_manager;
use super::events::get_event_log;
use super::settings::load_config_for_collect;

/// 飞书采集器 ID
const FEISHU_COLLECTOR_ID: &str = "feishu";

/// 触发飞书消息采集
///
/// 通过 CollectorManager 调用，支持 enable/disable 状态检查。
/// 采集成功后通过 Tauri 事件系统通知前端。
///
/// # 参数
/// - `app`: Tauri AppHandle，用于发射事件
/// - `chat_id`: 可选的飞书会话 ID；若不传则从配置读取
/// - `limit`: 最大采集数量
#[tauri::command]
pub async fn trigger_feishu_collect(
    app: tauri::AppHandle,
    chat_id: Option<String>,
    limit: Option<u32>,
) -> Result<usize, String> {
    let manager = get_collector_manager();

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
        // 用户显式传入 chat_id，直接调用采集（绕过 Manager 中注册的默认 chat_id）
        FeishuMessageCollector::collect(cid, limit)
            .map_err(|e| format!("飞书采集失败: {e}"))?
    } else {
        // 未传入 chat_id，尝试通过 Manager 采集（使用注册时的配置）
        match manager.collect_one(FEISHU_COLLECTOR_ID).await {
            Some(Ok(events)) => events,
            Some(Err(e)) => return Err(format!("飞书采集失败: {e}")),
            None => {
                // 采集器未注册到 Manager，从配置读取 chat_id
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

    let log = get_event_log().lock().await;
    for event in &events {
        log.append(event).await.map_err(|e| e.to_string())?;
    }
    drop(log);

    // 从采集的消息中自动发现任务
    let mut discovered_count = 0;
    for event in &events {
        let text = match &event.content {
            serde_json::Value::Object(obj) => {
                obj.get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            }
            serde_json::Value::String(s) => s.clone(),
            _ => continue,
        };

        if text.is_empty() {
            continue;
        }

        let discovery = super::tasks::get_task_discovery();
        let mut disc = discovery.lock().await;
        let tasks = disc.discover_from_message(&text);
        discovered_count += tasks.len();
    }

    if discovered_count > 0 {
        eprintln!(
            "[collect] 飞书采集完成: {} 条事件, 发现 {} 个待确认任务",
            count, discovered_count
        );
    }

    // 通知前端采集完成
    app.emit("feishu:collect-complete", count)
        .map_err(|e| format!("发射事件失败: {e}"))?;

    Ok(count)
}

/// 手动捕获事件
#[tauri::command]
pub async fn trigger_manual_capture(text: String) -> Result<Event, String> {
    let event = Event::new(
        Source::UserCapture,
        Confidence::High,
        EventType::ManualNote,
        json!({"text": text}),
        json!({"text": text}).to_string(),
    );

    let log = get_event_log().lock().await;
    log.append(&event).await.map_err(|e| e.to_string())?;

    Ok(event)
}
