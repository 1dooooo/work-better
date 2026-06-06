import { useState, useEffect, useCallback } from "react";
import {
  getEvents,
  getUnprocessedCount,
  triggerManualCapture,
  type Event,
} from "../lib/tauri";

export default function MenuBar() {
  const [unprocessedCount, setUnprocessedCount] = useState(0);
  const [events, setEvents] = useState<Event[]>([]);
  const [captureText, setCaptureText] = useState("");
  const [isCapturing, setIsCapturing] = useState(false);
  const [status, setStatus] = useState<"idle" | "loading" | "success" | "error">("idle");

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

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      handleCapture();
    }
  };

  return (
    <div className="menu-bar">
      <header className="menu-bar__header">
        <h1 className="menu-bar__title">Work Better</h1>
        <div className="menu-bar__badge" data-count={unprocessedCount}>
          {unprocessedCount > 0 ? `${unprocessedCount} 待处理` : "✓ 已同步"}
        </div>
      </header>

      <section className="menu-bar__capture">
        <textarea
          className="menu-bar__input"
          placeholder="快速记录... (⌘+Enter 发送)"
          value={captureText}
          onChange={(e) => setCaptureText(e.target.value)}
          onKeyDown={handleKeyDown}
          rows={3}
          disabled={isCapturing}
        />
        <button
          className="menu-bar__submit"
          onClick={handleCapture}
          disabled={isCapturing || !captureText.trim()}
        >
          {status === "loading" ? "记录中..." : status === "success" ? "✓ 已记录" : "记录"}
        </button>
      </section>

      <section className="menu-bar__events">
        <h2 className="menu-bar__section-title">最近事件</h2>
        {events.length === 0 ? (
          <p className="menu-bar__empty">暂无事件</p>
        ) : (
          <ul className="menu-bar__list">
            {events.map((event) => (
              <li key={event.id} className="menu-bar__event">
                <span className="menu-bar__event-type">{event.type}</span>
                <span className="menu-bar__event-source">{event.source}</span>
                <span className="menu-bar__event-time">
                  {new Date(event.timestamp).toLocaleString("zh-CN")}
                </span>
              </li>
            ))}
          </ul>
        )}
      </section>

      <footer className="menu-bar__footer">
        <span className={`menu-bar__status menu-bar__status--${status}`}>
          {status === "error" ? "操作失败" : "运行中"}
        </span>
        <button className="menu-bar__refresh" onClick={refresh}>
          刷新
        </button>
      </footer>
    </div>
  );
}
