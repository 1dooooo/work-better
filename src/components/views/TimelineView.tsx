import { useState, useEffect, useCallback } from "react";
import { getEvents, type Event } from "@/lib/tauri";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { RefreshCw, Loader2, Clock, Inbox } from "lucide-react";

interface TimeGroup {
  label: string;
  events: Event[];
}

function groupByHour(events: Event[]): TimeGroup[] {
  const groups = new Map<string, Event[]>();

  for (const event of events) {
    const date = new Date(event.timestamp);
    const hour = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")} ${String(date.getHours()).padStart(2, "0")}:00`;
    const existing = groups.get(hour) ?? [];
    existing.push(event);
    groups.set(hour, existing);
  }

  return Array.from(groups.entries())
    .sort(([a], [b]) => b.localeCompare(a))
    .map(([label, evts]) => ({ label, events: evts }));
}

export default function TimelineView() {
  const [events, setEvents] = useState<Event[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const data = await getEvents(200);
      setEvents(data);
    } catch (err) {
      console.error("Failed to load timeline:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const timeGroups = groupByHour(events);

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-border px-6 py-4">
        <div className="flex items-center gap-3">
          <h1 className="text-lg font-semibold">时间线</h1>
          <Badge variant="secondary" className="text-xs">
            {events.length} 条记录
          </Badge>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={refresh}
          className="h-8 gap-1.5"
        >
          <RefreshCw className="h-3.5 w-3.5" />
          刷新
        </Button>
      </header>

      {/* Content */}
      <ScrollArea className="flex-1 px-6 py-4">
        {loading ? (
          <div className="flex h-40 items-center justify-center text-muted-foreground">
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            加载中...
          </div>
        ) : timeGroups.length === 0 ? (
          <div className="flex h-40 flex-col items-center justify-center gap-2 text-muted-foreground">
            <Inbox className="h-8 w-8" />
            <span className="text-sm">暂无事件记录</span>
          </div>
        ) : (
          <div className="relative pl-6">
            {/* Vertical timeline line */}
            <div className="absolute left-[7px] top-2 bottom-2 w-px bg-border" />

            {timeGroups.map((group) => (
              <div key={group.label} className="relative mb-6 last:mb-0">
                {/* Time marker dot */}
                <div className="absolute -left-6 top-1 flex h-3.5 w-3.5 items-center justify-center">
                  <div className="h-2 w-2 rounded-full bg-primary" />
                </div>

                {/* Time label */}
                <div className="mb-2 flex items-center gap-2">
                  <Clock className="h-3.5 w-3.5 text-muted-foreground" />
                  <span className="text-xs font-medium text-muted-foreground">
                    {group.label}
                  </span>
                </div>

                {/* Events in this time group */}
                <div className="flex flex-col gap-1">
                  {group.events.map((event) => (
                    <div
                      key={event.id}
                      className="flex items-start gap-2 rounded-md px-3 py-2 transition-colors hover:bg-muted/50"
                    >
                      <Badge variant="outline" className="mt-0.5 shrink-0 text-[10px]">
                        {event.type}
                      </Badge>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <Badge variant="secondary" className="text-[10px]">
                            {event.source}
                          </Badge>
                          <span className="text-[11px] text-muted-foreground">
                            {new Date(event.timestamp).toLocaleTimeString("zh-CN", {
                              hour: "2-digit",
                              minute: "2-digit",
                            })}
                          </span>
                        </div>
                        <p className="mt-0.5 truncate text-sm text-foreground/80">
                          {typeof event.content === "string"
                            ? event.content.slice(0, 100)
                            : JSON.stringify(event.content).slice(0, 100)}
                        </p>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </ScrollArea>
    </div>
  );
}
