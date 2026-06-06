//! 采集相关 Tauri 命令

use serde_json::json;
use wb_core::event::{Confidence, Event, EventLog, EventType, Source};
use wb_collector::feishu::messages::FeishuMessageCollector;

use super::events::get_event_log;

/// 触发飞书消息采集
#[tauri::command]
pub async fn trigger_feishu_collect(chat_id: String, limit: Option<u32>) -> Result<usize, String> {
    let limit = limit.unwrap_or(50);

    let events = FeishuMessageCollector::collect(&chat_id, limit).map_err(|e| e.to_string())?;

    let count = events.len();

    let log = get_event_log().lock().await;
    for event in &events {
        log.append(event).await.map_err(|e| e.to_string())?;
    }

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
