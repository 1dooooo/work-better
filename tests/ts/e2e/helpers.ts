/**
 * E2E 测试辅助函数
 *
 * 提供真实 Tauri 环境下的测试辅助功能。
 * 使用 data-testid 选择器和真实 Tauri IPC。
 *
 * 两种模式：
 * 1. Mock 模式（默认）：使用 addInitScript 注入 mock IPC，适用于纯 UI 测试
 * 2. 真实 IPC 模式：调用真实 Tauri 后端，使用 test mode 隔离数据
 */
import { type Page, expect } from "@playwright/test";

// ─── Mock 状态 ─────────────────────────────────────────────────

interface MockState {
  testMode: boolean;
  testDir: string | null;
  events: any[];
  unprocessedCount: number;
}

const mockState: MockState = {
  testMode: false,
  testDir: null,
  events: [
    {
      id: "mock-001",
      timestamp: new Date().toISOString(),
      source: "manual",
      type: "note",
      content: "Mock 事件 1",
    },
    {
      id: "mock-002",
      timestamp: new Date().toISOString(),
      source: "feishu",
      type: "message",
      content: "Mock 飞书消息",
    },
  ],
  unprocessedCount: 2,
};

/**
 * 检测是否在 Tauri WebView 中运行
 */
async function isTauriEnvironment(page: Page): Promise<boolean> {
  return page.evaluate(() => {
    return typeof (window as any).__TAURI__ !== "undefined";
  });
}

/**
 * 获取 mock 响应（在 Node.js 环境中执行）
 */
function getMockResponse(command: string, args?: Record<string, any>): any {
  switch (command) {
    case "set_test_mode":
      mockState.testMode = args?.enabled ?? false;
      mockState.testDir = args?.dataDir ?? null;
      return null;
    case "cleanup_test_data":
      mockState.testMode = false;
      mockState.testDir = null;
      return null;
    case "get_events":
      return mockState.events.slice(0, args?.limit ?? 50);
    case "get_unprocessed_count":
      return mockState.unprocessedCount;
    case "trigger_manual_capture": {
      const newEvent = {
        id: `mock-${Date.now()}`,
        timestamp: new Date().toISOString(),
        source: "manual",
        type: "note",
        content: args?.text ?? "",
      };
      mockState.events.unshift(newEvent);
      mockState.unprocessedCount += 1;
      return newEvent;
    }
    default:
      console.warn(`[Mock] Unhandled command: ${command}`);
      return null;
  }
}

/**
 * 执行 Tauri IPC 调用（自动检测环境）
 */
async function invokeTauriCommand(
  page: Page,
  command: string,
  args?: Record<string, any>,
): Promise<any> {
  const isTauri = await isTauriEnvironment(page);

  if (isTauri) {
    // 真实 Tauri 环境
    return page.evaluate(
      ({ cmd, args }) => {
        return (window as any).__TAURI__.core.invoke(cmd, args);
      },
      { cmd: command, args },
    );
  } else {
    // Mock 环境 - 返回预设的 mock 数据
    return getMockResponse(command, args);
  }
}

// ─── 测试环境配置 ─────────────────────────────────────────────

/**
 * 设置测试环境
 *
 * 调用 Tauri IPC 启用测试模式，使用临时数据目录。
 * 必须在 page.goto() 之后调用。
 */
export async function setupTestEnvironment(page: Page): Promise<string> {
  const testDir = `/tmp/work-better-test-${Date.now()}-${Math.random().toString(36).slice(2)}`;

  await invokeTauriCommand(page, "set_test_mode", {
    enabled: true,
    dataDir: testDir,
  });

  return testDir;
}

/**
 * 清理测试环境
 *
 * 调用 Tauri IPC 清理测试数据并禁用测试模式。
 */
export async function cleanupTestEnvironment(page: Page): Promise<void> {
  await invokeTauriCommand(page, "cleanup_test_data");
}

// ─── 窗口导航 ─────────────────────────────────────────────────

/**
 * 等待主窗口加载完成
 *
 * 使用 data-testid="sidebar" 作为就绪标志。
 */
