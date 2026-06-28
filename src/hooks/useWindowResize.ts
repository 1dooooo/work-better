/**
 * useWindowResize — 窗口尺寸调整 Hook
 *
 * 使用 ResizeObserver + 延迟测量确保内容完全渲染后再调整窗口大小
 * 包含最小/最大高度保护
 *
 * 日志：通过invoke输出到Tauri终端，实现日志统一观测
 */

import { useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize } from "@tauri-apps/api/dpi";
import { createLogger } from "@/lib/logger";

// ─── 常量 ─────────────────────────────────────────────────────

/** 最小窗口高度 */
const MIN_HEIGHT = 100;

/** 最大窗口高度 */
const MAX_HEIGHT = 500;

/** 初始调整延迟（毫秒） */
const INITIAL_DELAY = 300;

/** 日志器 */
const logger = createLogger("WindowResize");

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
        // 使用 offsetHeight 获取实际渲染高度（包含 border）
        const rawHeight = node.offsetHeight;
        const height = Math.min(Math.max(rawHeight, MIN_HEIGHT), MAX_HEIGHT);

        logger.info(`height: ${rawHeight} → ${height}`);

        const win = getCurrentWindow();
        win.setSize(new LogicalSize(width, height)).catch(() => {});
      };

      // 延迟初始调整，确保内容完全渲染
      const initialTimer = setTimeout(() => {
        resizeWindow();
      }, INITIAL_DELAY);

      // 监听内容变化
      const observer = new ResizeObserver(() => {
        resizeWindow();
      });
      observer.observe(node);

      return () => {
        clearTimeout(initialTimer);
        observer.disconnect();
      };
    },
    [width],
  );

  return { rootRef };
}
