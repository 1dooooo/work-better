/**
 * F2: Feishu Collection Flow
 *
 * Tests the Feishu (Lark) collection flow: UI triggers collection,
 * lark-cli mock returns events, UI updates count.
 */
import {
  test,
  expect,
  getMockState,
  getInvokeLog,
  injectTauriMock,
  createDefaultMockState,
  navigateToView,
  overrideInvoke,
} from "./helpers";

test.describe("F2: Feishu Collection Flow", () => {
  test("F2-01: UI triggers collection via lark-cli and updates event count", async ({
    page,
    mockState,
  }) => {
    // Start on the main window events view
    await navigateToView(page, "事件");

    // Verify the events view loaded
    await expect(page.locator(".events-view")).toBeVisible();
    await expect(page.locator(".view__title")).toHaveText("事件流");

    // Record initial event count
    const initialCount = mockState.events.length;

    // Click the "采集飞书" button
    const collectBtn = page.locator("button:has-text('采集飞书')");
    await expect(collectBtn).toBeVisible();
    await expect(collectBtn).toBeEnabled();

    await collectBtn.click();

    // The mock has a 30ms delay + 50ms event fire, so loading state is brief.
    // Wait for collection to complete (button text changes back).
    await expect(collectBtn).toHaveText("采集飞书", { timeout: 5000 });

    // Verify the invoke log contains the correct commands
    const log = await getInvokeLog(page);
    const chatIdCall = log.find((l) => l.cmd === "get_feishu_chat_id");
    expect(chatIdCall).toBeDefined();

    const collectCall = log.find((l) => l.cmd === "trigger_feishu_collect");
    expect(collectCall).toBeDefined();
    expect(collectCall!.args.limit).toBe(20);

    // Verify new events appeared in state
    const state = await getMockState(page);
    expect(state.events.length).toBeGreaterThan(initialCount);

    // Verify new events have source "feishu"
    const feishuEvents = state.events.filter((e) => e.source === "feishu");
    expect(feishuEvents.length).toBeGreaterThan(0);

    // Verify the unprocessed count increased
    expect(state.unprocessedCount).toBeGreaterThan(3);
  });

  test("F2-02: Specified chat_id overrides config when collecting", async ({
    page,
  }) => {
    const state = createDefaultMockState();
    state.feishuChatId = "oc_original_chat";
    await injectTauriMock(page, state);

    await page.goto("/");
    await navigateToView(page, "事件");

    // Click collect button
    const collectBtn = page.locator("button:has-text('采集飞书')");
    await collectBtn.click();
    await expect(collectBtn).toHaveText("采集飞书", { timeout: 5000 });

    // Verify the chat_id from config was used
    const log = await getInvokeLog(page);
    const collectCall = log.find((l) => l.cmd === "trigger_feishu_collect");
    expect(collectCall).toBeDefined();
    expect(collectCall!.args.chatId).toBe("oc_original_chat");

    // Now change the chat_id via settings
    await navigateToView(page, "设置");

    // Find the chat ID input and change it
    const chatIdInput = page.locator('input[placeholder="输入飞书会话 ID"]');
    await expect(chatIdInput).toBeVisible();
    await chatIdInput.fill("oc_new_chat_id");
    await chatIdInput.blur(); // triggers save on blur

    // Wait for save to complete
    await page.waitForTimeout(500);

    // Verify save_feishu_chat_id was called with new value
    const saveLog = await getInvokeLog(page);
    const saveCall = saveLog.find((l) => l.cmd === "save_feishu_chat_id");
    expect(saveCall).toBeDefined();
    expect(saveCall!.args.chatId).toBe("oc_new_chat_id");

    // Go back to events and collect again
    await navigateToView(page, "事件");
    await page.locator("button:has-text('采集飞书')").click();
    await page.waitForTimeout(1000);

    // Verify the new chat_id was used
    const finalLog = await getInvokeLog(page);
    const secondCollect = finalLog.filter(
      (l) => l.cmd === "trigger_feishu_collect",
    );
    expect(secondCollect.length).toBe(2);
    expect(secondCollect[1].args.chatId).toBe("oc_new_chat_id");
  });

  test("F2-03: Disabled collector returns error on collection attempt", async ({
    page,
  }) => {
    const state = createDefaultMockState();
    state.collectors = [
      { id: "feishu", name: "飞书采集器", enabled: false, healthy: false },
    ];
    await injectTauriMock(page, state);

    await page.goto("/");

    // Override invoke to simulate error when collector is disabled
    await overrideInvoke(page, (cmd, args, original) => {
      if (cmd === "trigger_feishu_collect") {
        const st = (window as any).__mockState;
        const collector = st.collectors.find((c: any) => c.id === "feishu");
        if (collector && !collector.enabled) {
          throw new Error("Collector feishu is disabled");
        }
      }
      return (original as Function)(cmd, args);
    });

    await navigateToView(page, "事件");

    // Try to collect
    const collectBtn = page.locator("button:has-text('采集飞书')");
    await collectBtn.click();

    // Wait for error to appear
    await page.waitForTimeout(1000);

    // Verify error message is displayed
    const errorMsg = page.locator(".events-view__error");
    await expect(errorMsg).toBeVisible();
    await expect(errorMsg).toContainText("采集失败");
  });
});
