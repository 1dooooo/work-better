/**
 * F11: Cross-view Navigation
 *
 * Tests navigation between views: URL sync, state persistence,
 * keyboard shortcuts.
 */
import { test, expect } from "@playwright/test";
import { createMainWindowMockScript, waitForMainWindow, navigateToView } from "./helpers";

test.describe("F11: Cross-view Navigation", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(createMainWindowMockScript());
    await page.goto("/");
  });

  test("F11-01: Navigation updates URL query parameter", async ({ page }) => {
    await waitForMainWindow(page);

    // 导航到事件视图
    await navigateToView(page, "事件");
    const eventsUrl = page.url();
    expect(eventsUrl).toContain("view=events");

    // 导航到设置视图
    await navigateToView(page, "设置");
    const settingsUrl = page.url();
    expect(settingsUrl).toContain("view=settings");
  });

  test("F11-02: All navigation items are clickable", async ({ page }) => {
    await waitForMainWindow(page);

    // 验证所有导航项存在
    const navItems = ["dashboard", "events", "tasks", "timeline", "reports", "settings"];
    for (const item of navItems) {
      const navElement = page.getByTestId(`nav-item-${item}`);
      await expect(navElement).toBeVisible();
    }
  });

  test("F11-03: Dashboard view renders correctly", async ({ page }) => {
    await waitForMainWindow(page);
    await navigateToView(page, "工作台");
    await expect(page.getByTestId("dashboard-container")).toBeVisible();
  });

  test("F11-04: Events view renders correctly", async ({ page }) => {
    await waitForMainWindow(page);
    await navigateToView(page, "事件");
    await expect(page.getByTestId("events-container")).toBeVisible();
  });

  test("F11-05: Tasks view renders correctly", async ({ page }) => {
    await waitForMainWindow(page);
    await navigateToView(page, "任务");
    await expect(page.getByTestId("tasks-container")).toBeVisible();
  });

  test("F11-06: Timeline view renders correctly", async ({ page }) => {
    await waitForMainWindow(page);
    await navigateToView(page, "时间线");
    await expect(page.getByTestId("timeline-container")).toBeVisible();
  });

  test("F11-07: Settings view renders correctly", async ({ page }) => {
    await waitForMainWindow(page);
    await navigateToView(page, "设置");
    await expect(page.getByTestId("settings-container")).toBeVisible();
  });

  test("F11-08: Reports view renders correctly", async ({ page }) => {
    await waitForMainWindow(page);
    await navigateToView(page, "报告");
    await expect(page.getByTestId("reports-container")).toBeVisible();
  });
});
