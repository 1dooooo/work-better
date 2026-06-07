import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import EventsView from "./EventsView";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(null),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
  emit: vi.fn(),
}));

vi.mock("../../lib/tauri", () => ({
  getEvents: vi.fn().mockResolvedValue([
    {
      id: "1",
      timestamp: "2026-06-06T10:00:00Z",
      collected_at: "2026-06-06T10:00:00Z",
      source: "manual",
      source_confidence: "high",
      type: "note",
      content: "Test event content",
      raw_payload: "{}",
      tags: ["test"],
      related_ids: [],
      attachments: [],
    },
  ]),
  markEventProcessed: vi.fn().mockResolvedValue(undefined),
  triggerFeishuCollect: vi.fn().mockResolvedValue(1),
  getFeishuChatId: vi.fn().mockResolvedValue("chat-123"),
  onFeishuCollectComplete: vi.fn().mockReturnValue(Promise.resolve(() => {})),
}));

describe("EventsView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders without crashing (D1-03)", async () => {
    const { container } = render(<EventsView />);
    expect(container).toBeTruthy();
    await waitFor(() => {
      expect(screen.getByText("事件流")).toBeInTheDocument();
    });
  });

  it("displays the view title", async () => {
    render(<EventsView />);
    await waitFor(() => {
      expect(screen.getByText("事件流")).toBeInTheDocument();
    });
  });

  it("shows the collect button", () => {
    render(<EventsView />);
    expect(screen.getByText("采集飞书")).toBeInTheDocument();
  });

  it("shows refresh button", () => {
    render(<EventsView />);
    expect(screen.getByText("刷新")).toBeInTheDocument();
  });
});
