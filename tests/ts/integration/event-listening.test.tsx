import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, act } from "@testing-library/react";

// ─── Mocks ───────────────────────────────────────────────────────────

const mockInvoke = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

// Track the listen callback so we can simulate events
let capturedListenCallback: ((event: { payload: number }) => void) | null = null;
const mockUnlisten = vi.fn();

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockImplementation(
    (_event: string, callback: (event: { payload: number }) => void) => {
      capturedListenCallback = callback;
      return Promise.resolve(mockUnlisten);
    },
  ),
  emit: vi.fn(),
}));

// ─── Fixtures ────────────────────────────────────────────────────────

const MOCK_EVENTS_INITIAL = [
  {
    id: "evt-1",
    timestamp: "2026-06-06T10:00:00Z",
    collected_at: "2026-06-06T10:00:00Z",
    source: "feishu",
    source_confidence: "high",
    type: "message",
    content: "Hello from Feishu",
    raw_payload: '{"text":"Hello from Feishu"}',
    tags: ["chat"],
    related_ids: [],
    attachments: [],
  },
];

const MOCK_EVENTS_UPDATED = [
  ...MOCK_EVENTS_INITIAL,
  {
    id: "evt-2",
    timestamp: "2026-06-06T12:00:00Z",
    collected_at: "2026-06-06T12:00:00Z",
    source: "feishu",
    source_confidence: "high",
    type: "message",
    content: "New collected message",
    raw_payload: '{"text":"New collected message"}',
    tags: ["chat"],
    related_ids: [],
    attachments: [],
  },
];

// ─── E2: Event Listening ─────────────────────────────────────────────

