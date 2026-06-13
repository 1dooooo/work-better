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
  processed: boolean;
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
  health_level: string; // "healthy" | "degraded" | "unhealthy" | "unknown"
  health_message: string | null;
}

export interface CollectorGroup {
  id: string;
  name: string;
  enabled: boolean;
  collectors: CollectorStatus[];
}

export interface CollectorHealth {
  level: string;
  message: string | null;
  error_count: number;
}

export async function getCollectorGroups(): Promise<CollectorGroup[]> {
  return invoke<CollectorGroup[]>("get_collector_groups");
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

export async function enableCollectorGroup(groupId: string): Promise<void> {
  return invoke("enable_collector_group", { groupId });
}

export async function disableCollectorGroup(groupId: string): Promise<void> {
  return invoke("disable_collector_group", { groupId });
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

// ─── Event Processing ───────────────────────────────────────────────

export interface ProcessResult {
  event_id: string;
  category: string;
  confidence: number;
  processing_path: string;
  model_used: string;
  review_status: ReviewStatus;
  persistence_status: PersistenceStatus;
  timestamp: string;
}

export type ReviewStatus =
  | { Pending: null }
  | { Approved: null }
  | { Rejected: { reason: string } };

export interface PersistenceStatus {
  obsidian: boolean;
  vector_db: boolean;
  sqlite: boolean;
}

export async function processEvent(eventId: string): Promise<ProcessResult> {
  return invoke<ProcessResult>("process_event", { eventId });
}

// ─── Audit Log ────────────────────────────────────────────────────

export interface ProcessingAuditRow {
  event_id: string;
  record_id: string | null;
  trace_id: string;
  step: string;
  timestamp: string;
  duration_ms: number;
  model: string;
  model_version: string;
  prompt_id: string;
  prompt_params: string;
  input_summary: string;
  output: string;
  confidence: number;
  token_input: number;
  token_output: number;
  cost_estimate: number;
  upgrade_reason: string | null;
  previous_model: string | null;
  review_verdict: string | null;
  review_issues: string | null;
  user_action: string | null;
  user_correction: string | null;
}

export interface ExecutionLogRow {
  id: string;
  task_id: string;
  task_name: string;
  status: string;
  started_at: string;
  finished_at: string;
  duration_ms: number;
  output: string | null;
  error: string | null;
  created_at: string;
}

export interface AuditSummary {
  total_processing_audits: number;
  total_execution_logs: number;
  total_tokens: number;
  total_cost: number;
  success_rate: number;
}

export async function getProcessingAudits(options?: {
  step?: string;
  traceId?: string;
  since?: string;
  until?: string;
  limit?: number;
}): Promise<ProcessingAuditRow[]> {
  return invoke<ProcessingAuditRow[]>("get_processing_audits", {
    step: options?.step ?? null,
    traceId: options?.traceId ?? null,
    since: options?.since ?? null,
    until: options?.until ?? null,
    limit: options?.limit ?? null,
  });
}

export async function getExecutionLogs(options?: {
  taskId?: string;
  status?: string;
  limit?: number;
}): Promise<ExecutionLogRow[]> {
  return invoke<ExecutionLogRow[]>("get_execution_logs", {
    taskId: options?.taskId ?? null,
    status: options?.status ?? null,
    limit: options?.limit ?? null,
  });
}

export async function getAuditSummary(): Promise<AuditSummary> {
  return invoke<AuditSummary>("get_audit_summary");
}

// ─── Model Config ─────────────────────────────────────────────────

export interface ModelConfig {
  api_endpoint: string;
  api_key: string;
  token_budget: number;
  small_model: string;
  large_model: string;
}

/**
 * 获取模型配置（用于检查 API Key 是否已配置）
 */
export async function getModelConfig(): Promise<ModelConfig> {
  return invoke<ModelConfig>("get_model_config");
}

// ─── Model Management ─────────────────────────────────────────────

export interface ModelInfo {
  id: string;
  name: string;
}

export interface TestModelResult {
  success: boolean;
  message: string;
  latency_ms: number;
}

/**
 * 获取可用模型列表（从 API 端点获取）
 */
export async function listModels(
  apiEndpoint: string,
  apiKey: string,
): Promise<ModelInfo[]> {
  return invoke<ModelInfo[]>("list_models", { apiEndpoint, apiKey });
}

/**
 * 测试模型连接
 */
export async function testModel(
  apiEndpoint: string,
  apiKey: string,
  model: string,
): Promise<TestModelResult> {
  return invoke<TestModelResult>("test_model", { apiEndpoint, apiKey, model });
}

// ─── Batch Processing ─────────────────────────────────────────────

export interface BatchProcessResult {
  total: number;
  success: number;
  failed: number;
  skipped: number;
  details: BatchProcessDetail[];
}

export interface BatchProcessDetail {
  event_id: string;
  status: string;
  category: string | null;
  error: string | null;
}

/**
 * 批量处理所有未处理事件（开发者模式 - 主动整理）
 */
export async function triggerBatchProcess(): Promise<BatchProcessResult> {
  return invoke<BatchProcessResult>("trigger_batch_process");
}

// ─── Developer Mode ───────────────────────────────────────────────

export async function getDeveloperMode(): Promise<boolean> {
  return invoke<boolean>("get_developer_mode");
}

export async function saveDeveloperMode(enabled: boolean): Promise<void> {
  return invoke("save_developer_mode", { enabled });
}

// ─── Task Management ─────────────────────────────────────────────

export interface TaskDto {
  id: string;
  title: string;
  description: string | null;
  status: string;
  priority: string;
  source: string;
  due_date: string | null;
  created_at: string;
  tags: string[];
}

export interface PendingTaskDto {
  id: string;
  title: string;
  description: string | null;
  source: string;
  priority: string;
  origin_text: string;
  created_at: string;
}

export async function discoverTasksFromText(
  text: string,
  source: string,
): Promise<PendingTaskDto[]> {
  return invoke<PendingTaskDto[]>("discover_tasks_from_text", { text, source });
}

export async function getPendingTasks(): Promise<PendingTaskDto[]> {
  return invoke<PendingTaskDto[]>("get_pending_tasks");
}

export async function confirmPendingTask(
  pendingId: string,
): Promise<TaskDto> {
  return invoke<TaskDto>("confirm_pending_task", { pendingId });
}

export async function rejectPendingTask(pendingId: string): Promise<void> {
  return invoke("reject_pending_task", { pendingId });
}

export async function listTasks(options?: {
  status?: string;
  priority?: string;
}): Promise<TaskDto[]> {
  return invoke<TaskDto[]>("list_tasks", {
    status: options?.status ?? null,
    priority: options?.priority ?? null,
  });
}

export async function createTask(
  title: string,
  priority?: string,
): Promise<TaskDto> {
  return invoke<TaskDto>("create_task", {
    title,
    priority: priority ?? null,
  });
}

export async function updateTaskStatus(
  taskId: string,
  newStatus: string,
): Promise<TaskDto> {
  return invoke<TaskDto>("update_task_status", { taskId, newStatus });
}
