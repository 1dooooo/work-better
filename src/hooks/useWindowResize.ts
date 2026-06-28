/**
 * useWindowResize — 窗口尺寸调整 Hook
 *
 * 使用 ResizeObserver + 延迟测量确保内容完全渲染后再调整窗口大小
 * 包含最小高度保护
 *
 * 调试：添加了详细日志来诊断窗口大小调整问题
 */

import { useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize } from "@tauri-apps/api/dpi";

// ─── 常量 ─────────────────────────────────────────────────────

/** 最小窗口高度 */
const MIN_HEIGHT = 100;

/** 最大窗口高度 */
const MAX_HEIGHT = 500;

/** 初始调整延迟（毫秒） */
const INITIAL_DELAY = 300;

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

        console.log("[useWindowResize] resizeWindow called:", {
          rawHeight,
          minHeight: MIN_HEIGHT,
          maxHeight: MAX_HEIGHT,
          finalHeight: height,
          width,
          nodeScrollHeight: node.scrollHeight,
          nodeClientHeight: node.clientHeight,
        });

        const win = getCurrentWindow();
        win.setSize(new LogicalSize(width, height))
          .then(() => {
            console.log("[useWindowResize] setSize success:", { width, height });
          })
          .catch((err) => {
            console.error("[useWindowResize] setSize failed:", err);
          });
      };

      // 延迟初始调整，确保内容完全渲染
      const initialTimer = setTimeout(() => {
        console.log("[useWindowResize] initial timer fired");
        resizeWindow();
      }, INITIAL_DELAY);

      // 监听内容变化
      const observer = new ResizeObserver((entries) => {
        console.log("[useWindowResize] ResizeObserver triggered:", {
          entriesCount: entries.length,
          entry: entries[0]
            ? {
                target: entries[0].target.tagName,
                contentRect: entries[0].contentRect,
              }
            : null,
        });
        resizeWindow();
      });
      observer.observe(node);

      console.log("[useWindowResize] hook initialized:", {
        width,
        minHeight: MIN_HEIGHT,
        maxHeight: MAX_HEIGHT,
        initialDelay: INITIAL_DELAY,
        isTauri: !!(window as any).__TAURI_INTERNALS__,
      });

      return () => {
        clearTimeout(initialTimer);
        observer.disconnect();
      };
    },
    [width],
  );

  return { rootRef };
}
