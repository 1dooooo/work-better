/**
 * MenuBar — macOS 菜单栏面板（组合根）
 *
 * 职责：组合 hooks 和子组件，协调数据流和用户交互
 * 设计：~100 行的薄层，不含业务逻辑
 */

import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { showCaptureWindow, triggerBatchProcess } from "../lib/tauri";
import { cn } from "@/lib/utils";

import { useMenuBarData } from "@/hooks/useMenuBarData";
import { useAutoRefresh } from "@/hooks/useAutoRefresh";
import { useWindowResize } from "@/hooks/useWindowResize";

import { MenuBarHeader } from "./menubar/MenuBarHeader";
import { MenuBarContent } from "./menubar/MenuBarContent";
import { MenuBarActions } from "./menubar/MenuBarActions";

// ─── 常量 ─────────────────────────────────────────────────────

const REFRESH_INTERVAL = 30_000;

// ─── 组件 ─────────────────────────────────────────────────────

export default function MenuBar() {
  const [processing, setProcessing] = useState(false);

  const {
    unprocessedCount,
    events,
    notifications,
    pendingTasks,
    systemStatus,
    loading,
    refresh,
    dismissNotification,
    handleNotificationClick,
  } = useMenuBarData();

  const { triggerRefresh } = useAutoRefresh({
    onRefresh: refresh,
    interval: REFRESH_INTERVAL,
  });

  const { rootRef } = useWindowResize();

  // ─── 事件处理 ─────────────────────────────────────────────

  const handleTriggerProcess = useCallback(async () => {
    if (processing) return;
    setProcessing(true);
    try {
      await triggerBatchProcess();
      await triggerRefresh();
    } catch (err) {
      console.error("[MenuBar] trigger process failed:", err);
    } finally {
      setProcessing(false);
    }
  }, [processing, triggerRefresh]);

  const handleOpenMainWindow = useCallback(() => {
    invoke("show_main_window").catch(() => {});
  }, []);

  const handleOpenCapture = useCallback(() => {
    showCaptureWindow().catch(() => {});
  }, []);

  const handleTakeScreenshot = useCallback(() => {
    invoke("take_screenshot").catch(() => {});
  }, []);

  // ─── 渲染 ─────────────────────────────────────────────────

  return (
    <div
      ref={rootRef}
      className={cn(
        "flex flex-col select-none",
        "bg-background/95 backdrop-blur-xl",
        "text-foreground font-sans",
        "rounded-xl border border-border",
      )}
    >
      <MenuBarHeader
        systemStatus={systemStatus}
        unprocessedCount={unprocessedCount}
      />

      <MenuBarContent
        events={events}
        pendingTasks={pendingTasks}
        notifications={notifications}
        loading={loading}
        onRefresh={triggerRefresh}
        onNotificationClick={handleNotificationClick}
        onDismissNotification={dismissNotification}
      />

      <MenuBarActions
        processing={processing}
        onOpenMainWindow={handleOpenMainWindow}
        onOpenCapture={handleOpenCapture}
        onTakeScreenshot={handleTakeScreenshot}
        onTriggerProcess={handleTriggerProcess}
      />
    </div>
  );
}
