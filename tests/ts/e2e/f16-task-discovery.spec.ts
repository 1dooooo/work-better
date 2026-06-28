/**
 * F16: AI Task Discovery
 */
import { test, expect } from "@playwright/test";
import { createMainWindowMockScript, waitForMainWindow } from "./helpers";

test.describe("F16: AI Task Discovery", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(createMainWindowMockScript());
    await page.goto("/");
  });

  test("F16-01: Discover tasks from text returns array", async ({ page }) => {
    await waitForMainWindow(page);
    const tasks = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("discover_tasks_from_text", { text: "明天完成报告", source: "manual" });
    });
    expect(Array.isArray(tasks)).toBe(true);
  });

  test("F16-02: List tasks with filters works", async ({ page }) => {
    await waitForMainWindow(page);
    const tasks = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("list_tasks", { status: null, priority: null });
    });
    expect(Array.isArray(tasks)).toBe(true);
  });

  test("F16-03: Create task with priority works", async ({ page }) => {
    await waitForMainWindow(page);
    const task = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("create_task", { title: "高优先级任务", priority: "high" });
    });
    expect(task).toHaveProperty("id");
    expect(task).toHaveProperty("title");
  });

  test("F16-04: Update task status works", async ({ page }) => {
    await waitForMainWindow(page);
    const result = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("update_task_status", { taskId: "task-1", status: "InProgress" });
    });
    expect(result).toBeNull();
  });
});
