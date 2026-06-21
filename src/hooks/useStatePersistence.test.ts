import { describe, it, expect, beforeEach, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useStatePersistence } from "./useStatePersistence";

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] || null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = value;
    }),
    removeItem: vi.fn((key: string) => {
      delete store[key];
    }),
    clear: vi.fn(() => {
      store = {};
    }),
  };
})();

Object.defineProperty(window, "localStorage", { value: localStorageMock });

describe("useStatePersistence", () => {
  beforeEach(() => {
    localStorageMock.clear();
    vi.clearAllMocks();
  });

  it("should return default state when localStorage is empty", () => {
    const { result } = renderHook(() => useStatePersistence());

    expect(result.current.state).toEqual({
      lastView: "dashboard",
      eventFilter: "all",
      taskSort: "priority",
      sidebarCollapsed: false,
    });
  });

  it("should load valid persisted state from localStorage", () => {
    const persistedState = {
      lastView: "tasks",
      eventFilter: "unread",
      taskSort: "created",
      sidebarCollapsed: true,
    };
    localStorageMock.setItem("work-better-state", JSON.stringify(persistedState));

    const { result } = renderHook(() => useStatePersistence());

    expect(result.current.state).toEqual(persistedState);
  });

  it("should reject invalid lastView and fall back to default", () => {
    const invalidState = { lastView: "invalid-view" };
    localStorageMock.setItem("work-better-state", JSON.stringify(invalidState));

    const { result } = renderHook(() => useStatePersistence());

    expect(result.current.state.lastView).toBe("dashboard");
  });

  it("should reject invalid eventFilter and fall back to default", () => {
    const invalidState = { eventFilter: "invalid" };
    localStorageMock.setItem("work-better-state", JSON.stringify(invalidState));

    const { result } = renderHook(() => useStatePersistence());

    expect(result.current.state.eventFilter).toBe("all");
  });

  it("should reject invalid taskSort and fall back to default", () => {
    const invalidState = { taskSort: "date" };
    localStorageMock.setItem("work-better-state", JSON.stringify(invalidState));

    const { result } = renderHook(() => useStatePersistence());

    expect(result.current.state.taskSort).toBe("priority");
  });

  it("should reject non-boolean sidebarCollapsed and fall back to default", () => {
    const invalidState = { sidebarCollapsed: "yes" };
    localStorageMock.setItem("work-better-state", JSON.stringify(invalidState));

    const { result } = renderHook(() => useStatePersistence());

    expect(result.current.state.sidebarCollapsed).toBe(false);
  });

  it("should handle corrupted JSON gracefully", () => {
    localStorageMock.setItem("work-better-state", "not-json");

    const { result } = renderHook(() => useStatePersistence());

    expect(result.current.state).toEqual({
      lastView: "dashboard",
      eventFilter: "all",
      taskSort: "priority",
      sidebarCollapsed: false,
    });
  });

  it("should handle null object gracefully", () => {
    localStorageMock.setItem("work-better-state", "null");

    const { result } = renderHook(() => useStatePersistence());

    expect(result.current.state).toEqual({
      lastView: "dashboard",
      eventFilter: "all",
      taskSort: "priority",
      sidebarCollapsed: false,
    });
  });

  it("should update single field via updateState", () => {
    const { result } = renderHook(() => useStatePersistence());

    act(() => {
      result.current.updateState("lastView", "tasks");
    });

    expect(result.current.state.lastView).toBe("tasks");
    expect(result.current.state.eventFilter).toBe("all");
  });

  it("should persist state changes to localStorage", () => {
    const { result } = renderHook(() => useStatePersistence());

    act(() => {
      result.current.updateState("lastView", "reports");
    });

    expect(localStorageMock.setItem).toHaveBeenCalledWith(
      "work-better-state",
      expect.stringContaining('"reports"')
    );
  });

  it("should reset state to defaults", () => {
    const { result } = renderHook(() => useStatePersistence());

    act(() => {
      result.current.updateState("lastView", "tasks");
      result.current.updateState("sidebarCollapsed", true);
    });

    act(() => {
      result.current.resetState();
    });

    expect(result.current.state).toEqual({
      lastView: "dashboard",
      eventFilter: "all",
      taskSort: "priority",
      sidebarCollapsed: false,
    });
  });

  it("should handle localStorage read errors gracefully", () => {
    localStorageMock.getItem.mockImplementation(() => {
      throw new Error("Storage error");
    });

    const { result } = renderHook(() => useStatePersistence());

    expect(result.current.state).toEqual({
      lastView: "dashboard",
      eventFilter: "all",
      taskSort: "priority",
      sidebarCollapsed: false,
    });
  });
});
