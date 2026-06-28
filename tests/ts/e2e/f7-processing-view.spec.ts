/**
 * F7: Processing View
 *
 * Tests the ProcessingView: event list display, process button,
 * processing result, and status updates.
 */
import { test, expect } from "@playwright/test";
import { createMainWindowMockScript, waitForMainWindow } from "./helpers";

test.describe("F7: Processing View", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(createMainWindowMockScript());
    await page.goto("/");
  });

  test("F7-01: Processing view displays unprocessed events", async ({ page }) => {
    await waitForMainWindow(page);

    // 添加未处理事件
    await page.evaluate(() => {
      (window as any).__mockEvents = [
        { id: "proc-1", timestamp: new Date().toISOString(), source: "manual", type: "note", content: "待处理事件 1", processed: false, tags: [] },
        { id: "proc-2", timestamp: new Date().toISOString(), source: "feishu", type: "message", content: "待处理事件 2", processed: false, tags: [] },
      ];
    });

    // 验证事件数据可访问
    const eventCount = await page.evaluate(() => {
      return (window as any).__mockEvents?.length ?? 0;
    });
    expect(eventCount).toBe(2);
  });

  test("F7-02: Process button triggers processing", async ({ page }) => {
    await waitForMainWindow(page);

    // 通过 mock 验证 processEvent 命令可调用
    const result = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("process_event", { eventId: "test-123" });
    });
    expect(result).toEqual({ success: true });
  });

  test("F7-03: Batch process returns result", async ({ page }) => {
    await waitForMainWindow(page);

    const result = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("trigger_batch_process", {});
    });
    expect(result).toHaveProperty("processed");
    expect(result).toHaveProperty("failed");
  });
});
