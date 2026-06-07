// ─── Time Formatting (D2-01) ─────────────────────────────────────────

/**
 * Format a timestamp relative to now (e.g., "2 minutes ago", "in 3 hours")
 */
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

  if (seconds < 60) {
    return isFuture ? "即将" : "刚刚";
  }
  if (minutes < 60) {
    return isFuture ? `${minutes} 分钟后` : `${minutes} 分钟前`;
  }
  if (hours < 24) {
    return isFuture ? `${hours} 小时后` : `${hours} 小时前`;
  }
  if (days < 7) {
    return isFuture ? `${days} 天后` : `${days} 天前`;
  }
  return date.toLocaleDateString("zh-CN");
}

/**
 * Format a date to a localized string
 */
export function formatDate(timestamp: string | Date, locale = "zh-CN"): string {
  const date = typeof timestamp === "string" ? new Date(timestamp) : timestamp;
  return date.toLocaleString(locale);
}

// ─── Status Color Mapping (D2-02) ─────────────────────────────────────

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

/**
 * Get the color associated with a status
 */
export function getStatusColor(status: StatusType): string {
  return STATUS_COLORS[status] ?? "#9ca3af";
}

// ─── Priority Label Mapping (D2-03) ───────────────────────────────────

type Priority = "high" | "medium" | "low";

const PRIORITY_LABELS: Record<Priority, string> = {
  high: "高",
  medium: "中",
  low: "低",
};

/**
 * Get the Chinese label for a priority level
 */
export function getPriorityLabel(priority: Priority): string {
  return PRIORITY_LABELS[priority] ?? "未知";
}

/**
 * Get the numeric order for sorting (lower = higher priority)
 */
export function getPriorityOrder(priority: Priority): number {
  const order: Record<Priority, number> = { high: 0, medium: 1, low: 2 };
  return order[priority] ?? 99;
}

// ─── Icon Mapping (D2-04) ─────────────────────────────────────────────

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

/**
 * Get the icon for a view
 */
export function getViewIcon(viewId: string): string {
  return VIEW_ICONS[viewId] ?? "📄";
}

/**
 * Get the icon for a status
 */
export function getStatusIcon(status: string): string {
  return STATUS_ICONS[status] ?? "◦";
}

// ─── Text Truncation (D2-05) ──────────────────────────────────────────

/**
 * Truncate text to a maximum length, adding ellipsis if truncated
 */
export function truncateText(text: string, maxLength: number): string {
  if (maxLength < 0) {
    throw new RangeError("maxLength must be non-negative");
  }
  if (text.length <= maxLength) {
    return text;
  }
  if (maxLength === 0) {
    return "";
  }
  if (maxLength === 1) {
    return "…";
  }
  return text.slice(0, maxLength - 1) + "…";
}

/**
 * Truncate text to a maximum number of words
 */
export function truncateWords(text: string, maxWords: number): string {
  if (maxWords < 0) {
    throw new RangeError("maxWords must be non-negative");
  }
  const words = text.split(/\s+/);
  if (words.length <= maxWords) {
    return text;
  }
  if (maxWords === 0) {
    return "";
  }
  return words.slice(0, maxWords).join(" ") + "…";
}