export async function waitForMainWindow(page: Page): Promise<void> {
  await page.getByTestId("sidebar").waitFor({ state: "visible", timeout: 30000 });
}

/**
 * 导航到指定视图（使用 data-testid）
 *
 * Sidebar 中每个导航项渲染为 <TooltipTrigger>，
 * 带有 data-testid={`nav-item-${id}`} 属性。
 *
 * @param page - Playwright page 对象
 * @param viewLabel - 视图标签文本（如 "事件"、"设置"）或视图 ID
 */
export async function navigateToView(
  page: Page,
  viewLabel: string,
): Promise<void> {
  await waitForMainWindow(page);

  // 映射视图名称到 nav-item data-testid
  const viewMap: Record<string, string> = {
    "事件": "events",
    "设置": "settings",
    "任务": "tasks",
    "报告": "reports",
    "时间线": "timeline",
    "工作台": "dashboard",
  };

  const viewId = viewMap[viewLabel] || viewLabel.toLowerCase();
  await page.getByTestId(`nav-item-${viewId}`).click();
}

// ─── 事件查询 ─────────────────────────────────────────────────

/**
 * 通过 Tauri IPC 查询事件列表
 *
 * 注意：Rust 后端 `get_events` 命令仅接受 `limit` 参数，不支持 `offset`。
 */
export async function getEvents(
  page: Page,
  limit: number = 50,
): Promise<any[]> {
  return invokeTauriCommand(page, "get_events", { limit });
}

/**
 * 通过 Tauri IPC 查询未处理事件数量
 */
export async function getUnprocessedCount(page: Page): Promise<number> {
  return invokeTauriCommand(page, "get_unprocessed_count");
}

/**
 * 获取当前环境类型（用于调试）
 */
export async function getEnvironmentType(page: Page): Promise<string> {
  const isTauri = await isTauriEnvironment(page);
  return isTauri ? "tauri" : "browser";
}

// ─── 主窗口 Mock 脚本生成器 ────────────────────────────────────

/**
 * 生成主窗口的完整 mock 脚本
 *
 * 在 page.goto() 之前调用 page.addInitScript() 注入此函数。
 * 包含 MainWindow 及其子组件需要的所有 Tauri IPC mock。
 *
 * @param customHandlers - 自定义的 mock 处理器（命令名 → 处理函数），会覆盖默认行为
 */
