/**
 * useWindowResize — 窗口尺寸调整 Hook
 *
 * 恢复原始实现：使用 node.scrollHeight + ResizeObserver
 */

import { useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize } from "@tauri-apps/api/dpi";

// ─── 类型定义 ─────────────────────────────────────────────────

interface UseWindowResizeOptions {
  /** 窗口宽度 */
  width?: number;
}

interface UseWindowResizeReturn {
  /** 根元素 ref 回调 */
  rootRef: (node: HTMLDivElement | null) => void;
}

// ─── Hook ─────────────────────────────────────────────────────

export function useWindowResize({
  width = 360,
}: UseWindowResizeOptions = {}): UseWindowResizeReturn {
  const rootRef = useCallback(
    (node: HTMLDivElement | null) => {
      if (!node) return;

      // 仅在 Tauri 环境下调整窗口大小
      if (!(window as any).__TAURI_INTERNALS__) return;
      if (typeof ResizeObserver === "undefined") return;

      const resizeWindow = () => {
        const height = node.scrollHeight;
        const win = getCurrentWindow();
        win.setSize(new LogicalSize(width, height)).catch(() => {});
      };

      // 初始调整
      resizeWindow();

      // 监听内容变化
      const observer = new ResizeObserver(resizeWindow);
      observer.observe(node);

      return () => observer.disconnect();
    },
    [width],
  );

  return { rootRef };
}
