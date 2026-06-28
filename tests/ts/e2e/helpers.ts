/**
 * E2E 测试辅助函数
 *
 * 提供真实 Tauri 环境下的测试辅助功能。
 * 使用 data-testid 选择器和真实 Tauri IPC。
 */
import { type Page } from "@playwright/test";

// ─── 测试环境配置 ─────────────────────────────────────────────

/**
 * 设置测试环境
 *
 * 调用 Tauri IPC 启用测试模式，使用临时数据目录。
 * 必须在 page.goto() 之后调用。
 */
export async function setupTestEnvironment(page: Page): Promise<string> {
  const testDir = `/tmp/work-better-test-${Date.now()}-${Math.random().toString(36).slice(2)}`;

  await page.evaluate(async (dataDir: string) => {
    await (window as any).__TAURI__.core.invoke("set_test_mode", {
      enabled: true,
      dataDir: dataDir,
    });
  }, testDir);

  return testDir;
}

/**
 * 清理测试环境
 *
 * 调用 Tauri IPC 清理测试数据并禁用测试模式。
 */
export async function cleanupTestEnvironment(page: Page): Promise<void> {
  await page.evaluate(async () => {
    await (window as any).__TAURI__.core.invoke("cleanup_test_data");
  });
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
  return page.evaluate(
    (limit: number) => {
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
