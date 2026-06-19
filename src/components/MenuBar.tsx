/**
 * MenuBar — 菜单栏常驻组件（信息展示中心）
 *
 * 产品定义 F6.1.4：即时信息展示、快捷操作入口
 * 设计参考：Linear/Raycast 风格紧凑型列表
 *
 * 布局：
 *   Header — 应用名 + 待处理计数
 *   最近事件 — 紧凑型列表（状态指示器 + 类型标签 + 来源 + 内容 + 时间）
 *   今日待办 — 截止日临近的任务
 *   待确认通知 — 通知列表 + 点击跳转 + 标记已读
 *   快捷操作 — 打开主窗口 / 速记 / 截图 / 处理
 *   Footer — 系统状态 + 版本号
 */

import { useState, useEffect, useCallback } from "react";
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
} from "../lib/tauri";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  Zap,
  RefreshCw,
  Bell,
  Monitor,
  PenLine,
  Camera,
  Check,
  Clock,
  Activity,
  ListTodo,
  Play,
  ExternalLink,
} from "lucide-react";
import { cn } from "@/lib/utils";

// ─── 事件类型配色 ──────────────────────────────────────────────

const EVENT_TYPE_CONFIG: Record<string, { bg: string; text: string; label: string }> = {
  message: { bg: "bg-blue-100 dark:bg-blue-900/30", text: "text-blue-700 dark:text-blue-400", label: "MSG" },
  issue: { bg: "bg-amber-100 dark:bg-amber-900/30", text: "text-amber-700 dark:text-amber-400", label: "ISSUE" },
  pr: { bg: "bg-green-100 dark:bg-green-900/30", text: "text-green-700 dark:text-green-400", label: "PR" },
  document: { bg: "bg-gray-100 dark:bg-gray-800", text: "text-gray-600 dark:text-gray-400", label: "DOC" },
  note: { bg: "bg-purple-100 dark:bg-purple-900/30", text: "text-purple-700 dark:text-purple-400", label: "NOTE" },
  task: { bg: "bg-orange-100 dark:bg-orange-900/30", text: "text-orange-700 dark:text-orange-400", label: "TASK" },
};

function getEventTypeStyle(type: string) {
  return EVENT_TYPE_CONFIG[type] ?? { bg: "bg-muted", text: "text-muted-foreground", label: type.toUpperCase() };
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
  if (diffMins < 60) return `${diffMins}分钟前`;
  if (diffHours < 24) return `${diffHours}小时前`;
  if (diffDays < 7) return `${diffDays}天前`;
  return new Date(timestamp).toLocaleDateString("zh-CN", { month: "short", day: "numeric" });
}

// ─── 事件内容摘要 ──────────────────────────────────────────────

