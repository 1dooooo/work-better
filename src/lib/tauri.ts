import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface Event {
  id: string;
  timestamp: string;
  collected_at: string;
  source: string;
  source_confidence: string;
  type: string;
  content: unknown;
  raw_payload: string;
  tags: string[];
  related_ids: string[];
  attachments: unknown[];
}

export async function getEvents(limit?: number): Promise<Event[]> {
  return invoke<Event[]>("get_events", { limit: limit ?? null });
}

export async function getUnprocessedCount(): Promise<number> {
  return invoke<number>("get_unprocessed_count");
}

export async function markEventProcessed(eventId: string): Promise<void> {
  return invoke("mark_event_processed", { eventId });
}

export async function triggerManualCapture(text: string): Promise<Event> {
  return invoke<Event>("trigger_manual_capture", { text });
}

export async function triggerFeishuCollect(
  chatId?: string,
  limit?: number,
): Promise<number> {
  return invoke<number>("trigger_feishu_collect", {
    chatId: chatId ?? null,
    limit: limit ?? null,
  });
}

// ─── Collector Management ───────────────────────────────────────────

export interface CollectorStatus {
  id: string;
  name: string;
  enabled: boolean;
  healthy: boolean;
}

export interface CollectorHealth {
  level: string;
  message: string | null;
  error_count: number;
}

export async function getCollectorStatuses(): Promise<CollectorStatus[]> {
  return invoke<CollectorStatus[]>("get_collector_statuses");
}

export async function listCollectors(): Promise<string[]> {
  return invoke<string[]>("list_collectors");
}

export async function enableCollector(id: string): Promise<void> {
  return invoke("enable_collector", { id });
}

export async function disableCollector(id: string): Promise<void> {
  return invoke("disable_collector", { id });
}

export async function checkCollectorHealth(
  id: string,
): Promise<CollectorHealth> {
  return invoke<CollectorHealth>("check_collector_health", { id });
}

// ─── Feishu Configuration ───────────────────────────────────────────

/**
 * 获取飞书采集模式（"cli" | "api"）
 */
export async function getFeishuMode(): Promise<string> {
  return invoke<string>("get_feishu_mode");
}

/**
 * 保存飞书采集模式
 */
export async function saveFeishuMode(mode: string): Promise<void> {
  return invoke("save_feishu_mode", { mode });
}

/**
 * 获取飞书会话 ID
 */
export async function getFeishuChatId(): Promise<string> {
  return invoke<string>("get_feishu_chat_id");
}

/**
 * 保存飞书会话 ID
 */
export async function saveFeishuChatId(chatId: string): Promise<void> {
  return invoke("save_feishu_chat_id", { chatId });
}

// ─── Feishu Events ──────────────────────────────────────────────────

/**
 * 监听飞书采集完成事件
 *
 * @param callback 采集完成时的回调，count 为本次采集的事件数
 * @returns 取消监听函数
 */
export async function onFeishuCollectComplete(
  callback: (count: number) => void,
): Promise<() => void> {
  return listen<number>("feishu:collect-complete", (event) => {
    callback(event.payload);
  });
}

// ─── Scheduler Management ───────────────────────────────────────────

export interface TaskInfo {
  id: string;
  name: string;
  layer: string;
  cron: string;
  sla_ms: number;
}

export async function listScheduledTasks(): Promise<TaskInfo[]> {
  return invoke<TaskInfo[]>("list_scheduled_tasks");
}

export async function pauseScheduler(): Promise<void> {
  return invoke("pause_scheduler");
}

export async function resumeScheduler(): Promise<void> {
  return invoke("resume_scheduler");
}

export async function isSchedulerPaused(): Promise<boolean> {
  return invoke<boolean>("is_scheduler_paused");
}
