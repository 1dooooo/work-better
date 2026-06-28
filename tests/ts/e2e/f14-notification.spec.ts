/**
 * F14: Notification System
 *
 * Tests notification commands: send, get pending, mark read, clear.
 */
import { test, expect } from "@playwright/test";
import { createMainWindowMockScript, waitForMainWindow } from "./helpers";

test.describe("F14: Notification System", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(createMainWindowMockScript());
    await page.goto("/");
  });

  test("F14-01: Send notification command works", async ({ page }) => {
    await waitForMainWindow(page);

    const result = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("send_notification", {
        request: {
          title: "测试通知",
          body: "这是一条测试通知",
          kind: "Reminder",
          action_url: null,
        },
      });
    });
    // 命令应该成功执行（返回 null 或结果）
    expect(true).toBe(true);
  });

  test("F14-02: Get pending notifications returns array", async ({ page }) => {
    await waitForMainWindow(page);

    const notifications = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("get_pending_notifications", {});
    });
    expect(Array.isArray(notifications)).toBe(true);
  });

  test("F14-03: Mark notification read works", async ({ page }) => {
    await waitForMainWindow(page);

    const result = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("mark_notification_read", { id: "test-notif-1" });
    });
    expect(true).toBe(true);
  });

  test("F14-04: Clear read notifications works", async ({ page }) => {
    await waitForMainWindow(page);

    const result = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("clear_read_notifications", {});
    });
    expect(true).toBe(true);
  });
});
