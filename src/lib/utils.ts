import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

// ─── Time Formatting ──────────────────────────────────────────────────

export function formatRelativeTime(timestamp: string | Date): string {
  const now = new Date();
  const date = typeof timestamp === "string" ? new Date(timestamp) : timestamp;
  const diffMs = date.getTime() - now.getTime();
  const absDiffMs = Math.abs(diffMs);
  const isFuture = diffMs > 0;

  const seconds = Math.floor(absDiffMs / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (seconds < 60) return isFuture ? "即将" : "刚刚";
  if (minutes < 60) return isFuture ? `${minutes} 分钟后` : `${minutes} 分钟前`;
  if (hours < 24) return isFuture ? `${hours} 小时后` : `${hours} 小时前`;
  if (days < 7) return isFuture ? `${days} 天后` : `${days} 天前`;
  return date.toLocaleDateString("zh-CN");
}

export function formatDate(timestamp: string | Date, locale = "zh-CN"): string {
  const date = typeof timestamp === "string" ? new Date(timestamp) : timestamp;
  return date.toLocaleString(locale);
}

// ─── Status & Priority ────────────────────────────────────────────────

type StatusType = "todo" | "in_progress" | "done" | "error" | "success" | "idle" | "loading";

const STATUS_COLORS: Record<StatusType, string> = {
  todo: "#6b7280",
  in_progress: "#3b82f6",
  done: "#10b981",
  error: "#ef4444",
  success: "#10b981",
  idle: "#9ca3af",
  loading: "#f59e0b",
};

export function getStatusColor(status: StatusType): string {
  return STATUS_COLORS[status] ?? "#9ca3af";
}

type Priority = "high" | "medium" | "low";

const PRIORITY_LABELS: Record<Priority, string> = {
  high: "高",
  medium: "中",
  low: "低",
};

export function getPriorityLabel(priority: Priority): string {
  return PRIORITY_LABELS[priority] ?? "未知";
}

export function getPriorityOrder(priority: Priority): number {
  const order: Record<Priority, number> = { high: 0, medium: 1, low: 2 };
  return order[priority] ?? 99;
}

// ─── Icon Mapping ─────────────────────────────────────────────────────

const VIEW_ICONS: Record<string, string> = {
  events: "📋",
  tasks: "✅",
  timeline: "📅",
  reports: "📊",
  settings: "⚙",
};

const STATUS_ICONS: Record<string, string> = {
  todo: "○",
  in_progress: "◐",
  done: "●",
  error: "✕",
  success: "✓",
  idle: "◦",
  loading: "◌",
};

export function getViewIcon(viewId: string): string {
  return VIEW_ICONS[viewId] ?? "📄";
}

export function getStatusIcon(status: string): string {
  return STATUS_ICONS[status] ?? "◦";
}

// ─── Text Truncation ──────────────────────────────────────────────────

export function truncateText(text: string, maxLength: number): string {
  if (maxLength < 0) throw new RangeError("maxLength must be non-negative");
  if (text.length <= maxLength) return text;
  if (maxLength === 0) return "";
  if (maxLength === 1) return "…";
  return text.slice(0, maxLength - 1) + "…";
}

export function truncateWords(text: string, maxWords: number): string {
  if (maxWords < 0) throw new RangeError("maxWords must be non-negative");
  const words = text.split(/\s+/);
  if (words.length <= maxWords) return text;
  if (maxWords === 0) return "";
  return words.slice(0, maxWords).join(" ") + "…";
}

// ─── Content Helpers ──────────────────────────────────────────────────

/**
 * 将 event.content 转换为字符串
 * content 可能是 string 或 object（需要 JSON.stringify）
 */
export function getContentString(content: unknown): string {
  if (typeof content === "string") return content;
  return JSON.stringify(content);
}

// ─── Event Type Mapping ───────────────────────────────────────────────

interface EventTypeConfig {
  label: string;
  className: string;
}

const EVENT_TYPE_CONFIG: Record<string, EventTypeConfig> = {
  message: { label: "MSG", className: "bg-event-blue-bg text-event-blue-text" },
  issue: { label: "ISS", className: "bg-event-amber-bg text-event-amber-text" },
  pr: { label: "PR", className: "bg-event-green-bg text-event-green-text" },
  document: { label: "DOC", className: "bg-event-gray-bg text-event-gray-text" },
  default: { label: "EVT", className: "bg-event-gray-bg text-event-gray-text" },
};

/**
 * 根据事件类型字符串返回对应的配置（label + className）
 * 支持中英文匹配
 */
export function getEventType(type: string): EventTypeConfig {
  const lowerType = type.toLowerCase();
  if (lowerType.includes("message") || lowerType.includes("消息")) return EVENT_TYPE_CONFIG.message;
  if (lowerType.includes("issue")) return EVENT_TYPE_CONFIG.issue;
  if (lowerType.includes("pr") || lowerType.includes("pull")) return EVENT_TYPE_CONFIG.pr;
  if (lowerType.includes("doc") || lowerType.includes("文档")) return EVENT_TYPE_CONFIG.document;
  return EVENT_TYPE_CONFIG.default;
}
