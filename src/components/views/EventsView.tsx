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
  ChevronRight,
  ChevronDown,
} from "lucide-react";
import { toast } from "sonner";
import { useListNavigation } from "@/hooks/useListNavigation";
import { EmptyEvents } from "@/components/ui/empty-state";

type FilterSource = "all" | string;

// 事件类型配置
const EVENT_TYPE_CONFIG: Record<string, { label: string; className: string }> = {
  message: { label: "MSG", className: "bg-event-blue-bg text-event-blue-text" },
  issue: { label: "ISS", className: "bg-event-amber-bg text-event-amber-text" },
  pr: { label: "PR", className: "bg-event-green-bg text-event-green-text" },
  document: { label: "DOC", className: "bg-event-gray-bg text-event-gray-text" },
  default: { label: "EVT", className: "bg-event-gray-bg text-event-gray-text" },
};

export default function EventsView() {
  const [events, setEvents] = useState<Event[]>([]);
  const [filter, setFilter] = useState<FilterSource>("all");
  const [loading, setLoading] = useState(false);
  const [collecting, setCollecting] = useState(false);
  const [expandedEvents, setExpandedEvents] = useState<Set<string>>(new Set());

  // T3.2 vim 风格列表导航
  const { focusedIndex, listRef } = useListNavigation({
    itemCount: events.length,
    onEnter: (index) => toggleEvent(events[index]?.id ?? ""),
    onSpace: (index) => {
      const event = events[index];
      if (event && !event.processed) {
        handleMarkProcessed(event.id);
      }
    },
  });

  const toggleEvent = useCallback((id: string) => {
    setExpandedEvents((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }, []);

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
      <div ref={listRef} className="flex-1 overflow-y-auto">
        {loading && events.length === 0 ? (
          <div className="flex flex-col items-center justify-center gap-2 py-12 text-muted-foreground">
            <Loader2 className="h-4 w-4 animate-spin" />
            <span className="text-xs">加载中...</span>
          </div>
        ) : filteredEvents.length === 0 ? (
          <EmptyEvents />
        ) : (
          filteredEvents.map((event, index) => {
            const typeConfig = getEventType(event.type);
            const isExpanded = expandedEvents.has(event.id);
            const contentStr =
              typeof event.content === "string"
                ? event.content
                : JSON.stringify(event.content);
            return (
              <div key={event.id} className={`border-b border-border/50 ${index === focusedIndex ? "border-l-2 border-l-primary bg-muted/30" : ""}`}>
                {/* Event row - clickable to toggle expand */}
                <div
                  className="group flex items-center px-5 py-2 hover:bg-muted/50 transition-colors cursor-pointer min-h-[40px]"
                  onClick={() => toggleEvent(event.id)}
                >
                  {/* Chevron indicator */}
                  {isExpanded ? (
                    <ChevronDown className="h-3 w-3 text-muted-foreground mr-1.5 flex-shrink-0" />
                  ) : (
                    <ChevronRight className="h-3 w-3 text-muted-foreground mr-1.5 flex-shrink-0" />
                  )}

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

                  {/* 内容摘要 (80 chars when collapsed) */}
                  <span className="flex-1 text-xs text-foreground truncate mr-3">
                    {truncateContent(contentStr, 80)}
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

                {/* Expanded details */}
                {isExpanded && (
                  <div className="bg-muted/30 rounded p-2 mx-5 mb-2 text-[11px]">
                    <div className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1">
                      <span className="text-muted-foreground">时间</span>
                      <span className="text-foreground">
                        {new Date(event.timestamp).toLocaleString("zh-CN")}
                      </span>
                      <span className="text-muted-foreground">ID</span>
                      <span className="text-foreground font-mono text-[10px] break-all">
                        {event.id}
                      </span>
                      <span className="text-muted-foreground">内容</span>
                      <span className="text-foreground whitespace-pre-wrap break-words">
                        {contentStr}
                      </span>
                    </div>
                  </div>
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
