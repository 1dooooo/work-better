import { useState, useEffect, useCallback, type FormEvent } from "react";
import {
  listScheduledTasks,
  pauseScheduler,
  resumeScheduler,
  isSchedulerPaused,
  listTasks,
  createTask,
  updateTaskStatus,
  getPendingTasks,
  confirmPendingTask,
  rejectPendingTask,
  type PendingTaskDto,
  type TaskInfo,
} from "@/lib/tauri";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Plus,
  ArrowRight,
  RotateCcw,
  CalendarClock,
  Circle,
  Clock,
  CheckCircle2,
  Loader2,
  Pause,
  Play,
  Check,
  X,
  Sparkles,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { toast } from "sonner";

type TaskStatus = "Open" | "InProgress" | "Done" | "Pending";

interface DisplayTask {
  id: string;
  title: string;
  status: TaskStatus;
  priority: "P0" | "P1" | "P2" | "P3";
  source: string;
  dueDate: string;
  createdAt: string;
}

const STATUS_META: Record<
  TaskStatus,
  { label: string; next: TaskStatus; icon: typeof Circle }
> = {
  Open: { label: "待处理", next: "InProgress", icon: Circle },
  InProgress: { label: "进行中", next: "Done", icon: Clock },
  Done: { label: "已完成", next: "Open", icon: CheckCircle2 },
  Pending: { label: "待确认", next: "Open", icon: Sparkles },
};

const PRIORITY_CONFIG: Record<
  string,
  { label: string; className: string }
> = {
  P0: {
    label: "紧急",
    className:
      "bg-destructive/10 text-destructive border-destructive/20",
  },
  P1: {
    label: "高",
    className: "bg-warning/10 text-warning-foreground border-warning/20",
  },
  P2: { label: "中", className: "bg-muted text-muted-foreground" },
  P3: { label: "低", className: "bg-muted text-muted-foreground" },
};

const PRIORITY_ORDER: Record<string, number> = {
  P0: 0,
  P1: 1,
  P2: 2,
  P3: 3,
};

