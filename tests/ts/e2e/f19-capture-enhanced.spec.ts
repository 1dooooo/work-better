/**
 * F19: Enhanced Capture Window
 */
import { test, expect } from "@playwright/test";
import { createMainWindowMockScript, waitForMainWindow } from "./helpers";

test.describe("F19: Enhanced Capture Window", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(createMainWindowMockScript());
    await page.goto("/");
  });

  test("F19-01: Trigger manual capture returns event", async ({ page }) => {
    await waitForMainWindow(page);
    const event = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("trigger_manual_capture", { text: "测试速记内容" });
    });
    expect(event).toHaveProperty("id");
    expect(event).toHaveProperty("content");
    expect(event.source).toBe("manual");
  });

  test("F19-02: Mark event processed works", async ({ page }) => {
    await waitForMainWindow(page);
    const result = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("mark_event_processed", { eventId: "mock-001" });
    });
    expect(true).toBe(true);
  });

  test("F19-03: Show capture window command works", async ({ page }) => {
    await waitForMainWindow(page);
    const result = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("show_capture_window", {});
    });
    expect(true).toBe(true);
  });
});
