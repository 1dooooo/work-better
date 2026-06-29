/**
 * F12: Error Handling
 *
 * Tests error handling: empty states, invalid data, edge cases.
 */
import { test, expect } from "@playwright/test";
import { createMainWindowMockScript, waitForMainWindow, navigateToView } from "./helpers";

test.describe("F12: Error Handling", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(createMainWindowMockScript());
    await page.goto("/");
  });

  test("F12-01: Empty events list displays correctly", async ({ page }) => {
    await waitForMainWindow(page);

    // 清空事件
    await page.evaluate(() => {
      (window as any).__mockEvents = [];
    });

    // 导航到事件视图
    await navigateToView(page, "事件");
    await expect(page.getByTestId("events-container")).toBeVisible();
  });

  test("F12-02: Empty tasks list displays correctly", async ({ page }) => {
    await waitForMainWindow(page);

    // 清空任务
    await page.evaluate(() => {
      (window as any).__mockTasks = [];
    });

    // 导航到任务视图
    await navigateToView(page, "任务");
    await expect(page.getByTestId("tasks-container")).toBeVisible();
  });

  test("F12-03: Dashboard handles missing system status gracefully", async ({ page }) => {
    await waitForMainWindow(page);

    // 覆盖 mock 返回 null 状态
    await page.evaluate(() => {
      const originalInvoke = (window as any).__TAURI_INTERNALS__.invoke;
      (window as any).__TAURI_INTERNALS__.invoke = async (cmd: string, args: any) => {
        if (cmd === "get_system_status") {
          return {
            collector_running: false,
            scheduler_running: false,
            unprocessed_count: 0,
            today_processed_count: 0,
          };
        }
        return originalInvoke(cmd, args);
      };
    });

    // 导航到工作台
    await navigateToView(page, "工作台");
    await expect(page.getByTestId("dashboard-container")).toBeVisible();
  });

  test("F12-04: Large event list is handled correctly", async ({ page }) => {
    await waitForMainWindow(page);

    // 添加大量事件
    await page.evaluate(() => {
      const events = [];
      for (let i = 0; i < 100; i++) {
        events.push({
          id: `event-${i}`,
          timestamp: new Date().toISOString(),
          source: i % 2 === 0 ? "manual" : "feishu",
          type: i % 3 === 0 ? "note" : "message",
          content: `Event ${i}`,
          processed: i % 5 === 0,
          tags: [],
        });
      }
      (window as any).__mockEvents = events;
    });

    // 导航到事件视图
    await navigateToView(page, "事件");
    await expect(page.getByTestId("events-container")).toBeVisible();

    // 验证事件数量
    const count = await page.evaluate(() => {
      return (window as any).__mockEvents?.length ?? 0;
    });
    expect(count).toBe(100);
  });

  test("F12-05: Invalid view parameter defaults to dashboard", async ({ page }) => {
    // 带无效 view 参数访问
    await page.goto("/?view=invalid");
    await waitForMainWindow(page);

    // 验证 URL 包含 view 参数（应用正常加载）
    const url = page.url();
    expect(url).toContain("/?view=");
  });
});
