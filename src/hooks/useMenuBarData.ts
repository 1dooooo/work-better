/**
 * useMenuBarData — MenuBar 数据获取 Hook
 *
 * 封装 MenuBar 所需的所有数据获取逻辑
 */

import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  getEvents,
  getUnprocessedCount,
  getPendingNotifications,
  markNotificationRead,
  getPendingTasks,
  getSystemStatus,
  type Event,
  type NotificationRecord,
  type PendingTaskDto,
  type SystemStatus,
} from "@/lib/tauri";

// ─── 常量 ─────────────────────────────────────────────────────

/** 最多显示的事件数 */
const MAX_EVENTS = 10;

// ─── Hook ─────────────────────────────────────────────────────

interface UseMenuBarDataReturn {
  /** 未处理事件数 */
  unprocessedCount: number;
  /** 最近事件列表 */
  events: Event[];
  /** 待处理通知 */
  notifications: NotificationRecord[];
  /** 待办任务 */
  pendingTasks: PendingTaskDto[];
  /** 系统状态 */
  systemStatus: SystemStatus | null;
  /** 是否加载中 */
  loading: boolean;
  /** 刷新数据 */
  refresh: () => Promise<void>;
  /** 关闭通知 */
  dismissNotification: (id: string) => Promise<void>;
  /** 点击通知 */
  handleNotificationClick: (notif: NotificationRecord) => Promise<void>;
}

export function useMenuBarData(): UseMenuBarDataReturn {
  const [unprocessedCount, setUnprocessedCount] = useState(0);
  const [events, setEvents] = useState<Event[]>([]);
  const [notifications, setNotifications] = useState<NotificationRecord[]>([]);
  const [pendingTasks, setPendingTasks] = useState<PendingTaskDto[]>([]);
  const [systemStatus, setSystemStatus] = useState<SystemStatus | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const [count, recentEvents, pendingNotifs, tasks, status] =
        await Promise.all([
          getUnprocessedCount(),
          getEvents(MAX_EVENTS),
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
      console.error("[useMenuBarData] refresh failed:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  const dismissNotification = useCallback(async (id: string) => {
    try {
      await markNotificationRead(id);
      setNotifications((prev) => prev.filter((n) => n.id !== id));
    } catch (err) {
      console.error("[useMenuBarData] dismiss notification failed:", err);
    }
  }, []);

  const handleNotificationClick = useCallback(
    async (notif: NotificationRecord) => {
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
        console.error("[useMenuBarData] notification click failed:", err);
      }
    },
    [],
  );

  return {
    unprocessedCount,
    events,
    notifications,
    pendingTasks,
    systemStatus,
    loading,
    refresh,
    dismissNotification,
    handleNotificationClick,
  };
}
