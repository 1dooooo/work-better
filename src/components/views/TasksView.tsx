import { useState, useEffect } from "react";
import { getEvents, type Event } from "../../lib/tauri";

interface TaskItem {
  id: string;
  title: string;
  status: "todo" | "in_progress" | "done" | "blocked";
  priority: "p0" | "p1" | "p2" | "p3";
  source: string;
  createdAt: string;
}

function extractTasks(events: Event[]): TaskItem[] {
  return events
    .filter((e) => e.type === "task_update")
    .map((e) => ({
      id: e.id,
      title:
        typeof e.content === "string"
          ? e.content
          : ((e.content as Record<string, unknown>)?.title as string) ?? "未命名任务",
      status: "todo" as const,
      priority: "p2" as const,
      source: e.source,
      createdAt: e.timestamp,
    }));
}

const STATUS_LABELS: Record<TaskItem["status"], string> = {
  todo: "待办",
  in_progress: "进行中",
  done: "已完成",
  blocked: "阻塞",
};

const PRIORITY_LABELS: Record<TaskItem["priority"], string> = {
  p0: "🔴 P0",
  p1: "🟠 P1",
  p2: "🟡 P2",
  p3: "⚪ P3",
};

export default function TasksView() {
  const [tasks, setTasks] = useState<TaskItem[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    getEvents(100)
      .then((events) => setTasks(extractTasks(events)))
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  const grouped = {
    todo: tasks.filter((t) => t.status === "todo"),
    in_progress: tasks.filter((t) => t.status === "in_progress"),
    done: tasks.filter((t) => t.status === "done"),
    blocked: tasks.filter((t) => t.status === "blocked"),
  };

  if (loading) return <div className="view__loading">加载中...</div>;

  return (
    <div className="view tasks-view">
      <header className="view__header">
        <h2 className="view__title">任务看板</h2>
        <span className="view__count">{tasks.length} 个任务</span>
      </header>

      {tasks.length === 0 ? (
        <div className="view__empty">
          暂无任务。采集飞书消息后，任务将自动提取。
        </div>
      ) : (
        <div className="tasks-view__board">
          {(Object.entries(grouped) as [TaskItem["status"], TaskItem[]][]).map(
            ([status, items]) => (
              <div key={status} className="tasks-view__column">
                <h3 className="tasks-view__column-title">
                  {STATUS_LABELS[status]}
                  <span className="tasks-view__column-count">{items.length}</span>
                </h3>
                <ul className="tasks-view__column-list">
                  {items.map((task) => (
                    <li key={task.id} className="tasks-view__card">
                      <div className="tasks-view__card-title">{task.title}</div>
                      <div className="tasks-view__card-meta">
                        <span className="tasks-view__priority">
                          {PRIORITY_LABELS[task.priority]}
                        </span>
                        <span className="tasks-view__card-source">
                          {task.source}
                        </span>
                      </div>
                    </li>
                  ))}
                </ul>
              </div>
            )
          )}
        </div>
      )}
    </div>
  );
}
