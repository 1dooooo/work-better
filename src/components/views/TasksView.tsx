import { useState, useEffect, useCallback, type FormEvent } from "react";
import { listen } from "@tauri-apps/api/event";
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

  // 监听任务更新事件（来自 trigger_manual_capture）
  useEffect(() => {
    const unlisten = listen("tasks:updated", () => {
      refreshTasks();
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [refreshTasks]);

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
    <div className="flex h-full flex-col" data-testid="tasks-container">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-border px-5 py-3 min-h-[48px]">
        <div className="flex items-center gap-3">
          <h1 className="text-sm font-semibold">任务管理</h1>
          <span className="text-[11px] text-muted-foreground bg-muted px-2 py-0.5 rounded-full">
            {tasks.length} 个任务
          </span>
          {pendingTasks.length > 0 && (
            <span className="text-[11px] text-info bg-info/10 px-2 py-0.5 rounded-full">
              <Sparkles className="inline h-3 w-3 mr-0.5" />
              {pendingTasks.length} 待确认
            </span>
          )}
        </div>
      </header>

      {/* Pending Tasks (from AI discovery) */}
      {pendingTasks.length > 0 && (
        <div className="border-b border-border bg-info/5 px-5 py-2">
          <div className="mb-1.5 flex items-center gap-2">
            <Sparkles className="h-3.5 w-3.5 text-info" />
            <h3 className="text-xs font-medium">AI 发现的待确认任务</h3>
          </div>
          <div className="flex flex-col gap-1">
            {pendingTasks.map((pt) => (
              <div key={pt.id} className="group flex items-center justify-between rounded-md px-2.5 py-1.5 border border-info/20 hover:bg-info/5 transition-colors" data-testid={`pending-task-${pt.id}`}>
                <div className="flex-1 min-w-0">
                  <div className="text-xs font-medium truncate">{pt.title}</div>
                  <div className="mt-0.5 flex items-center gap-2">
                    <span className="text-[10px] bg-muted px-1.5 py-0.5 rounded">{pt.source}</span>
                    <span className="text-[10px] text-muted-foreground truncate">
                      来源: {pt.origin_text.slice(0, 60)}
                      {pt.origin_text.length > 60 ? "..." : ""}
                    </span>
                  </div>
                </div>
                <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-6 gap-1 px-1.5 text-[11px] text-success"
                    onClick={() => handleConfirm(pt.id)}
                  >
                    <Check className="h-3 w-3" />
                    确认
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-6 gap-1 px-1.5 text-[11px] text-muted-foreground"
                    onClick={() => handleReject(pt.id)}
                  >
                    <X className="h-3 w-3" />
                    拒绝
                  </Button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Create Form */}
      <form
        onSubmit={handleCreate}
        className="flex items-center gap-2 border-b border-border px-5 py-2"
        data-testid="task-create-form"
      >
        <Input
          placeholder="新任务标题..."
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          className="h-8 flex-1 text-sm"
          data-testid="task-title-input"
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
        <Button type="submit" size="sm" className="h-8 gap-1.5" data-testid="task-create-button">
          <Plus className="h-3.5 w-3.5" />
          添加
        </Button>
      </form>

      {/* Kanban Board */}
      <div className="flex flex-1 gap-3 overflow-auto p-5" data-testid="tasks-kanban">
        {loadingTasks ? (
          <div className="flex flex-1 items-center justify-center text-muted-foreground">
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            加载中...
          </div>
        ) : (
          grouped.map(({ status, label, icon: StatusIcon, items }) => (
            <div key={status} className="flex flex-1 flex-col gap-1.5">
              <div className="flex items-center gap-2 px-1">
                <StatusIcon
                  className={cn(
                    "h-3.5 w-3.5",
                    status === "Open" && "text-muted-foreground",
                    status === "InProgress" && "text-info",
                    status === "Done" && "text-success",
                  )}
                />
                <h3 className="text-xs font-medium">{label}</h3>
                <span className="text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded-full">
                  {items.length}
                </span>
              </div>
              <div className="flex flex-col gap-1">
                {items.map((task) => (
                  <div
                    key={task.id}
                    className="group rounded-md border border-border px-3 py-2 transition-colors hover:bg-muted/50"
                    data-testid={`task-item-${task.id}`}
                  >
                    <div className="text-xs font-medium">{task.title}</div>
                    <div className="mt-1 flex items-center gap-2">
                      <span
                        className={cn(
                          "text-[10px] px-1.5 py-0.5 rounded",
                          PRIORITY_CONFIG[task.priority]?.className ??
                            "bg-muted text-muted-foreground",
                        )}
                      >
                        {PRIORITY_CONFIG[task.priority]?.label ?? task.priority}
                      </span>
                      <span className="text-[10px] text-muted-foreground">
                        {task.dueDate}
                      </span>
                      {task.source !== "Manual" && (
                        <span className="text-[10px] bg-muted px-1.5 py-0.5 rounded">
                          {task.source}
                        </span>
                      )}
                    </div>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="mt-1 h-5 gap-1 px-1 text-[10px] text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity"
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
                  </div>
                ))}
                {items.length === 0 && (
                  <div className="flex h-16 items-center justify-center rounded-lg border border-dashed text-[11px] text-muted-foreground">
                    暂无任务
                  </div>
                )}
              </div>
            </div>
          ))
        )}
      </div>

      {/* Scheduled Tasks */}
      <div className="border-t border-border px-5 py-3">
        <div className="mb-2 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <CalendarClock className="h-3.5 w-3.5 text-muted-foreground" />
            <h3 className="text-xs font-medium">定时任务</h3>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={handleToggleScheduler}
            className="h-6 gap-1 text-[11px]"
            data-testid="scheduler-toggle"
          >
            {schedulerPaused ? (
              <>
                <Play className="h-3 w-3" />
                恢复
              </>
            ) : (
              <>
                <Pause className="h-3 w-3" />
                暂停
              </>
            )}
          </Button>
        </div>
        {loadingScheduled ? (
          <div className="flex items-center justify-center py-3 text-muted-foreground">
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            加载中...
          </div>
        ) : scheduledTasks.length > 0 ? (
          <div className="flex flex-col">
            {scheduledTasks.map((task) => (
              <div
                key={task.id}
                className="flex items-center justify-between px-2 py-1.5 text-xs hover:bg-muted/50 transition-colors rounded"
                data-testid={`scheduled-task-${task.id}`}
              >
                <div className="flex items-center gap-2">
                  <span className="text-foreground">{task.name}</span>
                  <span className="text-[10px] bg-muted px-1.5 py-0.5 rounded">
                    {task.layer}
                  </span>
                </div>
                <div className="flex items-center gap-3">
                  <span className="text-[11px] text-muted-foreground">
                    {task.cron}
                  </span>
                  <span className="text-[10px] text-muted-foreground">
                    SLA: {task.sla_ms}ms
                  </span>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="flex items-center justify-center py-3 text-[11px] text-muted-foreground">
            暂无定时任务
          </div>
        )}
      </div>
    </div>
  );
}
