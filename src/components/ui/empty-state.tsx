/**
 * EmptyState 组件 — 统一的空状态设计
 *
 * 功能：
 * - 图标 + 标题 + 描述 + 可选操作
 * - 统一的设计风格
 * - 支持自定义图标和操作
 */

import type { ReactNode } from "react";
import { cn } from "@/lib/utils";
import type { LucideIcon } from "lucide-react";

interface EmptyStateProps {
  /** 图标 */
  icon: LucideIcon;
  /** 标题 */
  title: string;
  /** 描述 */
  description?: string;
  /** 可选操作 */
  action?: ReactNode;
  /** 自定义类名 */
  className?: string;
}

export function EmptyState({
  icon: Icon,
  title,
  description,
  action,
  className,
}: EmptyStateProps) {
  return (
    <div
      className={cn(
        "flex flex-col items-center justify-center py-12 px-4 text-center",
        className
      )}
    >
      <Icon className="h-12 w-12 text-muted-foreground mb-4" />
      <h3 className="text-lg font-medium text-foreground mb-1">{title}</h3>
      {description && (
        <p className="text-sm text-muted-foreground max-w-sm mb-4">
          {description}
        </p>
      )}
      {action}
    </div>
  );
}

// ─── 预设空状态 ──────────────────────────────────────────────

import {
  Activity,
  CheckCircle2,
  Clock,
  ListTodo,
  Radio,
  FileText,
} from "lucide-react";

export function EmptyEvents() {
  return (
    <EmptyState
      icon={Activity}
      title="暂无事件"
      description="当有新事件时会自动显示在这里"
    />
  );
}

export function EmptyTasks() {
  return (
    <EmptyState
      icon={ListTodo}
      title="暂无任务"
      description="创建一个新任务开始工作"
    />
  );
}

export function EmptyTimeline() {
  return (
    <EmptyState
      icon={Clock}
      title="暂无时间线"
      description="当有事件发生时会自动记录"
    />
  );
}

export function EmptyReports() {
  return (
    <EmptyState
      icon={FileText}
      title="暂无报告"
      description="生成一份报告查看工作概览"
    />
  );
}

export function EmptyCollectors() {
  return (
    <EmptyState
      icon={Radio}
      title="暂无采集器"
      description="配置一个采集器开始收集数据"
    />
  );
}

export function AllDone() {
  return (
    <EmptyState
      icon={CheckCircle2}
      title="全部完成"
      description="所有任务都已处理"
    />
  );
}
