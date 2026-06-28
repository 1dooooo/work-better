/**
 * F10: MenuBar
 *
 * Tests the MenuBar components: header, content, actions.
 */
import { test, expect } from "@playwright/test";
import { createMainWindowMockScript, waitForMainWindow } from "./helpers";

test.describe("F10: MenuBar", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(createMainWindowMockScript());
    await page.goto("/");
  });

  test("F10-01: System status returns valid data", async ({ page }) => {
    await waitForMainWindow(page);

    const status = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("get_system_status", {});
    });

    expect(status).toHaveProperty("collector_running");
    expect(status).toHaveProperty("scheduler_running");
    expect(status).toHaveProperty("unprocessed_count");
    expect(status).toHaveProperty("today_processed_count");
    expect(typeof status.collector_running).toBe("boolean");
    expect(typeof status.scheduler_running).toBe("boolean");
    expect(typeof status.unprocessed_count).toBe("number");
  });

  test("F10-02: Events can be fetched for menubar display", async ({ page }) => {
    await waitForMainWindow(page);

    const events = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("get_events", { limit: 5 });
    });

    expect(Array.isArray(events)).toBe(true);
  });

  test("F10-03: Pending tasks can be fetched", async ({ page }) => {
    await waitForMainWindow(page);

    const tasks = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("get_pending_tasks", {});
    });

    expect(Array.isArray(tasks)).toBe(true);
  });
});
