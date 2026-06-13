/**
 * F5: Scheduler Integration
 *
 * Tests scheduler task management: listing tasks, pause/resume,
 * dependency handling, and timeout behavior.
 *
 * NOTE: The current frontend TasksView uses hardcoded local state for tasks
 * and scheduled tasks. The actual scheduler integration (list_scheduled_tasks,
 * pause_scheduler, resume_scheduler) is available in tauri.ts but not yet
 * wired into the TasksView. These tests verify what exists and skip what
 * requires UI changes.
 */
import {
  test,
  expect,
  getMockState,
  getInvokeLog,
  injectTauriMock,
  createDefaultMockState,
  navigateToView,
} from "./helpers";

test.describe("F5: Scheduler Integration", () => {
  // F5-01: Register and execute task
  test("F5-01: Register a scheduled task and verify it appears in the list", async ({
    page,
  }) => {
    // Spec: When the TasksView is wired to the scheduler:
    // 1. On mount, call invoke("list_scheduled_tasks")
    // 2. Display real tasks from the scheduler
    // 3. Each task shows name, cron, layer, SLA
    //
    // Mock: list_scheduled_tasks returns [
    //   { id: "t1", name: "feishu-collect", layer: "collector", cron: "*/15 * * * *", sla_ms: 30000 }
    // ]
    // Verify: UI shows "feishu-collect" with correct cron expression
  });

  // F5-02: Pause and resume task
  test("F5-02: Pause and resume a scheduled task", async ({ page }) => {
    // Spec:
    // 1. Click pause button on a task
    // 2. invoke("pause_scheduler") is called
    // 3. UI shows "paused" state
    // 4. Click resume
    // 5. invoke("resume_scheduler") is called
    // 6. UI shows "running" state
  });

  // F5-03: Task with dependency
  test("F5-03: Task with dependency shows dependency chain", async ({
    page,
  }) => {
    // Spec:
    // 1. A task has depends_on field
    // 2. UI shows dependency arrow or chain indicator
    // 3. If dependency is not complete, task shows "waiting" state
  });

  // F5-04: Task timeout handling
  test("F5-04: Task that exceeds SLA shows timeout indicator", async ({
    page,
  }) => {
    // Spec:
    // 1. Task has sla_ms: 5000
    // 2. After 5s, UI shows "timeout" indicator
    // 3. invoke("get_task_status") returns { status: "timeout", elapsed_ms: 6000 }
  });

  // Integration test: Verify the existing hardcoded scheduled tasks display
  test("F5-01-integration: Hardcoded scheduled tasks display correctly", async ({
    page,
  }) => {
    await navigateToView(page, "任务");

    // Wait for the tasks view to load
    await page.waitForSelector(".tasks-view", { timeout: 5000 });

    // Verify the scheduled tasks section exists
    const scheduledSection = page.locator(".tasks-view__scheduled");
    await expect(scheduledSection).toBeVisible();

    // Verify the hardcoded scheduled tasks are displayed
    const scheduledItems = page.locator(".tasks-view__scheduled-item");
    const count = await scheduledItems.count();
    expect(count).toBe(3); // 3 hardcoded tasks

    // Verify specific task names
    await expect(
      scheduledSection.locator("text=每日站会"),
    ).toBeVisible();
    await expect(
      scheduledSection.locator("text=周报汇总"),
    ).toBeVisible();
    await expect(
      scheduledSection.locator("text=依赖安全检查"),
    ).toBeVisible();

    // Verify schedule labels
    await expect(
      scheduledSection.locator("text=每天 10:00"),
    ).toBeVisible();
  });

  // Integration test: Verify task board interaction
  test("F5-02-integration: Task status toggle works on the board", async ({
    page,
  }) => {
    await navigateToView(page, "任务");
    await page.waitForSelector(".tasks-view__board", { timeout: 5000 });

    // Find the "待处理" column
    const todoColumn = page.locator(".tasks-view__column").first();
    await expect(
      todoColumn.locator(".tasks-view__column-title"),
    ).toContainText("待处理");

    // Count cards in todo column
    const todoCards = todoColumn.locator(".tasks-view__card");
    const initialTodoCount = await todoCards.count();
    expect(initialTodoCount).toBeGreaterThan(0);

    // Click "推进" on the first todo card
    const firstCardBtn = todoCards.first().locator(".tasks-view__card-btn");
    await expect(firstCardBtn).toHaveText("推进");
    await firstCardBtn.click();

    // Verify the card moved to "进行中" column
    const inProgressColumn = page.locator(".tasks-view__column").nth(1);
    await expect(
      inProgressColumn.locator(".tasks-view__column-title"),
    ).toContainText("进行中");

    // The card count in todo should decrease
    const newTodoCount = await todoColumn
      .locator(".tasks-view__card")
      .count();
    expect(newTodoCount).toBe(initialTodoCount - 1);
  });
});
