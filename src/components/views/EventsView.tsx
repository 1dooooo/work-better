import { useState, useEffect, useCallback } from "react";
import {
  getEvents,
  markEventProcessed,
  triggerFeishuCollect,
  getFeishuChatId,
  onFeishuCollectComplete,
  type Event,
} from "@/lib/tauri";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  RefreshCw,
  Download,
  CheckCircle2,
  Loader2,
  Inbox,
} from "lucide-react";
import { toast } from "sonner";

type FilterSource = "all" | string;

// 事件类型配置
const EVENT_TYPE_CONFIG: Record<string, { label: string; className: string }> = {
  message: { label: "MSG", className: "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400" },
  issue: { label: "ISS", className: "bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400" },
  pr: { label: "PR", className: "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400" },
  document: { label: "DOC", className: "bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400" },
  default: { label: "EVT", className: "bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400" },
};

export default function EventsView() {
  const [events, setEvents] = useState<Event[]>([]);
  const [filter, setFilter] = useState<FilterSource>("all");
  const [loading, setLoading] = useState(false);
  const [collecting, setCollecting] = useState(false);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const data = await getEvents(50);
      setEvents(data);
    } catch (err) {
      console.error("Failed to load events:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
    const unlisten = onFeishuCollectComplete(() => {
      refresh();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [refresh]);

  const handleCollect = async () => {
    setCollecting(true);
    try {
      const chatId = await getFeishuChatId();
      await triggerFeishuCollect(chatId || undefined, 20);
      await refresh();
      toast.success("采集完成");
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      console.error("Collect failed:", err);
      toast.error("采集失败", { description: msg });
    } finally {
      setCollecting(false);
    }
  };

  const handleMarkProcessed = async (id: string) => {
    // Optimistic update
    setEvents((prev) =>
      prev.map((e) => (e.id === id ? { ...e, processed: true } : e)),
    );
    try {
      await markEventProcessed(id);
      toast.success("已标记为已处理");
    } catch (err) {
      // Rollback on failure
      setEvents((prev) =>
        prev.map((e) => (e.id === id ? { ...e, processed: false } : e)),
      );
      console.error("Mark processed failed:", err);
      toast.error("标记失败");
    }
  };

  const filteredEvents =
    filter === "all"
      ? events
      : events.filter((e) => e.source === filter);

  const sources = Array.from(new Set(events.map((e) => e.source)));

  const formatTime = (timestamp: string) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return "刚刚";
    if (diffMins < 60) return `${diffMins}分钟前`;
    if (diffHours < 24) return `${diffHours}小时前`;
    if (diffDays < 7) return `${diffDays}天前`;
    return date.toLocaleDateString("zh-CN", { month: "short", day: "numeric" });
  };

  const truncateContent = (content: string, maxLength: number = 100) => {
    if (content.length <= maxLength) return content;
    return content.slice(0, maxLength) + "...";
  };

  const getEventType = (type: string) => {
    const lowerType = type.toLowerCase();
    if (lowerType.includes("message") || lowerType.includes("消息")) return EVENT_TYPE_CONFIG.message;
    if (lowerType.includes("issue")) return EVENT_TYPE_CONFIG.issue;
    if (lowerType.includes("pr") || lowerType.includes("pull")) return EVENT_TYPE_CONFIG.pr;
    if (lowerType.includes("doc") || lowerType.includes("文档")) return EVENT_TYPE_CONFIG.document;
    return EVENT_TYPE_CONFIG.default;
  };

  const unprocessedCount = filteredEvents.filter((e) => !e.processed).length;

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-border px-5 py-3 min-h-[48px]">
        <div className="flex items-center gap-3">
          <h1 className="text-sm font-semibold text-foreground">事件流</h1>
          <span className="text-[11px] text-muted-foreground bg-muted px-2 py-0.5 rounded-full">
            {filteredEvents.length}
          </span>
        </div>
        <div className="flex items-center gap-2">
          <Select value={filter} onValueChange={(v) => v !== null && setFilter(v)}>
            <SelectTrigger className="h-7 w-[100px] text-[11px]">
              <SelectValue placeholder="全部来源" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部来源</SelectItem>
              {sources.map((src) => (
                <SelectItem key={src} value={src}>
                  {src}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Button
            size="sm"
            onClick={handleCollect}
            disabled={collecting}
            className="h-7 gap-1 text-[11px] px-3"
          >
            {collecting ? (
              <Loader2 className="h-3 w-3 animate-spin" />
            ) : (
              <Download className="h-3 w-3" />
            )}
            {collecting ? "采集中..." : "采集"}
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={refresh}
            className="h-7 gap-1 text-[11px] px-3"
          >
            <RefreshCw className="h-3 w-3" />
            刷新
          </Button>
        </div>
      </header>

      {/* Event List */}
      <div className="flex-1 overflow-y-auto">
        {loading && events.length === 0 ? (
          <div className="flex flex-col items-center justify-center gap-2 py-12 text-muted-foreground">
            <Loader2 className="h-4 w-4 animate-spin" />
            <span className="text-xs">加载中...</span>
          </div>
        ) : filteredEvents.length === 0 ? (
          <div className="flex flex-col items-center justify-center gap-2 py-12 text-muted-foreground">
            <Inbox className="h-6 w-6 opacity-50" />
            <span className="text-xs">暂无事件</span>
          </div>
        ) : (
          filteredEvents.map((event) => {
            const typeConfig = getEventType(event.type);
            return (
              <div
                key={event.id}
                className="group flex items-center px-5 py-2 border-b border-border/50 hover:bg-muted/50 transition-colors cursor-pointer min-h-[40px]"
                onClick={() => !event.processed && handleMarkProcessed(event.id)}
              >
                {/* 状态指示器 */}
                <div
                  className={`w-1.5 h-1.5 rounded-full mr-3 flex-shrink-0 ${
                    event.processed ? "bg-border" : "bg-primary"
                  }`}
                />

                {/* 类型标签 */}
                <span
                  className={`text-[10px] font-semibold px-1.5 py-0.5 rounded mr-2 flex-shrink-0 uppercase tracking-wider leading-none ${typeConfig.className}`}
                >
                  {typeConfig.label}
                </span>

                {/* 来源 */}
                <span className="text-[11px] text-muted-foreground mr-3 flex-shrink-0 min-w-[80px]">
                  {event.source}
                </span>

                {/* 内容摘要 */}
                <span className="flex-1 text-xs text-foreground truncate mr-3">
                  {truncateContent(
                    typeof event.content === "string"
                      ? event.content
                      : JSON.stringify(event.content)
                  )}
                </span>

                {/* 标签 */}
                {event.tags.length > 0 && (
                  <div className="flex gap-1 mr-3 flex-shrink-0">
                    {event.tags.slice(0, 3).map((tag) => (
                      <span
                        key={tag}
                        className="text-[10px] px-1 py-0.5 bg-muted text-muted-foreground rounded"
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
                )}

                {/* 时间 */}
                <span className="text-[10px] text-muted-foreground flex-shrink-0 min-w-[60px] text-right mr-3">
                  {formatTime(event.timestamp)}
                </span>

                {/* 操作按钮 */}
                {!event.processed && (
                  <button
                    className="flex items-center gap-1 text-[10px] px-2 py-1 bg-background border border-border rounded text-muted-foreground opacity-0 group-hover:opacity-100 transition-all hover:bg-primary hover:text-primary-foreground hover:border-primary flex-shrink-0"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleMarkProcessed(event.id);
                    }}
                  >
                    <CheckCircle2 className="h-3 w-3" />
                    已处理
                  </button>
                )}
              </div>
            );
          })
        )}
      </div>

      {/* Footer */}
      <footer className="flex items-center justify-between px-5 py-2 border-t border-border bg-muted/30 text-[10px] text-muted-foreground">
        <span>共 {filteredEvents.length} 个事件</span>
        <span>{unprocessedCount} 个未处理</span>
      </footer>
    </div>
  );
}
