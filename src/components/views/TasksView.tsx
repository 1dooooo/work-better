import { useState, useCallback, type FormEvent } from "react";

interface Task {
  id: string;
  title: string;
  status: "todo" | "in_progress" | "done";
  priority: "high" | "medium" | "low";
  dueDate: string;
  createdAt: string;
}

const INITIAL_TASKS: Task[] = [
  { id: "1", title: "整理周报", status: "todo", priority: "high", dueDate: "2026-06-07", createdAt: "2026-06-06" },
  { id: "2", title: "Review PR #42", status: "todo", priority: "medium", dueDate: "2026-06-08", createdAt: "2026-06-06" },
  { id: "3", title: "更新项目文档", status: "in_progress", priority: "low", dueDate: "2026-06-10", createdAt: "2026-06-05" },
  { id: "4", title: "修复登录 Bug", status: "in_progress", priority: "high", dueDate: "2026-06-06", createdAt: "2026-06-04" },
  { id: "5", title: "设计新功能原型", status: "done", priority: "medium", dueDate: "2026-06-05", createdAt: "2026-06-01" },
];

const SCHEDULED_TASKS = [
  { id: "s1", title: "每日站会", schedule: "每天 10:00" },
  { id: "s2", title: "周报汇总", schedule: "每周五 17:00" },
  { id: "s3", title: "依赖安全检查", schedule: "每周一 09:00" },
];

const STATUS_META: Record<Task["status"], { label: string; next: Task["status"] }> = {
  todo: { label: "待处理", next: "in_progress" },
  in_progress: { label: "进行中", next: "done" },
  done: { label: "已完成", next: "todo" },
};

const PRIORITY_LABELS: Record<Task["priority"], string> = {
  high: "高",
  medium: "中",
  low: "低",
};

const PRIORITY_ORDER: Record<Task["priority"], number> = { high: 0, medium: 1, low: 2 };

let nextId = 100;

export default function TaskView() {
  const [tasks, setTasks] = useState<Task[]>(INITIAL_TASKS);
  const [title, setTitle] = useState("");
  const [priority, setPriority] = useState<Task["priority"]>("medium");
  const [dueDate, setDueDate] = useState("");

  const handleCreate = useCallback(
    (e: FormEvent) => {
      e.preventDefault();
      const trimmed = title.trim();
      if (!trimmed) return;
      setTasks((prev) => [
        ...prev,
        {
          id: String(nextId++),
          title: trimmed,
          status: "todo",
          priority,
          dueDate: dueDate || "未设定",
          createdAt: new Date().toISOString().slice(0, 10),
        },
      ]);
      setTitle("");
      setDueDate("");
    },
    [title, priority, dueDate],
  );

  const toggleStatus = useCallback((id: string) => {
    setTasks((prev) =>
      prev.map((t) =>
        t.id === id ? { ...t, status: STATUS_META[t.status].next } : t,
      ),
    );
  }, []);

  const grouped = (["todo", "in_progress", "done"] as const).map((s) => ({
    status: s,
    label: STATUS_META[s].label,
    items: tasks
      .filter((t) => t.status === s)
      .sort((a, b) => PRIORITY_ORDER[a.priority] - PRIORITY_ORDER[b.priority]),
  }));

  return (
    <div className="view tasks-view">
      <div className="view__header">
        <h2 className="view__title">任务管理</h2>
        <span className="view__count">{tasks.length} 个任务</span>
      </div>

      <form className="tasks-view__form" onSubmit={handleCreate}>
        <input
          className="tasks-view__input"
          placeholder="新任务标题..."
          value={title}
          onChange={(e) => setTitle(e.target.value)}
        />
        <select
          className="tasks-view__select"
          value={priority}
          onChange={(e) => setPriority(e.target.value as Task["priority"])}
        >
          <option value="high">高优先级</option>
          <option value="medium">中优先级</option>
          <option value="low">低优先级</option>
        </select>
        <input
          className="tasks-view__input tasks-view__input--date"
          type="date"
          value={dueDate}
          onChange={(e) => setDueDate(e.target.value)}
        />
        <button className="tasks-view__btn" type="submit">
          添加
        </button>
      </form>

      <div className="tasks-view__board">
        {grouped.map(({ status, label, items }) => (
          <div key={status} className="tasks-view__column">
            <h3 className="tasks-view__column-title">
              {label}
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
                    <span className="tasks-view__card-date">{task.dueDate}</span>
                  </div>
                  <button
                    className="tasks-view__card-btn"
                    onClick={() => toggleStatus(task.id)}
                  >
                    {status === "done" ? "重新打开" : "推进"}
                  </button>
                </li>
              ))}
              {items.length === 0 && (
                <li className="tasks-view__empty-col">暂无任务</li>
              )}
            </ul>
          </div>
        ))}
      </div>

      <section className="tasks-view__scheduled">
        <h3 className="tasks-view__section-title">定时任务</h3>
        <ul className="tasks-view__scheduled-list">
          {SCHEDULED_TASKS.map((s) => (
            <li key={s.id} className="tasks-view__scheduled-item">
              <span>{s.title}</span>
              <span className="tasks-view__schedule">{s.schedule}</span>
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}
