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
 */
export async function waitForMainWindow(page: Page): Promise<void> {
  await page.waitForSelector(".sidebar", { timeout: 30000 });
}

/**
 * 导航到指定视图
 *
 * @param page - Playwright page 对象
 * @param viewLabel - 视图标签文本（如 "事件"、"设置"）
 */
export async function navigateToView(
  page: Page,
  viewLabel: string,
): Promise<void> {
  await waitForMainWindow(page);
  await page.click(`.sidebar__item:has-text("${viewLabel}")`);
}

// ─── 事件查询 ─────────────────────────────────────────────────

/**
 * 通过 Tauri IPC 查询事件列表
 */
export async function getEvents(
  page: Page,
  limit: number = 50,
  offset: number = 0,
): Promise<any[]> {
  return page.evaluate(
    ({ limit, offset }) => {
      return (window as any).__TAURI__.core.invoke("get_events", {
        limit,
        offset,
      });
    },
    { limit, offset },
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

// ─── 导出测试框架 ─────────────────────────────────────────────

export { expect };
export { test as base };
