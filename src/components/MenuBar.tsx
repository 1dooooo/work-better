/**
 * MenuBar — macOS 菜单栏面板（Raycast/Fantastical 风格）
 *
 * 设计参考：
 * - Raycast: 深色毛玻璃、高密度、原生 macOS 语言
 * - Fantastical: 清晰分组、语义化颜色、一键操作
 * - Apple HIG: SF Pro 字体、8pt 圆角、系统强调色
 */

import { useState, useEffect, useCallback, useMemo } from "react";
import {
  getEvents,
  getUnprocessedCount,
  getPendingNotifications,
  markNotificationRead,
  getPendingTasks,
  getSystemStatus,
  showCaptureWindow,
  triggerBatchProcess,
  type Event,
  type NotificationRecord,
  type PendingTaskDto,
  type SystemStatus,
  type NotifyKind,
} from "../lib/tauri";
import { invoke } from "@tauri-apps/api/core";
import {
  Zap,
  Monitor,
  PenLine,
  Camera,
  Check,
  Clock,
  Activity,
  ListTodo,
  Play,
  ExternalLink,
  AlertTriangle,
  Info,
  CheckCircle2,
  ChevronRight,
  RefreshCw,
  Bell,
} from "lucide-react";
import { cn } from "@/lib/utils";

// ─── 事件类型配色（macOS 系统色）─────────────────────────────────

const EVENT_TYPE_CONFIG: Record<string, { color: string; bg: string; label: string }> = {
  // PascalCase（后端 Rust 枚举序列化格式）
  Message: { color: "text-macos-blue", bg: "bg-macos-blue/10", label: "MSG" },
  DocumentChange: { color: "text-macos-gray", bg: "bg-white/5", label: "DOC" },
  TaskUpdate: { color: "text-macos-pink", bg: "bg-macos-pink/10", label: "TASK" },
  Meeting: { color: "text-macos-orange", bg: "bg-macos-orange/10", label: "MTG" },
  CalendarEvent: { color: "text-macos-orange", bg: "bg-macos-orange/10", label: "CAL" },
  Email: { color: "text-macos-blue", bg: "bg-macos-blue/10", label: "MAIL" },
  Approval: { color: "text-macos-green", bg: "bg-macos-green/10", label: "APPR" },
  OkrUpdate: { color: "text-macos-purple", bg: "bg-macos-purple/10", label: "OKR" },
  Browsing: { color: "text-macos-gray", bg: "bg-white/5", label: "WEB" },
  AppActivity: { color: "text-macos-blue", bg: "bg-macos-blue/10", label: "APP" },
  ManualNote: { color: "text-macos-purple", bg: "bg-macos-purple/10", label: "NOTE" },
  // snake_case 兼容
  message: { color: "text-macos-blue", bg: "bg-macos-blue/10", label: "MSG" },
  issue: { color: "text-macos-orange", bg: "bg-macos-orange/10", label: "ISS" },
  pr: { color: "text-macos-green", bg: "bg-macos-green/10", label: "PR" },
  document: { color: "text-macos-gray", bg: "bg-white/5", label: "DOC" },
  note: { color: "text-macos-purple", bg: "bg-macos-purple/10", label: "NOTE" },
  task: { color: "text-macos-pink", bg: "bg-macos-pink/10", label: "TASK" },
};

function getEventTypeConfig(type: string) {
  return EVENT_TYPE_CONFIG[type] ?? { color: "text-macos-gray", bg: "bg-white/5", label: type.slice(0, 4).toUpperCase() };
}

// ─── 通知分组配置 ─────────────────────────────────────────────

interface NotifyGroupConfig {
  label: string;
  icon: typeof Bell;
  color: string;
  bg: string;
  border: string;
}