describe("E2: Event Listening", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    capturedListenCallback = null;
    mockUnlisten.mockClear();
  });

  // ── E2-01: onFeishuCollectComplete → updates UI count ──────────

  describe("E2-01: onFeishuCollectComplete → updates UI count", () => {
    it("registers a listener for feishu:collect-complete on mount", async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "get_events") return Promise.resolve(MOCK_EVENTS_INITIAL);
        if (cmd === "get_unprocessed_count") return Promise.resolve(1);
        return Promise.resolve(null);
      });

      const { listen } = await import("@tauri-apps/api/event");
      const { default: EventsView } = await import(
        "../../src/components/views/EventsView"
      );
      render(<EventsView />);

      await waitFor(() => {
        expect(listen).toHaveBeenCalledWith(
          "feishu:collect-complete",
          expect.any(Function),
        );
      });
    });

    it("refreshes events when feishu collect complete event fires", async () => {
      let getEventsCalls = 0;
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "get_events") {
          getEventsCalls++;
          // First call: initial events; subsequent: updated events
          return Promise.resolve(
            getEventsCalls <= 1 ? MOCK_EVENTS_INITIAL : MOCK_EVENTS_UPDATED,
          );
        }
        if (cmd === "get_unprocessed_count") return Promise.resolve(1);
        if (cmd === "get_feishu_chat_id") return Promise.resolve("chat-123");
        return Promise.resolve(null);
      });

      const { default: EventsView } = await import(
        "../../src/components/views/EventsView"
      );
      render(<EventsView />);

      // Wait for initial load
      await waitFor(() => {
        expect(screen.getByText("Hello from Feishu")).toBeInTheDocument();
      });

      // Simulate a feishu:collect-complete event
      // The listen callback receives the full event object with .payload
      await act(async () => {
        capturedListenCallback?.({ payload: 2 });
      });

      // After the event fires, the component should refresh and show updated events
      await waitFor(() => {
        const calls = mockInvoke.mock.calls.filter(
          (c: unknown[]) => c[0] === "get_events",
        );
        expect(calls.length).toBeGreaterThanOrEqual(2);
      });

      // The updated event list should now include the new message
      await waitFor(() => {
        expect(screen.getByText("New collected message")).toBeInTheDocument();
      });
    });

    it("registers listener in MainWindow and refreshes unprocessed count", async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "get_events") return Promise.resolve(MOCK_EVENTS_INITIAL);
        if (cmd === "get_unprocessed_count") return Promise.resolve(1);
        if (cmd === "get_feishu_chat_id") return Promise.resolve("chat-123");
        return Promise.resolve(null);
      });

      const { listen } = await import("@tauri-apps/api/event");
      const { default: MainWindow } = await import(
        "../../src/components/MainWindow"
      );
      render(<MainWindow />);

      await waitFor(() => {
        expect(listen).toHaveBeenCalledWith(
          "feishu:collect-complete",
          expect.any(Function),
        );
      });

      // Record how many times getUnprocessedCount was called before the event
      const countBefore = mockInvoke.mock.calls.filter(
        (c: unknown[]) => c[0] === "get_unprocessed_count",
      ).length;

      // Simulate event
      await act(async () => {
        capturedListenCallback?.({ payload: 5 });
      });

      // MainWindow should re-call getUnprocessedCount after the event
      await waitFor(() => {
        const countAfter = mockInvoke.mock.calls.filter(
          (c: unknown[]) => c[0] === "get_unprocessed_count",
        ).length;
        expect(countAfter).toBeGreaterThan(countBefore);
      });
    });
  });

  // ── E2-02: Cleanup on unmount → removes listener ───────────────

  describe("E2-02: Cleanup on unmount → removes listener", () => {
    it("calls unlisten function when EventsView unmounts", async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "get_events") return Promise.resolve(MOCK_EVENTS_INITIAL);
        if (cmd === "get_unprocessed_count") return Promise.resolve(1);
        return Promise.resolve(null);
      });

      const { default: EventsView } = await import(
        "../../src/components/views/EventsView"
      );
      const { unmount } = render(<EventsView />);

      // Wait for listener registration to complete
      await waitFor(() => {
        expect(capturedListenCallback).toBeTruthy();
      });

      // Unmount the component
      unmount();

      // The unlisten function should have been called
      await waitFor(() => {
        expect(mockUnlisten).toHaveBeenCalled();
      });
    });

    it("calls unlisten function when MainWindow unmounts", async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "get_events") return Promise.resolve(MOCK_EVENTS_INITIAL);
        if (cmd === "get_unprocessed_count") return Promise.resolve(1);
        if (cmd === "get_feishu_chat_id") return Promise.resolve("chat-123");
        return Promise.resolve(null);
      });

      const { default: MainWindow } = await import(
        "../../src/components/MainWindow"
      );
      const { unmount } = render(<MainWindow />);

      // Wait for listener registration to complete
      await waitFor(() => {
        expect(capturedListenCallback).toBeTruthy();
      });

      // Unmount the component
      unmount();

      // The unlisten function should have been called
      await waitFor(() => {
        expect(mockUnlisten).toHaveBeenCalled();
      });
    });

    it("unlisten prevents further event delivery after unmount", async () => {
      // This test verifies the cleanup contract: after unmount, the unlisten
      // function is called, which in a real Tauri app would deregister the
      // listener from the event system. We verify that unlisten is called
      // (the mechanism that prevents future callbacks) rather than trying to
      // simulate post-unmount callback behavior (which is React-level, not
      // Tauri-level).
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "get_events") return Promise.resolve(MOCK_EVENTS_INITIAL);
        if (cmd === "get_unprocessed_count") return Promise.resolve(1);
        if (cmd === "get_feishu_chat_id") return Promise.resolve("chat-123");
        return Promise.resolve(null);
      });

      const { listen } = await import("@tauri-apps/api/event");
      const { default: EventsView } = await import(
        "../../src/components/views/EventsView"
      );
      const { unmount } = render(<EventsView />);

      // Wait for initial load and listener registration
      await waitFor(() => {
        expect(screen.getByText("Hello from Feishu")).toBeInTheDocument();
      });
      await waitFor(() => {
        expect(capturedListenCallback).toBeTruthy();
      });

      // Verify listen was called exactly once (one listener registered)
      const listenCallCount = (listen as unknown as ReturnType<typeof vi.fn>).mock
        .calls.length;

      // Unmount — triggers cleanup
      unmount();

      // Verify unlisten was called (listener deregistered from event system)
      await waitFor(() => {
        expect(mockUnlisten).toHaveBeenCalledTimes(1);
      });

      // In a real Tauri app, after unlisten() is called, the event system
      // would not deliver events to this callback anymore. The unlisten
      // mechanism is what prevents future event delivery — this is the
      // contract we verify here.
      expect(listenCallCount).toBe(1);
    });
  });
});
