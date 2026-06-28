/**
 * F17: Collector Groups & Health
 */
import { test, expect } from "@playwright/test";
import { createMainWindowMockScript, waitForMainWindow } from "./helpers";

test.describe("F17: Collector Groups & Health", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(createMainWindowMockScript());
    await page.goto("/");
  });

  test("F17-01: Get collector groups returns data", async ({ page }) => {
    await waitForMainWindow(page);
    const groups = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("get_collector_groups", {});
    });
    expect(Array.isArray(groups)).toBe(true);
    expect(groups.length).toBeGreaterThan(0);
  });

  test("F17-02: Get collector statuses returns data", async ({ page }) => {
    await waitForMainWindow(page);
    const statuses = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("get_collector_statuses", {});
    });
    expect(Array.isArray(statuses)).toBe(true);
  });

  test("F17-03: List collectors returns array", async ({ page }) => {
    await waitForMainWindow(page);
    const collectors = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("list_collectors", {});
    });
    expect(Array.isArray(collectors)).toBe(true);
  });

  test("F17-04: Check collector health returns result", async ({ page }) => {
    await waitForMainWindow(page);
    const health = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("check_collector_health", { id: "feishu" });
    });
    expect(health).toHaveProperty("level");
  });

  test("F17-05: Enable/disable collector group works", async ({ page }) => {
    await waitForMainWindow(page);
    await page.evaluate(async () => {
      await (window as any).__TAURI_INTERNALS__.invoke("enable_collector_group", { groupId: "feishu-group" });
    });
    await page.evaluate(async () => {
      await (window as any).__TAURI_INTERNALS__.invoke("disable_collector_group", { groupId: "feishu-group" });
    });
    expect(true).toBe(true);
  });
});
