/**
 * F4: Settings Propagation
 *
 * Tests that settings changes in the UI correctly propagate to the
 * backend via Tauri invoke calls.
 */
import {
  test,
  expect,
  getMockState,
  getInvokeLog,
  injectTauriMock,
  createDefaultMockState,
  navigateToView,
} from "./helpers";

test.describe("F4: Settings Propagation", () => {
  test("F4-01: chat_id setting change propagates to collector", async ({
    page,
    mockState,
  }) => {
    await navigateToView(page, "设置");

    // Wait for collector settings to load
    await page.waitForSelector(".settings__collector-list", { timeout: 5000 });

    // Find the chat ID input
    const chatIdInput = page.locator('input[placeholder="输入飞书会话 ID"]');
    await expect(chatIdInput).toBeVisible();

    // Verify initial value is loaded from backend
    const initialValue = await chatIdInput.inputValue();
    expect(initialValue).toBe("oc_test_chat_id");

    // Change the chat ID
    await chatIdInput.fill("oc_new_propagated_chat");
    await chatIdInput.blur(); // triggers save on blur

    // Wait for save to complete
    await page.waitForTimeout(500);

    // Verify invoke was called with correct args
    const log = await getInvokeLog(page);
    const saveCall = log.find((l) => l.cmd === "save_feishu_chat_id");
    expect(saveCall).toBeDefined();
    expect(saveCall!.args.chatId).toBe("oc_new_propagated_chat");

    // Verify "已保存" indicator appears
    await expect(page.locator(".settings__saved")).toBeVisible();

    // Verify the mock state was updated
    const state = await getMockState(page);
    expect(state.feishuChatId).toBe("oc_new_propagated_chat");
  });

  test("F4-02: mode setting change propagates to collector", async ({
    page,
    mockState,
  }) => {
    await navigateToView(page, "设置");
    await page.waitForSelector(".settings__collector-list", { timeout: 5000 });

    // The radio inputs don't have value attributes in the HTML.
    // Select by the label text next to them instead.
    // The CLI radio should be checked initially (mock state: "cli").
    const cliLabel = page.locator(".settings__radio:has-text('lark-cli')");
    const apiLabel = page.locator(".settings__radio:has-text('API 直连')");

    // Verify CLI is selected (its input should be checked)
    const cliInput = cliLabel.locator("input[type='radio']");
    await expect(cliInput).toBeChecked();

    // Switch to API mode by clicking the API label
    const apiInput = apiLabel.locator("input[type='radio']");
    await apiInput.click({ force: true });

    // Wait for save
    await page.waitForTimeout(500);

    // Verify save_feishu_mode was called with "api"
    const log = await getInvokeLog(page);
    const saveCall = log.find((l) => l.cmd === "save_feishu_mode");
    expect(saveCall).toBeDefined();
    expect(saveCall!.args.mode).toBe("api");

    // Verify "已保存" indicator
    await expect(page.locator(".settings__saved")).toBeVisible();

    // Verify hint text updated
    const hint = page.locator(".settings__hint").filter({
      hasText: "飞书开放平台 API",
    });
    await expect(hint).toBeVisible();

    // Verify mock state updated
    const state = await getMockState(page);
    expect(state.feishuMode).toBe("api");
  });

  test("F4-03: disable collector propagates to backend", async ({
    page,
    mockState,
  }) => {
    await navigateToView(page, "设置");
    await page.waitForSelector(".settings__collector-list", { timeout: 5000 });

    // Verify the collector is initially enabled
    const toggle = page.locator(
      ".settings__collector-item input[type='checkbox']",
    );
    await expect(toggle.first()).toBeChecked();

    // Disable the collector by clicking the toggle label/slider
    const toggleLabel = page.locator(".settings__toggle").first();
    await toggleLabel.click();

    // Wait for the toggle handler to complete
    await page.waitForTimeout(500);

    // Verify disable_collector was called
    const log = await getInvokeLog(page);
    const disableCall = log.find((l) => l.cmd === "disable_collector");
    expect(disableCall).toBeDefined();
    expect(disableCall!.args.id).toBe("feishu");

    // Verify mock state updated
    const state = await getMockState(page);
    const feishuCollector = state.collectors.find((c) => c.id === "feishu");
    expect(feishuCollector).toBeDefined();
    expect(feishuCollector!.enabled).toBe(false);

    // Re-enable and verify
    await toggleLabel.click();
    await page.waitForTimeout(500);

    const logAfter = await getInvokeLog(page);
    const enableCall = logAfter.find((l) => l.cmd === "enable_collector");
    expect(enableCall).toBeDefined();
    expect(enableCall!.args.id).toBe("feishu");
  });

  test("F4-04: vault_path change propagates to writer via storage config", async ({
    page,
    mockState,
  }) => {
    await navigateToView(page, "设置");
    await page.waitForSelector(
      ".settings__section-title:has-text('存储配置')",
      { timeout: 5000 },
    );

    // Find the vault path input
    const vaultInput = page.locator("#vault-path");
    await expect(vaultInput).toBeVisible();

    // Verify initial value
    await expect(vaultInput).toHaveValue("~/Documents/Obsidian");

    // Change the vault path
    await vaultInput.fill("/Users/test/MyVault");

    // Click save
    const saveBtn = page.locator(
      ".settings__section:has-text('存储配置') button:has-text('保存')",
    );
    await saveBtn.click();

    // Wait for save
    await page.waitForTimeout(500);

    // Verify save_storage_config was called with new path
    const log = await getInvokeLog(page);
    const saveCall = log.find((l) => l.cmd === "save_storage_config");
    expect(saveCall).toBeDefined();
    expect((saveCall!.args.config as any).vault_path).toBe(
      "/Users/test/MyVault",
    );

    // Verify "已保存" indicator
    const savedIndicator = page.locator(
      ".settings__section:has-text('存储配置') .settings__saved",
    );
    await expect(savedIndicator).toBeVisible();

    // Verify mock state updated
    const state = await getMockState(page);
    expect(state.storageConfig.vault_path).toBe("/Users/test/MyVault");
  });
});
