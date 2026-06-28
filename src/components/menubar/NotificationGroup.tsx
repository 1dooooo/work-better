/**
 * NotificationGroup — 通知分组组件
 *
 * 按类型分组显示通知，支持点击和关闭操作
 */

import { useMemo } from "react";
import type { NotificationRecord, NotifyKind } from "@/lib/tauri";
import { cn } from "@/lib/utils";
import {
  AlertTriangle,
  Info,
  CheckCircle2,
  ExternalLink,
  Check,
} from "lucide-react";

// ─── 常量 ─────────────────────────────────────────────────────

/** 每组通知最多显示条数 */
const MAX_NOTIFICATIONS_PER_GROUP = 3;

// ─── 通知分组配置 ─────────────────────────────────────────────

interface NotifyGroupConfig {
  label: string;
  icon: typeof AlertTriangle;
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

// ─── 类型定义 ─────────────────────────────────────────────────

interface NotificationGroupProps {
  /** 通知列表 */
  notifications: NotificationRecord[];
  /** 点击通知回调 */
  onNotificationClick: (notif: NotificationRecord) => void;
  /** 关闭通知回调 */
  onDismiss: (id: string) => void;
}

// ─── Hook ─────────────────────────────────────────────────────

export function useGroupedNotifications(notifications: NotificationRecord[]) {
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

// ─── 组件 ─────────────────────────────────────────────────────

export function NotificationGroup({
  notifications,
  onNotificationClick,
  onDismiss,
}: NotificationGroupProps) {
  const groupedNotifications = useGroupedNotifications(notifications);

  const activeGroups = (
    ["Confirm", "Reminder", "TaskDone"] as NotifyKind[]
  ).filter((kind) => groupedNotifications[kind].length > 0);

  if (activeGroups.length === 0) return null;

  return (
    <div className="space-y-0.5">
      {activeGroups.map((kind) => {
        const group = NOTIFY_GROUP_CONFIG[kind];
        const items = groupedNotifications[kind];
        const GroupIcon = group.icon;

        return (
          <div key={kind}>
            {/* 分组标题 */}
            <div className="flex items-center gap-1 px-1 mb-0.5">
              <GroupIcon
                className={cn("h-3 w-3", group.color)}
                strokeWidth={1.8}
              />
              <span className={cn("text-[9px] font-medium", group.color)}>
                {group.label}
              </span>
              <span className="text-[9px] text-muted-foreground/50 tabular-nums">
                {items.length}
              </span>
            </div>

            {/* 通知列表 */}
            <div className="space-y-px">
              {items.slice(0, MAX_NOTIFICATIONS_PER_GROUP).map((notif) => (
                <div
                  key={notif.id}
                  className={cn(
                    "group flex items-center gap-2 rounded-lg border-l-2 px-2 py-[4px] cursor-pointer transition-colors",
                    group.bg,
                    group.border,
                    "hover:bg-accent",
                  )}
                  onClick={() => onNotificationClick(notif)}
                >
                  <div className="min-w-0 flex-1">
                    <p className="text-[11px] text-foreground truncate leading-tight">
                      {notif.title}
                    </p>
                    {notif.body && (
                      <p className="text-[9px] text-muted-foreground truncate mt-0.5">
                        {notif.body}
                      </p>
                    )}
                  </div>
                  <div className="flex items-center gap-0.5 flex-shrink-0">
                    {notif.action_url && (
                      <ExternalLink className="h-3 w-3 text-muted-foreground/40 opacity-0 group-hover:opacity-100 transition-opacity" />
                    )}
                    <button
                      className="flex h-5 w-5 items-center justify-center rounded-md text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
                      onClick={(e) => {
                        e.stopPropagation();
                        onDismiss(notif.id);
                      }}
                      aria-label="标记已读"
                    >
                      <Check className="h-3 w-3" />
                    </button>
                  </div>
                </div>
              ))}
              {items.length > MAX_NOTIFICATIONS_PER_GROUP && (
                <p className="text-[9px] text-muted-foreground/40 pl-2 py-0.5">
                  +{items.length - MAX_NOTIFICATIONS_PER_GROUP} 更多
                </p>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}
