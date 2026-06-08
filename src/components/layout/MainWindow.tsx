import { useState, useEffect, useCallback } from "react";
import Sidebar, { type ViewId } from "./Sidebar";
import EventsView from "../views/EventsView";
import TasksView from "../views/TasksView";
import TimelineView from "../views/TimelineView";
import ReportsView from "../views/ReportsView";
import SettingsView from "../views/SettingsView";
import { getUnprocessedCount, onFeishuCollectComplete } from "@/lib/tauri";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Toaster } from "@/components/ui/sonner";

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
    const unlisten = onFeishuCollectComplete(() => {
      refreshCount();
    });
    return () => {
      clearInterval(interval);
      unlisten.then((fn) => fn());
    };
  }, [refreshCount]);

  const ActiveComponent = VIEW_COMPONENTS[activeView];

  return (
    <TooltipProvider>
      <div className="flex h-screen w-screen overflow-hidden bg-background text-foreground">
        <Sidebar
          activeView={activeView}
          onViewChange={setActiveView}
          unprocessedCount={unprocessedCount}
        />
        <main className="flex-1 overflow-auto">
          <ActiveComponent />
        </main>
      </div>
      <Toaster />
    </TooltipProvider>
  );
}
