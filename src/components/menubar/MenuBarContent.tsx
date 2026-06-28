/**
 * MenuBarContent — 菜单栏内容区组件
 *
 * 显示最近事件、今日待办和通知中心
 */

import type {
  Event,
  NotificationRecord,
  PendingTaskDto,
} from "@/lib/tauri";
import {
  Clock,
  ListTodo,
  Bell,
  Activity,
  RefreshCw,
} from "lucide-react";
import { EventListItem } from "./EventListItem";
import { TaskListItem } from "./TaskListItem";
import { NotificationGroup } from "./NotificationGroup";

// ─── 常量 ─────────────────────────────────────────────────────

/** 最多显示的待办数 */
const MAX_PENDING_TASKS = 3;

// ─── 子组件：区域标题 ─────────────────────────────────────────

function SectionHeader({
  icon: Icon,
  label,
  count,
  action,
}: {
  icon: typeof Clock;
  label: string;
  count?: number;
  action?: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between px-3 py-1.5">
      <div className="flex items-center gap-1.5">
        <Icon className="h-3 w-3 text-muted-foreground" />
        <span className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider">
          {label}
        </span>
        {count !== undefined && count > 0 && (
          <span className="text-[9px] text-muted-foreground/60 tabular-nums">
            {count}
          </span>
        )}
      </div>
      {action}
    </div>
  );
}

// ─── 子组件：骨架屏 ───────────────────────────────────────────

function SkeletonRow() {
  return (
    <div className="flex items-center gap-2 px-3 py-1.5">
      <div className="h-4 w-8 rounded bg-accent animate-pulse" />
      <div className="flex-1 h-3.5 rounded bg-accent animate-pulse" />
      <div className="h-3 w-6 rounded bg-accent animate-pulse" />
    </div>
  );
}

function LoadingSkeleton() {
  return (
    <div className="space-y-1 py-2">
      {Array.from({ length: 4 }, (_, i) => (
        <SkeletonRow key={i} />
      ))}
    </div>
  );
}

// ─── 类型定义 ─────────────────────────────────────────────────

interface MenuBarContentProps {
  /** 最近事件列表 */
  events: Event[];
  /** 待办任务 */
  pendingTasks: PendingTaskDto[];
  /** 通知列表 */
  notifications: NotificationRecord[];
  /** 是否加载中 */
  loading: boolean;
  /** 刷新回调 */
  onRefresh: () => void;
  /** 点击通知回调 */
  onNotificationClick: (notif: NotificationRecord) => void;
  /** 关闭通知回调 */
  onDismissNotification: (id: string) => void;
}

// ─── 组件 ─────────────────────────────────────────────────────

export function MenuBarContent({
  events,
  pendingTasks,
  notifications,
  loading,
  onRefresh,
  onNotificationClick,
  onDismissNotification,
}: MenuBarContentProps) {
  return (
    <div className="flex-1 overflow-y-auto min-h-0" data-testid="menubar-content">
      {/* ── 最近事件 ── */}
      <SectionHeader
        icon={Clock}
        label="最近事件"
        action={
          <button
            className="flex h-5 w-5 items-center justify-center rounded-md text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
            onClick={onRefresh}
            aria-label="刷新"
          >
            <RefreshCw className="h-3 w-3" />
          </button>
        }
      />

      <div className="px-2">
        {loading ? (
          <LoadingSkeleton />
        ) : events.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
            <Activity className="h-5 w-5 mb-1.5" strokeWidth={1.5} />
            <span className="text-[11px]">暂无事件</span>
            <span className="text-[9px] text-muted-foreground/60 mt-0.5">
              采集器运行后会自动显示
            </span>
          </div>
        ) : (
          <div className="space-y-px">
            {events.map((event) => (
              <EventListItem key={event.id} event={event} />
            ))}
          </div>
        )}
      </div>

      {/* ── 今日待办 ── */}
      {pendingTasks.length > 0 && (
        <div className="border-t border-border">
          <SectionHeader
            icon={ListTodo}
            label="今日待办"
            count={pendingTasks.length}
          />
          <div className="px-2 pb-1 space-y-px">
            {pendingTasks.slice(0, MAX_PENDING_TASKS).map((task) => (
              <TaskListItem key={task.id} task={task} />
            ))}
          </div>
        </div>
      )}

      {/* ── 通知中心 ── */}
      {notifications.length > 0 && (
        <div className="border-t border-border">
          <SectionHeader
            icon={Bell}
            label="通知"
            count={notifications.length}
          />
          <div className="px-2 pb-1">
            <NotificationGroup
              notifications={notifications}
              onNotificationClick={onNotificationClick}
              onDismiss={onDismissNotification}
            />
          </div>
        </div>
      )}
    </div>
  );
}
