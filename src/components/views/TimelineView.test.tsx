import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import TimelineView from "./TimelineView";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(null),
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
      content: "Timeline event",
      raw_payload: "{}",
      tags: [],
      related_ids: [],
      attachments: [],
    },
  ]),
}));

describe("TimelineView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders without crashing (D1-05)", async () => {
    const { container } = render(<TimelineView />);
    expect(container).toBeTruthy();
    await waitFor(() => {
      expect(screen.getByText("时间线")).toBeInTheDocument();
    });
  });

  it("displays the view title", async () => {
    render(<TimelineView />);
    await waitFor(() => {
      expect(screen.getByText("时间线")).toBeInTheDocument();
    });
  });

  it("shows the refresh button", async () => {
    render(<TimelineView />);
    await waitFor(() => {
      expect(screen.getByText("刷新")).toBeInTheDocument();
    });
  });
});
