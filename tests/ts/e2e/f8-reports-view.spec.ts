/**
 * F8: Reports View
 *
 * Tests the ReportsView: report type selection, placeholder display.
 */
import { test, expect } from "@playwright/test";
import { createMainWindowMockScript, waitForMainWindow, navigateToView } from "./helpers";

test.describe("F8: Reports View", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(createMainWindowMockScript());
    await page.goto("/");
  });

  test("F8-01: Reports view renders correctly", async ({ page }) => {
    await waitForMainWindow(page);
    await navigateToView(page, "报告");
    await expect(page.getByTestId("reports-container")).toBeVisible();
  });

  test("F8-02: Reports view shows placeholder message", async ({ page }) => {
    await waitForMainWindow(page);
    await navigateToView(page, "报告");
    await expect(page.getByTestId("reports-container")).toBeVisible();

    // 验证"即将推出"提示（精确匹配）
    const badge = page.getByText("即将推出", { exact: true });
    await expect(badge).toBeVisible();
  });

  test("F8-03: Report type cards are visible", async ({ page }) => {
    await waitForMainWindow(page);
    await navigateToView(page, "报告");
    await expect(page.getByTestId("reports-container")).toBeVisible();

    // 验证三种报告类型卡片（使用 CardTitle 的精确匹配）
    await expect(page.getByText("日报", { exact: true })).toBeVisible();
    await expect(page.getByText("周报", { exact: true })).toBeVisible();
    await expect(page.getByText("月报", { exact: true })).toBeVisible();
  });
});
