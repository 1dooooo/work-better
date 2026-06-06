import { useState, useEffect, useCallback } from "react";
import Sidebar, { type ViewId } from "./Sidebar";
import EventsView from "./views/EventsView";
import TasksView from "./views/TasksView";
import TimelineView from "./views/TimelineView";
import ReportsView from "./views/ReportsView";
import SettingsView from "./views/SettingsView";
import { getUnprocessedCount } from "../lib/tauri";

const VIEW_COMPONENTS: Record<ViewId, React.ComponentType> = {
  events: EventsView,
  tasks: TasksView,
  timeline: TimelineView,
  reports: ReportsView,
  settings: SettingsView,
};

export default function MainWindow() {
  const [activeView, setActiveView] = useState<ViewId>("events");
  const [unprocessedCount, setUnprocessedCount] = useState(0);

  const refreshCount = useCallback(async () => {
    try {
      const count = await getUnprocessedCount();
      setUnprocessedCount(count);
    } catch (err) {
      console.error("Failed to get unprocessed count:", err);
    }
  }, []);

  useEffect(() => {
    refreshCount();
    const interval = setInterval(refreshCount, 30_000);
    return () => clearInterval(interval);
  }, [refreshCount]);

  const ActiveComponent = VIEW_COMPONENTS[activeView];

  return (
    <div className="main-window">
      <Sidebar
        activeView={activeView}
        onViewChange={setActiveView}
        unprocessedCount={unprocessedCount}
      />
      <main className="main-window__content">
        <ActiveComponent />
      </main>
    </div>
  );
}
