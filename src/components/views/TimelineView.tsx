import { useState, useEffect, useCallback } from "react";
import { getEvents, type Event } from "@/lib/tauri";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { RefreshCw, Loader2, Clock, Inbox, ChevronRight, ChevronDown } from "lucide-react";

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
  const [expandedGroups, setExpandedGroups] = useState<Set<string>>(new Set());

  const toggleGroup = useCallback((label: string) => {
    setExpandedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(label)) {
        next.delete(label);
      } else {
        next.add(label);
      }
      return next;
    });
  }, []);

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
      <header className="flex items-center justify-between border-b border-border px-5 py-3 min-h-[48px]">
        <div className="flex items-center gap-3">
          <h1 className="text-sm font-semibold">时间线</h1>
          <span className="text-[11px] text-muted-foreground bg-muted px-2 py-0.5 rounded-full">
            {events.length} 条记录
          </span>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={refresh}
          className="h-6 gap-1 text-[11px]"
        >
          <RefreshCw className="h-3 w-3" />
          刷新
        </Button>
      </header>

      {/* Content */}
      <ScrollArea className="flex-1 px-5 py-3">
        {loading ? (
          <div className="flex h-32 items-center justify-center text-muted-foreground">
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            加载中...
          </div>
        ) : timeGroups.length === 0 ? (
          <div className="flex h-32 flex-col items-center justify-center gap-1.5 text-muted-foreground">
            <Inbox className="h-6 w-6" />
            <span className="text-xs">暂无事件记录</span>
          </div>
        ) : (
          <div className="relative pl-5">
            {/* Vertical timeline line */}
            <div className="absolute left-[5px] top-1 bottom-1 w-px bg-border" />

            {timeGroups.map((group) => {
              const isExpanded = expandedGroups.has(group.label);
              return (
              <div key={group.label} className="relative mb-4 last:mb-0">
                {/* Time marker dot */}
                <div className="absolute -left-5 top-0.5 flex h-3 w-3 items-center justify-center">
                  <div className="h-1.5 w-1.5 rounded-full bg-primary" />
                </div>

                {/* Time label - clickable to toggle */}
                <div
                  className="mb-1.5 flex items-center gap-1.5 cursor-pointer hover:bg-muted/50 rounded px-2 py-1 transition-colors"
                  onClick={() => toggleGroup(group.label)}
                  role="button"
                  tabIndex={0}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                      e.preventDefault();
                      toggleGroup(group.label);
                    }
                  }}
                >
                  {isExpanded ? (
                    <ChevronDown className="h-3 w-3 text-muted-foreground flex-shrink-0" />
                  ) : (
                    <ChevronRight className="h-3 w-3 text-muted-foreground flex-shrink-0" />
                  )}
                  <Clock className="h-3 w-3 text-muted-foreground" />
                  <span className="text-[11px] font-medium text-muted-foreground">
                    {group.label}
                  </span>
                  <span className="text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded-full">
                    {group.events.length} 条事件
                  </span>
                </div>

                {/* Events in this time group - collapsible */}
                <div
                  className={`flex flex-col overflow-hidden transition-all duration-200 ease-in-out ${
                    isExpanded ? "max-h-[2000px] opacity-100" : "max-h-0 opacity-0"
                  }`}
                >
                  {group.events.map((event) => (
                    <div
                      key={event.id}
                      className="group flex items-center gap-2.5 rounded px-2 py-1.5 transition-colors hover:bg-muted/50"
                    >
                      <span className="text-[10px] font-semibold bg-muted px-1.5 py-0.5 rounded uppercase tracking-wider flex-shrink-0">
                        {event.type}
                      </span>
                      <span className="text-[10px] text-muted-foreground flex-shrink-0 min-w-[40px]">
                        {new Date(event.timestamp).toLocaleTimeString("zh-CN", {
                          hour: "2-digit",
                          minute: "2-digit",
                        })}
                      </span>
                      <span className="text-[10px] text-muted-foreground bg-muted/50 px-1.5 py-0.5 rounded flex-shrink-0">
                        {event.source}
                      </span>
                      <span className="flex-1 text-xs text-foreground truncate">
                        {typeof event.content === "string"
                          ? event.content.slice(0, 100)
                          : JSON.stringify(event.content).slice(0, 100)}
                      </span>
                    </div>
                  ))}
                </div>
              </div>
              );
            })}
          </div>
        )}
      </ScrollArea>
    </div>
  );
}