export function createMainWindowMockScript(): string {
  return `
    // 初始化 Tauri internals
    (window).__TAURI_INTERNALS__ = (window).__TAURI_INTERNALS__ || {};
    (window).__TAURI_INTERNALS__.metadata = {
      currentWindow: { label: "main" },
      currentWebview: { windowLabel: "main", label: "main" },
    };

    // Mock invoke 函数
    (window).__TAURI_INTERNALS__.invoke = async (cmd, args) => {
      console.log('[Mock] invoke: ' + cmd, args);

      switch (cmd) {
        // ─── 事件相关 ─────────────────────────────────
        case "get_events":
          return ((window).__mockEvents || []).slice(0, args?.limit ?? 50);
        case "get_unprocessed_count":
          return ((window).__mockEvents || []).filter(e => !e.processed).length;
        case "trigger_manual_capture":
          const newEvent = {
            id: 'mock-' + Date.now(),
            timestamp: new Date().toISOString(),
            source: "manual",
            type: "note",
            content: args?.text || "",
            processed: false,
            attachments: [],
            tags: [],
          };
          (window).__mockEvents = (window).__mockEvents || [];
          (window).__mockEvents.unshift(newEvent);
          return newEvent;
        case "mark_event_processed":
          if ((window).__mockEvents) {
            const evt = (window).__mockEvents.find(e => e.id === args?.eventId);
            if (evt) evt.processed = true;
          }
          return null;

        // ─── 系统状态 ─────────────────────────────────
        case "get_system_status":
          return {
            collector_running: true,
            scheduler_running: true,
            unprocessed_count: ((window).__mockEvents || []).filter(e => !e.processed).length,
            today_processed_count: 0,
          };
        case "get_developer_mode":
          return (window).__developerMode || false;
        case "save_developer_mode":
          (window).__developerMode = args?.enabled;
          return null;

        // ─── 采集器相关 ───────────────────────────────
        case "get_collector_statuses":
          return (window).__collectorStatuses || [
            { id: "feishu", name: "飞书采集器", enabled: true, health_level: "healthy", health_message: null },
          ];
        case "get_collector_groups":
          return (window).__collectorGroups || [
            {
              id: "feishu-group",
              name: "飞书采集器组",
              enabled: true,
              collectors: [
                { id: "feishu", name: "飞书采集器", enabled: true, health_level: "healthy", health_message: null },
              ],
            },
          ];
        case "list_collectors":
          return ["feishu"];
        case "enable_collector":
        case "disable_collector":
          return null;
        case "trigger_feishu_collect":
          const count = args?.limit ?? 5;
          for (let i = 0; i < Math.min(count, 3); i++) {
            (window).__mockEvents = (window).__mockEvents || [];
            (window).__mockEvents.unshift({
              id: 'feishu-' + Date.now() + '-' + i,
              timestamp: new Date().toISOString(),
              source: "feishu",
              type: "message",
              content: '飞书消息 ' + (i + 1),
              processed: false,
              attachments: [],
              tags: [],
            });
          }
          return count;
        case "get_feishu_chat_id":
          return (window).__feishuChatId || "oc_default_chat";
        case "save_feishu_chat_id":
          (window).__feishuChatId = args?.chatId;
          return null;
        case "get_feishu_mode":
          return "cli";
        case "save_feishu_mode":
          return null;
        case "check_collector_health":
          return { level: "healthy", message: null };

        // ─── 任务相关 ─────────────────────────────────
        case "list_tasks":
          return (window).__mockTasks || [];
        case "list_scheduled_tasks":
          return [];
        case "get_pending_tasks":
          return [];
        case "create_task":
          const newTask = {
            id: 'task-' + Date.now(),
            title: args?.title || "新任务",
            description: null,
            status: "pending",
            priority: args?.priority || "medium",
            created_at: new Date().toISOString(),
          };
          (window).__mockTasks = (window).__mockTasks || [];
          (window).__mockTasks.push(newTask);
          return newTask;
        case "update_task_status":
          return null;
        case "pause_scheduler":
        case "resume_scheduler":
          return null;
        case "is_scheduler_paused":
          return false;

        // ─── 处理相关 ─────────────────────────────────
        case "trigger_batch_process":
          return { processed: 0, failed: 0 };
        case "process_event":
          return { success: true };
        case "get_processing_audits":
          return [];
        case "get_execution_logs":
          return [];
        case "get_audit_summary":
          return { total: 0, passed: 0, failed: 0 };

        // ─── 模型相关 ─────────────────────────────────
        case "get_model_config":
          return { provider: "mock", model: "mock-model", api_key: "" };
        case "save_model_config":
          return null;
        case "list_models":
          return [];
        case "test_model":
          return { success: true };

        // ─── 快捷键相关 ───────────────────────────────
        case "get_shortcut_config":
          return [];
        case "save_shortcut_config":
          return null;

        // ─── 通知相关 ─────────────────────────────────
        case "send_notification":
          return null;
        case "get_pending_notifications":
          return [];
        case "mark_notification_read":
          return null;
        case "clear_read_notifications":
          return null;

        // ─── 任务发现 ─────────────────────────────────
        case "discover_tasks_from_text":
          return [];
        case "confirm_pending_task":
          return { id: args?.pendingId || "unknown", title: "确认的任务", status: "Open", priority: "medium" };
        case "reject_pending_task":
          return null;

        // ─── 采集器分组 ───────────────────────────────
        case "enable_collector_group":
        case "disable_collector_group":
          return null;

        // ─── 窗口管理 ─────────────────────────────────
        case "show_capture_window":
        case "hide_capture_window":
          return null;

        // ─── 存储配置 ─────────────────────────────────
        case "get_storage_config":
          return { vault_path: "/default/vault" };
        case "save_storage_config":
          return null;

        // ─── 测试模式 ─────────────────────────────────
        case "set_test_mode":
          return null;
        case "cleanup_test_data":
          return null;

        // ─── 事件监听 ─────────────────────────────────
        case "plugin:event|listen":
          return "mock-event-" + Math.random().toString(36).slice(2);
        case "plugin:event|unlisten":
          return null;

        default:
          console.warn('[Mock] Unhandled command: ' + cmd);
          return null;
      }
    };

    // Mock transformCallback（事件系统需要）
    (window).__TAURI_INTERNALS__.transformCallback = (cb, once) => {
      const id = Math.random().toString(36).slice(2);
      (window).__callbacks = (window).__callbacks || {};
      (window).__callbacks[id] = cb;
      return id;
    };

    // Mock __TAURI_EVENT_PLUGIN_INTERNALS__（listen/unlisten 需要）
    (window).__TAURI_EVENT_PLUGIN_INTERNALS__ = {
      registerListener: (event, eventId, handler) => {
        console.log('[Mock] registerListener: ' + event + ' ' + eventId);
      },
      unregisterListener: (event, eventId) => {
        console.log('[Mock] unregisterListener: ' + event + ' ' + eventId);
      },
    };

    // 初始化默认 mock 状态
    (window).__mockEvents = (window).__mockEvents || [
      {
        id: "mock-001",
        timestamp: new Date().toISOString(),
        source: "manual",
        type: "note",
        content: "初始事件",
        processed: false,
        attachments: [],
        tags: [],
      },
    ];
    (window).__mockTasks = (window).__mockTasks || [];
    (window).__feishuChatId = "oc_default_chat";
    (window).__developerMode = false;
    (window).__collectorStatuses = [
      { id: "feishu", name: "飞书采集器", enabled: true, health_level: "healthy", health_message: null },
    ];
  `;
}

