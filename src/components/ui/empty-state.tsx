/**
 * EmptyState 组件 — 统一的空状态设计
 *
 * 功能：
 * - 图标 + 标题 + 描述 + 可选操作
 * - 统一的设计风格
 * - 支持自定义图标和操作
 * - 渐进式引导支持
 */

import type { ReactNode } from "react";
import { cn } from "@/lib/utils";
import type { LucideIcon } from "lucide-react";
import {
  Activity,
  CheckCircle2,
  Clock,
  ListTodo,
  Radio,
  FileText,
  Download,
  Plus,
} from "lucide-react";
import { Button } from "@/components/ui/button";

interface EmptyStateProps {
  /** 图标 */
  icon: LucideIcon;
  /** 标题 */
  title: string;
  /** 描述 */
  description?: string;
  /** 可选操作 */
  action?: ReactNode;
  /** 引导提示 */
  hint?: string;
  /** 自定义类名 */
  className?: string;
}

export function EmptyState({
  icon: Icon,
  title,
  description,
  action,
  hint,
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
      {hint && (
        <p className="text-xs text-muted-foreground mt-2">{hint}</p>
      )}
    </div>
  );
}

// ─── 预设空状态（渐进式引导）──────────────────────────────────────

interface EmptyEventsProps {
  onCollect?: () => void;
}

export function EmptyEvents({ onCollect }: EmptyEventsProps) {
  return (
    <EmptyState
      icon={Activity}
      title="暂无事件"
      description="采集数据后，事件会自动显示在这里"
      action={
        onCollect && (
          <Button onClick={onCollect} className="gap-2">
            <Download className="h-4 w-4" />
            立即采集
          </Button>
        )
      }
      hint="配置飞书数据源开始采集"
    />
  );
}

interface EmptyTasksProps {
  onCreateTask?: () => void;
}

export function EmptyTasks({ onCreateTask }: EmptyTasksProps) {
  return (
    <EmptyState
      icon={ListTodo}
      title="暂无任务"
      description="创建第一个任务开始工作"
      action={
        onCreateTask && (
          <Button onClick={onCreateTask} className="gap-2">
            <Plus className="h-4 w-4" />
            新建任务
          </Button>
        )
      }
      hint="任务可以从事件中自动发现，也可以手动创建"
    />
  );
}

export function EmptyTimeline() {
  return (
    <EmptyState
      icon={Clock}
      title="暂无时间线"
      description="当有事件发生时会自动记录"
      hint="时间线按小时分组展示事件流"
    />
  );
}

export function EmptyReports() {
  return (
    <EmptyState
      icon={FileText}
      title="暂无报告"
      description="生成一份报告查看工作概览"
      hint="支持日报、周报、月报自动生成"
    />
  );
}

export function EmptyCollectors() {
  return (
    <EmptyState
      icon={Radio}
      title="暂无采集器"
      description="配置一个采集器开始收集数据"
      hint="在设置中配置飞书数据源"
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
