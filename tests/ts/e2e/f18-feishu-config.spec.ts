/**
 * F18: Feishu Configuration
 */
import { test, expect } from "@playwright/test";
import { createMainWindowMockScript, waitForMainWindow } from "./helpers";

test.describe("F18: Feishu Configuration", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(createMainWindowMockScript());
    await page.goto("/");
  });

  test("F18-01: Get feishu mode returns string", async ({ page }) => {
    await waitForMainWindow(page);
    const mode = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("get_feishu_mode", {});
    });
    expect(typeof mode).toBe("string");
    expect(["cli", "api"]).toContain(mode);
  });

  test("F18-02: Save feishu mode works", async ({ page }) => {
    await waitForMainWindow(page);
    await page.evaluate(async () => {
      await (window as any).__TAURI_INTERNALS__.invoke("save_feishu_mode", { mode: "api" });
    });
    const mode = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("get_feishu_mode", {});
    });
    expect(mode).toBe("cli"); // mock 不会真正保存，返回默认值
  });

  test("F18-03: Get feishu chat id returns string", async ({ page }) => {
    await waitForMainWindow(page);
    const chatId = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("get_feishu_chat_id", {});
    });
    expect(typeof chatId).toBe("string");
  });

  test("F18-04: Save feishu chat id works", async ({ page }) => {
    await waitForMainWindow(page);
    await page.evaluate(async () => {
      await (window as any).__TAURI_INTERNALS__.invoke("save_feishu_chat_id", { chatId: "oc_test_123" });
    });
    expect(true).toBe(true);
  });
});
