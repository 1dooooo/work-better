export type ViewId = "events" | "tasks" | "timeline" | "reports" | "settings";

interface NavItem {
  id: ViewId;
  label: string;
  icon: string;
}

const NAV_ITEMS: NavItem[] = [
  { id: "events", label: "事件", icon: "📋" },
  { id: "tasks", label: "任务", icon: "✅" },
  { id: "timeline", label: "时间线", icon: "📅" },
  { id: "reports", label: "报告", icon: "📊" },
  { id: "settings", label: "设置", icon: "⚙" },
];

interface SidebarProps {
  activeView: ViewId;
  onViewChange: (view: ViewId) => void;
  unprocessedCount: number;
}

export default function Sidebar({
  activeView,
  onViewChange,
  unprocessedCount,
}: SidebarProps) {
  return (
    <nav className="sidebar">
      <div className="sidebar__brand">
        <span className="sidebar__logo">⚡</span>
        <span className="sidebar__app-name">Work Better</span>
      </div>

      <ul className="sidebar__nav">
        {NAV_ITEMS.map((item) => (
          <li key={item.id}>
            <button
              className={`sidebar__item ${
                activeView === item.id ? "sidebar__item--active" : ""
              }`}
              onClick={() => onViewChange(item.id)}
            >
              <span className="sidebar__icon">{item.icon}</span>
              <span className="sidebar__label">{item.label}</span>
              {item.id === "events" && unprocessedCount > 0 && (
                <span className="sidebar__badge">{unprocessedCount}</span>
              )}
            </button>
          </li>
        ))}
      </ul>

      <div className="sidebar__footer">
        <span className="sidebar__version">v0.1.0</span>
      </div>
    </nav>
  );
}