export default function TasksView() {
  const [tasks, setTasks] = useState<DisplayTask[]>([]);
  const [pendingTasks, setPendingTasks] = useState<PendingTaskDto[]>([]);
  const [title, setTitle] = useState("");
  const [priority, setPriority] = useState<string>("P2");
  const [dueDate, setDueDate] = useState("");
  const [scheduledTasks, setScheduledTasks] = useState<TaskInfo[]>([]);
  const [schedulerPaused, setSchedulerPaused] = useState(false);
  const [loadingScheduled, setLoadingScheduled] = useState(false);
  const [loadingTasks, setLoadingTasks] = useState(true);

  // 加载任务数据
  const refreshTasks = useCallback(async () => {
    setLoadingTasks(true);
    try {
      const [backendTasks, pending] = await Promise.all([
        listTasks(),
        getPendingTasks(),
      ]);
      setTasks(
        backendTasks.map((t) => ({
          id: t.id,
          title: t.title,
          status: (["Open", "InProgress", "Done", "Pending"].includes(t.status)
            ? t.status
            : "Open") as TaskStatus,
          priority: (["P0", "P1", "P2", "P3"].includes(t.priority)
            ? t.priority
            : "P2") as DisplayTask["priority"],
          source: t.source,
          dueDate: t.due_date ?? "未设定",
          createdAt: t.created_at,
        })),
      );
      setPendingTasks(pending);
    } catch (err) {
      console.error("Failed to load tasks:", err);
    } finally {
      setLoadingTasks(false);
    }
  }, []);

  // 加载调度器任务
  useEffect(() => {
    const loadScheduledTasks = async () => {
      setLoadingScheduled(true);
      try {
        const [schedTasks, paused] = await Promise.all([
          listScheduledTasks(),
          isSchedulerPaused(),
        ]);
        setScheduledTasks(schedTasks);
        setSchedulerPaused(paused);
      } catch (err) {
        console.error("Failed to load scheduled tasks:", err);
      } finally {
        setLoadingScheduled(false);
      }
    };
    loadScheduledTasks();
    refreshTasks();
  }, [refreshTasks]);

  // 暂停/恢复调度器
  const handleToggleScheduler = useCallback(async () => {
    try {
      if (schedulerPaused) {
        await resumeScheduler();
        setSchedulerPaused(false);
        toast.success("调度器已恢复");
      } else {
        await pauseScheduler();
        setSchedulerPaused(true);
        toast.success("调度器已暂停");
      }
    } catch (err) {
      console.error("Failed to toggle scheduler:", err);
      toast.error("操作失败");
    }
  }, [schedulerPaused]);

  // 创建任务
  const handleCreate = useCallback(
    async (e: FormEvent) => {
      e.preventDefault();
      const trimmed = title.trim();
      if (!trimmed) return;
      try {
        await createTask(trimmed, priority);
        setTitle("");
        setDueDate("");
        toast.success("任务已创建");
        refreshTasks();
      } catch (err) {
        console.error("Failed to create task:", err);
        toast.error("创建失败");
      }
    },
    [title, priority, refreshTasks],
  );

  // 推进任务状态
  const toggleStatus = useCallback(
    async (id: string, currentStatus: TaskStatus) => {
      const nextStatus = STATUS_META[currentStatus].next;
      try {
        await updateTaskStatus(id, nextStatus);
        toast.success(`状态已更新为${STATUS_META[nextStatus].label}`);
        refreshTasks();
      } catch (err) {
        console.error("Failed to update task status:", err);
        toast.error("状态更新失败");
      }
    },
    [refreshTasks],
  );

  // 确认待确认任务
  const handleConfirm = useCallback(
    async (pendingId: string) => {
      try {
        await confirmPendingTask(pendingId);
        toast.success("任务已确认");
        refreshTasks();
      } catch (err) {
        console.error("Failed to confirm task:", err);
        toast.error("确认失败");
      }
    },
    [refreshTasks],
  );

  // 拒绝待确认任务
  const handleReject = useCallback(
    async (pendingId: string) => {
      try {
        await rejectPendingTask(pendingId);
        toast.success("已拒绝");
        refreshTasks();
      } catch (err) {
        console.error("Failed to reject task:", err);
        toast.error("拒绝失败");
      }
    },
    [refreshTasks],
  );

  const grouped = (["Open", "InProgress", "Done"] as const).map((s) => ({
    status: s,
    label: STATUS_META[s].label,
    icon: STATUS_META[s].icon,
    items: tasks
      .filter((t) => t.status === s)
      .sort(
        (a, b) =>
          (PRIORITY_ORDER[a.priority] ?? 9) - (PRIORITY_ORDER[b.priority] ?? 9),
      ),
  }));

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-border px-6 py-4">
        <div className="flex items-center gap-3">
          <h1 className="text-lg font-semibold">任务管理</h1>
          <Badge variant="secondary" className="text-xs">
            {tasks.length} 个任务
          </Badge>
          {pendingTasks.length > 0 && (
            <Badge variant="outline" className="gap-1 text-xs text-info">
              <Sparkles className="h-3 w-3" />
              {pendingTasks.length} 个待确认
            </Badge>
          )}
        </div>
      </header>

      {/* Pending Tasks (from AI discovery) */}
      {pendingTasks.length > 0 && (
        <div className="border-b border-border bg-info/5 px-6 py-3">
          <div className="mb-2 flex items-center gap-2">
            <Sparkles className="h-4 w-4 text-info" />
            <h3 className="text-sm font-medium">AI 发现的待确认任务</h3>
          </div>
          <div className="flex flex-col gap-1.5">
            {pendingTasks.map((pt) => (
              <Card key={pt.id} className="border-info/20">
                <CardContent className="flex items-center justify-between px-3 py-2">
                  <div className="flex-1">
                    <div className="text-sm font-medium">{pt.title}</div>
                    <div className="mt-0.5 flex items-center gap-2">
                      <Badge variant="outline" className="text-[10px]">
                        {pt.source}
                      </Badge>
                      <span className="text-[10px] text-muted-foreground">
                        来源: {pt.origin_text.slice(0, 60)}
                        {pt.origin_text.length > 60 ? "..." : ""}
                      </span>
                    </div>
                  </div>
                  <div className="flex items-center gap-1.5">
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-7 gap-1 px-2 text-xs text-success"
                      onClick={() => handleConfirm(pt.id)}
                    >
                      <Check className="h-3 w-3" />
                      确认
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-7 gap-1 px-2 text-xs text-muted-foreground"
                      onClick={() => handleReject(pt.id)}
                    >
                      <X className="h-3 w-3" />
                      拒绝
                    </Button>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      )}

      {/* Create Form */}
      <form
        onSubmit={handleCreate}
        className="flex items-center gap-2 border-b border-border px-6 py-3"
      >
        <Input
          placeholder="新任务标题..."
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          className="h-8 flex-1 text-sm"
        />
        <Select
          value={priority}
          onValueChange={(v) => v !== null && setPriority(v)}
        >
          <SelectTrigger className="h-8 w-[100px] text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="P0">紧急</SelectItem>
            <SelectItem value="P1">高优先级</SelectItem>
            <SelectItem value="P2">中优先级</SelectItem>
            <SelectItem value="P3">低优先级</SelectItem>
          </SelectContent>
        </Select>
        <Input
          type="date"
          value={dueDate}
          onChange={(e) => setDueDate(e.target.value)}
          className="h-8 w-[130px] text-xs"
        />
        <Button type="submit" size="sm" className="h-8 gap-1.5">
          <Plus className="h-3.5 w-3.5" />
          添加
        </Button>
      </form>

      {/* Kanban Board */}
      <div className="flex flex-1 gap-4 overflow-auto p-6">
        {loadingTasks ? (
          <div className="flex flex-1 items-center justify-center text-muted-foreground">
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            加载中...
          </div>
        ) : (
          grouped.map(({ status, label, icon: StatusIcon, items }) => (
            <div key={status} className="flex flex-1 flex-col gap-2">
              <div className="flex items-center gap-2 px-1">
                <StatusIcon
                  className={cn(
                    "h-4 w-4",
                    status === "Open" && "text-muted-foreground",
                    status === "InProgress" && "text-info",
                    status === "Done" && "text-success",
                  )}
                />
                <h3 className="text-sm font-medium">{label}</h3>
                <Badge variant="secondary" className="h-5 text-[10px]">
                  {items.length}
                </Badge>
              </div>
              <div className="flex flex-col gap-1.5">
                {items.map((task) => (
                  <Card
                    key={task.id}
                    className="border-border transition-shadow hover:shadow-sm"
                  >
                    <CardContent className="px-3 py-2.5">
                      <div className="text-sm font-medium">{task.title}</div>
                      <div className="mt-1.5 flex items-center gap-2">
                        <Badge
                          variant="outline"
                          className={cn(
                            "text-[10px]",
                            PRIORITY_CONFIG[task.priority]?.className ??
                              "bg-muted text-muted-foreground",
                          )}
                        >
                          {PRIORITY_CONFIG[task.priority]?.label ?? task.priority}
                        </Badge>
                        <span className="text-[11px] text-muted-foreground">
                          {task.dueDate}
                        </span>
                        {task.source !== "Manual" && (
                          <Badge variant="outline" className="text-[10px]">
                            {task.source}
                          </Badge>
                        )}
                      </div>
                      <Button
                        variant="ghost"
                        size="sm"
                        className="mt-1.5 h-6 gap-1 px-1 text-[11px] text-muted-foreground"
                        onClick={() => toggleStatus(task.id, task.status)}
                      >
                        {status === "Done" ? (
                          <>
                            <RotateCcw className="h-3 w-3" />
                            重新打开
                          </>
                        ) : (
                          <>
                            <ArrowRight className="h-3 w-3" />
                            推进
                          </>
                        )}
                      </Button>
                    </CardContent>
                  </Card>
                ))}
                {items.length === 0 && (
                  <div className="flex h-20 items-center justify-center rounded-lg border border-dashed text-xs text-muted-foreground">
                    暂无任务
                  </div>
                )}
              </div>
            </div>
          ))
        )}
      </div>

      {/* Scheduled Tasks */}
      <div className="border-t border-border px-6 py-4">
        <div className="mb-3 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <CalendarClock className="h-4 w-4 text-muted-foreground" />
            <h3 className="text-sm font-medium">定时任务</h3>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={handleToggleScheduler}
            className="h-7 gap-1.5"
          >
            {schedulerPaused ? (
              <>
                <Play className="h-3.5 w-3.5" />
                恢复
              </>
            ) : (
              <>
                <Pause className="h-3.5 w-3.5" />
                暂停
              </>
            )}
          </Button>
        </div>
        {loadingScheduled ? (
          <div className="flex items-center justify-center py-4 text-muted-foreground">
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            加载中...
          </div>
        ) : scheduledTasks.length > 0 ? (
          <div className="flex flex-col gap-1">
            {scheduledTasks.map((task) => (
              <div
                key={task.id}
                className="flex items-center justify-between rounded-md px-3 py-2 text-sm hover:bg-muted/50"
              >
                <div className="flex items-center gap-2">
                  <span>{task.name}</span>
                  <Badge variant="outline" className="text-[10px]">
                    {task.layer}
                  </Badge>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-xs text-muted-foreground">
                    {task.cron}
                  </span>
                  <span className="text-xs text-muted-foreground">
                    SLA: {task.sla_ms}ms
                  </span>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="flex items-center justify-center py-4 text-xs text-muted-foreground">
            暂无定时任务
          </div>
        )}
      </div>
    </div>
  );
}
