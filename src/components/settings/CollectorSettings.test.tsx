import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import CollectorSettings from "./CollectorSettings";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(null),
}));

vi.mock("../../lib/tauri", () => ({
  getCollectorStatuses: vi.fn().mockResolvedValue([
    { id: "feishu", name: "飞书采集器", enabled: true, healthy: true },
  ]),
  getFeishuMode: vi.fn().mockResolvedValue("cli"),
  getFeishuChatId: vi.fn().mockResolvedValue("chat-123"),
  enableCollector: vi.fn().mockResolvedValue(undefined),
  disableCollector: vi.fn().mockResolvedValue(undefined),
  saveFeishuMode: vi.fn().mockResolvedValue(undefined),
  saveFeishuChatId: vi.fn().mockResolvedValue(undefined),
}));

describe("CollectorSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders without crashing (D1-10)", async () => {
    const { container } = render(<CollectorSettings />);
    expect(container).toBeTruthy();
    await waitFor(() => {
      expect(screen.getByText("采集器")).toBeInTheDocument();
    });
  });

  it("displays the section title", async () => {
    render(<CollectorSettings />);
    await waitFor(() => {
      expect(screen.getByText("采集器")).toBeInTheDocument();
    });
  });

  it("shows collector list", async () => {
    render(<CollectorSettings />);
    await waitFor(() => {
      expect(screen.getByText("飞书采集器")).toBeInTheDocument();
    });
  });

  it("shows feishu mode radio buttons", async () => {
    render(<CollectorSettings />);
    await waitFor(() => {
      expect(screen.getByText("API 直连")).toBeInTheDocument();
      expect(screen.getByText("lark-cli")).toBeInTheDocument();
    });
  });

  it("shows chat id input", async () => {
    render(<CollectorSettings />);
    await waitFor(() => {
      expect(screen.getByPlaceholderText("输入飞书会话 ID")).toBeInTheDocument();
    });
  });

  it("shows 未启用 badge when collector is disabled", async () => {
    const { getCollectorStatuses } = await import("../../lib/tauri");
    vi.mocked(getCollectorStatuses).mockResolvedValueOnce([
      { id: "feishu", name: "飞书", enabled: false, healthy: false },
      { id: "manual", name: "手动输入", enabled: false, healthy: false },
    ]);
    render(<CollectorSettings />);
    await waitFor(() => {
      expect(screen.getAllByText("未启用")).toHaveLength(2);
      expect(screen.queryByText("异常")).not.toBeInTheDocument();
    });
  });

  it("shows 异常 badge when enabled but unhealthy", async () => {
    const { getCollectorStatuses } = await import("../../lib/tauri");
    vi.mocked(getCollectorStatuses).mockResolvedValueOnce([
      { id: "feishu", name: "飞书", enabled: true, healthy: false },
    ]);
    render(<CollectorSettings />);
    await waitFor(() => {
      expect(screen.getByText("异常")).toBeInTheDocument();
      expect(screen.queryByText("未启用")).not.toBeInTheDocument();
    });
  });

  it("shows 正常 badge when enabled and healthy", async () => {
    const { getCollectorStatuses } = await import("../../lib/tauri");
    vi.mocked(getCollectorStatuses).mockResolvedValueOnce([
      { id: "feishu", name: "飞书", enabled: true, healthy: true },
    ]);
    render(<CollectorSettings />);
    await waitFor(() => {
      expect(screen.getByText("正常")).toBeInTheDocument();
      expect(screen.queryByText("未启用")).not.toBeInTheDocument();
      expect(screen.queryByText("异常")).not.toBeInTheDocument();
    });
  });
});
