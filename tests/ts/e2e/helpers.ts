/**
 * E2E 测试辅助函数
 *
 * 提供真实 Tauri 环境下的测试辅助功能。
 * 使用 data-testid 选择器和真实 Tauri IPC。
 *
 * 环境检测：
 * - 在 Tauri WebView 中：使用真实的 __TAURI__.core.invoke
 * - 在普通浏览器中（Playwright）：使用 Tauri 官方 mockIPC
 */
import { type Page } from "@playwright/test";
import { mockIPC, clearMocks, mockWindows } from "@tauri-apps/api/mocks";

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

/**
 * 直接 mock invoke（在 page.evaluate 内部使用）
 */
function mockInvokeDirect(command: string, args?: any): any {
  switch (command) {
    case "set_test_mode":
      return null;
    case "cleanup_test_data":
      return null;
    case "get_events":
      return [
        {
          id: "mock-001",
          timestamp: new Date().toISOString(),
          source: "manual",
          type: "note",
          content: "Mock 事件 1",
        },
      ];
    case "get_unprocessed_count":
      return 2;
    case "trigger_manual_capture":
      return {
        id: `mock-${Date.now()}`,
        timestamp: new Date().toISOString(),
        source: "manual",
        type: "note",
        content: args?.text ?? "",
      };
    default:
      return null;
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

// ─── 导出 Tauri Mock 工具 ─────────────────────────────────────

export { mockIPC, clearMocks, mockWindows };
