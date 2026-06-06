import { useState } from "react";

interface Shortcut {
  id: string;
  label: string;
  key: string;
  modifiers: string[];
}

const DEFAULT_SHORTCUTS: Shortcut[] = [
  { id: "capture", label: "快速捕获", key: "Space", modifiers: ["cmd", "shift"] },
  { id: "search", label: "全局搜索", key: "K", modifiers: ["cmd"] },
  { id: "task", label: "新建任务", key: "N", modifiers: ["cmd", "shift"] },
];

export default function ShortcutSettings() {
  const [shortcuts, setShortcuts] = useState<Shortcut[]>(DEFAULT_SHORTCUTS);
  const [editing, setEditing] = useState<string | null>(null);

  const formatKey = (s: Shortcut) =>
    [...s.modifiers.map((m) => (m === "cmd" ? "⌘" : m === "shift" ? "⇧" : m === "alt" ? "⌥" : "⌃")), s.key].join(" + ");

  const handleReset = () => {
    setShortcuts(DEFAULT_SHORTCUTS);
    setEditing(null);
  };

  return (
    <section className="settings-section">
      <h3>快捷键配置</h3>
      <p className="settings-hint">自定义全局快捷键</p>

      <div className="shortcut-list">
        {shortcuts.map((s) => (
          <div key={s.id} className={`shortcut-item ${editing === s.id ? "shortcut-item--editing" : ""}`}>
            <span className="shortcut-label">{s.label}</span>
            <kbd className="shortcut-key" onClick={() => setEditing(editing === s.id ? null : s.id)}>
              {formatKey(s)}
            </kbd>
            {editing === s.id && <span className="shortcut-hint">按下新的快捷键组合...</span>}
          </div>
        ))}
      </div>

      <button className="btn btn--outline" onClick={handleReset}>
        恢复默认
      </button>
    </section>
  );
}
