/**
 * useCommandData — 命令面板数据获取 hook
 *
 * 功能：
 * - 提供事件、任务数据
 * - 支持模糊搜索
 * - 实时更新
 */

import { useState, useEffect, useCallback, useMemo } from "react";
import { getEvents, listTasks, type Event, type TaskDto } from "@/lib/tauri";

interface CommandData {
  events: Event[];
  tasks: TaskDto[];
  loading: boolean;
  error: string | null;
}

export function useCommandData(searchQuery: string = "") {
  const [data, setData] = useState<CommandData>({
    events: [],
    tasks: [],
    loading: true,
    error: null,
  });

  // 加载数据
  const fetchData = useCallback(async () => {
    try {
      const [events, tasks] = await Promise.all([
        getEvents(50),
        listTasks(),
      ]);
      setData({
        events,
        tasks,
        loading: false,
        error: null,
      });
    } catch (err) {
      setData((prev) => ({
        ...prev,
        loading: false,
        error: err instanceof Error ? err.message : "加载失败",
      }));
    }
  }, []);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  // 模糊搜索
  const filteredEvents = useMemo(() => {
    if (!searchQuery) return data.events;
    const query = searchQuery.toLowerCase();
    return data.events.filter((event) => {
      const content =
        typeof event.content === "string"
          ? event.content
          : JSON.stringify(event.content);
      return (
        content.toLowerCase().includes(query) ||
        event.source.toLowerCase().includes(query) ||
        event.type.toLowerCase().includes(query)
      );
    });
  }, [data.events, searchQuery]);

  const filteredTasks = useMemo(() => {
    if (!searchQuery) return data.tasks;
    const query = searchQuery.toLowerCase();
    return data.tasks.filter(
      (task) =>
        task.title.toLowerCase().includes(query) ||
        task.status.toLowerCase().includes(query) ||
        task.priority.toLowerCase().includes(query)
    );
  }, [data.tasks, searchQuery]);

  return {
    events: filteredEvents,
    tasks: filteredTasks,
    loading: data.loading,
    error: data.error,
    refresh: fetchData,
  };
}
