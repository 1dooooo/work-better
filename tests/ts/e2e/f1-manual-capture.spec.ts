/**
 * F1: Manual Capture Flow
 *
 * Tests the manual capture functionality across CaptureWindow and MenuBar.
 * Verifies that typed text triggers Tauri invoke and events appear in state.
 *
 * 运行在真实 Tauri 环境中，使用测试模式隔离数据。
 */
import { test, expect } from "@playwright/test";
import {
  setupTestEnvironment,
  cleanupTestEnvironment,
  getEvents,
} from "./helpers";

test.describe("F1: Manual Capture Flow", () => {
  // 每个测试前启用测试模式
  test.beforeEach(async ({ page }) => {
    await page.goto("/?view=capture");
    await setupTestEnvironment(page);
  });

  // 每个测试后清理
  test.afterEach(async ({ page }) => {
    await cleanupTestEnvironment(page);
  });

  test("F1-01: Type text and manual capture creates event", async ({
    page,
  }) => {
    // 等待速记窗口加载
    const textarea = page.getByPlaceholder("记录点什么...");
    await expect(textarea).toBeVisible();

    // 输入笔记内容
    await textarea.fill("This is a test capture note");

    // 点击提交按钮
    const submitBtn = page.getByRole("button", { name: "提交" });
    await expect(submitBtn).toBeEnabled();
    await submitBtn.click();

    // 验证成功提示
    await expect(page.getByText("已捕获")).toBeVisible({ timeout: 5000 });

    // 等待后端处理完成（使用 expect.poll 替代 waitForTimeout）
    await expect
      .poll(
        async () => {
          const events = await getEvents(page, 10);
          return events.length;
        },
        { timeout: 5000 },
      )
      .toBeGreaterThan(0);

    // 验证事件已创建
    const events = await getEvents(page, 10);
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
    const textarea = page.getByPlaceholder("记录点什么...");
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
