import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import MenuBar from "./MenuBar";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(null),
}));

vi.mock("../lib/tauri", () => ({
  getEvents: vi.fn().mockResolvedValue([]),
  getUnprocessedCount: vi.fn().mockResolvedValue(0),
  triggerManualCapture: vi.fn().mockResolvedValue({
    id: "1",
    timestamp: new Date().toISOString(),
    collected_at: new Date().toISOString(),
    source: "manual",
    source_confidence: "high",
    type: "note",
    content: "test",
    raw_payload: "{}",
    tags: [],
    related_ids: [],
    attachments: [],
  }),
}));

describe("MenuBar", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders without crashing (D1-01)", () => {
    const { container } = render(<MenuBar />);
    expect(container).toBeTruthy();
  });

  it("displays the app title", () => {
    render(<MenuBar />);
    expect(screen.getByText("Work Better")).toBeInTheDocument();
  });

  it("shows the capture textarea", () => {
    render(<MenuBar />);
    expect(screen.getByPlaceholderText(/记录想法/)).toBeInTheDocument();
  });

  it("shows empty state when no events", async () => {
    render(<MenuBar />);
    await waitFor(() => {
      expect(screen.getByText("暂无事件")).toBeInTheDocument();
    });
  });

  it("disables submit when input is empty", () => {
    render(<MenuBar />);
    const button = screen.getByRole("button", { name: "记录" });
    expect(button).toBeDisabled();
  });
});
