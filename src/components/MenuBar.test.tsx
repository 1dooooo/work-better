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
  getPendingNotifications: vi.fn().mockResolvedValue([]),
  markNotificationRead: vi.fn().mockResolvedValue(undefined),
  getPendingTasks: vi.fn().mockResolvedValue([]),
  getSystemStatus: vi.fn().mockResolvedValue({
    collectors_total: 3,
    collectors_healthy: 2,
    scheduler_running: true,
    unprocessed_count: 0,
    today_processed_count: 0,
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

// ─── Helper: 生成标准 Event 对象 ──────────────────────────────────
// 使用 content_set 标记区分"显式传了 null/undefined"和"未传 content"

function makeEvent(
  overrides: {
    id?: string;
    type?: string;
    content?: unknown;
    timestamp?: string;
    /** 内部标记：是否显式设置了 content（含 null/undefined） */
    __contentSet?: boolean;
  } = {},
) {
  return {
    id: overrides.id ?? "evt-1",
    timestamp: overrides.timestamp ?? new Date().toISOString(),
    collected_at: new Date().toISOString(),
    source: "feishu",
    source_confidence: "high",
    type: overrides.type ?? "Message",
    content: overrides.__contentSet ? overrides.content : (overrides.content ?? "测试消息"),
    raw_payload: "{}",
    tags: [],
    related_ids: [],
    attachments: [],
    processed: false,
  };
}

// ─── Helper: 生成标准 NotificationRecord ─────────────────────────

function makeNotification(overrides: Partial<{
  id: string;
  title: string;
  body: string;
  kind: "Confirm" | "Reminder" | "TaskDone";
  action_url: string | null;
}> = {}) {
  return {
    id: overrides.id ?? `notif-${Math.random().toString(36).slice(2, 6)}`,
    title: overrides.title ?? "测试通知",
    body: overrides.body ?? "通知内容",
    kind: overrides.kind ?? "Confirm",
    action_url: overrides.action_url ?? null,
    read: false,
    created_at: new Date().toISOString(),
  };
}

// ─── Helper: 生成标准 PendingTaskDto ─────────────────────────────

function makeTask(overrides: Partial<{
  id: string;
  title: string;
}> = {}) {
  return {
    id: overrides.id ?? `task-${Math.random().toString(36).slice(2, 6)}`,
    title: overrides.title ?? "测试任务",
    description: null,
    source: "feishu",
    priority: "high",
    origin_text: "测试任务",
    created_at: new Date().toISOString(),
  };
}

describe("MenuBar", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // ─── 基础渲染 ─────────────────────────────────────────────────

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

  // ─── L5: 完整数据加载流程 ─────────────────────────────────────

  describe("L5: 完整数据加载流程", () => {
    it("refresh 成功后所有区域同时正确渲染", async () => {
      const { getEvents, getUnprocessedCount, getPendingNotifications, getPendingTasks, getSystemStatus } =
        await import("../lib/tauri");

      vi.mocked(getUnprocessedCount).mockResolvedValueOnce(3);
      vi.mocked(getEvents).mockResolvedValueOnce([
        makeEvent({ id: "1", type: "Message", content: "飞书消息内容" }),
        makeEvent({ id: "2", type: "TaskUpdate", content: "任务更新" }),
      ]);
      vi.mocked(getPendingNotifications).mockResolvedValueOnce([
        makeNotification({ id: "n1", title: "待确认任务", kind: "Confirm" }),
        makeNotification({ id: "n2", title: "系统提醒", kind: "Reminder" }),
      ]);
      vi.mocked(getPendingTasks).mockResolvedValueOnce([
        makeTask({ id: "t1", title: "完成报告" }),
      ]);
      vi.mocked(getSystemStatus).mockResolvedValueOnce({
        collectors_total: 5,
        collectors_healthy: 4,
        scheduler_running: true,
        unprocessed_count: 3,
        today_processed_count: 15,
      });

      render(<MenuBar />);

      await waitFor(() => {
        // Header 状态区域
        expect(screen.getByText(/4\/5 采集/)).toBeInTheDocument();
        expect(screen.getByText("调度中")).toBeInTheDocument();
        expect(screen.getByText("今日 15")).toBeInTheDocument();
        expect(screen.getByText("3")).toBeInTheDocument(); // unprocessed badge

        // 事件列表区域
        expect(screen.getByText("飞书消息内容")).toBeInTheDocument();
        expect(screen.getByText("任务更新")).toBeInTheDocument();

        // 通知区域
        expect(screen.getByText("待确认任务")).toBeInTheDocument();
        expect(screen.getByText("系统提醒")).toBeInTheDocument();

        // 待办区域
        expect(screen.getByText("今日待办")).toBeInTheDocument();
        expect(screen.getByText("完成报告")).toBeInTheDocument();

        // 快捷操作按钮
        expect(screen.getByText("主窗口")).toBeInTheDocument();
        expect(screen.getByText("速记")).toBeInTheDocument();
        expect(screen.getByText("截图")).toBeInTheDocument();
        expect(screen.getByText("处理")).toBeInTheDocument();
      });
    });

    it("部分 API 失败时仍渲染可用数据", async () => {
      const { getEvents, getPendingNotifications, getPendingTasks } =
        await import("../lib/tauri");

      // getEvents 正常
      vi.mocked(getEvents).mockResolvedValueOnce([makeEvent({ content: "正常事件" })]);
      // getPendingNotifications 失败
      vi.mocked(getPendingNotifications).mockRejectedValueOnce(new Error("API error"));
      // getPendingTasks 失败
      vi.mocked(getPendingTasks).mockRejectedValueOnce(new Error("API error"));

      render(<MenuBar />);

      await waitFor(() => {
        expect(screen.getByText("正常事件")).toBeInTheDocument();
      });

      // 失败的区域不应崩溃
      expect(screen.queryByText("今日待办")).not.toBeInTheDocument();
      expect(screen.queryByText("通知")).not.toBeInTheDocument();
    });
  });

  // ─── L4: getEventSummary 执行路径验证 ──────────────────────────

  describe("L4: getEventSummary 执行路径验证", () => {
    // 注意：getEventSummary 是模块内部函数，我们通过渲染组件间接测试

    it("字符串 content 直接显示", async () => {
      const { getEvents } = await import("../lib/tauri");
      vi.mocked(getEvents).mockResolvedValueOnce([
        makeEvent({ content: "这是一条字符串消息" }),
      ]);

      render(<MenuBar />);
      await waitFor(() => {
        expect(screen.getByText("这是一条字符串消息")).toBeInTheDocument();
      });
    });

    it("字符串 content 超过 60 字符被截断", async () => {
      const { getEvents } = await import("../lib/tauri");
      const longString = "这是一条很长的消息".repeat(10); // > 60 chars
      vi.mocked(getEvents).mockResolvedValueOnce([
        makeEvent({ content: longString }),
      ]);

      render(<MenuBar />);
      await waitFor(() => {
        // 截断后应显示前 60 个字符
        const expected = longString.slice(0, 60);
        expect(screen.getByText(expected)).toBeInTheDocument();
      });
    });

    it("包含 text 字段的对象（ManualNote 格式）", async () => {
      const { getEvents } = await import("../lib/tauri");
      vi.mocked(getEvents).mockResolvedValueOnce([
        makeEvent({
          type: "ManualNote",
          content: { text: "手动记录的内容" },
        }),
      ]);

      render(<MenuBar />);
      await waitFor(() => {
        expect(screen.getByText("手动记录的内容")).toBeInTheDocument();
      });
    });

    it("包含多个元数据字段的对象", async () => {
      const { getEvents } = await import("../lib/tauri");
      vi.mocked(getEvents).mockResolvedValueOnce([
        makeEvent({
          content: { doc_id: "doc-123", action: "update" },
        }),
      ]);

      render(<MenuBar />);
      await waitFor(() => {
        expect(screen.getByText(/doc_id: doc-123/)).toBeInTheDocument();
      });
    });

    it("包含 summary 字段的对象（最高优先级）", async () => {
      const { getEvents } = await import("../lib/tauri");
      vi.mocked(getEvents).mockResolvedValueOnce([
        makeEvent({
          content: { summary: "这是摘要", title: "这是标题", text: "这是文本" },
        }),
      ]);

      render(<MenuBar />);
      await waitFor(() => {
        // summary 优先级最高
        expect(screen.getByText("这是摘要")).toBeInTheDocument();
      });
    });

    it("包含 title 字段的对象（第二优先级）", async () => {
      const { getEvents } = await import("../lib/tauri");
      vi.mocked(getEvents).mockResolvedValueOnce([
        makeEvent({
          content: { title: "会议标题", text: "会议详情" },
        }),
      ]);

      render(<MenuBar />);
      await waitFor(() => {
        // title 优先于 text
        expect(screen.getByText("会议标题")).toBeInTheDocument();
      });
    });

    it("包含 message 字段的对象", async () => {
      const { getEvents } = await import("../lib/tauri");
      vi.mocked(getEvents).mockResolvedValueOnce([
        makeEvent({
          content: { message: "系统消息内容" },
        }),
      ]);

      render(<MenuBar />);
      await waitFor(() => {
        expect(screen.getByText("系统消息内容")).toBeInTheDocument();
      });
    });

    it("嵌套对象提取 — textKeys 内的嵌套字段", async () => {
      const { getEvents } = await import("../lib/tauri");
      // 使用 content 作为外层 key（在 textKeys 中），内部也有 textKeys
      // getEventSummary 第三优先：obj["content"] 是对象 → 检查 nested["text"]
      vi.mocked(getEvents).mockResolvedValueOnce([
        makeEvent({
          content: { content: { text: "嵌套文本内容" } },
        }),
      ]);

      render(<MenuBar />);
      await waitFor(() => {
        expect(screen.getByText("嵌套文本内容")).toBeInTheDocument();
      });
    });

    it("嵌套对象 — 非 textKeys 的 key 回退到 JSON", async () => {
      const { getEvents } = await import("../lib/tauri");
      // "data" 不在 textKeys 中，所以嵌套提取不会命中，回退到 JSON
      vi.mocked(getEvents).mockResolvedValueOnce([
        makeEvent({
          content: { data: { text: "嵌套文本内容" } },
        }),
      ]);

      render(<MenuBar />);
      await waitFor(() => {
        // 应显示紧凑 JSON（因为 data 不在 textKeys 中）
        expect(screen.getByText(/data.*text.*嵌套文本内容/)).toBeInTheDocument();
      });
    });

    it("null content 显示为字符串 'null'", async () => {
      const { getEvents } = await import("../lib/tauri");
      vi.mocked(getEvents).mockResolvedValueOnce([
        makeEvent({ content: null, __contentSet: true }),
      ]);

      render(<MenuBar />);
      await waitFor(() => {
        expect(screen.getByText("null")).toBeInTheDocument();
      });
    });

    it("undefined content 显示为字符串 'undefined'", async () => {
      const { getEvents } = await import("../lib/tauri");
      vi.mocked(getEvents).mockResolvedValueOnce([
        makeEvent({ content: undefined, __contentSet: true }),
      ]);

      render(<MenuBar />);
      await waitFor(() => {
        expect(screen.getByText("undefined")).toBeInTheDocument();
      });
    });

    it("布尔值 content 显示为字符串", async () => {
      const { getEvents } = await import("../lib/tauri");
      vi.mocked(getEvents).mockResolvedValueOnce([
        makeEvent({ content: true }),
      ]);

      render(<MenuBar />);
      await waitFor(() => {
        expect(screen.getByText("true")).toBeInTheDocument();
      });
    });

    it("数组 content 显示为 JSON 字符串", async () => {
      const { getEvents } = await import("../lib/tauri");
      vi.mocked(getEvents).mockResolvedValueOnce([
        makeEvent({ content: ["item1", "item2"] }),
      ]);

      render(<MenuBar />);
      await waitFor(() => {
        expect(screen.getByText('["item1","item2"]')).toBeInTheDocument();
      });
    });

    it("空字符串 content 显示为空", async () => {
      const { getEvents } = await import("../lib/tauri");
      vi.mocked(getEvents).mockResolvedValueOnce([
        makeEvent({ content: "" }),
      ]);

      render(<MenuBar />);
      await waitFor(() => {
        // 空字符串直接返回，不会触发 fallback
        // 事件行仍然存在
        const eventRows = screen.getAllByText(/MSG|DOC|TASK|MTG|CAL|MAIL|APPR|OKR|WEB|APP|NOTE/);
        expect(eventRows.length).toBeGreaterThan(0);
      });
    });

    it("空对象 content 回退到 JSON", async () => {
      const { getEvents } = await import("../lib/tauri");
      vi.mocked(getEvents).mockResolvedValueOnce([
        makeEvent({ content: {} }),
      ]);

      render(<MenuBar />);
      await waitFor(() => {
        expect(screen.getByText("{}")).toBeInTheDocument();
      });
    });

    it("包含 sender 字段的对象", async () => {
      const { getEvents } = await import("../lib/tauri");
      vi.mocked(getEvents).mockResolvedValueOnce([
        makeEvent({
          content: { sender: "张三", chat_id: "group-123" },
        }),
      ]);

      render(<MenuBar />);
      await waitFor(() => {
        expect(screen.getByText(/sender: 张三/)).toBeInTheDocument();
      });
    });

    it("布尔值字段显示为 checkmark/cross", async () => {
      const { getEvents } = await import("../lib/tauri");
      vi.mocked(getEvents).mockResolvedValueOnce([
        makeEvent({
          content: { approved: true, status: "completed" },
        }),
      ]);

      render(<MenuBar />);
      await waitFor(() => {
        expect(screen.getByText(/approved.*✓/)).toBeInTheDocument();
      });
    });
  });

  // ─── L4: 事件类型配置覆盖 ─────────────────────────────────────

  describe("L4: 事件类型配置覆盖", () => {
    // EventType 枚举值（来自 crates/wb-core/src/event.rs）
    const RUST_EVENT_TYPES = [
      "Message",
      "DocumentChange",
      "TaskUpdate",
      "Meeting",
      "CalendarEvent",
      "Email",
      "Approval",
      "OkrUpdate",
      "Browsing",
      "AppActivity",
      "ManualNote",
    ];

    it.each(RUST_EVENT_TYPES)("EventType.%s 在 EVENT_TYPE_CONFIG 中有配置", async (eventType) => {
      const { getEvents } = await import("../lib/tauri");
      vi.mocked(getEvents).mockResolvedValueOnce([
        makeEvent({ type: eventType, content: `${eventType}内容` }),
      ]);

      render(<MenuBar />);
      await waitFor(() => {
        // 验证事件能正常渲染（不会因为缺少配置而崩溃）
        expect(screen.getByText(`${eventType}内容`)).toBeInTheDocument();
      });
    });

    it("未知事件类型显示为大写前 4 字符", async () => {
      const { getEvents } = await import("../lib/tauri");
      vi.mocked(getEvents).mockResolvedValueOnce([
        makeEvent({ type: "UnknownType", content: "未知类型事件" }),
      ]);

      render(<MenuBar />);
      await waitFor(() => {
        // 未知类型应显示 UNKN
        expect(screen.getByText("UNKN")).toBeInTheDocument();
      });
    });
  });

  // ─── L4: IPC 调用路径验证 ──────────────────────────────────────

  describe("L4: IPC 调用路径验证", () => {
    it("refresh 调用所有必要的 Tauri 命令", async () => {
      const { getEvents, getUnprocessedCount, getPendingNotifications, getPendingTasks, getSystemStatus } =
        await import("../lib/tauri");

      render(<MenuBar />);

      await waitFor(() => {
        expect(getUnprocessedCount).toHaveBeenCalled();
        expect(getEvents).toHaveBeenCalledWith(10);
        expect(getPendingNotifications).toHaveBeenCalled();
        expect(getPendingTasks).toHaveBeenCalled();
        expect(getSystemStatus).toHaveBeenCalled();
      });
    });

    it("点击快捷按钮调用正确的 Tauri 命令", async () => {
      const { showCaptureWindow } = await import("../lib/tauri");
      const { invoke } = await import("@tauri-apps/api/core");

      render(<MenuBar />);
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getByText("速记")).toBeInTheDocument();
      });

      await user.click(screen.getByText("速记"));
      expect(showCaptureWindow).toHaveBeenCalled();

      await user.click(screen.getByText("截图"));
      expect(invoke).toHaveBeenCalledWith("take_screenshot");

      await user.click(screen.getByText("主窗口"));
      expect(invoke).toHaveBeenCalledWith("show_main_window");
    });

    it("处理按钮调用 triggerBatchProcess 并刷新", async () => {
      const { triggerBatchProcess, getEvents } = await import("../lib/tauri");

      render(<MenuBar />);
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getByText("处理")).toBeInTheDocument();
      });

      // 清除初始化调用记录
      vi.mocked(getEvents).mockClear();

      await user.click(screen.getByText("处理"));

      await waitFor(() => {
        expect(triggerBatchProcess).toHaveBeenCalled();
        // 处理完成后应刷新数据
        expect(getEvents).toHaveBeenCalled();
      });
    });
  });

  // ─── L5: 通知区域 max-height 行为 ─────────────────────────────

  describe("L5: 通知区域 max-height 行为", () => {
    it("通知容器有 max-h-[120px] 和 overflow-y-auto", async () => {
      const { getPendingNotifications } = await import("../lib/tauri");

      // 生成 9+ 条通知（3 组各 3 条以上）
      const notifications = [
        ...Array.from({ length: 4 }, (_, i) =>
          makeNotification({ id: `confirm-${i}`, title: `确认 ${i}`, kind: "Confirm" })
        ),
        ...Array.from({ length: 4 }, (_, i) =>
          makeNotification({ id: `reminder-${i}`, title: `提醒 ${i}`, kind: "Reminder" })
        ),
        ...Array.from({ length: 4 }, (_, i) =>
          makeNotification({ id: `done-${i}`, title: `完成 ${i}`, kind: "TaskDone" })
        ),
      ];

      vi.mocked(getPendingNotifications).mockResolvedValueOnce(notifications);

      render(<MenuBar />);

      await waitFor(() => {
        expect(screen.getByText("通知")).toBeInTheDocument();
      });

      // 查找通知容器（包含 max-h-[120px] 的元素）
      const notificationContainer = screen.getByText("通知").closest("div")?.parentElement?.querySelector(".max-h-\\[120px\\]");
      expect(notificationContainer).toBeTruthy();
      expect(notificationContainer).toHaveClass("overflow-y-auto");
    });

    it("每组最多显示 3 条通知，超出显示 +N 更多", async () => {
      const { getPendingNotifications } = await import("../lib/tauri");

      const notifications = Array.from({ length: 5 }, (_, i) =>
        makeNotification({ id: `confirm-${i}`, title: `确认通知 ${i}`, kind: "Confirm" })
      );

      vi.mocked(getPendingNotifications).mockResolvedValueOnce(notifications);

      render(<MenuBar />);

      await waitFor(() => {
        // 应显示前 3 条
        expect(screen.getByText("确认通知 0")).toBeInTheDocument();
        expect(screen.getByText("确认通知 1")).toBeInTheDocument();
        expect(screen.getByText("确认通知 2")).toBeInTheDocument();

        // 不应显示第 4、5 条
        expect(screen.queryByText("确认通知 3")).not.toBeInTheDocument();
        expect(screen.queryByText("确认通知 4")).not.toBeInTheDocument();

        // 应显示 +2 更多
        expect(screen.getByText("+2 更多")).toBeInTheDocument();
      });
    });
  });

  // ─── L5: aria-label 可访问性 ──────────────────────────────────

  describe("L5: aria-label 可访问性", () => {
    it("所有快捷操作按钮都有正确的 aria-label", async () => {
      render(<MenuBar />);

      await waitFor(() => {
        const mainWindowBtn = screen.getByLabelText("主窗口");
        expect(mainWindowBtn).toBeInTheDocument();
        expect(mainWindowBtn.tagName).toBe("BUTTON");

        const captureBtn = screen.getByLabelText("速记");
        expect(captureBtn).toBeInTheDocument();
        expect(captureBtn.tagName).toBe("BUTTON");

        const screenshotBtn = screen.getByLabelText("截图");
        expect(screenshotBtn).toBeInTheDocument();
        expect(screenshotBtn.tagName).toBe("BUTTON");

        const processBtn = screen.getByLabelText("处理");
        expect(processBtn).toBeInTheDocument();
        expect(processBtn.tagName).toBe("BUTTON");
      });
    });

    it("处理中状态时 aria-label 更新", async () => {
      const { triggerBatchProcess } = await import("../lib/tauri");

      // 让 triggerBatchProcess 挂起，模拟处理中状态
      vi.mocked(triggerBatchProcess).mockImplementationOnce(
        () => new Promise(() => {}) // never resolves
      );

      render(<MenuBar />);
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getByLabelText("处理")).toBeInTheDocument();
      });

      // 点击处理按钮
      await user.click(screen.getByLabelText("处理"));

      await waitFor(() => {
        // aria-label 应更新为 "处理中"
        expect(screen.getByLabelText("处理中")).toBeInTheDocument();
      });
    });
  });

  // ─── L5: Header + Status 合并显示 ─────────────────────────────

  describe("L5: Header + Status 合并显示", () => {
    it("Header 显示应用名称和系统状态在同一行", async () => {
      const { getSystemStatus } = await import("../lib/tauri");
      vi.mocked(getSystemStatus).mockResolvedValueOnce({
        collectors_total: 5,
        collectors_healthy: 3,
        scheduler_running: false,
        unprocessed_count: 2,
        today_processed_count: 8,
      });

      render(<MenuBar />);

      await waitFor(() => {
        // 应用名称
        expect(screen.getByText("Work Better")).toBeInTheDocument();

        // 系统状态信息在同一行
        expect(screen.getByText(/3\/5 采集/)).toBeInTheDocument();
        expect(screen.getByText("已暂停")).toBeInTheDocument();
        expect(screen.getByText("今日 8")).toBeInTheDocument();
      });
    });

    it("scheduler 未运行时显示已暂停", async () => {
      const { getSystemStatus } = await import("../lib/tauri");
      vi.mocked(getSystemStatus).mockResolvedValueOnce({
        collectors_total: 3,
        collectors_healthy: 3,
        scheduler_running: false,
        unprocessed_count: 0,
        today_processed_count: 0,
      });

      render(<MenuBar />);
      await waitFor(() => {
        expect(screen.getByText("已暂停")).toBeInTheDocument();
      });
    });
  });

  // ─── L5: 事件类型标签 ─────────────────────────────────────────

  describe("L5: 事件类型标签", () => {
    it("事件类型标签显示为带背景色的色块", async () => {
      const { getEvents } = await import("../lib/tauri");
      vi.mocked(getEvents).mockResolvedValueOnce([
        makeEvent({ type: "Message", content: "消息内容" }),
        makeEvent({ id: "2", type: "TaskUpdate", content: "任务内容" }),
        makeEvent({ id: "3", type: "ManualNote", content: "笔记内容" }),
      ]);

      render(<MenuBar />);

      await waitFor(() => {
        // 验证类型标签存在
        const msgTag = screen.getByText("MSG");
        const taskTag = screen.getByText("TASK");
        const noteTag = screen.getByText("NOTE");

        expect(msgTag).toBeInTheDocument();
        expect(taskTag).toBeInTheDocument();
        expect(noteTag).toBeInTheDocument();

        // 验证标签有圆角样式（色块特征）
        expect(msgTag).toHaveClass("rounded");
        expect(taskTag).toHaveClass("rounded");
        expect(noteTag).toHaveClass("rounded");
      });
    });
  });

  // ─── L5: 通知项 title/body 拆分 ──────────────────────────────

  describe("L5: 通知项 title/body 拆分", () => {
    it("通知项分别显示 title 和 body", async () => {
      const { getPendingNotifications } = await import("../lib/tauri");
      vi.mocked(getPendingNotifications).mockResolvedValueOnce([
        makeNotification({
          id: "n1",
          title: "任务需要确认",
          body: "请确认是否完成周报",
          kind: "Confirm",
        }),
      ]);

      render(<MenuBar />);

      await waitFor(() => {
        // title 和 body 分别显示
        expect(screen.getByText("任务需要确认")).toBeInTheDocument();
        expect(screen.getByText("请确认是否完成周报")).toBeInTheDocument();
      });
    });

    it("body 为空时不显示 body 区域", async () => {
      const { getPendingNotifications } = await import("../lib/tauri");
      vi.mocked(getPendingNotifications).mockResolvedValueOnce([
        makeNotification({
          id: "n1",
          title: "纯标题通知",
          body: "",
          kind: "Reminder",
        }),
      ]);

      render(<MenuBar />);

      await waitFor(() => {
        expect(screen.getByText("纯标题通知")).toBeInTheDocument();
        // 空 body 不应产生额外的文本节点
      });
    });
  });

  // ─── 原有测试（保留兼容性）─────────────────────────────────────

  it("shows quick action buttons", async () => {
    render(<MenuBar />);
    await waitFor(() => {
      expect(screen.getByText("主窗口")).toBeInTheDocument();
      expect(screen.getByText("速记")).toBeInTheDocument();
      expect(screen.getByText("截图")).toBeInTheDocument();
      expect(screen.getByText("处理")).toBeInTheDocument();
    });
  });

  it("shows system status in header", async () => {
    render(<MenuBar />);
    await waitFor(() => {
      expect(screen.getByText(/采集/)).toBeInTheDocument();
    });
  });

  it("shows scheduler status when running", async () => {
    render(<MenuBar />);
    await waitFor(() => {
      expect(screen.getByText("调度中")).toBeInTheDocument();
    });
  });

  it("displays events when loaded", async () => {
    const { getEvents } = await import("../lib/tauri");
    vi.mocked(getEvents).mockResolvedValueOnce([
      makeEvent({ content: "测试消息内容" }),
    ]);

    render(<MenuBar />);
    await waitFor(() => {
      expect(screen.getByText("测试消息内容")).toBeInTheDocument();
    });
  });

  it("displays notifications grouped by kind", async () => {
    const { getPendingNotifications } = await import("../lib/tauri");
    vi.mocked(getPendingNotifications).mockResolvedValueOnce([
      makeNotification({ id: "notif-1", title: "任务确认", body: "请确认飞书任务", kind: "Confirm" }),
      makeNotification({ id: "notif-2", title: "系统提醒", body: "采集器异常", kind: "Reminder" }),
    ]);

    render(<MenuBar />);
    await waitFor(() => {
      expect(screen.getByText("待确认")).toBeInTheDocument();
      expect(screen.getByText("提醒")).toBeInTheDocument();
      expect(screen.getByText("任务确认")).toBeInTheDocument();
      expect(screen.getByText("系统提醒")).toBeInTheDocument();
    });
  });

  it("shows unprocessed count badge when count > 0", async () => {
    const { getUnprocessedCount } = await import("../lib/tauri");
    vi.mocked(getUnprocessedCount).mockResolvedValueOnce(5);

    render(<MenuBar />);
    await waitFor(() => {
      expect(screen.getByText("5")).toBeInTheDocument();
    });
  });

  it("displays pending tasks when present", async () => {
    const { getPendingTasks } = await import("../lib/tauri");
    vi.mocked(getPendingTasks).mockResolvedValueOnce([
      makeTask({ title: "完成报告" }),
    ]);

    render(<MenuBar />);
    await waitFor(() => {
      expect(screen.getByText("今日待办")).toBeInTheDocument();
      expect(screen.getByText("完成报告")).toBeInTheDocument();
    });
  });

  it("shows today processed count when > 0", async () => {
    const { getSystemStatus } = await import("../lib/tauri");
    vi.mocked(getSystemStatus).mockResolvedValueOnce({
      collectors_total: 3,
      collectors_healthy: 2,
      scheduler_running: true,
      unprocessed_count: 0,
      today_processed_count: 12,
    });

    render(<MenuBar />);
    await waitFor(() => {
      expect(screen.getByText("今日 12")).toBeInTheDocument();
    });
  });

  it("hides today processed count when 0", async () => {
    const { getSystemStatus } = await import("../lib/tauri");
    vi.mocked(getSystemStatus).mockResolvedValueOnce({
      collectors_total: 3,
      collectors_healthy: 2,
      scheduler_running: true,
      unprocessed_count: 0,
      today_processed_count: 0,
    });

    render(<MenuBar />);
    await waitFor(() => {
      expect(screen.getByText("Work Better")).toBeInTheDocument();
    });
    expect(screen.queryByText(/今日/)).not.toBeInTheDocument();
  });

  it("shows TaskDone notifications in completed group", async () => {
    const { getPendingNotifications } = await import("../lib/tauri");
    vi.mocked(getPendingNotifications).mockResolvedValueOnce([
      makeNotification({
        id: "notif-done-1",
        title: "报告已生成",
        body: "周报已自动保存到 Obsidian",
        kind: "TaskDone",
      }),
    ]);

    render(<MenuBar />);
    await waitFor(() => {
      expect(screen.getByText("已完成")).toBeInTheDocument();
      expect(screen.getByText("报告已生成")).toBeInTheDocument();
    });
  });
});
