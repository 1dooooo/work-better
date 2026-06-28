/**
 * TaskListItem — 任务列表项组件
 *
 * 显示待办任务标题和操作提示
 */

import type { PendingTaskDto } from "@/lib/tauri";
import { ListTodo, ChevronRight } from "lucide-react";

// ─── 类型定义 ─────────────────────────────────────────────────

interface TaskListItemProps {
  /** 任务数据 */
  task: PendingTaskDto;
}

// ─── 组件 ─────────────────────────────────────────────────────

export function TaskListItem({ task }: TaskListItemProps) {
  return (
    <div className="group flex items-center gap-2 rounded-lg px-2 py-[5px] hover:bg-[var(--color-glass-hover)] transition-colors cursor-pointer">
      <div className="flex h-3.5 w-3.5 items-center justify-center rounded border border-[var(--color-glass-border)]">
        <ListTodo className="h-2 w-2 text-[var(--color-text-hint)]" />
      </div>
      <span className="flex-1 text-[11px] text-foreground/70 truncate min-w-0">
        {task.title}
      </span>
      <ChevronRight className="h-3 w-3 text-[var(--color-glass-muted)] opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0" />
    </div>
  );
}
