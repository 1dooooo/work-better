/**
 * F2: Feishu Collection Flow
 *
 * Tests the Feishu (Lark) collection flow: UI triggers collection,
 * lark-cli mock returns events, UI updates count.
 *
 * 运行在真实 Tauri 环境中，使用真实的 Tauri IPC。
 * 飞书 API 使用 MSW mock server 隔离。
 */
import { test, expect } from "@playwright/test";
import {
  navigateToView,
  getEvents,
  getUnprocessedCount,
} from "./helpers";
import { feishuServer } from "./mock-feishu-server";

// 启动飞书 API mock server
test.beforeAll(() => {
  feishuServer.listen();
});

test.afterAll(() => {
  feishuServer.close();
});

// TODO: add setupTestEnvironment/cleanupTestEnvironment when backend commands are implemented
test.afterEach(async ({ page }) => {
  feishuServer.resetHandlers();

  // Restore state mutated by F2-02 (chat ID) and F2-03 (feishu toggle)
  try {
    await page.goto("/");
    await navigateToView(page, "设置");

    // Restore chat ID to empty/default
    const chatIdInput = page.locator('input[placeholder="输入飞书会话 ID"]');
    if (await chatIdInput.isVisible({ timeout: 2000 }).catch(() => false)) {
      await chatIdInput.fill("");
      await chatIdInput.blur();
      await page.waitForTimeout(300);
    }

    // Re-enable feishu collector if it was disabled
    const feishuToggle = page.locator(
      '[data-collector="feishu"] .collector-toggle',
    );
    if (await feishuToggle.isVisible({ timeout: 2000 }).catch(() => false)) {
      const isDisabled =
        (await feishuToggle.getAttribute("aria-checked")) === "false";
      if (isDisabled) {
        await feishuToggle.click();
        await page.waitForTimeout(300);
      }
    }
  } catch {
    // Best-effort cleanup — don't fail the test on cleanup errors
  }
});

test.describe("F2: Feishu Collection Flow", () => {
  test("F2-01: UI triggers collection via lark-cli and updates event count", async ({
    page,
  }) => {
    // 导航到事件视图
    await page.goto("/");
    await navigateToView(page, "事件");

    // 等待事件视图加载
    await expect(page.locator(".events-view")).toBeVisible();
    await expect(page.locator(".view__title")).toHaveText("事件流");

    // 记录初始事件数量
    const initialCount = await getUnprocessedCount(page);

    // 点击 "采集飞书" 按钮
    const collectBtn = page.locator("button:has-text('采集飞书')");
    await expect(collectBtn).toBeVisible();
    await expect(collectBtn).toBeEnabled();

    await collectBtn.click();

    // 等待采集完成（按钮文本恢复）
    await expect(collectBtn).toHaveText("采集飞书", { timeout: 5000 });

    // 等待后端处理完成
    await page.waitForTimeout(1000);

    // 验证新事件出现
    const events = await getEvents(page, 50, 0);
    const feishuEvents = events.filter((e: any) => e.source === "feishu");
    expect(feishuEvents.length).toBeGreaterThan(0);

    // 验证未处理事件数量增加
    const newCount = await getUnprocessedCount(page);
    expect(newCount).toBeGreaterThan(initialCount);
  });

  test("F2-02: Specified chat_id overrides config when collecting", async ({
    page,
  }) => {
    // 导航到事件视图
    await page.goto("/");
    await navigateToView(page, "事件");

    // 等待事件视图加载
    await page.waitForSelector(".events-view", { timeout: 5000 });

    // 点击采集按钮
    const collectBtn = page.locator("button:has-text('采集飞书')");
    await collectBtn.click();

    // 等待采集完成
    await expect(collectBtn).toHaveText("采集飞书", { timeout: 5000 });

    // 导航到设置页面
    await navigateToView(page, "设置");

    // 找到会话 ID 输入框并修改
    const chatIdInput = page.locator('input[placeholder="输入飞书会话 ID"]');
    await expect(chatIdInput).toBeVisible();
    await chatIdInput.fill("oc_new_chat_id");
    await chatIdInput.blur(); // 触发保存

    // 等待保存完成
    await page.waitForTimeout(500);

    // 返回事件视图
    await navigateToView(page, "事件");

    // 再次采集
    await page.locator("button:has-text('采集飞书')").click();
    await page.waitForTimeout(1000);

    // 验证使用了新的会话 ID
    const events = await getEvents(page, 50, 0);
    const newEvents = events.filter(
      (e: any) => e.content && e.content.includes("oc_new_chat_id"),
    );
    expect(newEvents.length).toBeGreaterThan(0);
  });

  test("F2-03: Disabled collector returns error on collection attempt", async ({
    page,
  }) => {
    // 导航到设置页面
    await page.goto("/");
    await navigateToView(page, "设置");

    // 等待设置页面加载
    await page.waitForSelector(".settings-view", { timeout: 5000 });

    // 找到飞书采集器开关并禁用
    const feishuToggle = page.locator(
      '[data-collector="feishu"] .collector-toggle',
    );
    await expect(feishuToggle).toBeVisible();
    await feishuToggle.click();

    // 等待状态更新
    await page.waitForTimeout(500);

    // 导航到事件视图
    await navigateToView(page, "事件");

    // 尝试采集
    const collectBtn = page.locator("button:has-text('采集飞书')");
    await collectBtn.click();

    // 等待错误出现
    await page.waitForTimeout(1000);

    // 验证错误消息
    const errorMsg = page.locator(".events-view__error");
    await expect(errorMsg).toBeVisible();
    await expect(errorMsg).toContainText("采集失败");
  });
});
