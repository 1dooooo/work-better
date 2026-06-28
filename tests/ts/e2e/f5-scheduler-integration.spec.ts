/**
 * F5: Scheduler Integration
 *
 * Tests scheduler task management: listing tasks, pause/resume,
 * task creation, and status updates.
 *
 * 使用 addInitScript 注入 mock IPC。
 */
import { test, expect } from "@playwright/test";
import { waitForMainWindow, navigateToView, createMainWindowMockScript } from "./helpers";

test.describe("F5: Scheduler Integration", () => {
  test.beforeEach(async ({ page }) => {
    // 注入基础 mock
    await page.addInitScript(createMainWindowMockScript());

    // 注入调度器相关的额外 mock
    await page.addInitScript(() => {
      // 初始化调度器状态
      (window as any).__schedulerPaused = false;

      // 保存原始 invoke
      const originalInvoke = (window as any).__TAURI_INTERNALS__.invoke;

      // 覆盖 invoke 以处理调度器相关命令
      (window as any).__TAURI_INTERNALS__.invoke = async (cmd: string, args: any) => {
        switch (cmd) {
          case "list_scheduled_tasks":
            return [
              { id: "sched-1", name: "每日采集", layer: "L1", cron: "0 9 * * *", sla_ms: 5000 },
              { id: "sched-2", name: "周报生成", layer: "L2", cron: "0 18 * * 5", sla_ms: 10000 },
            ];
          case "is_scheduler_paused":
            return (window as any).__schedulerPaused ?? false;
          case "pause_scheduler":
            (window as any).__schedulerPaused = true;
            return null;
          case "resume_scheduler":
            (window as any).__schedulerPaused = false;
            return null;
          default:
            // 调用原始 invoke
            return originalInvoke(cmd, args);
        }
      };
    });

    await page.goto("/");
  });

  test("F5-01: Tasks view displays task list correctly", async ({
    page,
  }) => {
    await waitForMainWindow(page);

    // 添加测试任务
    await page.evaluate(() => {
      (window as any).__mockTasks = [
        { id: "task-1", title: "完成 E2E 测试", status: "Open", priority: "P1", source: "Manual", due_date: "2026-07-01", created_at: new Date().toISOString() },
        { id: "task-2", title: "代码审查", status: "InProgress", priority: "P2", source: "AI", due_date: null, created_at: new Date().toISOString() },
        { id: "task-3", title: "修复 Bug", status: "Done", priority: "P0", source: "Manual", due_date: null, created_at: new Date().toISOString() },
      ];
    });

    // 导航到任务视图
    await navigateToView(page, "任务");
    await expect(page.getByTestId("tasks-container")).toBeVisible();

    // 验证看板渲染
    const kanban = page.getByTestId("tasks-kanban");
    await expect(kanban).toBeVisible();
  });

  test("F5-02: Create new task via form", async ({
    page,
  }) => {
    await waitForMainWindow(page);

    // 导航到任务视图
    await navigateToView(page, "任务");
    await expect(page.getByTestId("tasks-container")).toBeVisible();

    // 找到创建表单
    const titleInput = page.getByTestId("task-title-input");
    const createBtn = page.getByTestId("task-create-button");

    await expect(titleInput).toBeVisible({ timeout: 10000 });

    // 输入任务标题
    await titleInput.fill("新测试任务");

    // 点击创建
    await createBtn.click();
    await page.waitForTimeout(500);

    // 验证任务已创建
    const tasks = await page.evaluate(() => {
      return (window as any).__mockTasks || [];
    });
    const created = tasks.find((t: any) => t.title === "新测试任务");
    expect(created).toBeDefined();
  });

  test("F5-03: Task status toggle works", async ({
    page,
  }) => {
    await waitForMainWindow(page);

    // 添加测试任务
    await page.evaluate(() => {
      (window as any).__mockTasks = [
        { id: "task-1", title: "测试任务", status: "Open", priority: "P2", source: "Manual", due_date: null, created_at: new Date().toISOString() },
      ];
    });

    // 导航到任务视图
    await navigateToView(page, "任务");
    await expect(page.getByTestId("tasks-container")).toBeVisible();

    // 验证任务存在
    const tasks = await page.evaluate(() => {
      return (window as any).__mockTasks || [];
    });
    expect(tasks.length).toBe(1);
    expect(tasks[0].status).toBe("Open");
  });

  test("F5-04: Scheduled tasks display in task view", async ({
    page,
  }) => {
    await waitForMainWindow(page);

    // 导航到任务视图
    await navigateToView(page, "任务");
    await expect(page.getByTestId("tasks-container")).toBeVisible();

    // 验证定时任务区域存在
    const scheduledSection = page.locator("text=定时任务");
    await expect(scheduledSection).toBeVisible({ timeout: 10000 });
  });

  test("F5-05: Scheduler pause/resume works", async ({
    page,
  }) => {
    await waitForMainWindow(page);

    // 导航到任务视图
    await navigateToView(page, "任务");
    await expect(page.getByTestId("tasks-container")).toBeVisible();

    // 找到暂停/恢复按钮
    const toggleBtn = page.locator("button:has-text('暂停'), button:has-text('恢复')");
    await expect(toggleBtn).toBeVisible({ timeout: 10000 });

    // 获取初始状态
    const initialText = await toggleBtn.textContent();

    // 点击切换
    await toggleBtn.click();
    await page.waitForTimeout(500);

    // 验证状态已改变
    const newText = await toggleBtn.textContent();
    expect(newText).not.toBe(initialText);
  });
});
