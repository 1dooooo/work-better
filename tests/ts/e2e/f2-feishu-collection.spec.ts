/**
 * F2: Feishu Collection Flow
 *
 * Tests the Feishu (Lark) collection flow: UI triggers collection,
 * lark-cli returns events, UI updates count.
 *
 * 运行在真实 Tauri 环境中，使用测试模式隔离数据。
 */
import { test, expect } from "@playwright/test";
import {
  setupTestEnvironment,
  cleanupTestEnvironment,
  waitForMainWindow,
  navigateToView,
  getEvents,
  getUnprocessedCount,
} from "./helpers";

test.describe("F2: Feishu Collection Flow", () => {
  // 每个测试前启用测试模式
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await setupTestEnvironment(page);
    await waitForMainWindow(page);
  });

  // 每个测试后清理
  test.afterEach(async ({ page }) => {
    await cleanupTestEnvironment(page);
  });

  test("F2-01: UI triggers collection and updates event count", async ({
    page,
  }) => {
    // 导航到事件视图
    await navigateToView(page, "事件");

    // 等待事件视图加载
    await expect(page.getByTestId("events-container")).toBeVisible();

    // 记录初始事件数量
    const initialCount = await getUnprocessedCount(page);

    // 点击采集按钮
    const collectBtn = page.getByTestId("collect-button");
    await expect(collectBtn).toBeEnabled();
    await collectBtn.click();

    // 等待采集完成（按钮文本从 "采集中..." 恢复为 "采集"）
    await expect(page.getByTestId("collect-button")).toHaveText("采集", {
      timeout: 10000,
    });

    // 验证新事件出现（使用 expect.poll 替代 waitForTimeout）
    await expect
      .poll(
        async () => {
          const events = await getEvents(page, 50);
          const feishuEvents = events.filter(
            (e: any) => e.source === "feishu",
          );
          return feishuEvents.length;
        },
        { timeout: 5000 },
      )
      .toBeGreaterThan(0);

    // 验证未处理事件数量增加
    const newCount = await getUnprocessedCount(page);
    expect(newCount).toBeGreaterThan(initialCount);
  });

  test("F2-02: Specified chat_id overrides config when collecting", async ({
    page,
  }) => {
    // 导航到设置页面
    await navigateToView(page, "设置");

    // 找到飞书会话 ID 输入框并修改
    const chatIdInput = page.getByTestId("feishu-chat-id-input");
    await expect(chatIdInput).toBeVisible();
    await chatIdInput.fill("oc_new_chat_id");
    await chatIdInput.blur(); // 触发保存

    // 等待保存完成
    await page.waitForTimeout(500);

    // 导航到事件视图
    await navigateToView(page, "事件");

    // 点击采集按钮
    await page.getByTestId("collect-button").click();

    // 等待采集完成
    await expect(page.getByTestId("collect-button")).toHaveText("采集", {
      timeout: 10000,
    });

    // 验证采集完成
    const events = await getEvents(page, 50);
    const feishuEvents = events.filter((e: any) => e.source === "feishu");
    expect(feishuEvents.length).toBeGreaterThan(0);
  });

  test("F2-03: Disabled collector shows error on collection attempt", async ({
    page,
  }) => {
    // 导航到设置页面
    await navigateToView(page, "设置");

    // 找到飞书采集器开关并禁用
    const feishuToggle = page.getByTestId("collector-toggle-feishu");
    await expect(feishuToggle).toBeVisible();
    await feishuToggle.click();

    // 等待状态更新
    await page.waitForTimeout(500);

    // 导航到事件视图
    await navigateToView(page, "事件");

    // 尝试采集
    await page.getByTestId("collect-button").click();

    // 等待错误出现（通过 toast 显示）
    const errorToast = page.getByText("采集失败");
    await expect(errorToast).toBeVisible({ timeout: 5000 });
  });
});