const NOTIFY_GROUP_CONFIG: Record<NotifyKind, NotifyGroupConfig> = {
  Confirm: {
    label: "待确认",
    icon: AlertTriangle,
    color: "text-macos-orange",
    bg: "bg-macos-orange/8",
    border: "border-l-macos-orange",
  },
  Reminder: {
    label: "提醒",
    icon: Info,
    color: "text-macos-blue",
    bg: "bg-macos-blue/8",
    border: "border-l-macos-blue",
  },
  TaskDone: {
    label: "已完成",
    icon: CheckCircle2,
    color: "text-macos-green",
    bg: "bg-macos-green/8",
    border: "border-l-macos-green",
  },
};

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
  return new Date(timestamp).toLocaleDateString("zh-CN", { month: "short", day: "numeric" });
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
  const textKeys = ["summary", "title", "text", "message", "content", "body", "description", "name", "subject"];
  for (const key of textKeys) {
    if (typeof obj[key] === "string" && obj[key].trim()) {
      return (obj[key] as string).slice(0, 60);
    }
  }

  // 第二优先：组合多个有意义字段
  const parts: string[] = [];
  const detailKeys = ["sender", "app", "url", "action", "status", "approved", "chat_id", "doc_id", "okr_id", "meeting_id", "event_id", "duration_min"];
  for (const key of detailKeys) {
    if (obj[key] !== undefined && obj[key] !== null) {
      const val = typeof obj[key] === "boolean" ? (obj[key] ? "✓" : "✗") : String(obj[key]);
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

// ─── 通知分组 Hook ─────────────────────────────────────────────

function useGroupedNotifications(notifications: NotificationRecord[]) {
  return useMemo(() => {
    const groups: Record<NotifyKind, NotificationRecord[]> = {
      Confirm: [],
      Reminder: [],
      TaskDone: [],
    };
    for (const n of notifications) {
      groups[n.kind].push(n);
    }
    return groups;
  }, [notifications]);
}

// ─── MenuBar 组件 ──────────────────────────────────────────────

export default function MenuBar() {
  const [unprocessedCount, setUnprocessedCount] = useState(0);
  const [events, setEvents] = useState<Event[]>([]);
  const [notifications, setNotifications] = useState<NotificationRecord[]>([]);
  const [pendingTasks, setPendingTasks] = useState<PendingTaskDto[]>([]);
  const [systemStatus, setSystemStatus] = useState<SystemStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [processing, setProcessing] = useState(false);

  const groupedNotifications = useGroupedNotifications(notifications);

  const refresh = useCallback(async () => {
    try {
      const [count, recentEvents, pendingNotifs, tasks, status] = await Promise.all([
        getUnprocessedCount(),
        getEvents(10),
        getPendingNotifications().catch(() => []),
        getPendingTasks().catch(() => []),
        getSystemStatus().catch(() => null),
      ]);
      setUnprocessedCount(count);
      setEvents(recentEvents);
      setNotifications(pendingNotifs);
      setPendingTasks(tasks);
      setSystemStatus(status);
    } catch (err) {
      console.error("[MenuBar] refresh failed:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
    const interval = setInterval(refresh, 30_000);
    return () => clearInterval(interval);
  }, [refresh]);

  const handleDismissNotification = async (id: string) => {
    try {
      await markNotificationRead(id);
      setNotifications((prev) => prev.filter((n) => n.id !== id));
    } catch (err) {
      console.error("[MenuBar] dismiss notification failed:", err);
    }
  };

  const handleNotificationClick = async (notif: NotificationRecord) => {
    try {
      await markNotificationRead(notif.id);
      setNotifications((prev) => prev.filter((n) => n.id !== notif.id));
      await invoke("show_main_window");
      if (notif.action_url) {
        const mainWindow = await invoke("get_main_window");
        if (mainWindow) {
          const { emit } = await import("@tauri-apps/api/event");
          await emit("navigate-to", { url: notif.action_url });
        }
      }
    } catch (err) {
      console.error("[MenuBar] notification click failed:", err);
    }
  };

  const handleTriggerProcess = async () => {
    if (processing) return;
    setProcessing(true);
    try {
      await triggerBatchProcess();
      await refresh();
    } catch (err) {
      console.error("[MenuBar] trigger process failed:", err);
    } finally {
      setProcessing(false);
    }
  };

  const handleOpenMainWindow = () => {
    invoke("show_main_window").catch(() => {});
  };

  const activeGroups = (["Confirm", "Reminder", "TaskDone"] as NotifyKind[]).filter(
    (kind) => groupedNotifications[kind].length > 0
  );

  return (
    <div className="flex h-full flex-col bg-[var(--glass-bg)] backdrop-blur-[var(--glass-blur)] text-white select-none overflow-hidden font-[-apple-system,BlinkMacSystemFont,'SF Pro Text','Helvetica Neue',sans-serif]">
      {/* ── Header (merged with status) ── */}
      <header className="flex items-center justify-between px-3 py-1.5">
        <div className="flex items-center gap-1.5">
          <div className="flex h-4 w-4 items-center justify-center rounded bg-macos-blue/20">
            <Zap className="h-2.5 w-2.5 text-macos-blue" />
          </div>
          <span className="text-[11px] font-semibold text-white/90">Work Better</span>
          {systemStatus && (
            <div className="flex items-center gap-1 ml-1.5">
              <span className={cn("h-1.5 w-1.5 rounded-full", systemStatus.scheduler_running ? "bg-macos-green" : "bg-macos-gray")} />
              <span className="text-[9px] text-macos-gray">
                {systemStatus.collectors_healthy}/{systemStatus.collectors_total} 采集
              </span>
              <span className="text-[9px] text-macos-dark-gray">·</span>
              <span className="text-[9px] text-macos-gray">
                {systemStatus.scheduler_running ? "调度中" : "已暂停"}
              </span>
            </div>
          )}
        </div>
        <div className="flex items-center gap-1.5">
          {systemStatus && systemStatus.today_processed_count > 0 && (
            <span className="text-[9px] text-macos-gray tabular-nums">今日 {systemStatus.today_processed_count}</span>
          )}
          {unprocessedCount > 0 && (
            <span className="flex h-3.5 min-w-3.5 items-center justify-center rounded-full bg-macos-blue px-1 text-[8px] font-semibold text-white tabular-nums">
              {unprocessedCount}
            </span>
          )}
        </div>
      </header>

      {/* ── 最近事件 ── */}
      <div className="flex-1 flex flex-col overflow-hidden min-h-0">
        <div className="flex items-center justify-between px-3 py-1">
          <div className="flex items-center gap-1">
            <Clock className="h-2.5 w-2.5 text-macos-dark-gray" />
            <span className="text-[9px] font-medium text-macos-dark-gray uppercase tracking-wider">最近事件</span>
          </div>
          <button
            className="flex h-4 w-4 items-center justify-center rounded text-macos-dark-gray hover:text-white hover:bg-white/10 transition-colors"
            onClick={refresh}
          >
            <RefreshCw className="h-2.5 w-2.5" />
          </button>
        </div>

        <div className="flex-1 overflow-y-auto px-1.5 min-h-0">
          {loading ? (
            <div className="flex items-center justify-center py-6 text-[11px] text-macos-dark-gray">加载中...</div>
          ) : events.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-6 text-macos-dark-gray">
              <Activity className="h-4 w-4 mb-1" />
              <span className="text-[11px]">暂无事件</span>
            </div>
          ) : (
            events.map((event) => {
              const typeConfig = getEventTypeConfig(event.type);
              return (
                <div
                  key={event.id}
                  className="group flex items-center gap-2 rounded-md px-2 py-[4px] hover:bg-white/[0.06] transition-colors cursor-default"
                >
                  {/* 类型色块 */}
                  <span className={cn("text-[8px] font-bold w-7 flex-shrink-0 tabular-nums rounded px-1 py-0.5 text-center", typeConfig.color, typeConfig.bg)}>
                    {typeConfig.label}
                  </span>
                  {/* 内容摘要 */}
                  <span className="flex-1 text-[11px] text-white/80 truncate min-w-0">{getEventSummary(event.content)}</span>
                  {/* 时间 */}
                  <span className="text-[8px] text-macos-dark-gray flex-shrink-0 tabular-nums">{formatRelativeTime(event.timestamp)}</span>
                </div>
              );
            })
          )}
        </div>
      </div>

      {/* ── 今日待办 ── */}
      {pendingTasks.length > 0 && (
        <div className="border-t border-white/[0.06]">
          <div className="flex items-center gap-1.5 px-3 py-1">
            <ListTodo className="h-2.5 w-2.5 text-macos-blue" />
            <span className="text-[9px] font-medium text-macos-dark-gray uppercase tracking-wider">
              今日待办
            </span>
            <span className="text-[8px] text-macos-dark-gray tabular-nums">{pendingTasks.length}</span>
          </div>
          <div className="px-2 pb-1 space-y-0.5">
            {pendingTasks.slice(0, 3).map((task) => (
              <div
                key={task.id}
                className="group flex items-center gap-2 rounded-md px-2 py-[4px] hover:bg-white/[0.06] transition-colors cursor-pointer"
              >
                <div className="flex h-3 w-3 items-center justify-center rounded-sm border border-macos-blue/40">
                  <ListTodo className="h-1.5 w-1.5 text-macos-blue" />
                </div>
                <span className="flex-1 text-[11px] text-white/80 truncate min-w-0">
                  {task.title}
                </span>
                <ChevronRight className="h-2.5 w-2.5 text-macos-dark-gray opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0" />
              </div>
            ))}
          </div>
        </div>
      )}

      {/* ── 通知中心 ── */}
      {activeGroups.length > 0 && (
        <div className="border-t border-white/[0.06]">
          <div className="flex items-center gap-1 px-3 py-1">
            <Bell className="h-2.5 w-2.5 text-macos-dark-gray" />
            <span className="text-[9px] font-medium text-macos-dark-gray uppercase tracking-wider">通知</span>
            <span className="text-[8px] text-macos-dark-gray tabular-nums">{notifications.length}</span>
          </div>
          <div className="max-h-[120px] overflow-y-auto px-1.5 pb-1 space-y-0.5">
            {activeGroups.map((kind) => {
              const group = NOTIFY_GROUP_CONFIG[kind];
              const items = groupedNotifications[kind];
              const GroupIcon = group.icon;
              return (
                <div key={kind}>
                  <div className="flex items-center gap-1 px-1 mb-0.5">
                    <GroupIcon className={cn("h-2.5 w-2.5", group.color)} />
                    <span className={cn("text-[8px] font-medium", group.color)}>{group.label}</span>
                    <span className="text-[8px] text-macos-dark-gray tabular-nums">{items.length}</span>
                  </div>
                  <div className="space-y-0.5">
                    {items.slice(0, 3).map((notif) => (
                      <div
                        key={notif.id}
                        className={cn(
                          "group flex items-center gap-2 rounded-md border-l-2 px-2 py-[3px] cursor-pointer transition-colors",
                          group.bg, group.border, "hover:bg-white/[0.06]"
                        )}
                        onClick={() => handleNotificationClick(notif)}
                      >
                        <div className="min-w-0 flex-1">
                          <p className="text-[11px] text-white/90 truncate leading-tight">{notif.title}</p>
                          {notif.body && <p className="text-[9px] text-macos-gray truncate mt-0.5">{notif.body}</p>}
                        </div>
                        <div className="flex items-center gap-0.5 flex-shrink-0">
                          {notif.action_url && (
                            <ExternalLink className="h-2.5 w-2.5 text-macos-dark-gray opacity-0 group-hover:opacity-100 transition-opacity" />
                          )}
                          <button
                            className="flex h-4 w-4 items-center justify-center rounded text-macos-dark-gray hover:text-white hover:bg-white/10 transition-colors"
                            onClick={(e) => { e.stopPropagation(); handleDismissNotification(notif.id); }}
                          >
                            <Check className="h-2.5 w-2.5" />
                          </button>
                        </div>
                      </div>
                    ))}
                    {items.length > 3 && (
                      <p className="text-[8px] text-macos-dark-gray pl-2 py-0.5">+{items.length - 3} 更多</p>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* ── 快捷操作 ── */}
      <div className="flex items-center justify-center gap-0.5 border-t border-white/[0.06] px-2 py-1.5">
        <ActionButton icon={Monitor} label="主窗口" onClick={handleOpenMainWindow} />
        <ActionButton icon={PenLine} label="速记" onClick={() => showCaptureWindow()} />
        <ActionButton icon={Camera} label="截图" onClick={() => invoke("take_screenshot")} />
        <ActionButton
          icon={Play}
          label={processing ? "处理中" : "处理"}
          onClick={handleTriggerProcess}
          disabled={processing}
          spinning={processing}
          accent
        />
      </div>
    </div>
  );
}

// ─── 快捷操作按钮 ──────────────────────────────────────────────

function ActionButton({
  icon: Icon,
  label,
  onClick,
  disabled,
  spinning,
  accent,
}: {
  icon: typeof Monitor;
  label: string;
  onClick: () => void;
  disabled?: boolean;
  spinning?: boolean;
  accent?: boolean;
}) {
  return (
    <button
      className={cn(
        "flex items-center gap-1 rounded-md px-2 py-1 transition-all duration-150",
        "active:scale-95 active:opacity-80",
        accent
          ? "bg-macos-blue/15 text-macos-blue hover:bg-macos-blue/25"
          : "text-white/60 hover:text-white hover:bg-white/[0.08]",
        disabled && "opacity-50 cursor-not-allowed"
      )}
      onClick={onClick}
      disabled={disabled}
      aria-label={label}
    >
      <Icon className={cn("h-3 w-3", spinning && "animate-spin")} />
      <span className="text-[9px] font-medium leading-none">{label}</span>
    </button>
  );
}
