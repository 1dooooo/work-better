import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import CollectorSettings from "./CollectorSettings";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(null),
}));

vi.mock("../../lib/tauri", () => ({
  getCollectorGroups: vi.fn().mockResolvedValue([
    {
      id: "feishu",
      name: "飞书",
      enabled: true,
      collectors: [
        { id: "feishu", name: "消息", enabled: true, health_level: "healthy", health_message: null },
      ],
    },
  ]),
  getFeishuMode: vi.fn().mockResolvedValue("cli"),
  getFeishuChatId: vi.fn().mockResolvedValue("chat-123"),
  enableCollector: vi.fn().mockResolvedValue(undefined),
  disableCollector: vi.fn().mockResolvedValue(undefined),
  enableCollectorGroup: vi.fn().mockResolvedValue(undefined),
  disableCollectorGroup: vi.fn().mockResolvedValue(undefined),
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

  it("shows collector group", async () => {
    render(<CollectorSettings />);
    await waitFor(() => {
      expect(screen.getByText("飞书")).toBeInTheDocument();
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
    const { getCollectorGroups } = await import("../../lib/tauri");
    vi.mocked(getCollectorGroups).mockResolvedValueOnce([
      {
        id: "feishu",
        name: "飞书",
        enabled: true,
        collectors: [
          { id: "feishu", name: "消息", enabled: false, health_level: "unhealthy", health_message: null },
        ],
      },
    ]);
    render(<CollectorSettings />);
    await waitFor(() => {
      expect(screen.getByText("飞书")).toBeInTheDocument();
    });
    fireEvent.click(screen.getByText("飞书"));
    await waitFor(() => {
      expect(screen.getByText("未启用")).toBeInTheDocument();
    });
  });

  it("shows 异常 badge when enabled and unhealthy", async () => {
    const { getCollectorGroups } = await import("../../lib/tauri");
    vi.mocked(getCollectorGroups).mockResolvedValueOnce([
      {
        id: "feishu",
        name: "飞书",
        enabled: true,
        collectors: [
          { id: "feishu", name: "消息", enabled: true, health_level: "unhealthy", health_message: "lark-cli 不可用" },
        ],
      },
    ]);
    render(<CollectorSettings />);
    await waitFor(() => {
      expect(screen.getByText("飞书")).toBeInTheDocument();
    });
    fireEvent.click(screen.getByText("飞书"));
    await waitFor(() => {
      expect(screen.getByText("异常")).toBeInTheDocument();
    });
  });

  it("shows 正常 badge when enabled and healthy", async () => {
    const { getCollectorGroups } = await import("../../lib/tauri");
    vi.mocked(getCollectorGroups).mockResolvedValueOnce([
      {
        id: "feishu",
        name: "飞书",
        enabled: true,
        collectors: [
          { id: "feishu", name: "消息", enabled: true, health_level: "healthy", health_message: null },
        ],
      },
    ]);
    render(<CollectorSettings />);
    await waitFor(() => {
      expect(screen.getByText("飞书")).toBeInTheDocument();
    });
    fireEvent.click(screen.getByText("飞书"));
    await waitFor(() => {
      expect(screen.getByText("正常")).toBeInTheDocument();
    });
  });

  it("shows 降级 badge when enabled and degraded", async () => {
    const { getCollectorGroups } = await import("../../lib/tauri");
    vi.mocked(getCollectorGroups).mockResolvedValueOnce([
      {
        id: "feishu",
        name: "飞书",
        enabled: true,
        collectors: [
          { id: "feishu", name: "消息", enabled: true, health_level: "degraded", health_message: "lark-cli 未安装" },
        ],
      },
    ]);
    render(<CollectorSettings />);
    await waitFor(() => {
      expect(screen.getByText("飞书")).toBeInTheDocument();
    });
    fireEvent.click(screen.getByText("飞书"));
    await waitFor(() => {
      expect(screen.getByText("降级")).toBeInTheDocument();
    });
  });
});
