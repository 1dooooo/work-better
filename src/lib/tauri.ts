import { invoke } from "@tauri-apps/api/core";

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
  chatId: string,
  limit?: number,
): Promise<number> {
  return invoke<number>("trigger_feishu_collect", {
    chatId,
    limit: limit ?? null,
  });
}