/**
 * 等待 Tauri 命令完成（通过等待 UI 更新）
 *
 * @param page - Playwright page 对象
 * @param selector - 要等待的选择器
 * @param timeout - 超时时间（毫秒）
 */
export async function waitForCommandCompletion(
  page: Page,
  selector: string,
  timeout: number = 5000
): Promise<void> {
  await page.locator(selector).waitFor({ state: "visible", timeout });
}

/**
 * 验证事件已创建（通过 UI 或 IPC）
 *
 * @param page - Playwright page 对象
 * @param expectedContent - 期望的事件内容
 * @param source - 事件来源
 */
export async function verifyEventCreated(
  page: Page,
  expectedContent: string,
  source?: string
): Promise<boolean> {
  // 等待事件出现在 UI 中
  const eventItems = page.getByTestId(/^event-item-/);
  const count = await eventItems.count();

  for (let i = 0; i < count; i++) {
    const item = eventItems.nth(i);
    const text = await item.textContent();
    if (text?.includes(expectedContent)) {
      return true;
    }
  }

  return false;
}

/**
 * 验证任务已创建（通过 UI）
 *
 * @param page - Playwright page 对象
 * @param expectedTitle - 期望的任务标题
 */
export async function verifyTaskCreated(
  page: Page,
  expectedTitle: string
): Promise<boolean> {
  const taskItems = page.locator('[data-testid^="task-item-"]');
  const count = await taskItems.count();

  for (let i = 0; i < count; i++) {
    const item = taskItems.nth(i);
    const text = await item.textContent();
    if (text?.includes(expectedTitle)) {
      return true;
    }
  }

  return false;
}

/**
 * 获取当前视图的容器选择器
 */
export function getViewContainer(viewId: string): string {
  const containerMap: Record<string, string> = {
    events: "events-container",
    settings: "settings-container",
    tasks: "tasks-container",
    reports: "reports-container",
    timeline: "timeline-container",
    dashboard: "dashboard-container",
  };

  return containerMap[viewId] || `${viewId}-container`;
}
