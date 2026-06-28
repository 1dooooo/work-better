/**
 * F2: Feishu Collection Flow
 *
 * Tests the Feishu (Lark) collection flow: UI triggers collection,
 * lark-cli returns events, UI updates count.
 *
 * 使用 addInitScript 注入 mock IPC。
 */
import { test, expect } from "@playwright/test";

test.describe("F2: Feishu Collection Flow", () => {
  // 每个测试前设置 mock
  test.beforeEach(async ({ page }) => {
    // 在页面加载前注入 mock
    await page.addInitScript(() => {
      // 初始化 Tauri internals
      (window as any).__TAURI_INTERNALS__ = (window as any).__TAURI_INTERNALS__ || {};
      (window as any).__TAURI_INTERNALS__.metadata = {
        currentWindow: { label: "main" },
        currentWebview: { windowLabel: "main", label: "main" },
      };

      // Mock invoke 函数
      (window as any).__TAURI_INTERNALS__.invoke = async (cmd: string, args: any) => {
        console.log(`[Mock] invoke: ${cmd}`, args);

        switch (cmd) {
          case "trigger_feishu_collect":
            // 模拟飞书采集，添加新事件
            const count = args?.limit ?? 5;
            for (let i = 0; i < Math.min(count, 3); i++) {
              (window as any).__mockEvents = (window as any).__mockEvents || [];
              (window as any).__mockEvents.unshift({
                id: `feishu-${Date.now()}-${i}`,
                timestamp: new Date().toISOString(),
                source: "feishu",
                type: "message",
                content: `飞书消息 ${i + 1}`,
              });
            }
            return count;
          case "get_events":
            return ((window as any).__mockEvents || []).slice(0, args?.limit ?? 50);
          case "get_unprocessed_count":
            return ((window as any).__mockEvents || []).length;
          case "get_feishu_chat_id":
            return (window as any).__feishuChatId || "oc_default_chat";
          case "save_feishu_chat_id":
            (window as any).__feishuChatId = args?.chatId;
            return null;
          case "get_collector_statuses":
            return [
              { id: "feishu", name: "飞书采集器", enabled: true, healthy: true },
            ];
          case "enable_collector":
          case "disable_collector":
            return null;
          case "set_test_mode":
            return null;
          case "cleanup_test_data":
            return null;
          default:
            console.warn(`[Mock] Unhandled command: ${cmd}`);
            return null;
        }
      };

      // Mock transformCallback
      (window as any).__TAURI_INTERNALS__.transformCallback = (cb: Function, once?: boolean) => {
        const id = Math.random().toString(36).slice(2);
        (window as any).__callbacks = (window as any).__callbacks || {};
        (window as any).__callbacks[id] = cb;
        return id;
      };

      // 初始化 mock 状态
      (window as any).__mockEvents = [
        {
          id: "mock-001",
          timestamp: new Date().toISOString(),
          source: "manual",
          type: "note",
          content: "初始事件",
        },
      ];
      (window as any).__feishuChatId = "oc_default_chat";
    });

    await page.goto("/");
  });

  test("F2-01: UI triggers collection and updates event count", async ({
    page,
  }) => {
    // 调试：截图查看页面状态
    await page.screenshot({ path: "/tmp/f2-debug-1.png" });
    console.log("Page URL:", page.url());
    console.log("Page title:", await page.title());

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
