/**
 * F1: Manual Capture Flow
 *
 * Tests the manual capture functionality across CaptureWindow and MenuBar.
 * Verifies that typed text triggers Tauri invoke and events appear in state.
 *
 * 使用 addInitScript 注入 mock IPC。
 */
import { test, expect } from "@playwright/test";

// Mock 事件存储
const capturedEvents: any[] = [];

test.describe("F1: Manual Capture Flow", () => {
  // 每个测试前设置 mock
  test.beforeEach(async ({ page }) => {
    // 清空事件
    capturedEvents.length = 0;

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
        // 记录调用
        console.log(`[Mock] invoke: ${cmd}`, args);

        switch (cmd) {
          case "trigger_manual_capture":
            const newEvent = {
              id: `mock-${Date.now()}`,
              timestamp: new Date().toISOString(),
              source: "manual",
              type: "note",
              content: args?.text ?? "",
            };
            // 存储到全局变量
            (window as any).__mockEvents = (window as any).__mockEvents || [];
            (window as any).__mockEvents.unshift(newEvent);
            return newEvent;
          case "get_events":
            return ((window as any).__mockEvents || []).slice(0, args?.limit ?? 50);
          case "get_unprocessed_count":
            return ((window as any).__mockEvents || []).length;
          case "set_test_mode":
            return null;
          case "cleanup_test_data":
            return null;
          case "hide_capture_window":
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
    });

    await page.goto("/?view=capture");
  });

  test("F1-01: Type text and manual capture creates event", async ({
    page,
  }) => {
    // 等待速记窗口加载
    const textarea = page.getByPlaceholder("记录一条想法、笔记或任务...支持粘贴图片");
    await expect(textarea).toBeVisible();

    // 输入笔记内容
    await textarea.fill("This is a test capture note");

    // 点击提交按钮
    const submitBtn = page.getByRole("button", { name: "提交" });
    await expect(submitBtn).toBeEnabled();
    await submitBtn.click();

    // 验证成功提示
    await expect(page.getByText("已捕获")).toBeVisible({ timeout: 5000 });

    // 验证事件已创建（通过页面状态）
    const events = await page.evaluate(() => {
      return (window as any).__mockEvents || [];
    });

    expect(events.length).toBeGreaterThan(0);
    const captured = events.find(
      (e: any) =>
        e.content === "This is a test capture note" && e.source === "manual",
    );
    expect(captured).toBeDefined();
    expect(captured?.type).toBe("note");
  });

  test("F1-02: Capture without image does not show preview", async ({
    page,
  }) => {
    // 等待速记窗口加载
    const textarea = page.getByPlaceholder("记录一条想法、笔记或任务...支持粘贴图片");
    await expect(textarea).toBeVisible();

    // 输入笔记内容
    await textarea.fill("Note without image");

    // 验证图片预览区域不可见（没有粘贴图片）
    await expect(page.locator("img[alt='粘贴的图片']")).not.toBeVisible();

    // 点击提交按钮
    await page.getByRole("button", { name: "提交" }).click();

    // 验证成功提示
    await expect(page.getByText("已捕获")).toBeVisible({ timeout: 5000 });
  });
});
