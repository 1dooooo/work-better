import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { useCommandData } from "./useCommandData";

vi.mock("@/lib/tauri", () => ({
  getEvents: vi.fn(),
  listTasks: vi.fn(),
}));

import { getEvents, listTasks } from "@/lib/tauri";

const mockGetEvents = vi.mocked(getEvents);
const mockListTasks = vi.mocked(listTasks);

const MOCK_EVENTS = [
  { id: "1", content: "飞书消息内容", source: "feishu", type: "message", timestamp: "", collected_at: "", source_confidence: "", raw_payload: "", tags: [], related_ids: [], attachments: [], processed: false },
  { id: "2", content: "GitHub Issue", source: "github", type: "issue", timestamp: "", collected_at: "", source_confidence: "", raw_payload: "", tags: [], related_ids: [], attachments: [], processed: true },
];

const MOCK_TASKS = [
  { id: "t1", title: "完成报告", description: null, status: "pending", priority: "high", source: "manual", due_date: null, created_at: "", tags: [] },
  { id: "t2", title: "代码审查", description: null, status: "done", priority: "low", source: "manual", due_date: null, created_at: "", tags: [] },
];

describe("useCommandData", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockGetEvents.mockResolvedValue(MOCK_EVENTS);
    mockListTasks.mockResolvedValue(MOCK_TASKS);
  });

  it("should load events and tasks on mount", async () => {
    const { result } = renderHook(() => useCommandData());

    expect(result.current.loading).toBe(true);

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.events).toHaveLength(2);
    expect(result.current.tasks).toHaveLength(2);
    expect(result.current.error).toBeNull();
  });

  it("should filter events by content", async () => {
    const { result } = renderHook(() => useCommandData("飞书"));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    await waitFor(() => {
      expect(result.current.events).toHaveLength(1);
    }, { timeout: 300 });
    expect(result.current.events[0].id).toBe("1");
  });

  it("should filter tasks by title", async () => {
    const { result } = renderHook(() => useCommandData("报告"));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    await waitFor(() => {
      expect(result.current.tasks).toHaveLength(1);
    }, { timeout: 300 });
    expect(result.current.tasks[0].id).toBe("t1");
  });

  it("should return all items when search is empty", async () => {
    const { result } = renderHook(() => useCommandData(""));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.events).toHaveLength(2);
    expect(result.current.tasks).toHaveLength(2);
  });

  it("should set error on API failure", async () => {
    mockGetEvents.mockRejectedValue(new Error("API error"));

    const { result } = renderHook(() => useCommandData());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.error).toBe("API error");
    expect(result.current.events).toHaveLength(0);
  });

  it("should refresh data when refresh is called", async () => {
    const { result } = renderHook(() => useCommandData());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    mockGetEvents.mockResolvedValue([MOCK_EVENTS[0]]);
    await result.current.refresh();

    await waitFor(() => {
      expect(result.current.events).toHaveLength(1);
    });
  });
});
