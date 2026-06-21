/**
 * useStatePersistence — 视图状态持久化 hook
 *
 * 功能：
 * - 使用 localStorage 存储用户偏好
 * - 支持视图状态、过滤条件、排序方式持久化
 * - 应用启动时恢复状态
 */

import { useState, useEffect, useCallback } from "react";

interface PersistedState {
  lastView: string;
  eventFilter: string;
  taskSort: string;
  sidebarCollapsed: boolean;
}

const STORAGE_KEY = "work-better-state";

const DEFAULT_STATE: PersistedState = {
  lastView: "dashboard",
  eventFilter: "all",
  taskSort: "priority",
  sidebarCollapsed: false,
};

// 从 localStorage 加载状态
function loadState(): PersistedState {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      return { ...DEFAULT_STATE, ...parsed };
    }
  } catch (err) {
    console.warn("Failed to load persisted state:", err);
  }
  return DEFAULT_STATE;
}

// 保存状态到 localStorage
function saveState(state: PersistedState): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch (err) {
    console.warn("Failed to persist state:", err);
  }
}

export function useStatePersistence() {
  const [state, setState] = useState<PersistedState>(loadState);

  // 状态变更时自动保存
  useEffect(() => {
    saveState(state);
  }, [state]);

  // 更新单个字段
  const updateState = useCallback(
    <K extends keyof PersistedState>(key: K, value: PersistedState[K]) => {
      setState((prev) => ({ ...prev, [key]: value }));
    },
    []
  );

  // 重置为默认值
  const resetState = useCallback(() => {
    setState(DEFAULT_STATE);
  }, []);

  return {
    state,
    updateState,
    resetState,
  };
}
