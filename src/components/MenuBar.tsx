import { useState, useEffect, useCallback } from "react";
import {
  getEvents,
  getUnprocessedCount,
  triggerManualCapture,
  type Event,
} from "../lib/tauri";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  Zap,
  PenLine,
  PlusCircle,
  RefreshCw,
  Loader2,
  Check,
  AlertCircle,
  Inbox,
  Clock,
} from "lucide-react";
import { cn } from "@/lib/utils";

export default function MenuBar() {
  const [unprocessedCount, setUnprocessedCount] = useState(0);
  const [events, setEvents] = useState<Event[]>([]);
  const [captureText, setCaptureText] = useState("");
  const [taskTitle, setTaskTitle] = useState("");
  const [isCapturing, setIsCapturing] = useState(false);
  const [isCreatingTask, setIsCreatingTask] = useState(false);
  const [status, setStatus] = useState<"idle" | "loading" | "success" | "error">("idle");
  const [activeTab, setActiveTab] = useState<"capture" | "task">("capture");

  const refresh = useCallback(async () => {
    try {
      const [count, recentEvents] = await Promise.all([
        getUnprocessedCount(),
        getEvents(10),
      ]);
      setUnprocessedCount(count);
      setEvents(recentEvents);
    } catch (err) {
      console.error("Failed to refresh:", err);
    }
  }, []);

  useEffect(() => {
    refresh();
    const interval = setInterval(refresh, 30_000);
    return () => clearInterval(interval);
  }, [refresh]);

  const handleCapture = async () => {
    if (!captureText.trim()) return;
    setIsCapturing(true);
    setStatus("loading");
    try {
      await triggerManualCapture(captureText.trim());
      setCaptureText("");
      setStatus("success");
      await refresh();
      setTimeout(() => setStatus("idle"), 2000);
    } catch (err) {
      console.error("Capture failed:", err);
      setStatus("error");
      setTimeout(() => setStatus("idle"), 3000);
    } finally {
      setIsCapturing(false);
    }
  };

  const handleCreateTask = async () => {
    if (!taskTitle.trim()) return;
    setIsCreatingTask(true);
    try {
      await triggerManualCapture(`[任务] ${taskTitle.trim()}`);
      setTaskTitle("");
      setStatus("success");
      await refresh();
      setTimeout(() => setStatus("idle"), 2000);
    } catch (err) {
      console.error("Task creation failed:", err);
      setStatus("error");
      setTimeout(() => setStatus("idle"), 3000);
    } finally {
      setIsCreatingTask(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent, handler: () => void) => {
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      handler();
    }
  };

  return (
    <div className="flex h-full flex-col bg-background text-foreground">
      {/* Header */}
      <header className="flex items-center justify-between px-4 py-3">
        <div className="flex items-center gap-2">
          <div className="flex h-6 w-6 items-center justify-center rounded-md bg-primary text-primary-foreground">
            <Zap className="h-3.5 w-3.5" />
          </div>
          <span className="text-sm font-semibold">Work Better</span>
        </div>
        <Badge
          variant={unprocessedCount > 0 ? "default" : "secondary"}
          className="text-[10px]"
        >
          {unprocessedCount > 0 ? `${unprocessedCount} 待处理` : "已同步"}
        </Badge>
      </header>

      <Separator />

      {/* Tab Switcher */}
      <div className="flex border-b border-border">
        <button
          onClick={() => setActiveTab("capture")}
          className={cn(
            "flex flex-1 items-center justify-center gap-1.5 py-2 text-xs font-medium transition-colors",
            "border-b-2",
            activeTab === "capture"
              ? "border-primary text-primary"
              : "border-transparent text-muted-foreground hover:text-foreground"
          )}
        >
          <PenLine className="h-3.5 w-3.5" />
          极速记录
        </button>
        <button
          onClick={() => setActiveTab("task")}
          className={cn(
            "flex flex-1 items-center justify-center gap-1.5 py-2 text-xs font-medium transition-colors",
            "border-b-2",
            activeTab === "task"
              ? "border-primary text-primary"
              : "border-transparent text-muted-foreground hover:text-foreground"
          )}
        >
          <PlusCircle className="h-3.5 w-3.5" />
          任务速建
        </button>
      </div>

      {/* Capture / Task Form */}
      <div className="px-4 py-3">
        {activeTab === "capture" ? (
          <div className="space-y-2">
            <Textarea
              placeholder="记录想法、笔记、待办... (⌘+Enter 发送)"
              value={captureText}
              onChange={(e) => setCaptureText(e.target.value)}
              onKeyDown={(e) => handleKeyDown(e, handleCapture)}
              rows={3}
              disabled={isCapturing}
              className="resize-none text-sm"
            />
            <Button
              size="sm"
              className="w-full gap-1.5"
              onClick={handleCapture}
              disabled={isCapturing || !captureText.trim()}
            >
              {isCapturing ? (
                <Loader2 className="h-3.5 w-3.5 animate-spin" />
              ) : status === "success" ? (
                <Check className="h-3.5 w-3.5" />
              ) : (
                <PenLine className="h-3.5 w-3.5" />
              )}
              {isCapturing ? "记录中..." : status === "success" ? "已记录" : "记录"}
            </Button>
          </div>
        ) : (
          <div className="space-y-2">
            <Input
              placeholder="任务标题... (⌘+Enter 创建)"
              value={taskTitle}
              onChange={(e) => setTaskTitle(e.target.value)}
              onKeyDown={(e) => handleKeyDown(e, handleCreateTask)}
              disabled={isCreatingTask}
              className="text-sm"
            />
            <Button
              size="sm"
              className="w-full gap-1.5"
              onClick={handleCreateTask}
              disabled={isCreatingTask || !taskTitle.trim()}
            >
              {isCreatingTask ? (
                <Loader2 className="h-3.5 w-3.5 animate-spin" />
              ) : (
                <PlusCircle className="h-3.5 w-3.5" />
              )}
              {isCreatingTask ? "创建中..." : "创建任务"}
            </Button>
          </div>
        )}

        {/* Status Message */}
        {status === "error" && (
          <div className="mt-2 flex items-center gap-1.5 text-xs text-destructive">
            <AlertCircle className="h-3.5 w-3.5" />
            操作失败
          </div>
        )}
      </div>

      <Separator />

      {/* Recent Events */}
      <div className="flex flex-1 flex-col overflow-hidden">
        <div className="flex items-center justify-between px-4 py-2">
          <div className="flex items-center gap-1.5">
            <Clock className="h-3.5 w-3.5 text-muted-foreground" />
            <span className="text-xs font-medium text-muted-foreground">
              最近事件
            </span>
          </div>
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6 text-muted-foreground"
            onClick={refresh}
          >
            <RefreshCw className="h-3 w-3" />
          </Button>
        </div>

        <ScrollArea className="flex-1 px-4">
          {events.length === 0 ? (
            <div className="flex h-24 flex-col items-center justify-center gap-1.5 text-muted-foreground">
              <Inbox className="h-5 w-5" />
              <span className="text-xs">暂无事件</span>
            </div>
          ) : (
            <div className="space-y-1 pb-3">
              {events.map((event) => (
                <div
                  key={event.id}
                  className="flex items-start gap-2 rounded-md px-2 py-1.5 transition-colors hover:bg-muted/50"
                >
                  <Badge
                    variant="outline"
                    className="mt-0.5 shrink-0 text-[9px]"
                  >
                    {event.type}
                  </Badge>
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-1.5">
                      <span className="text-[11px] font-medium">
                        {event.source}
                      </span>
                      <span className="text-[10px] text-muted-foreground">
                        {new Date(event.timestamp).toLocaleTimeString("zh-CN", {
                          hour: "2-digit",
                          minute: "2-digit",
                        })}
                      </span>
                    </div>
                    <p className="truncate text-[11px] text-muted-foreground">
                      {typeof event.content === "string"
                        ? event.content.slice(0, 60)
                        : JSON.stringify(event.content).slice(0, 60)}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          )}
        </ScrollArea>
      </div>

      <Separator />

      {/* Footer */}
      <footer className="flex items-center justify-between px-4 py-2">
        <span className="text-[10px] text-muted-foreground">
          {status === "error" ? "操作失败" : "运行中"}
        </span>
        <span className="text-[10px] text-muted-foreground">v0.1.0</span>
      </footer>
    </div>
  );
}
