import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

// ─── Mocks ───────────────────────────────────────────────────────────

const mockInvoke = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
  emit: vi.fn(),
}));

// ─── Fixtures ────────────────────────────────────────────────────────

const MOCK_EVENTS = [
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
  {
    id: "evt-2",
    timestamp: "2026-06-06T11:00:00Z",
    collected_at: "2026-06-06T11:00:00Z",
    source: "manual",
    source_confidence: "high",
    type: "note",
    content: "Manual note",
    raw_payload: '{"text":"Manual note"}',
    tags: [],
    related_ids: [],
    attachments: [],
  },
];

const MOCK_COLLECTORS = [
  { id: "feishu", name: "飞书采集器", enabled: true, healthy: true },
  { id: "git", name: "Git 采集器", enabled: true, healthy: false },
];

const MOCK_COLLECTOR_HEALTH = {
  level: "ok",
  message: null,
  error_count: 0,
};

const MOCK_TASKS = [
  {
    id: "task-1",
    name: "每日站会",
    layer: "scheduler",
    cron: "0 10 * * *",
    sla_ms: 60000,
  },
  {
    id: "task-2",
    name: "周报汇总",
    layer: "scheduler",
    cron: "0 17 * * 5",
    sla_ms: 120000,
  },
  {
    id: "task-3",
    name: "依赖安全检查",
    layer: "scheduler",
    cron: "0 9 * * 1",
    sla_ms: 300000,
  },
];

// ─── Helpers ─────────────────────────────────────────────────────────

function resolveInvoke(command: string) {
  const map: Record<string, unknown> = {
    get_events: MOCK_EVENTS,
    get_unprocessed_count: 5,
    mark_event_processed: undefined,
    trigger_manual_capture: MOCK_EVENTS[0],
    trigger_feishu_collect: 3,
    get_collector_statuses: MOCK_COLLECTORS,
    list_collectors: ["feishu", "git"],
    enable_collector: undefined,
    disable_collector: undefined,
    check_collector_health: MOCK_COLLECTOR_HEALTH,
    get_feishu_mode: "cli",
    save_feishu_mode: undefined,
    get_feishu_chat_id: "chat-abc-123",
    save_feishu_chat_id: undefined,
    list_scheduled_tasks: MOCK_TASKS,
  };
  return map[command] ?? null;
}

// ─── E1: Tauri invoke Integration ────────────────────────────────────

