/**
 * F6: MenuBar Data Flow
 *
 * Tests the MenuBar component (tray view): event count, health status,
 * and scheduler status flow from backend to the menu bar display.
 */
import {
  test,
  expect,
  getMockState,
  getInvokeLog,
  injectTauriMock,
  createDefaultMockState,
  overrideInvoke,
} from "./helpers";

test.describe("F6: MenuBar Data Flow", () => {
  test("F6-01: Event count flows to MenuBar badge", async ({ page }) => {
    // Create state with a specific count and inject before navigation
    const state = createDefaultMockState();
    state.unprocessedCount = 7;
    await injectTauriMock(page, state);

    // Navigate to the tray view
    await page.goto("/?view=tray");

    // Wait for MenuBar to load
    await page.waitForSelector(".menu-bar", { timeout: 5000 });

    // Verify the badge shows the count
    const badge = page.locator(".menu-bar__badge");
    await expect(badge).toBeVisible();
    await expect(badge).toContainText("7 待处理");
    await expect(badge).toHaveAttribute("data-count", "7");

    // Verify get_unprocessed_count was called
    const log = await getInvokeLog(page);
    const countCall = log.find((l) => l.cmd === "get_unprocessed_count");
    expect(countCall).toBeDefined();

    // Verify getEvents was also called (for recent events list)
    const eventsCall = log.find((l) => l.cmd === "get_events");
    expect(eventsCall).toBeDefined();
    expect(eventsCall!.args.limit).toBe(10);
  });

  test("F6-01b: Event count shows sync status when zero", async ({ page }) => {
    const state = createDefaultMockState();
    state.unprocessedCount = 0;
    await injectTauriMock(page, state);

    await page.goto("/?view=tray");
    await page.waitForSelector(".menu-bar", { timeout: 5000 });

    const badge = page.locator(".menu-bar__badge");
    await expect(badge).toContainText("已同步");
    await expect(badge).toHaveAttribute("data-count", "0");
  });

  test("F6-02: Health status flows to MenuBar footer", async ({
    page,
    mockState,
  }) => {
    await page.goto("/?view=tray");
    await page.waitForSelector(".menu-bar", { timeout: 5000 });

    // The MenuBar shows a status indicator in the footer
    const statusEl = page.locator(".menu-bar__status");
    await expect(statusEl).toBeVisible();

    // When status is "idle", it should show "运行中"
    await expect(statusEl).toContainText("运行中");
    await expect(statusEl).toHaveClass(/menu-bar__status--idle/);
  });

  test("F6-02b: Error status flows to MenuBar footer", async ({
    page,
    mockState,
  }) => {
    // mockState fixture injects the mock and navigates to /
    // Now navigate to the tray view
    await page.goto("/?view=tray");
    await page.waitForSelector(".menu-bar", { timeout: 5000 });

    // Override invoke to simulate error on capture
    await overrideInvoke(page, (cmd, args, original) => {
      if (cmd === "trigger_manual_capture") {
        throw new Error("Network error");
      }
      return (original as Function)(cmd, args);
    });

    // Type something and try to capture
    const input = page.locator(".menu-bar__input");
    await input.fill("test error");
    const submitBtn = page.locator(".menu-bar__submit");
    await submitBtn.click();

    // Wait for error state
    await page.waitForTimeout(1000);

    // Verify error status
    const statusEl = page.locator(".menu-bar__status");
    await expect(statusEl).toContainText("操作失败");
    await expect(statusEl).toHaveClass(/menu-bar__status--error/);
  });

  test("F6-03: Scheduler status flows to MenuBar via event count refresh", async ({
    page,
    mockState,
  }) => {
    // The MenuBar doesn't directly show scheduler status, but it refreshes
    // the event count periodically. Verify the refresh mechanism works.
    await page.goto("/?view=tray");
    await page.waitForSelector(".menu-bar", { timeout: 5000 });

    // Verify initial state
    const badge = page.locator(".menu-bar__badge");
    await expect(badge).toContainText("待处理");

    // Click refresh button
    const refreshBtn = page.locator(".menu-bar__refresh");
    await expect(refreshBtn).toBeVisible();
    await refreshBtn.click();

    // Wait for refresh to complete
    await page.waitForTimeout(500);

    // Verify get_unprocessed_count was called again
    const log = await getInvokeLog(page);
    const countCalls = log.filter(
      (l) => l.cmd === "get_unprocessed_count",
    );
    // At least 2 calls: initial load + manual refresh
    expect(countCalls.length).toBeGreaterThanOrEqual(2);
  });

  test("F6-03b: MenuBar capture flow updates count after success", async ({
    page,
  }) => {
    // Create state with specific initial count
    const state = createDefaultMockState();
    state.unprocessedCount = 2;
    await injectTauriMock(page, state);

    await page.goto("/?view=tray");
    await page.waitForSelector(".menu-bar", { timeout: 5000 });

    // Verify initial count
    const badge = page.locator(".menu-bar__badge");
    await expect(badge).toContainText("2 待处理");

    // Type and capture
    const input = page.locator(".menu-bar__input");
    await input.fill("Quick note from menu bar");
    const submitBtn = page.locator(".menu-bar__submit");

    // The mock has a 30ms delay, so loading state should be observable
    await submitBtn.click();

    // Wait for success (the mock resolves quickly, so we just check final state)
    await expect(submitBtn).toContainText("已记录", { timeout: 5000 });

    // After capture, MenuBar refreshes, so count should update
    // The mock adds 1 to unprocessedCount on capture
    await page.waitForTimeout(1000);
    await expect(badge).toContainText("3 待处理");

    // Verify recent events list updated
    const eventList = page.locator(".menu-bar__list");
    await expect(eventList).toBeVisible();
  });
});
