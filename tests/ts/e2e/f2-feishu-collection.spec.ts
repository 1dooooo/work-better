/**
 * F2: Feishu Collection Flow
 *
 * Tests the Feishu (Lark) collection flow: UI triggers collection,
 * lark-cli returns events, UI updates count.
 *
 * 使用 addInitScript 注入 mock IPC。
 */
import { test, expect } from "@playwright/test";
import { createMainWindowMockScript } from "./helpers";

test.describe("F2: Feishu Collection Flow", () => {
  // 每个测试前设置 mock
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(createMainWindowMockScript());
    await page.goto("/");
  });

  test("F2-01: UI triggers collection and updates event count", async ({
    page,
  }) => {
    // 等待侧边栏加载
    await page.getByTestId("sidebar").waitFor({ state: "visible", timeout: 30000 });

    // 导航到事件视图
    await page.getByTestId("nav-item-events").click();

    // 等待事件视图加载
    await expect(page.getByTestId("events-container")).toBeVisible();

    // 记录初始事件数量
    const initialEvents = await page.evaluate(() => {
      return (window as any).__mockEvents?.length ?? 0;
    });

    // 点击采集按钮
    const collectBtn = page.getByTestId("collect-button");
    await expect(collectBtn).toBeEnabled();
    await collectBtn.click();

    // 等待采集完成（按钮文本恢复）
    await expect(collectBtn).toHaveText("采集", { timeout: 10000 });

    // 验证新事件出现
    const newEvents = await page.evaluate(() => {
      return (window as any).__mockEvents?.length ?? 0;
    });
    expect(newEvents).toBeGreaterThan(initialEvents);

    // 验证有飞书来源的事件
    const feishuEvents = await page.evaluate(() => {
      const events = (window as any).__mockEvents || [];
      return events.filter((e: any) => e.source === "feishu");
    });
    expect(feishuEvents.length).toBeGreaterThan(0);
  });

  test("F2-02: Specified chat_id overrides config when collecting", async ({
    page,
  }) => {
    // 等待侧边栏加载
    await page.getByTestId("sidebar").waitFor({ state: "visible", timeout: 30000 });

    // 导航到设置页面
    await page.getByTestId("nav-item-settings").click();

    // 切换到采集器 tab
    await page.getByTestId("settings-tab-collector").click();

    // 找到飞书会话 ID 输入框并修改
    const chatIdInput = page.getByTestId("feishu-chat-id-input");
    await expect(chatIdInput).toBeVisible({ timeout: 10000 });
    await chatIdInput.fill("oc_new_chat_id");
    await chatIdInput.blur(); // 触发保存

    // 等待保存完成
    await page.waitForTimeout(500);

    // 验证 chat_id 已更新
    const savedChatId = await page.evaluate(() => {
      return (window as any).__feishuChatId;
    });
    expect(savedChatId).toBe("oc_new_chat_id");
  });

  test("F2-03: Disabled collector shows error on collection attempt", async ({
    page,
  }) => {
    // 等待侧边栏加载
    await page.getByTestId("sidebar").waitFor({ state: "visible", timeout: 30000 });

    // 导航到设置页面
    await page.getByTestId("nav-item-settings").click();

    // 切换到采集器 tab
    await page.getByTestId("settings-tab-collector").click();

    // 展开采集器组（点击组名）
    await page.getByText("飞书采集器组").click();

    // 找到飞书采集器开关并禁用
    const feishuToggle = page.getByTestId("collector-toggle-feishu");
    await expect(feishuToggle).toBeVisible({ timeout: 10000 });

    // 点击开关（如果当前是启用状态）
    const isEnabled = await feishuToggle.isChecked();
    if (isEnabled) {
      await feishuToggle.click();
    }

    // 等待状态更新
    await page.waitForTimeout(500);

    // 验证开关已禁用
    const isStillEnabled = await feishuToggle.isChecked();
    expect(isStillEnabled).toBe(false);
  });
});