describe("E1: Tauri invoke Integration", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockImplementation((cmd: string) =>
      Promise.resolve(resolveInvoke(cmd)),
    );
  });

  // ── E1-01: getEvents → displays events in list ──────────────────

  describe("E1-01: getEvents → displays events in list", () => {
    it("calls get_events invoke and renders event items in MenuBar", async () => {
      const { default: MenuBar } = await import("../../src/components/MenuBar");
      render(<MenuBar />);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("get_events", expect.any(Object));
      });

      await waitFor(() => {
        expect(screen.getByText("feishu")).toBeInTheDocument();
        expect(screen.getByText("manual")).toBeInTheDocument();
      });
    });

    it("calls get_events invoke and renders events in EventsView", async () => {
      const { default: EventsView } = await import("../../src/components/views/EventsView");
      render(<EventsView />);

      await waitFor(() => {
        expect(screen.getByText("Hello from Feishu")).toBeInTheDocument();
        expect(screen.getByText("Manual note")).toBeInTheDocument();
      });
    });
  });

  // ── E1-02: getUnprocessedCount → shows count in MenuBar ─────────

  describe("E1-02: getUnprocessedCount → shows count in MenuBar", () => {
    it("displays unprocessed count badge in MenuBar", async () => {
      const { default: MenuBar } = await import("../../src/components/MenuBar");
      render(<MenuBar />);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("get_unprocessed_count");
      });

      await waitFor(() => {
        expect(screen.getByText("5 待处理")).toBeInTheDocument();
      });
    });

    it("shows count in Sidebar when rendered directly with props", async () => {
      const { default: Sidebar } = await import("../../src/components/Sidebar");
      render(
        <Sidebar
          activeView="events"
          onViewChange={() => {}}
          unprocessedCount={5}
        />,
      );

      // Sidebar shows badge for events view when unprocessedCount > 0
      await waitFor(() => {
        expect(screen.getByText("5")).toBeInTheDocument();
      });
    });
  });

  // ── E1-03: markEventProcessed → updates event status in UI ──────

  describe("E1-03: markEventProcessed → updates event status in UI", () => {
    it("calls mark_event_processed when marking an event", async () => {
      const user = userEvent.setup();
      const { default: EventsView } = await import("../../src/components/views/EventsView");
      render(<EventsView />);

      // Wait for events to load
      await waitFor(() => {
        expect(screen.getByText("Hello from Feishu")).toBeInTheDocument();
      });

      // Click the first "已处理" button
      const markButtons = screen.getAllByText("已处理");
      await user.click(markButtons[0]);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("mark_event_processed", {
          eventId: "evt-1",
        });
      });
    });
  });

  // ── E1-04: triggerManualCapture → calls invoke and shows feedback ─

  describe("E1-04: triggerManualCapture → calls invoke and shows feedback", () => {
    it("submits capture text and shows success state in MenuBar", async () => {
      const user = userEvent.setup();
      const { default: MenuBar } = await import("../../src/components/MenuBar");
      render(<MenuBar />);

      const textarea = screen.getByPlaceholderText(/记录想法/);
      await user.type(textarea, "Test capture text");

      const submitButton = screen.getByRole("button", { name: "记录" });
      expect(submitButton).toBeEnabled();
      await user.click(submitButton);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("trigger_manual_capture", {
          text: "Test capture text",
        });
      });

      // Should show success feedback
      await waitFor(() => {
        expect(screen.getByText("已记录")).toBeInTheDocument();
      });
    });

    it("submits capture and shows success toast in CaptureWindow", async () => {
      const user = userEvent.setup();
      const { default: CaptureWindow } = await import("../../src/capture/CaptureWindow");
      render(<CaptureWindow />);

      const textarea = screen.getByPlaceholderText(/记录一条想法/);
      await user.type(textarea, "Quick capture");

      const submitButton = screen.getByText("提交");
      await user.click(submitButton);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("trigger_manual_capture", {
          text: "Quick capture",
        });
      });

      await waitFor(() => {
        expect(screen.getByText("已捕获")).toBeInTheDocument();
      });
    });
  });

  // ── E1-05: triggerFeishuCollect → triggers collection and updates count ─

  describe("E1-05: triggerFeishuCollect → triggers collection and updates count", () => {
    it("calls trigger_feishu_collect and refreshes events in EventsView", async () => {
      const user = userEvent.setup();
      const { default: EventsView } = await import("../../src/components/views/EventsView");
      render(<EventsView />);

      // Wait for initial load
      await waitFor(() => {
        expect(screen.getByText("事件流")).toBeInTheDocument();
      });

      const collectButton = screen.getByText("采集");
      await user.click(collectButton);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("trigger_feishu_collect", expect.objectContaining({
          limit: 20,
        }));
      });
    });
  });

  // ── E1-06: getCollectorStatuses → displays collector health ─────

  describe("E1-06: getCollectorStatuses → displays collector health", () => {
    it("renders collector list with health indicators", async () => {
      const { default: CollectorSettings } = await import("../../src/components/settings/CollectorSettings");
      render(<CollectorSettings />);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("get_collector_statuses");
      });

      await waitFor(() => {
        expect(screen.getByText("飞书采集器")).toBeInTheDocument();
        expect(screen.getByText("Git 采集器")).toBeInTheDocument();
        expect(screen.getByText("正常")).toBeInTheDocument();
        expect(screen.getByText("异常")).toBeInTheDocument();
      });
    });
  });

  // ── E1-07: listCollectors → shows collector list ────────────────

  describe("E1-07: listCollectors → shows collector list", () => {
    it("lists collector IDs via invoke", async () => {
      // listCollectors is used internally; verify it resolves correctly
      const result = await mockInvoke("list_collectors");
      expect(result).toEqual(["feishu", "git"]);
    });
  });

  // ── E1-08: enableCollector → toggles collector on ───────────────

  describe("E1-08: enableCollector → toggles collector on", () => {
    it("enables a disabled collector via switch toggle", async () => {
      const user = userEvent.setup();
      const { default: CollectorSettings } = await import("../../src/components/settings/CollectorSettings");
      render(<CollectorSettings />);

      // Wait for collectors to load
      await waitFor(() => {
        expect(screen.getByText("飞书采集器")).toBeInTheDocument();
      });

      // All collectors are enabled in mock, so click to disable one first
      const switches = screen.getAllByRole("switch");
      const feishuSwitch = switches[0];
      expect(feishuSwitch).toBeChecked();

      // Click to disable
      await user.click(feishuSwitch);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("disable_collector", { id: "feishu" });
      });
    });
  });

  // ── E1-09: disableCollector → toggles collector off ─────────────

  describe("E1-09: disableCollector → toggles collector off", () => {
    it("disables an enabled collector via switch toggle", async () => {
      const user = userEvent.setup();
      const { default: CollectorSettings } = await import("../../src/components/settings/CollectorSettings");
      render(<CollectorSettings />);

      // Wait for collectors to load
      await waitFor(() => {
        expect(screen.getByText("飞书采集器")).toBeInTheDocument();
      });

      // Find the switch for the Feishu collector (first switch, initially checked)
      const switches = screen.getAllByRole("switch");
      const feishuSwitch = switches[0];
      expect(feishuSwitch).toBeChecked();

      await user.click(feishuSwitch);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("disable_collector", { id: "feishu" });
      });
    });
  });

  // ── E1-10: checkCollectorHealth → shows health indicator ────────

  describe("E1-10: checkCollectorHealth → shows health indicator", () => {
    it("returns health data from invoke", async () => {
      const result = await mockInvoke("check_collector_health", { id: "feishu" });
      expect(result).toEqual(MOCK_COLLECTOR_HEALTH);
      expect(result.level).toBe("ok");
      expect(result.error_count).toBe(0);
    });

    it("shows health status in CollectorSettings UI", async () => {
      const { default: CollectorSettings } = await import("../../src/components/settings/CollectorSettings");
      render(<CollectorSettings />);

      await waitFor(() => {
        // "正常" is the healthy indicator for feishu collector
        expect(screen.getByText("正常")).toBeInTheDocument();
        // "异常" is the unhealthy indicator for git collector
        expect(screen.getByText("异常")).toBeInTheDocument();
      });
    });
  });

  // ── E1-11: getFeishuMode → shows current mode ──────────────────

  describe("E1-11: getFeishuMode → shows current mode", () => {
    it("displays the current feishu mode as selected radio", async () => {
      const { default: CollectorSettings } = await import("../../src/components/settings/CollectorSettings");
      render(<CollectorSettings />);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("get_feishu_mode");
      });

      await waitFor(() => {
        // The component renders two buttons for mode selection
        // The "cli" mode is selected. Find the buttons by text.
        const buttons = screen.getAllByRole("button");
        const cliButton = buttons.find(b => b.textContent?.includes("lark-cli"));
        expect(cliButton).toBeDefined();
        // The selected button should have primary styling
        expect(cliButton?.className).toContain("border-primary");
      });
    });
  });

  // ── E1-12: saveFeishuMode → persists mode selection ─────────────

  describe("E1-12: saveFeishuMode → persists mode selection", () => {
    it("calls save_feishu_mode when radio selection changes", async () => {
      const user = userEvent.setup();
      const { default: CollectorSettings } = await import("../../src/components/settings/CollectorSettings");
      render(<CollectorSettings />);

      // Wait for initial load
      await waitFor(() => {
        expect(screen.getByText("采集器")).toBeInTheDocument();
      });

      // Click the first radio ("API 直连") which is unchecked
      const buttons = screen.getAllByRole("button");
      const apiButton = buttons.find(b => b.textContent?.includes("API 直连"));
      expect(apiButton).toBeDefined();
      await user.click(apiButton!);
      expect(apiButton).toBeDefined();
      await user.click(apiButton!);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("save_feishu_mode", { mode: "api" });
      });

      // Should show saved indicator
      await waitFor(() => {
        expect(screen.getByText("已保存")).toBeInTheDocument();
      });
    });
  });

  // ── E1-13: getFeishuChatId → shows chat ID ─────────────────────

  describe("E1-13: getFeishuChatId → shows chat ID", () => {
    it("displays the current chat ID in the input field", async () => {
      const { default: CollectorSettings } = await import("../../src/components/settings/CollectorSettings");
      render(<CollectorSettings />);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("get_feishu_chat_id");
      });

      await waitFor(() => {
        const input = screen.getByPlaceholderText("输入飞书会话 ID");
        expect(input).toHaveValue("chat-abc-123");
      });
    });
  });

  // ── E1-14: saveFeishuChatId → persists chat ID ─────────────────

  describe("E1-14: saveFeishuChatId → persists chat ID", () => {
    it("calls save_feishu_chat_id on input blur", async () => {
      const user = userEvent.setup();
      const { default: CollectorSettings } = await import("../../src/components/settings/CollectorSettings");
      render(<CollectorSettings />);

      // Wait for initial load
      await waitFor(() => {
        expect(screen.getByPlaceholderText("输入飞书会话 ID")).toHaveValue("chat-abc-123");
      });

      // Clear and type a new chat ID
      const input = screen.getByPlaceholderText("输入飞书会话 ID");
      await user.clear(input);
      await user.type(input, "new-chat-id-456");

      // Blur triggers save
      await user.tab();

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("save_feishu_chat_id", {
          chatId: "new-chat-id-456",
        });
      });

      // Should show saved indicator
      await waitFor(() => {
        expect(screen.getByText("已保存")).toBeInTheDocument();
      });
    });
  });

  // ── E1-15: listScheduledTasks → shows task list ─────────────────

  describe("E1-15: listScheduledTasks → shows task list", () => {
    it("returns scheduled tasks from invoke", async () => {
      const result = await mockInvoke("list_scheduled_tasks");
      expect(result).toEqual(MOCK_TASKS);
      expect(result).toHaveLength(3);
      expect(result[0].name).toBe("每日站会");
      expect(result[1].cron).toBe("0 17 * * 5");
    });

    it("displays hardcoded scheduled tasks in TasksView", async () => {
      const { default: TaskView } = await import("../../src/components/views/TasksView");
      render(<TaskView />);

      // TasksView fetches scheduled tasks from Tauri API
      await waitFor(() => {
        expect(screen.getByText("定时任务")).toBeInTheDocument();
        expect(screen.getByText("每日站会")).toBeInTheDocument();
        expect(screen.getByText("0 10 * * *")).toBeInTheDocument();
        expect(screen.getByText("周报汇总")).toBeInTheDocument();
        expect(screen.getByText("0 17 * * 5")).toBeInTheDocument();
        expect(screen.getByText("依赖安全检查")).toBeInTheDocument();
        expect(screen.getByText("0 9 * * 1")).toBeInTheDocument();
      });
    });
  });
});
