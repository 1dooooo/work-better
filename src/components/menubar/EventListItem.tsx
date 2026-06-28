/**
 * EventListItem — 事件列表项组件
 *
 * 显示事件类型标签、内容摘要和相对时间
 */

import type { Event } from "@/lib/tauri";
import { cn } from "@/lib/utils";

// ─── 事件类型配置 ─────────────────────────────────────────────

interface EventTypeConfig {
  color: string;
  bg: string;
  label: string;
}

const EVENT_TYPE_CONFIG: Record<string, EventTypeConfig> = {
  // PascalCase（后端 Rust 枚举序列化格式）
  Message: { color: "text-macos-blue", bg: "bg-macos-blue/10", label: "MSG" },
  DocumentChange: { color: "text-macos-gray", bg: "bg-macos-gray/10", label: "DOC" },
  TaskUpdate: { color: "text-macos-pink", bg: "bg-macos-pink/10", label: "TASK" },
  Meeting: { color: "text-macos-orange", bg: "bg-macos-orange/10", label: "MTG" },
  CalendarEvent: { color: "text-macos-orange", bg: "bg-macos-orange/10", label: "CAL" },
  Email: { color: "text-macos-blue", bg: "bg-macos-blue/10", label: "MAIL" },
  Approval: { color: "text-macos-green", bg: "bg-macos-green/10", label: "APPR" },
  OkrUpdate: { color: "text-macos-purple", bg: "bg-macos-purple/10", label: "OKR" },
  Browsing: { color: "text-macos-gray", bg: "bg-macos-gray/10", label: "WEB" },
  AppActivity: { color: "text-macos-blue", bg: "bg-macos-blue/10", label: "APP" },
  ManualNote: { color: "text-macos-purple", bg: "bg-macos-purple/10", label: "NOTE" },
  // snake_case 兼容
  message: { color: "text-macos-blue", bg: "bg-macos-blue/10", label: "MSG" },
  issue: { color: "text-macos-orange", bg: "bg-macos-orange/10", label: "ISS" },
  pr: { color: "text-macos-green", bg: "bg-macos-green/10", label: "PR" },
  document: { color: "text-macos-gray", bg: "bg-macos-gray/10", label: "DOC" },
  note: { color: "text-macos-purple", bg: "bg-macos-purple/10", label: "NOTE" },
  task: { color: "text-macos-pink", bg: "bg-macos-pink/10", label: "TASK" },
};

function getEventTypeConfig(type: string): EventTypeConfig {
  return (
    EVENT_TYPE_CONFIG[type] ?? {
      color: "text-macos-gray",
      bg: "bg-macos-gray/10",
      label: type.slice(0, 4).toUpperCase(),
    }
  );
}

// ─── 时间格式化 ────────────────────────────────────────────────

function formatRelativeTime(timestamp: string): string {
  const now = Date.now();
  const then = new Date(timestamp).getTime();
  const diffMs = now - then;
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return "刚刚";
  if (diffMins < 60) return `${diffMins}分`;
  if (diffHours < 24) return `${diffHours}时`;
  if (diffDays < 7) return `${diffDays}天`;
  return new Date(timestamp).toLocaleDateString("zh-CN", {
    month: "short",
    day: "numeric",
  });
}

// ─── 事件内容摘要 ──────────────────────────────────────────────

function getEventSummary(content: unknown): string {
  // 字符串直接返回
  if (typeof content === "string") return content.slice(0, 60);

  // 非对象或 null
  if (typeof content !== "object" || content === null) {
    return String(content).slice(0, 60);
  }

  const obj = content as Record<string, unknown>;

  // 第一优先：按优先级提取可读文本字段
  const textKeys = [
    "summary", "title", "text", "message", "content",
    "body", "description", "name", "subject",
  ];
  for (const key of textKeys) {
    if (typeof obj[key] === "string" && obj[key].trim()) {
      return (obj[key] as string).slice(0, 60);
    }
  }

  // 第二优先：组合多个有意义字段
  const parts: string[] = [];
  const detailKeys = [
    "sender", "app", "url", "action", "status", "approved",
    "chat_id", "doc_id", "okr_id", "meeting_id", "event_id", "duration_min",
  ];
  for (const key of detailKeys) {
    if (obj[key] !== undefined && obj[key] !== null) {
      const val =
        typeof obj[key] === "boolean"
          ? obj[key] ? "✓" : "✗"
          : String(obj[key]);
      parts.push(`${key}: ${val}`);
      if (parts.length >= 2) break;
    }
  }
  if (parts.length > 0) return parts.join(", ").slice(0, 60);

  // 第三优先：尝试从嵌套对象提取
  for (const key of textKeys) {
    if (typeof obj[key] === "object" && obj[key] !== null) {
      const nested = obj[key] as Record<string, unknown>;
      for (const nk of textKeys) {
        if (typeof nested[nk] === "string" && nested[nk].trim()) {
          return (nested[nk] as string).slice(0, 60);
        }
      }
    }
  }

  // 最终 fallback：紧凑 JSON
  try {
    const json = JSON.stringify(content);
    return json.length > 60 ? json.slice(0, 57) + "..." : json;
  } catch {
    return String(content).slice(0, 60);
  }
}

// ─── 类型定义 ─────────────────────────────────────────────────

interface EventListItemProps {
  /** 事件数据 */
  event: Event;
}

// ─── 组件 ─────────────────────────────────────────────────────

export function EventListItem({ event }: EventListItemProps) {
  const typeConfig = getEventTypeConfig(event.type);

  return (
    <div className="group flex items-center gap-2 rounded-lg px-2 py-[5px] hover:bg-accent transition-colors cursor-default">
      {/* 类型标签 */}
      <span
        className={cn(
          "text-[8px] font-bold w-8 flex-shrink-0 tabular-nums rounded px-1 py-0.5 text-center",
          typeConfig.color,
          typeConfig.bg,
        )}
      >
        {typeConfig.label}
      </span>
      {/* 内容摘要 */}
      <span className="flex-1 text-[11px] text-muted-foreground truncate min-w-0">
        {getEventSummary(event.content)}
      </span>
      {/* 时间 */}
      <span className="text-[9px] text-muted-foreground/50 flex-shrink-0 tabular-nums">
        {formatRelativeTime(event.timestamp)}
      </span>
    </div>
  );
}
