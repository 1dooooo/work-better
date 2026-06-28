/**
 * useAutoRefresh — 自动刷新 Hook
 *
 * 提供定时自动刷新功能，支持配置间隔时间
 */

import { useEffect, useCallback, useRef } from "react";

// ─── 常量 ─────────────────────────────────────────────────────

/** 默认刷新间隔（毫秒） */
const DEFAULT_INTERVAL = 30_000;

// ─── Hook ─────────────────────────────────────────────────────

interface UseAutoRefreshOptions {
  /** 刷新回调函数 */
  onRefresh: () => Promise<void> | void;
  /** 刷新间隔（毫秒），默认 30000 */
  interval?: number;
  /** 是否启用自动刷新 */
  enabled?: boolean;
}

interface UseAutoRefreshReturn {
  /** 手动触发刷新 */
  triggerRefresh: () => Promise<void>;
}

export function useAutoRefresh({
  onRefresh,
  interval = DEFAULT_INTERVAL,
  enabled = true,
}: UseAutoRefreshOptions): UseAutoRefreshReturn {
  const callbackRef = useRef(onRefresh);
  callbackRef.current = onRefresh;

  const triggerRefresh = useCallback(async () => {
    await callbackRef.current();
  }, []);

  useEffect(() => {
    if (!enabled) return;

    // 初始刷新
    triggerRefresh();

    // 设置定时器
    const timer = setInterval(() => {
      triggerRefresh();
    }, interval);

    return () => clearInterval(timer);
  }, [enabled, interval, triggerRefresh]);

  return { triggerRefresh };
}
