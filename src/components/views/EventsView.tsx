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
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  RefreshCw,
  Download,
  CheckCircle2,
  X,
  Loader2,
  Inbox,
} from "lucide-react";
import { toast } from "sonner";

type FilterSource = "all" | string;

export default function EventsView() {
  const [events, setEvents] = useState<Event[]>([]);
  const [filter, setFilter] = useState<FilterSource>("all");
  const [loading, setLoading] = useState(false);
  const [collecting, setCollecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

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
    setError(null);
    try {
      const chatId = await getFeishuChatId();
      await triggerFeishuCollect(chatId || undefined, 20);
      await refresh();
      toast.success("采集完成");
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      console.error("Collect failed:", err);
      setError(`采集失败: ${msg}`);
      toast.error("采集失败", { description: msg });
    } finally {
      setCollecting(false);
    }
  };

  const handleMarkProcessed = async (id: string) => {
    try {
      await markEventProcessed(id);
      setEvents((prev) =>
        prev.map((e) => (e.id === id ? { ...e, processed: true } : e))
      );
      toast.success("已标记为已处理");
    } catch (err) {
      console.error("Mark processed failed:", err);
      toast.error("标记失败");
    }
  };

  const filteredEvents =
    filter === "all"
      ? events
      : events.filter((e) => e.source === filter);

  const sources = Array.from(new Set(events.map((e) => e.source)));

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-border px-6 py-4">
        <div className="flex items-center gap-3">
          <h1 className="text-lg font-semibold">事件流</h1>
          <Badge variant="secondary" className="text-xs">
            {filteredEvents.length}
          </Badge>
        </div>
        <div className="flex items-center gap-2">
          <Select value={filter} onValueChange={(v) => v !== null && setFilter(v)}>
            <SelectTrigger className="h-8 w-[130px] text-xs">
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
            className="h-8 gap-1.5"
          >
            {collecting ? (
              <Loader2 className="h-3.5 w-3.5 animate-spin" />
            ) : (
              <Download className="h-3.5 w-3.5" />
            )}
            {collecting ? "采集中..." : "采集飞书"}
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={refresh}
            className="h-8 gap-1.5"
          >
            <RefreshCw className="h-3.5 w-3.5" />
            刷新
          </Button>
        </div>
      </header>

      {/* Error Banner */}
      {error && (
        <div className="flex items-center gap-2 bg-destructive/10 px-6 py-2 text-sm text-destructive">
          <span className="flex-1">{error}</span>
          <Button
            variant="ghost"
            size="icon"
            className="h-5 w-5 text-destructive"
            onClick={() => setError(null)}
          >
            <X className="h-3.5 w-3.5" />
          </Button>
        </div>
      )}

      {/* Content */}
      <ScrollArea className="flex-1 px-6 py-4">
        {loading && events.length === 0 ? (
          <div className="flex h-40 items-center justify-center text-muted-foreground">
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            加载中...
          </div>
        ) : filteredEvents.length === 0 ? (
          <div className="flex h-40 flex-col items-center justify-center gap-2 text-muted-foreground">
            <Inbox className="h-8 w-8" />
            <span className="text-sm">暂无事件</span>
          </div>
        ) : (
          <div className="flex flex-col gap-2">
            {filteredEvents.map((event) => (
              <Card key={event.id} className="border-border">
                <CardHeader className="flex flex-row items-center gap-2 px-4 py-2.5">
                  <Badge variant="outline" className="text-[11px]">
                    {event.type}
                  </Badge>
                  <Badge variant="secondary" className="text-[11px]">
                    {event.source}
                  </Badge>
                  <span className="ml-auto text-[11px] text-muted-foreground">
                    {new Date(event.timestamp).toLocaleString("zh-CN")}
                  </span>
                </CardHeader>
                <CardContent className="px-4 pb-3 pt-0">
                  <pre className="whitespace-pre-wrap break-words text-sm text-foreground/90">
                    {typeof event.content === "string"
                      ? event.content
                      : JSON.stringify(event.content, null, 2)}
                  </pre>
                  {event.tags.length > 0 && (
                    <div className="mt-2 flex flex-wrap gap-1">
                      {event.tags.map((tag) => (
                        <Badge
                          key={tag}
                          variant="secondary"
                          className="text-[10px]"
                        >
                          {tag}
                        </Badge>
                      ))}
                    </div>
                  )}
                  <Separator className="my-2" />
                  <div className="flex justify-end">
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-7 gap-1 text-xs text-muted-foreground"
                      onClick={() => handleMarkProcessed(event.id)}
                    >
                      <CheckCircle2 className="h-3.5 w-3.5" />
                      标记已处理
                    </Button>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        )}
      </ScrollArea>
    </div>
  );
}
