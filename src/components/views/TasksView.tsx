import { useState, useEffect, useCallback, type FormEvent } from "react";
import {
  listScheduledTasks,
  pauseScheduler,
  resumeScheduler,
  isSchedulerPaused,
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
  AlertCircle,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { toast } from "sonner";

interface Task {
  id: string;
  title: string;
  status: "todo" | "in_progress" | "done";
  priority: "high" | "medium" | "low";
  dueDate: string;
  createdAt: string;
}

const INITIAL_TASKS: Task[] = [
  { id: "1", title: "整理周报", status: "todo", priority: "high", dueDate: "2026-06-07", createdAt: "2026-06-06" },
  { id: "2", title: "Review PR #42", status: "todo", priority: "medium", dueDate: "2026-06-08", createdAt: "2026-06-06" },
  { id: "3", title: "更新项目文档", status: "in_progress", priority: "low", dueDate: "2026-06-10", createdAt: "2026-06-05" },
  { id: "4", title: "修复登录 Bug", status: "in_progress", priority: "high", dueDate: "2026-06-06", createdAt: "2026-06-04" },
  { id: "5", title: "设计新功能原型", status: "done", priority: "medium", dueDate: "2026-06-05", createdAt: "2026-06-01" },
];

const SCHEDULED_TASKS = [
  { id: "s1", title: "每日站会", schedule: "每天 10:00" },
  { id: "s2", title: "周报汇总", schedule: "每周五 17:00" },
  { id: "s3", title: "依赖安全检查", schedule: "每周一 09:00" },
];

const STATUS_META: Record<Task["status"], { label: string; next: Task["status"]; icon: typeof Circle }> = {
  todo: { label: "待处理", next: "in_progress", icon: Circle },
  in_progress: { label: "进行中", next: "done", icon: Clock },
  done: { label: "已完成", next: "todo", icon: CheckCircle2 },
};

const PRIORITY_CONFIG: Record<Task["priority"], { label: string; className: string }> = {
  high: { label: "高", className: "bg-destructive/10 text-destructive border-destructive/20" },
  medium: { label: "中", className: "bg-warning/10 text-warning-foreground border-warning/20" },
  low: { label: "低", className: "bg-muted text-muted-foreground" },
};

const PRIORITY_ORDER: Record<Task["priority"], number> = { high: 0, medium: 1, low: 2 };

let nextId = 100;

export default function TasksView() {
  const [tasks, setTasks] = useState<Task[]>(INITIAL_TASKS);
  const [title, setTitle] = useState("");
  const [priority, setPriority] = useState<Task["priority"]>("medium");
  const [dueDate, setDueDate] = useState("");
  const [scheduledTasks, setScheduledTasks] = useState<TaskInfo[]>([]);
  const [schedulerPaused, setSchedulerPaused] = useState(false);
  const [loadingScheduled, setLoadingScheduled] = useState(false);

  // 加载调度器任务
  useEffect(() => {
    const loadScheduledTasks = async () => {
      setLoadingScheduled(true);
      try {
        const [tasks, paused] = await Promise.all([
          listScheduledTasks(),
          isSchedulerPaused(),
        ]);
        setScheduledTasks(tasks);
        setSchedulerPaused(paused);
      } catch (err) {
        console.error("Failed to load scheduled tasks:", err);
      } finally {
        setLoadingScheduled(false);
      }
    };
    loadScheduledTasks();
  }, []);

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

  const handleCreate = useCallback(
    (e: FormEvent) => {
      e.preventDefault();
      const trimmed = title.trim();
      if (!trimmed) return;
      setTasks((prev) => [
        ...prev,
        {
          id: String(nextId++),
          title: trimmed,
          status: "todo",
          priority,
          dueDate: dueDate || "未设定",
          createdAt: new Date().toISOString().slice(0, 10),
        },
      ]);
      setTitle("");
      setDueDate("");
    },
    [title, priority, dueDate],
  );

  const toggleStatus = useCallback((id: string) => {
    setTasks((prev) =>
      prev.map((t) =>
        t.id === id ? { ...t, status: STATUS_META[t.status].next } : t,
      ),
    );
  }, []);

  const grouped = (["todo", "in_progress", "done"] as const).map((s) => ({
    status: s,
    label: STATUS_META[s].label,
    icon: STATUS_META[s].icon,
    items: tasks
      .filter((t) => t.status === s)
      .sort((a, b) => PRIORITY_ORDER[a.priority] - PRIORITY_ORDER[b.priority]),
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
        </div>
      </header>

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
          onValueChange={(v) => setPriority(v as Task["priority"])}
        >
          <SelectTrigger className="h-8 w-[100px] text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="high">高优先级</SelectItem>
            <SelectItem value="medium">中优先级</SelectItem>
            <SelectItem value="low">低优先级</SelectItem>
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
        {grouped.map(({ status, label, icon: StatusIcon, items }) => (
          <div key={status} className="flex flex-1 flex-col gap-2">
            <div className="flex items-center gap-2 px-1">
              <StatusIcon
                className={cn(
                  "h-4 w-4",
                  status === "todo" && "text-muted-foreground",
                  status === "in_progress" && "text-info",
                  status === "done" && "text-success"
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
                        className={cn("text-[10px]", PRIORITY_CONFIG[task.priority].className)}
                      >
                        {PRIORITY_CONFIG[task.priority].label}
                      </Badge>
                      <span className="text-[11px] text-muted-foreground">
                        {task.dueDate}
                      </span>
                    </div>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="mt-1.5 h-6 gap-1 px-1 text-[11px] text-muted-foreground"
                      onClick={() => toggleStatus(task.id)}
                    >
                      {status === "done" ? (
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
        ))}
      </div>

      {/* Scheduled Tasks */}
      <div className="border-t border-border px-6 py-4">
        <div className="flex items-center justify-between mb-3">
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
          <div className="flex flex-col gap-1">
            {SCHEDULED_TASKS.map((s) => (
              <div
                key={s.id}
                className="flex items-center justify-between rounded-md px-3 py-2 text-sm hover:bg-muted/50"
              >
                <span>{s.title}</span>
                <span className="text-xs text-muted-foreground">
                  {s.schedule}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
