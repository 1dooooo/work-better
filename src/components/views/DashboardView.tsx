/**
 * DashboardView — Bento Grid 仪表盘
 *
 * 设计参考：
 * - Linear：深色背景 + 色差分层
 * - Raycast：高密度信息展示
 * - Bento Grid：不对称网格布局
 */

import { useState, useEffect, useCallback } from "react";
import {
  Activity,
  CheckCircle2,
  Clock,
  ListTodo,
  Radio,
  TrendingUp,
  Zap,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import { MotionCard } from "@/components/ui/motion";
import {
  getEvents,
  getUnprocessedCount,
  getSystemStatus,
  getPendingTasks,
  type Event,
  type SystemStatus,
  type PendingTaskDto,
} from "@/lib/tauri";

// ─── 统计卡片 ──────────────────────────────────────────────

function StatCard({
  title,
  value,
  subtitle,
  icon: Icon,
  trend,
  className,
}: {
  title: string;
  value: string | number;
  subtitle?: string;
  icon: typeof Activity;
  trend?: "up" | "down" | "neutral";
  className?: string;
}) {
  return (
    <MotionCard className={className}>
      <Card className="relative overflow-hidden">
        <CardHeader className="pb-2">
          <div className="flex items-center justify-between">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              {title}
            </CardTitle>
            <Icon className="h-4 w-4 text-muted-foreground" />
          </div>
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{value}</div>
          {subtitle && (
            <p className="text-xs text-muted-foreground mt-1">{subtitle}</p>
          )}
          {trend && (
            <div
              className={cn(
                "absolute bottom-2 right-2 flex items-center gap-1 text-xs",
                trend === "up" && "text-success",
                trend === "down" && "text-destructive",
                trend === "neutral" && "text-muted-foreground"
              )}
            >
              <TrendingUp
                className={cn(
                  "h-3 w-3",
                  trend === "down" && "rotate-180"
                )}
              />
            </div>
          )}
        </CardContent>
      </Card>
    </MotionCard>
  );
}

// ─── 采集器状态卡片 ──────────────────────────────────────────

function CollectorStatusCard({
  status,
}: {
  status: SystemStatus | null;
}) {
  if (!status) {
    return (
      <MotionCard>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              采集器状态
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-sm text-muted-foreground">加载中...</div>
          </CardContent>
        </Card>
      </MotionCard>
    );
  }

  const healthPercentage = status.collectors_total > 0
    ? Math.round((status.collectors_healthy / status.collectors_total) * 100)
    : 0;

  return (
    <MotionCard>
    <Card>
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-medium text-muted-foreground">
            采集器状态
          </CardTitle>
          <Radio className="h-4 w-4 text-muted-foreground" />
        </div>
      </CardHeader>
      <CardContent>
        <div className="flex items-center gap-3">
          <div className="relative h-12 w-12">
            <svg className="h-12 w-12 -rotate-90" viewBox="0 0 36 36">
              <circle
                cx="18"
                cy="18"
                r="16"
                fill="none"
                className="stroke-muted"
                strokeWidth="3"
              />
              <circle
                cx="18"
                cy="18"
                r="16"
                fill="none"
                className={cn(
                  "stroke-current",
                  healthPercentage >= 80
                    ? "text-success"
                    : healthPercentage >= 50
                    ? "text-warning"
                    : "text-destructive"
                )}
                strokeWidth="3"
                strokeDasharray={`${healthPercentage * 1.005} 100.5`}
                strokeLinecap="round"
              />
            </svg>
            <div className="absolute inset-0 flex items-center justify-center text-xs font-medium">
              {healthPercentage}%
            </div>
          </div>
          <div className="space-y-1">
            <div className="text-sm">
              <span className="font-medium">{status.collectors_healthy}</span>
              <span className="text-muted-foreground">/{status.collectors_total} 活跃</span>
            </div>
            <div className="flex items-center gap-1.5">
              <span
                className={cn(
                  "h-1.5 w-1.5 rounded-full",
                  status.scheduler_running ? "bg-success" : "bg-muted"
                )}
              />
              <span className="text-xs text-muted-foreground">
                调度{status.scheduler_running ? "中" : "已暂停"}
              </span>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
    </MotionCard>
  );
}

// ─── 最近事件卡片 ──────────────────────────────────────────

function RecentEventsCard({ events }: { events: Event[] }) {
  const typeColors: Record<string, string> = {
    message: "bg-info/10 text-info",
    issue: "bg-warning/10 text-warning",
    pr: "bg-success/10 text-success",
    document: "bg-muted text-muted-foreground",
  };

  return (
    <MotionCard className="col-span-2">
    <Card className="col-span-2">
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-medium text-muted-foreground">
            最近事件
          </CardTitle>
          <Clock className="h-4 w-4 text-muted-foreground" />
        </div>
      </CardHeader>
      <CardContent>
        {events.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-6 text-muted-foreground">
            <Activity className="h-8 w-8 mb-2" />
            <span className="text-sm">暂无事件</span>
          </div>
        ) : (
          <div className="space-y-2">
            {events.slice(0, 5).map((event) => (
              <div
                key={event.id}
                className="flex items-center gap-3 rounded-lg p-2 hover:bg-muted/50 transition-colors"
              >
                <Badge
                  variant="secondary"
                  className={cn(
                    "text-[10px] font-medium",
                    typeColors[event.type] || "bg-muted text-muted-foreground"
                  )}
                >
                  {event.type.slice(0, 3).toUpperCase()}
                </Badge>
                <span className="flex-1 text-sm truncate">
                  {getEventSummary(event.content)}
                </span>
                <span className="text-xs text-muted-foreground">
                  {formatRelativeTime(event.timestamp)}
                </span>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
    </MotionCard>
  );
}

// ─── 待办任务卡片 ──────────────────────────────────────────

function PendingTasksCard({ tasks }: { tasks: PendingTaskDto[] }) {
  return (
    <MotionCard>
    <Card>
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-medium text-muted-foreground">
            待办任务
          </CardTitle>
          <ListTodo className="h-4 w-4 text-muted-foreground" />
        </div>
      </CardHeader>
      <CardContent>
        {tasks.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-4 text-muted-foreground">
            <CheckCircle2 className="h-6 w-6 mb-1" />
            <span className="text-sm">全部完成</span>
          </div>
        ) : (
          <div className="space-y-2">
            {tasks.slice(0, 3).map((task) => (
              <div
                key={task.id}
                className="flex items-center gap-2 rounded-lg p-2 hover:bg-muted/50 transition-colors"
              >
                <div className="h-2 w-2 rounded-full bg-primary" />
                <span className="flex-1 text-sm truncate">{task.title}</span>
              </div>
            ))}
            {tasks.length > 3 && (
              <p className="text-xs text-muted-foreground text-center">
                +{tasks.length - 3} 更多
              </p>
            )}
          </div>
        )}
      </CardContent>
    </Card>
    </MotionCard>
  );
}

// ─── 辅助函数 ──────────────────────────────────────────────

function getEventSummary(content: unknown): string {
  if (typeof content !== "string") {
    return String(content).slice(0, 50);
  }
  try {
    const parsed = JSON.parse(content);
    return parsed.title || parsed.summary || content.slice(0, 50);
  } catch {
    return content.slice(0, 50);
  }
}

function formatRelativeTime(timestamp: string): string {
  const now = Date.now();
  const then = new Date(timestamp).getTime();
  const diff = now - then;

  if (diff < 60_000) return "刚刚";
  if (diff < 3600_000) return `${Math.floor(diff / 60_000)}分钟前`;
  if (diff < 86400_000) return `${Math.floor(diff / 3600_000)}小时前`;
  return `${Math.floor(diff / 86400_000)}天前`;
}

// ─── 主组件 ──────────────────────────────────────────────

export default function DashboardView() {
  const [events, setEvents] = useState<Event[]>([]);
  const [unprocessedCount, setUnprocessedCount] = useState(0);
  const [systemStatus, setSystemStatus] = useState<SystemStatus | null>(null);
  const [pendingTasks, setPendingTasks] = useState<PendingTaskDto[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const [eventsData, count, status, tasks] = await Promise.all([
        getEvents(),
        getUnprocessedCount(),
        getSystemStatus(),
        getPendingTasks(),
      ]);
      setEvents(eventsData);
      setUnprocessedCount(count);
      setSystemStatus(status);
      setPendingTasks(tasks);
    } catch (err) {
      console.error("Failed to refresh dashboard:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
    const interval = setInterval(refresh, 30_000);
    return () => clearInterval(interval);
  }, [refresh]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-muted-foreground">加载中...</div>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold">仪表盘</h1>
        <p className="text-muted-foreground">今日工作概览</p>
      </div>

      {/* Bento Grid */}
      <div className="grid grid-cols-4 gap-4">
        {/* Row 1: Stats */}
        <StatCard
          title="今日处理"
          value={systemStatus?.today_processed_count ?? 0}
          subtitle="事件已处理"
          icon={Zap}
          trend="up"
        />
        <StatCard
          title="待处理"
          value={unprocessedCount}
          subtitle="事件待处理"
          icon={Activity}
          trend={unprocessedCount > 0 ? "up" : "neutral"}
        />
        <CollectorStatusCard status={systemStatus} />
        <PendingTasksCard tasks={pendingTasks} />

        {/* Row 2: Recent Events (wide) */}
        <RecentEventsCard events={events} />

        {/* Row 2: Quick Actions */}
        <MotionCard className="col-span-2">
        <Card className="col-span-2">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              快速操作
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 gap-2">
              <button className="flex items-center gap-2 rounded-lg border p-3 text-sm hover:bg-muted/50 transition-colors">
                <ListTodo className="h-4 w-4" />
                新建任务
              </button>
              <button className="flex items-center gap-2 rounded-lg border p-3 text-sm hover:bg-muted/50 transition-colors">
                <Activity className="h-4 w-4" />
                查看事件
              </button>
              <button className="flex items-center gap-2 rounded-lg border p-3 text-sm hover:bg-muted/50 transition-colors">
                <Clock className="h-4 w-4" />
                时间线
              </button>
              <button className="flex items-center gap-2 rounded-lg border p-3 text-sm hover:bg-muted/50 transition-colors">
                <CheckCircle2 className="h-4 w-4" />
                处理待办
              </button>
            </div>
          </CardContent>
        </Card>
        </MotionCard>
      </div>
    </div>
  );
}
