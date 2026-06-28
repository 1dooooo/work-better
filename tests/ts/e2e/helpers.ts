/**
 * E2E 测试辅助函数
 *
 * 提供真实 Tauri 环境下的测试辅助功能。
 * 移除了所有 mock 相关代码，使用真实的 Tauri IPC。
 */
import { test as base, expect, type Page } from "@playwright/test";

// ─── 测试环境配置 ─────────────────────────────────────────────

/**
 * 设置测试环境
 *
 * 通过 Tauri IPC 设置测试模式，使用临时数据目录。
 * 必须在 page.goto() 之后调用。
 */
export async function setupTestEnvironment(page: Page): Promise<string> {
  const testDir = `/tmp/work-better-test-${Date.now()}`;

  await page.evaluate((dataDir) => {
    return (window as any).__TAURI__.core.invoke("set_test_mode", {
      enabled: true,
      data_dir: dataDir,
    });
  }, testDir);

  return testDir;
}

/**
 * 清理测试环境
 *
 * 清理测试数据，必须在每个测试结束后调用。
 */
export async function cleanupTestEnvironment(page: Page): Promise<void> {
  await page.evaluate(() => {
    return (window as any).__TAURI__.core.invoke("cleanup_test_data");
  });
}

// ─── 窗口导航 ─────────────────────────────────────────────────

/**
 * 等待主窗口加载完成
 *
 * 使用 aside 元素（Sidebar 组件的根元素）作为就绪标志。
 * Sidebar 组件渲染为 <aside>，包含 "Work Better" 品牌文字。
 */
export async function waitForMainWindow(page: Page): Promise<void> {
  await page.locator("aside").waitFor({ state: "visible", timeout: 30000 });
}

/**
 * 导航到指定视图
 *
 * Sidebar 中每个导航项渲染为 <TooltipTrigger> 按钮，
 * 包含图标和标签文本（如 "事件"、"设置"）。
 * 使用 getByRole + 精确匹配来定位。
 *
 * @param page - Playwright page 对象
 * @param viewLabel - 视图标签文本（如 "事件"、"设置"）
 */
export async function navigateToView(
  page: Page,
  viewLabel: string,
): Promise<void> {
  await waitForMainWindow(page);
  await page.getByRole("button", { name: viewLabel, exact: true }).click();
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
  return page.evaluate(
    (limit) => {
      return (window as any).__TAURI__.core.invoke("get_events", {
        limit,
      });
    },
    limit,
  );
}

/**
 * 通过 Tauri IPC 查询未处理事件数量
 */
export async function getUnprocessedCount(page: Page): Promise<number> {
  return page.evaluate(() => {
    return (window as any).__TAURI__.core.invoke("get_unprocessed_count");
  });
}

// ─── Mock 辅助函数（遗留 stub，待迁移至真实 Tauri IPC） ──────────

export interface MockState {
  events: any[];
  invokeLog: string[];
}

export function createDefaultMockState(): MockState {
  return { events: [], invokeLog: [] };
}

export async function injectTauriMock(
  _page: Page,
  _state: MockState,
): Promise<void> {
  // TODO: 替换为真实 Tauri IPC 调用
}

export async function getMockState(_page: Page): Promise<MockState> {
  // TODO: 替换为真实 Tauri IPC 调用
  return createDefaultMockState();
}

export async function getInvokeLog(_page: Page): Promise<string[]> {
  // TODO: 替换为真实 Tauri IPC 调用
  return [];
}

export async function overrideInvoke(
  _page: Page,
  _command: string,
  _handler: (...args: any[]) => any,
): Promise<void> {
  // TODO: 替换为真实 Tauri IPC 调用
}

export function createMockEvent(overrides: Record<string, any> = {}): any {
  return {
    id: `mock-${Date.now()}`,
    source: "manual",
    content: "test event",
    created_at: new Date().toISOString(),
    processed: false,
    ...overrides,
  };
}

// ─── 自定义 Test Fixtures ──────────────────────────────────────
// 仅 F3-F6 使用（当前已 skip），F1/F2 直接从 @playwright/test 导入 test

type TestFixtures = {
  mockState: MockState;
};

const testWithFixtures = base.extend<TestFixtures>({
  mockState: async ({}, use) => {
    await use(createDefaultMockState());
  },
});

// ─── 导出测试框架 ─────────────────────────────────────────────

export { expect };
export { base };
export { testWithFixtures as test };
