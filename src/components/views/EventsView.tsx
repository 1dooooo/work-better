import { useState, useEffect, useCallback } from "react";
import {
  getEvents,
  markEventProcessed,
  triggerFeishuCollect,
  getFeishuChatId,
  onFeishuCollectComplete,
  type Event,
} from "../../lib/tauri";

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
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      console.error("Collect failed:", err);
      setError(`采集失败: ${msg}`);
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
    } catch (err) {
      console.error("Mark processed failed:", err);
    }
  };

  const filteredEvents =
    filter === "all"
      ? events
      : events.filter((e) => e.source === filter);

  const sources = Array.from(new Set(events.map((e) => e.source)));

  return (
    <div className="view events-view">
      <header className="view__header">
        <h2 className="view__title">事件流</h2>
        <div className="view__actions">
          <select
            className="view__filter"
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
          >
            <option value="all">全部来源</option>
            {sources.map((src) => (
              <option key={src} value={src}>
                {src}
              </option>
            ))}
          </select>
          <button
            className="view__btn"
            onClick={handleCollect}
            disabled={collecting}
          >
            {collecting ? "采集中..." : "采集飞书"}
          </button>
          <button className="view__btn view__btn--secondary" onClick={refresh}>
            刷新
          </button>
        </div>
      </header>

      {error && (
        <div className="events-view__error">
          {error}
          <button
            className="events-view__action-btn"
            onClick={() => setError(null)}
            style={{ marginLeft: "0.5rem" }}
          >
            ✕
          </button>
        </div>
      )}

      {loading && events.length === 0 ? (
        <div className="view__loading">加载中...</div>
      ) : filteredEvents.length === 0 ? (
        <div className="view__empty">暂无事件</div>
      ) : (
        <ul className="events-view__list">
          {filteredEvents.map((event) => (
            <li key={event.id} className="events-view__card">
              <div className="events-view__card-header">
                <span className="events-view__type">{event.type}</span>
                <span className="events-view__source">{event.source}</span>
                <span className="events-view__time">
                  {new Date(event.timestamp).toLocaleString("zh-CN")}
                </span>
              </div>
              <div className="events-view__card-body">
                <pre className="events-view__content">
                  {typeof event.content === "string"
                    ? event.content
                    : JSON.stringify(event.content, null, 2)}
                </pre>
              </div>
              {event.tags.length > 0 && (
                <div className="events-view__tags">
                  {event.tags.map((tag) => (
                    <span key={tag} className="events-view__tag">
                      {tag}
                    </span>
                  ))}
                </div>
              )}
              <div className="events-view__card-actions">
                <button
                  className="events-view__action-btn"
                  onClick={() => handleMarkProcessed(event.id)}
                >
                  标记已处理
                </button>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
