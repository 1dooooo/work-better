/**
 * F4: Settings Propagation
 *
 * Tests that settings changes in the UI correctly propagate to the
 * backend via Tauri invoke calls.
 *
 * 使用 addInitScript 注入 mock IPC。
 */
import { test, expect } from "@playwright/test";
import { waitForMainWindow, navigateToView, createMainWindowMockScript } from "./helpers";

test.describe("F4: Settings Propagation", () => {
  test.beforeEach(async ({ page }) => {
    // 注入基础 mock
    await page.addInitScript(createMainWindowMockScript());

    // 注入设置相关的额外 mock
    await page.addInitScript(() => {
      // 初始化设置状态
      (window as any).__mockSettings = {
        feishu_chat_id: "oc_default_chat",
        feishu_enabled: true,
        vault_path: "/default/vault",
      };

      // 保存原始 invoke
      const originalInvoke = (window as any).__TAURI_INTERNALS__.invoke;

      // 覆盖 invoke 以处理设置相关命令
      (window as any).__TAURI_INTERNALS__.invoke = async (cmd: string, args: any) => {
        switch (cmd) {
          case "get_feishu_chat_id":
            return (window as any).__mockSettings?.feishu_chat_id ?? "oc_default_chat";
          case "save_feishu_chat_id":
            (window as any).__mockSettings = (window as any).__mockSettings || {};
            (window as any).__mockSettings.feishu_chat_id = args?.chatId;
            return null;
          case "get_collector_statuses":
            return [
              {
                id: "feishu",
                name: "飞书采集器",
                enabled: (window as any).__mockSettings?.feishu_enabled ?? true,
                health_level: "healthy",
                health_message: null,
              },
            ];
          case "enable_collector":
            (window as any).__mockSettings = (window as any).__mockSettings || {};
            (window as any).__mockSettings.feishu_enabled = true;
            return null;
          case "disable_collector":
            (window as any).__mockSettings = (window as any).__mockSettings || {};
            (window as any).__mockSettings.feishu_enabled = false;
            return null;
          default:
            // 调用原始 invoke
            return originalInvoke(cmd, args);
        }
      };
    });

    await page.goto("/");
  });

  test("F4-01: chat_id setting change propagates to collector", async ({
    page,
  }) => {
    await waitForMainWindow(page);

    // 导航到设置页面
    await navigateToView(page, "设置");
    await expect(page.getByTestId("settings-container")).toBeVisible();

    // 切换到采集器 tab
    await page.getByTestId("settings-tab-collector").click();

    // 找到飞书会话 ID 输入框
    const chatIdInput = page.getByTestId("feishu-chat-id-input");
    await expect(chatIdInput).toBeVisible({ timeout: 10000 });

    // 修改 chat_id
    await chatIdInput.fill("oc_new_chat_id");
    await chatIdInput.blur(); // 触发保存

    // 等待保存完成
    await page.waitForTimeout(500);

    // 验证 chat_id 已更新
    const savedChatId = await page.evaluate(() => {
      return (window as any).__mockSettings?.feishu_chat_id;
    });
    expect(savedChatId).toBe("oc_new_chat_id");
  });

  test("F4-02: Collector toggle propagates to backend", async ({
    page,
  }) => {
    await waitForMainWindow(page);

    // 导航到设置页面
    await navigateToView(page, "设置");
    await expect(page.getByTestId("settings-container")).toBeVisible();

    // 切换到采集器 tab
    await page.getByTestId("settings-tab-collector").click();

    // 展开采集器组（点击组名）
    await page.getByText("飞书采集器组").click();

    // 找到飞书采集器开关
    const feishuToggle = page.getByTestId("collector-toggle-feishu");
    await expect(feishuToggle).toBeVisible({ timeout: 10000 });

    // 获取当前状态
    const initialState = await feishuToggle.isChecked();

    // 切换状态
    await feishuToggle.click();
    await page.waitForTimeout(500);

    // 验证状态已改变
    const newState = await feishuToggle.isChecked();
    expect(newState).toBe(!initialState);
  });

  test("F4-03: Settings tab navigation works", async ({
    page,
  }) => {
    await waitForMainWindow(page);

    // 导航到设置页面
    await navigateToView(page, "设置");
    await expect(page.getByTestId("settings-container")).toBeVisible();

    // 验证设置标签存在
    const settingsTabs = page.getByTestId(/^settings-tab-/);
    const tabCount = await settingsTabs.count();
    expect(tabCount).toBeGreaterThan(0);
  });

  test("F4-04: Collector list displays correctly", async ({
    page,
  }) => {
    await waitForMainWindow(page);

    // 导航到设置页面
    await navigateToView(page, "设置");
    await expect(page.getByTestId("settings-container")).toBeVisible();

    // 切换到采集器 tab
    await page.getByTestId("settings-tab-collector").click();

    // 验证采集器列表
    const collectorList = page.getByTestId("collector-list");
    await expect(collectorList).toBeVisible({ timeout: 10000 });
  });
});
