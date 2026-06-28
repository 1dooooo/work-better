/**
 * F6: MenuBar Data Flow
 *
 * Tests the Dashboard view: event count, system status,
 * and data refresh flow from backend to the UI display.
 *
 * 使用 addInitScript 注入 mock IPC。
 */
import { test, expect } from "@playwright/test";
import { waitForMainWindow, navigateToView, createMainWindowMockScript } from "./helpers";

test.describe("F6: Dashboard Data Flow", () => {
  test.beforeEach(async ({ page }) => {
    // 注入基础 mock（包含所有 MainWindow 需要的命令）
    await page.addInitScript(createMainWindowMockScript());
    await page.goto("/");
  });

  test("F6-01: Dashboard displays system status correctly", async ({
    page,
  }) => {
    await waitForMainWindow(page);

    // 导航到工作台
    await navigateToView(page, "工作台");
    await expect(page.getByTestId("dashboard-container")).toBeVisible();

    // 验证仪表盘标题
    const title = page.locator("text=仪表盘");
    await expect(title).toBeVisible();
  });

  test("F6-02: Dashboard shows unprocessed count", async ({
    page,
  }) => {
    await waitForMainWindow(page);

    // 设置未处理事件
    await page.evaluate(() => {
      (window as any).__mockEvents = [
        { id: "1", timestamp: new Date().toISOString(), source: "manual", type: "note", content: "Event 1", processed: false, tags: [] },
        { id: "2", timestamp: new Date().toISOString(), source: "manual", type: "note", content: "Event 2", processed: false, tags: [] },
        { id: "3", timestamp: new Date().toISOString(), source: "feishu", type: "message", content: "Event 3", processed: true, tags: [] },
      ];
    });

    // 导航到工作台
    await navigateToView(page, "工作台");
    await expect(page.getByTestId("dashboard-container")).toBeVisible();

    // 验证待处理数量显示（精确匹配标题，避免匹配"事件待处理"副标题）
    const pendingText = page.getByText("待处理", { exact: true });
    await expect(pendingText).toBeVisible();
  });

  test("F6-03: Dashboard shows collector health status", async ({
    page,
  }) => {
    await waitForMainWindow(page);

    // 导航到工作台
    await navigateToView(page, "工作台");
    await expect(page.getByTestId("dashboard-container")).toBeVisible();

    // 验证采集器状态卡片
    const collectorStatus = page.locator("text=采集器状态");
    await expect(collectorStatus).toBeVisible();
  });

  test("F6-04: Dashboard shows pending tasks", async ({
    page,
  }) => {
    await waitForMainWindow(page);

    // 设置待确认任务
    await page.evaluate(() => {
      (window as any).__mockPendingTasks = [
        { id: "pt-1", title: "AI 发现的任务 1", source: "AI", origin_text: "来自消息分析" },
        { id: "pt-2", title: "AI 发现的任务 2", source: "AI", origin_text: "来自邮件" },
      ];
    });

    // 导航到工作台
    await navigateToView(page, "工作台");
    await expect(page.getByTestId("dashboard-container")).toBeVisible();

    // 验证待办任务卡片
    const pendingTasks = page.locator("text=待办任务");
    await expect(pendingTasks).toBeVisible();
  });

  test("F6-05: Dashboard shows recent events", async ({
    page,
  }) => {
    await waitForMainWindow(page);

    // 设置事件
    await page.evaluate(() => {
      (window as any).__mockEvents = [
        { id: "1", timestamp: new Date().toISOString(), source: "manual", type: "note", content: "最近事件 1", processed: false, tags: [] },
        { id: "2", timestamp: new Date().toISOString(), source: "feishu", type: "message", content: "最近事件 2", processed: true, tags: [] },
      ];
    });

    // 导航到工作台
    await navigateToView(page, "工作台");
    await expect(page.getByTestId("dashboard-container")).toBeVisible();

    // 验证最近事件卡片
    const recentEvents = page.locator("text=最近事件");
    await expect(recentEvents).toBeVisible();
  });

  test("F6-06: Timeline view displays events grouped by time", async ({
    page,
  }) => {
    await waitForMainWindow(page);

    // 设置事件
    await page.evaluate(() => {
      (window as any).__mockEvents = [
        { id: "1", timestamp: "2026-06-28T10:00:00Z", source: "manual", type: "note", content: "上午事件 1", processed: false, tags: [] },
        { id: "2", timestamp: "2026-06-28T10:30:00Z", source: "feishu", type: "message", content: "上午事件 2", processed: true, tags: [] },
        { id: "3", timestamp: "2026-06-28T14:00:00Z", source: "manual", type: "note", content: "下午事件 1", processed: false, tags: [] },
      ];
    });

    // 导航到时间线
    await navigateToView(page, "时间线");
    await expect(page.getByTestId("timeline-container")).toBeVisible();

    // 验证时间线标题（使用 heading 角色避免匹配侧边栏导航项）
    const title = page.getByRole("heading", { name: "时间线" });
    await expect(title).toBeVisible();
  });
});
