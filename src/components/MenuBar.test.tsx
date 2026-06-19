import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import MenuBar from "./MenuBar";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(null),
}));

vi.mock("../lib/tauri", () => ({
  getEvents: vi.fn().mockResolvedValue([]),
  getUnprocessedCount: vi.fn().mockResolvedValue(0),
  getPendingNotifications: vi.fn().mockResolvedValue([]),
  markNotificationRead: vi.fn().mockResolvedValue(undefined),
  getPendingTasks: vi.fn().mockResolvedValue([]),
  getSystemStatus: vi.fn().mockResolvedValue({
    collectors_total: 3,
    collectors_healthy: 2,
    scheduler_running: true,
    unprocessed_count: 0,
  }),
  showCaptureWindow: vi.fn().mockResolvedValue(undefined),
  triggerBatchProcess: vi.fn().mockResolvedValue({
    total: 0,
    success: 0,
    failed: 0,
    skipped: 0,
    details: [],
  }),
}));

describe("MenuBar", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders without crashing", () => {
    const { container } = render(<MenuBar />);
    expect(container).toBeTruthy();
  });

  it("displays the app title", () => {
    render(<MenuBar />);
    expect(screen.getByText("Work Better")).toBeInTheDocument();
  });

  it("shows empty state when no events", async () => {
    render(<MenuBar />);
    await waitFor(() => {
      expect(screen.getByText("暂无事件")).toBeInTheDocument();
    });
  });

  it("shows the recent events section header", async () => {
    render(<MenuBar />);
    await waitFor(() => {
      expect(screen.getByText("最近事件")).toBeInTheDocument();
    });
  });

  it("shows quick action buttons", async () => {
    render(<MenuBar />);
    await waitFor(() => {
      expect(screen.getByText("主窗口")).toBeInTheDocument();
      expect(screen.getByText("速记")).toBeInTheDocument();
      expect(screen.getByText("截图")).toBeInTheDocument();
      expect(screen.getByText("处理")).toBeInTheDocument();
    });
  });

  it("shows system status in footer", async () => {
    render(<MenuBar />);
    await waitFor(() => {
      expect(screen.getByText(/采集器/)).toBeInTheDocument();
    });
  });

  it("shows scheduler status when running", async () => {
    render(<MenuBar />);
    await waitFor(() => {
      expect(screen.getByText("调度")).toBeInTheDocument();
    });
  });

  it("displays events when loaded", async () => {
    const { getEvents } = await import("../lib/tauri");
    vi.mocked(getEvents).mockResolvedValueOnce([
      {
        id: "1",
        timestamp: new Date().toISOString(),
        collected_at: new Date().toISOString(),
        source: "feishu",
        source_confidence: "high",
        type: "message",
        content: "测试消息内容",
        raw_payload: "{}",
        tags: [],
        related_ids: [],
        attachments: [],
        processed: false,
      },
    ]);

    render(<MenuBar />);
    await waitFor(() => {
      expect(screen.getByText("测试消息内容")).toBeInTheDocument();
    });
  });

  it("displays notifications when present", async () => {
    const { getPendingNotifications } = await import("../lib/tauri");
    vi.mocked(getPendingNotifications).mockResolvedValueOnce([
      {
        id: "notif-1",
        title: "任务确认",
        body: "请确认飞书任务",
        kind: "Confirm",
        action_url: null,
        read: false,
        created_at: new Date().toISOString(),
      },
    ]);

    render(<MenuBar />);
    await waitFor(() => {
      expect(screen.getByText("待确认")).toBeInTheDocument();
      expect(screen.getByText("任务确认")).toBeInTheDocument();
    });
  });

  it("shows unprocessed count badge when count > 0", async () => {
    const { getUnprocessedCount } = await import("../lib/tauri");
    vi.mocked(getUnprocessedCount).mockResolvedValueOnce(5);

    render(<MenuBar />);
    await waitFor(() => {
      expect(screen.getByText("5 待处理")).toBeInTheDocument();
    });
  });

  it("displays pending tasks when present", async () => {
    const { getPendingTasks } = await import("../lib/tauri");
    vi.mocked(getPendingTasks).mockResolvedValueOnce([
      {
        id: "task-1",
        title: "完成报告",
        description: null,
        source: "feishu",
        priority: "high",
        origin_text: "完成报告",
        created_at: new Date().toISOString(),
      },
    ]);

    render(<MenuBar />);
    await waitFor(() => {
      expect(screen.getByText("今日待办")).toBeInTheDocument();
      expect(screen.getByText("完成报告")).toBeInTheDocument();
    });
  });
});
