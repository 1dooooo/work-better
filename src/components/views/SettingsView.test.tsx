import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import SettingsView from "./SettingsView";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockImplementation((cmd: string) => {
    switch (cmd) {
      case "get_model_config":
        return Promise.resolve({
          api_endpoint: "https://api.openai.com/v1",
          api_key: "sk-test",
          token_budget: 4096,
        });
      case "get_storage_config":
        return Promise.resolve({
          vault_path: "~/Documents/Obsidian",
          db_path: "~/.work-better/data.db",
        });
      default:
        return Promise.resolve(null);
    }
  }),
}));

vi.mock("../../lib/tauri", () => ({
  getCollectorStatuses: vi.fn().mockResolvedValue([]),
  getFeishuMode: vi.fn().mockResolvedValue("cli"),
  getFeishuChatId: vi.fn().mockResolvedValue(""),
  enableCollector: vi.fn().mockResolvedValue(undefined),
  disableCollector: vi.fn().mockResolvedValue(undefined),
  saveFeishuMode: vi.fn().mockResolvedValue(undefined),
  saveFeishuChatId: vi.fn().mockResolvedValue(undefined),
}));

describe("SettingsView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders without crashing (D1-07)", () => {
    const { container } = render(<SettingsView />);
    expect(container).toBeTruthy();
  });

  it("displays the view title", () => {
    render(<SettingsView />);
    expect(screen.getByText("设置")).toBeInTheDocument();
  });

  it("renders all settings sections", async () => {
    render(<SettingsView />);
    await waitFor(() => {
      expect(screen.getByText("模型配置")).toBeInTheDocument();
    });
    expect(screen.getByText("采集器配置")).toBeInTheDocument();
    expect(screen.getByText("存储配置")).toBeInTheDocument();
    expect(screen.getByText("快捷键配置")).toBeInTheDocument();
    expect(screen.getByText("保鲜规则配置")).toBeInTheDocument();
    expect(screen.getByText("报告配置")).toBeInTheDocument();
  });
});
