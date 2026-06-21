import { useState, useEffect, useCallback } from "react";
import Sidebar, { type ViewId } from "./Sidebar";
import DashboardView from "../views/DashboardView";
import EventsView from "../views/EventsView";
import TasksView from "../views/TasksView";
import TimelineView from "../views/TimelineView";
import ReportsView from "../views/ReportsView";
import SettingsView from "../views/SettingsView";
import AuditView from "../views/AuditView";
import CommandPalette from "../CommandPalette";
import { getUnprocessedCount, onFeishuCollectComplete, getDeveloperMode } from "@/lib/tauri";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Toaster } from "@/components/ui/sonner";
import { useKeyboardShortcuts, SHORTCUTS } from "@/hooks/useKeyboardShortcuts";

const VIEW_COMPONENTS: Record<ViewId, React.ComponentType> = {
  dashboard: DashboardView,
  events: EventsView,
  tasks: TasksView,
  timeline: TimelineView,
  reports: ReportsView,
  settings: SettingsView,
  audit: AuditView,
};

// 从 URL 读取初始视图（模块级，避免每次渲染重新创建）
function getInitialView(): ViewId {
  const params = new URLSearchParams(window.location.search);
  const view = params.get("view") as ViewId;
  if (view && view in VIEW_COMPONENTS) {
    return view;
  }
  return "dashboard";
}

export default function MainWindow() {
  const [activeView, setActiveView] = useState<ViewId>(getInitialView);
  const [unprocessedCount, setUnprocessedCount] = useState(0);
  const [developerMode, setDeveloperMode] = useState(false);

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
    getDeveloperMode().then(setDeveloperMode).catch((err) => {
      console.error("Failed to get developer mode:", err);
    });
    return () => {
      clearInterval(interval);
      unlisten.then((fn) => fn());
    };
  }, [refreshCount]);

  // 同步视图状态到 URL
  useEffect(() => {
    const url = new URL(window.location.href);
    url.searchParams.set("view", activeView);
    window.history.replaceState({}, "", url.toString());
  }, [activeView]);

  // 监听浏览器前进/后退
  useEffect(() => {
    const handlePopState = () => {
      const params = new URLSearchParams(window.location.search);
      const view = params.get("view") as ViewId;
      if (view && view in VIEW_COMPONENTS) {
        setActiveView(view);
      }
    };

    window.addEventListener("popstate", handlePopState);
    return () => window.removeEventListener("popstate", handlePopState);
  }, []);

  // 切换视图时刷新开发者模式（从设置页返回时立即生效）
  const handleViewChange = useCallback((view: ViewId) => {
    setActiveView(view);
    getDeveloperMode().then(setDeveloperMode).catch(() => {});
  }, []);

  // 命令面板操作处理
  const handleCommandAction = useCallback((action: string) => {
    console.log("Command action:", action);
    // TODO: 实现具体操作（新建任务、触发采集等）
  }, []);

  // T3.1 全局键盘快捷键
  useKeyboardShortcuts([
    { ...SHORTCUTS.VIEW_EVENTS, handler: () => handleViewChange("events") },
    { ...SHORTCUTS.VIEW_TASKS, handler: () => handleViewChange("tasks") },
    { ...SHORTCUTS.VIEW_TIMELINE, handler: () => handleViewChange("timeline") },
    { ...SHORTCUTS.VIEW_REPORTS, handler: () => handleViewChange("reports") },
    { ...SHORTCUTS.VIEW_SETTINGS, handler: () => handleViewChange("settings") },
  ]);

  const ActiveComponent = VIEW_COMPONENTS[activeView];

  return (
    <TooltipProvider>
      <div className="flex h-screen w-screen overflow-hidden bg-background text-foreground">
        <Sidebar
          activeView={activeView}
          onViewChange={handleViewChange}
          unprocessedCount={unprocessedCount}
          developerMode={developerMode}
        />
        <main className="flex-1 overflow-auto">
          <ActiveComponent />
        </main>
      </div>
      <CommandPalette onNavigate={handleViewChange} onAction={handleCommandAction} />
      <Toaster />
    </TooltipProvider>
  );
}
