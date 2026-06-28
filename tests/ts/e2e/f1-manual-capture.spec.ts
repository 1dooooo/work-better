/**
 * F1: Manual Capture Flow
 *
 * Tests the manual capture functionality across CaptureWindow and MenuBar.
 * Verifies that typed text triggers Tauri invoke and events appear in state.
 *
 * 运行在真实 Tauri 环境中，使用真实的 Tauri IPC。
 */
import { test, expect } from "@playwright/test";
import { getEvents } from "./helpers";

// TODO: add setupTestEnvironment/cleanupTestEnvironment when backend commands are implemented
// (set_test_mode / cleanup_test_data in Rust backend)

test.describe("F1: Manual Capture Flow", () => {
  test("F1-01: Type text and manual capture creates event in state", async ({
    page,
  }) => {
    // 导航到速记窗口
    await page.goto("/?view=capture");

    // 等待速记窗口加载
    const textarea = page.locator("textarea");
    await expect(textarea).toBeVisible();

    // 输入笔记内容
    await textarea.fill("This is a test capture note");

    // 点击提交按钮
    const submitBtn = page.getByRole("button", { name: "提交" });
    await expect(submitBtn).toBeEnabled();
    await submitBtn.click();

    // 验证成功提示
    await expect(page.getByText("已捕获")).toBeVisible();

    // 等待后端处理完成
    await page.waitForTimeout(1000);

    // 通过 Tauri IPC 查询事件
    const events = await getEvents(page, 10);

    // 验证事件已创建
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
    // 导航到速记窗口
    await page.goto("/?view=capture");

    // 等待速记窗口加载
    const textarea = page.locator("textarea");
    await expect(textarea).toBeVisible();

    // 输入笔记内容
    await textarea.fill("Note with potential image");

    // 验证图片预览区域不可见（没有粘贴图片）
    // CaptureWindow 中图片预览只在粘贴图片后显示
    const previewContainer = page.locator("img[alt='粘贴的图片']");
    await expect(previewContainer).not.toBeVisible();

    // 点击提交按钮
    const submitBtn = page.getByRole("button", { name: "提交" });
    await submitBtn.click();

    // 验证成功提示
    await expect(page.getByText("已捕获")).toBeVisible({ timeout: 5000 });

    // 等待后端处理完成
    await page.waitForTimeout(1000);

    // 通过 Tauri IPC 查询事件
    const events = await getEvents(page, 10);

    // 验证事件已创建
    const captured = events.find(
      (e: any) => e.content === "Note with potential image",
    );
    expect(captured).toBeDefined();

    // 验证图片预览结构不可见（没有粘贴图片）
    await expect(previewContainer).not.toBeVisible();
  });
});
