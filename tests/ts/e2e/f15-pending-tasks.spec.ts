/**
 * F15: Pending Task Confirmation
 *
 * Tests confirm/reject pending tasks.
 */
import { test, expect } from "@playwright/test";
import { createMainWindowMockScript, waitForMainWindow } from "./helpers";

test.describe("F15: Pending Task Confirmation", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(createMainWindowMockScript());
    await page.goto("/");
  });

  test("F15-01: Get pending tasks returns array", async ({ page }) => {
    await waitForMainWindow(page);

    const tasks = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("get_pending_tasks", {});
    });
    expect(Array.isArray(tasks)).toBe(true);
  });

  test("F15-02: Confirm pending task returns task", async ({ page }) => {
    await waitForMainWindow(page);

    const task = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("confirm_pending_task", { pendingId: "test-pending-1" });
    });
    expect(task).toHaveProperty("id");
    expect(task).toHaveProperty("title");
  });

  test("F15-03: Reject pending task works", async ({ page }) => {
    await waitForMainWindow(page);

    const result = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("reject_pending_task", { pendingId: "test-pending-1" });
    });
    expect(result).toBeNull();
  });
});
