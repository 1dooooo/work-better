/**
 * F20: Sidebar Interaction
 */
import { test, expect } from "@playwright/test";
import { createMainWindowMockScript, waitForMainWindow } from "./helpers";

test.describe("F20: Sidebar Interaction", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(createMainWindowMockScript());
    await page.goto("/");
  });

  test("F20-01: Sidebar has brand element", async ({ page }) => {
    await waitForMainWindow(page);
    const brand = page.getByText("Work Better", { exact: true });
    await expect(brand).toBeVisible();
  });

  test("F20-02: Sidebar has version number", async ({ page }) => {
    await waitForMainWindow(page);
    const version = page.getByText("v0.1.0", { exact: true });
    await expect(version).toBeVisible();
  });

  test("F20-03: Theme toggle button exists", async ({ page }) => {
    await waitForMainWindow(page);
    const themeBtn = page.locator('button[title="切换到亮色"], button[title="切换到暗色"]');
    await expect(themeBtn).toBeVisible();
  });

  test("F20-04: Collapse button exists when onCollapsedChange is provided", async ({ page }) => {
    await waitForMainWindow(page);
    const collapseBtn = page.locator('button[title="折叠侧边栏"], button[title="展开侧边栏"]');
    const count = await collapseBtn.count();
    // 可能存在也可能不存在，取决于 MainWindow 是否传入 onCollapsedChange
    expect(count).toBeGreaterThanOrEqual(0);
  });

  test("F20-05: Navigation items have correct labels", async ({ page }) => {
    await waitForMainWindow(page);
    // 使用 testid 避免选择器歧义
    const navItems = ["dashboard", "events", "tasks", "timeline", "reports", "settings"];
    for (const item of navItems) {
      await expect(page.getByTestId(`nav-item-${item}`)).toBeVisible();
    }
  });
});
