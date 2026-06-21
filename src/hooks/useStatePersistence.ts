/**
 * useStatePersistence — 视图状态持久化 hook
 *
 * 功能：
 * - 使用 localStorage 存储用户偏好
 * - 支持视图状态、过滤条件、排序方式持久化
 * - 应用启动时恢复状态
 * - 带 schema 验证，防止损坏数据导致崩溃
 */

import { useState, useEffect, useCallback } from "react";
import type { ViewId } from "@/components/layout/Sidebar";

type EventFilter = "all" | "unread" | "processed";
type TaskSort = "priority" | "created" | "due";

interface PersistedState {
  lastView: ViewId;
  eventFilter: EventFilter;
  taskSort: TaskSort;
  sidebarCollapsed: boolean;
}

const STORAGE_KEY = "work-better-state";

const VALID_VIEWS: ViewId[] = ["dashboard", "events", "tasks", "timeline", "reports", "settings", "audit"];
const VALID_EVENT_FILTERS: EventFilter[] = ["all", "unread", "processed"];
const VALID_TASK_SORTS: TaskSort[] = ["priority", "created", "due"];

const DEFAULT_STATE: PersistedState = {
  lastView: "dashboard",
  eventFilter: "all",
  taskSort: "priority",
  sidebarCollapsed: false,
};

// 验证并修复加载的状态（exported for getInitialView）
export function validateState(raw: unknown): PersistedState {
  if (typeof raw !== "object" || raw === null) return DEFAULT_STATE;
  const obj = raw as Record<string, unknown>;

  return {
    lastView: VALID_VIEWS.includes(obj.lastView as ViewId) ? (obj.lastView as ViewId) : DEFAULT_STATE.lastView,
    eventFilter: VALID_EVENT_FILTERS.includes(obj.eventFilter as EventFilter) ? (obj.eventFilter as EventFilter) : DEFAULT_STATE.eventFilter,
    taskSort: VALID_TASK_SORTS.includes(obj.taskSort as TaskSort) ? (obj.taskSort as TaskSort) : DEFAULT_STATE.taskSort,
    sidebarCollapsed: typeof obj.sidebarCollapsed === "boolean" ? obj.sidebarCollapsed : DEFAULT_STATE.sidebarCollapsed,
  };
}

// 从 localStorage 加载状态
function loadState(): PersistedState {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed: unknown = JSON.parse(stored);
      return validateState(parsed);
    }
  } catch {
    // 损坏数据，静默回退到默认值
  }
  return DEFAULT_STATE;
}

// 保存状态到 localStorage
function saveState(state: PersistedState): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch {
    // localStorage 不可用（隐私模式等），静默忽略
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