function getEventSummary(content: unknown): string {
  if (typeof content === "string") return content.slice(0, 80);
  try {
    return JSON.stringify(content).slice(0, 80);
  } catch {
    return String(content).slice(0, 80);
  }
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

  const refresh = useCallback(async () => {
    try {
      const [count, recentEvents, pendingNotifs, tasks, status] = await Promise.all([
        getUnprocessedCount(),
        getEvents(15),
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

  // P0-2: 通知点击跳转 — 打开主窗口并导航到 action_url
  const handleNotificationClick = async (notif: NotificationRecord) => {
    try {
      // 先标记为已读
      await markNotificationRead(notif.id);
      setNotifications((prev) => prev.filter((n) => n.id !== notif.id));

      // 打开主窗口
      await invoke("show_main_window");

      // 如果有 action_url，通过事件通知前端导航
      if (notif.action_url) {
        // 发送导航事件给主窗口
        const mainWindow = await invoke("get_main_window");
        if (mainWindow) {
          // 使用 Tauri event 系统通知主窗口导航
          const { emit } = await import("@tauri-apps/api/event");
          await emit("navigate-to", { url: notif.action_url });
        }
      }
    } catch (err) {
      console.error("[MenuBar] notification click failed:", err);
    }
  };

  // P1-2: 手动触发处理
  const handleTriggerProcess = async () => {
    if (processing) return;
    setProcessing(true);
    try {
      await triggerBatchProcess();
      // 处理完成后刷新数据
      await refresh();
    } catch (err) {
      console.error("[MenuBar] trigger process failed:", err);
    } finally {
      setProcessing(false);
    }
  };

  const handleOpenMainWindow = () => {
    invoke("show_main_window").catch(() => {
      // fallback: 尝试通过事件打开
    });
  };

  return (
    <div className="flex h-full flex-col bg-background text-foreground select-none">
      {/* ── Header ────────────────────────────────────────── */}
      <header className="flex items-center justify-between px-4 py-3 min-h-[44px]">
        <div className="flex items-center gap-2">
          <div className="flex h-5 w-5 items-center justify-center rounded bg-primary text-primary-foreground">
            <Zap className="h-3 w-3" />
          </div>
          <span className="text-xs font-semibold">Work Better</span>
        </div>
        {unprocessedCount > 0 && (
          <Badge variant="default" className="text-[10px] px-1.5 py-0">
            {unprocessedCount} 待处理
          </Badge>
        )}
      </header>

      <Separator />

      {/* ── 最近事件（紧凑型列表）────────────────────────── */}
      <div className="flex flex-1 flex-col overflow-hidden">
        <div className="flex items-center justify-between px-4 py-2">
          <div className="flex items-center gap-1.5">
            <Clock className="h-3.5 w-3.5 text-muted-foreground" />
            <span className="text-[11px] font-medium text-muted-foreground">
              最近事件
            </span>
            {events.length > 0 && (
              <span className="text-[10px] text-muted-foreground bg-muted px-1.5 py-0 rounded-full">
                {events.length}
              </span>
            )}
          </div>
          <Button
            variant="ghost"
            size="icon"
            className="h-5 w-5 text-muted-foreground"
            onClick={refresh}
          >
            <RefreshCw className="h-3 w-3" />
          </Button>
        </div>

        <ScrollArea className="flex-1">
          {loading ? (
            <div className="flex h-20 items-center justify-center text-[11px] text-muted-foreground">
              加载中...
            </div>
          ) : events.length === 0 ? (
            <div className="flex h-20 flex-col items-center justify-center gap-1 text-muted-foreground">
              <Activity className="h-4 w-4" />
              <span className="text-[11px]">暂无事件</span>
            </div>
          ) : (
            <div className="divide-y divide-border/50">
              {events.map((event) => {
                const typeStyle = getEventTypeStyle(event.type);
                return (
                  <div
                    key={event.id}
                    className="group flex items-center px-4 py-1.5 hover:bg-muted/50 transition-colors cursor-default min-h-[36px]"
                  >
                    {/* 状态指示器 */}
                    <div
                      className={cn(
                        "w-1.5 h-1.5 rounded-full mr-2 flex-shrink-0",
                        event.processed ? "bg-muted-foreground/30" : "bg-primary"
                      )}
                    />

                    {/* 类型标签 */}
                    <span
                      className={cn(
                        "text-[9px] font-semibold px-1 py-0.5 rounded mr-2 flex-shrink-0 uppercase tracking-wider leading-none",
                        typeStyle.bg, typeStyle.text
                      )}
                    >
                      {typeStyle.label}
                    </span>

                    {/* 来源 */}
                    <span className="text-[10px] text-muted-foreground mr-2 flex-shrink-0 min-w-[50px] truncate">
                      {event.source}
                    </span>

                    {/* 内容摘要 */}
                    <span className="flex-1 text-[11px] text-foreground truncate mr-2">
                      {getEventSummary(event.content)}
                    </span>

                    {/* 时间 */}
                    <span className="text-[10px] text-muted-foreground flex-shrink-0">
                      {formatRelativeTime(event.timestamp)}
                    </span>
                  </div>
                );
              })}
            </div>
          )}
        </ScrollArea>
      </div>

      {/* ── P1-1: 今日待办 ────────────────────────────────── */}
      {pendingTasks.length > 0 && (
        <>
          <Separator />
          <div className="px-4 py-2">
            <div className="flex items-center gap-1.5 mb-1.5">
              <ListTodo className="h-3.5 w-3.5 text-info" />
              <span className="text-[11px] font-medium text-muted-foreground">
                今日待办
              </span>
              <span className="text-[10px] text-muted-foreground bg-muted px-1.5 py-0 rounded-full">
                {pendingTasks.length}
              </span>
            </div>
            <div className="space-y-1">
              {pendingTasks.slice(0, 3).map((task) => (
                <div
                  key={task.id}
                  className="flex items-center gap-2 rounded bg-info/10 px-2 py-1.5 min-h-[36px]"
                >
                  <div className="min-w-0 flex-1">
                    <p className="text-[11px] font-medium truncate">{task.title}</p>
                    <p className="text-[10px] text-muted-foreground truncate">
                      {task.source} · {task.priority}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </>
      )}

      {/* ── P0-2: 待确认通知（支持点击跳转）───────────────── */}
      {notifications.length > 0 && (
        <>
          <Separator />
          <div className="px-4 py-2">
            <div className="flex items-center gap-1.5 mb-1.5">
              <Bell className="h-3.5 w-3.5 text-warning" />
              <span className="text-[11px] font-medium text-muted-foreground">
                待确认
              </span>
              <span className="text-[10px] text-muted-foreground bg-muted px-1.5 py-0 rounded-full">
                {notifications.length}
              </span>
            </div>
            <div className="space-y-1">
              {notifications.slice(0, 3).map((notif) => (
                <div
                  key={notif.id}
                  className="group flex items-start gap-2 rounded bg-warning/10 px-2 py-1.5 cursor-pointer hover:bg-warning/20 transition-colors"
                  onClick={() => handleNotificationClick(notif)}
                >
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-1">
                      <p className="text-[11px] font-medium truncate">{notif.title}</p>
                      {notif.action_url && (
                        <ExternalLink className="h-3 w-3 text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0" />
                      )}
                    </div>
                    <p className="text-[10px] text-muted-foreground truncate">{notif.body}</p>
                  </div>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-5 w-5 shrink-0 text-muted-foreground hover:text-foreground"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDismissNotification(notif.id);
                    }}
                  >
                    <Check className="h-3 w-3" />
                  </Button>
                </div>
              ))}
            </div>
          </div>
        </>
      )}

      {/* ── 快捷操作（含 P1-2 处理按钮）───────────────────── */}
      <Separator />
      <div className="flex items-center gap-1 px-4 py-2">
        <Button
          variant="outline"
          size="sm"
          className="h-7 flex-1 gap-1 text-[11px]"
          onClick={handleOpenMainWindow}
        >
          <Monitor className="h-3 w-3" />
          主窗口
        </Button>
        <Button
          variant="outline"
          size="sm"
          className="h-7 flex-1 gap-1 text-[11px]"
          onClick={() => showCaptureWindow()}
        >
          <PenLine className="h-3 w-3" />
          速记
        </Button>
        <Button
          variant="outline"
          size="sm"
          className="h-7 flex-1 gap-1 text-[11px]"
          onClick={() => invoke("take_screenshot")}
        >
          <Camera className="h-3 w-3" />
          截图
        </Button>
        <Button
          variant="outline"
          size="sm"
          className={cn(
            "h-7 flex-1 gap-1 text-[11px]",
            processing && "opacity-50 cursor-not-allowed"
          )}
          onClick={handleTriggerProcess}
          disabled={processing}
        >
          <Play className={cn("h-3 w-3", processing && "animate-spin")} />
          {processing ? "处理中" : "处理"}
        </Button>
      </div>

      {/* ── Footer（系统状态）─────────────────────────────── */}
      <Separator />
      <footer className="flex items-center justify-between px-4 py-1.5 text-[10px] text-muted-foreground">
        <span>
          {systemStatus
            ? `采集器 ${systemStatus.collectors_healthy}/${systemStatus.collectors_total}`
            : "运行中"}
        </span>
        <div className="flex items-center gap-2">
          {systemStatus?.scheduler_running && (
            <span className="flex items-center gap-0.5">
              <span className="h-1.5 w-1.5 rounded-full bg-success" />
              调度
            </span>
          )}
          <span>v0.1.0</span>
        </div>
      </footer>
    </div>
  );
}
