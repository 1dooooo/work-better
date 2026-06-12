import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import TasksView from "./TasksView";

vi.mock("@/lib/tauri", () => ({
  listScheduledTasks: vi.fn().mockResolvedValue([
    { name: "每日站会", cron: "0 9 * * *", enabled: true },
  ]),
  pauseScheduler: vi.fn().mockResolvedValue(undefined),
  resumeScheduler: vi.fn().mockResolvedValue(undefined),
  isSchedulerPaused: vi.fn().mockResolvedValue(false),
  listTasks: vi.fn().mockResolvedValue([
    {
      id: "1",
      title: "整理周报",
      status: "Open",
      priority: "P1",
      source: "manual",
      dueDate: "",
      createdAt: "2026-01-01",
    },
    {
      id: "2",
      title: "Review PR #42",
      status: "InProgress",
      priority: "P2",
      source: "feishu",
      dueDate: "2026-01-02",
      createdAt: "2026-01-01",
    },
  ]),
  createTask: vi.fn().mockResolvedValue(undefined),
  updateTaskStatus: vi.fn().mockResolvedValue(undefined),
  getPendingTasks: vi.fn().mockResolvedValue([]),
  confirmPendingTask: vi.fn().mockResolvedValue(undefined),
  rejectPendingTask: vi.fn().mockResolvedValue(undefined),
}));

describe("TasksView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders without crashing (D1-04)", async () => {
    const { container } = render(<TasksView />);
    expect(container).toBeTruthy();
    await waitFor(() => {
      expect(screen.getByText("任务管理")).toBeInTheDocument();
    });
  });

  it("displays the view title", async () => {
    render(<TasksView />);
    await waitFor(() => {
      expect(screen.getByText("任务管理")).toBeInTheDocument();
    });
  });

  it("shows the task creation form", async () => {
    render(<TasksView />);
    await waitFor(() => {
      expect(screen.getByPlaceholderText("新任务标题...")).toBeInTheDocument();
      expect(screen.getByText("添加")).toBeInTheDocument();
    });
  });

  it("displays initial tasks", async () => {
    render(<TasksView />);
    await waitFor(() => {
      expect(screen.getByText("整理周报")).toBeInTheDocument();
      expect(screen.getByText("Review PR #42")).toBeInTheDocument();
    });
  });

  it("shows task count", async () => {
    render(<TasksView />);
    await waitFor(() => {
      expect(screen.getByText(/个任务/)).toBeInTheDocument();
    });
  });

  it("displays scheduled tasks section", async () => {
    render(<TasksView />);
    await waitFor(() => {
      expect(screen.getByText("定时任务")).toBeInTheDocument();
      expect(screen.getByText("每日站会")).toBeInTheDocument();
    });
  });
});
