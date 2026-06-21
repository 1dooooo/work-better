import { useState, useEffect, useCallback } from "react";
import Sidebar, { type ViewId } from "./Sidebar";
import DashboardView from "../views/DashboardView";
import EventsView from "../views/EventsView";
import TasksView from "../views/TasksView";
import TimelineView from "../views/TimelineView";
import ReportsView from "../views/ReportsView";
import SettingsView from "../views/SettingsView";
import AuditView from "../views/AuditView";
import CommandPalette from "../command-palette/CommandPalette";
import { getUnprocessedCount, onFeishuCollectComplete, getDeveloperMode, triggerFeishuCollect, createTask, getEvents, markEventProcessed } from "@/lib/tauri";
import { validateState } from "@/hooks/useStatePersistence";
import { toast } from "sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Toaster } from "@/components/ui/sonner";
import { useKeyboardShortcuts, SHORTCUTS } from "@/hooks/useKeyboardShortcuts";
import { useStatePersistence } from "@/hooks/useStatePersistence";

const VIEW_COMPONENTS: Record<ViewId, React.ComponentType> = {
  dashboard: DashboardView,
  events: EventsView,
  tasks: TasksView,
  timeline: TimelineView,
  reports: ReportsView,
  settings: SettingsView,
  audit: AuditView,
};

// 从 URL 或 localStorage 读取初始视图（模块级，避免每次渲染重新创建）
function getInitialView(): ViewId {
  // 优先从 URL 读取
  const params = new URLSearchParams(window.location.search);
  const view = params.get("view") as ViewId;
  if (view && view in VIEW_COMPONENTS) {
    return view;
  }

  // 其次从 localStorage 读取（带 schema 验证）
  try {
    const stored = localStorage.getItem("work-better-state");
    if (stored) {
      const validated = validateState(JSON.parse(stored));
      if (validated.lastView in VIEW_COMPONENTS) {
        return validated.lastView;
      }
    }
  } catch {
    // 损坏数据，静默回退
  }

  return "dashboard";
}

export default function MainWindow() {
  const { state: persistedState, updateState: updatePersistedState } = useStatePersistence();
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
    updatePersistedState("lastView", view);
    getDeveloperMode().then(setDeveloperMode).catch((err) => {
      console.error("Failed to refresh developer mode:", err);
    });
  }, [updatePersistedState]);

  // 命令面板操作处理
  const handleCommandAction = useCallback(async (action: string) => {
    switch (action) {
      case "new-task": {
        const title = prompt("请输入任务标题");
        if (title?.trim()) {
          try {
            await createTask(title.trim());
            toast.success("任务已创建");
          } catch (err) {
            console.error("Failed to create task:", err);
            toast.error("创建任务失败");
          }
        }
        break;
      }
      case "trigger-collect": {
        try {
          const count = await triggerFeishuCollect();
          toast.success(`采集完成，获取 ${count} 条事件`);
        } catch (err) {
          console.error("Failed to trigger collect:", err);
          toast.error("采集失败");
        }
        break;
      }
      case "mark-processed": {
        try {
          const events = await getEvents(50);
          const unprocessed = events.filter((e) => !e.processed);
          if (unprocessed.length === 0) {
            toast.info("没有待处理的事件");
            break;
          }
          const count = Math.min(unprocessed.length, 10);
          await Promise.all(
            unprocessed.slice(0, count).map((e) => markEventProcessed(e.id))
          );
          toast.success(`已标记 ${count} 条事件为已处理`);
          refreshCount();
        } catch (err) {
          console.error("Failed to mark events processed:", err);
          toast.error("标记事件失败");
        }
        break;
      }
      case "ai-generate-report": {
        handleViewChange("reports");
        toast.info("请在报告页面生成报告");
        break;
      }
      default:
        break;
    }
  }, [handleViewChange, refreshCount]);

  // T3.1 全局键盘快捷键
  useKeyboardShortcuts([
    { ...SHORTCUTS.VIEW_DASHBOARD, handler: () => handleViewChange("dashboard") },
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
          collapsed={persistedState.sidebarCollapsed}
          onCollapsedChange={(collapsed) => updatePersistedState("sidebarCollapsed", collapsed)}
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
