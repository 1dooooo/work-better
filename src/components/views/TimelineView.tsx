import { useState, useEffect, useCallback } from "react";
import { getEvents, type Event } from "../../lib/tauri";

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

  if (loading) return <div className="view__loading">加载中...</div>;

  return (
    <div className="view timeline-view">
      <header className="view__header">
        <h2 className="view__title">时间线</h2>
        <button className="view__btn view__btn--secondary" onClick={refresh}>
          刷新
        </button>
      </header>

      {timeGroups.length === 0 ? (
        <div className="view__empty">暂无事件记录</div>
      ) : (
        <div className="timeline-view__container">
          {timeGroups.map((group) => (
            <div key={group.label} className="timeline-view__group">
              <div className="timeline-view__time-marker">
                <span className="timeline-view__dot" />
                <span className="timeline-view__time">{group.label}</span>
              </div>
              <div className="timeline-view__items">
                {group.events.map((event) => (
                  <div key={event.id} className="timeline-view__item">
                    <span className="timeline-view__item-type">
                      {event.type}
                    </span>
                    <span className="timeline-view__item-source">
                      {event.source}
                    </span>
                    <span className="timeline-view__item-preview">
                      {typeof event.content === "string"
                        ? event.content.slice(0, 80)
                        : JSON.stringify(event.content).slice(0, 80)}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
