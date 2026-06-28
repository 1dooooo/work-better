/**
 * F13: Settings Sub-pages
 *
 * Tests all settings tabs: Model, Storage, Shortcut, Freshness, Report, Developer.
 */
import { test, expect } from "@playwright/test";
import { createMainWindowMockScript, waitForMainWindow, navigateToView } from "./helpers";

test.describe("F13: Settings Sub-pages", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(createMainWindowMockScript());
    await page.goto("/");
  });

  test("F13-01: Model settings tab renders", async ({ page }) => {
    await waitForMainWindow(page);
    await navigateToView(page, "设置");
    await expect(page.getByTestId("settings-container")).toBeVisible();

    // 默认就是 model tab
    const modelTab = page.getByTestId("settings-tab-model");
    await expect(modelTab).toBeVisible();
    await modelTab.click();
    await page.waitForTimeout(300);
  });

  test("F13-02: Storage settings tab renders", async ({ page }) => {
    await waitForMainWindow(page);
    await navigateToView(page, "设置");
    await page.getByTestId("settings-tab-storage").click();
    await page.waitForTimeout(300);
  });

  test("F13-03: Shortcut settings tab renders", async ({ page }) => {
    await waitForMainWindow(page);
    await navigateToView(page, "设置");
    await page.getByTestId("settings-tab-shortcuts").click();
    await page.waitForTimeout(300);
  });

  test("F13-04: Freshness settings tab renders", async ({ page }) => {
    await waitForMainWindow(page);
    await navigateToView(page, "设置");
    await page.getByTestId("settings-tab-freshness").click();
    await page.waitForTimeout(300);
  });

  test("F13-05: Report settings tab renders", async ({ page }) => {
    await waitForMainWindow(page);
    await navigateToView(page, "设置");
    await page.getByTestId("settings-tab-reports").click();
    await page.waitForTimeout(300);
  });

  test("F13-06: Developer settings tab renders", async ({ page }) => {
    await waitForMainWindow(page);
    await navigateToView(page, "设置");
    await page.getByTestId("settings-tab-developer").click();
    await page.waitForTimeout(300);
  });

  test("F13-07: All settings tabs are clickable", async ({ page }) => {
    await waitForMainWindow(page);
    await navigateToView(page, "设置");

    const tabs = ["model", "collector", "storage", "shortcuts", "freshness", "reports", "developer"];
    for (const tab of tabs) {
      const tabElement = page.getByTestId(`settings-tab-${tab}`);
      await expect(tabElement).toBeVisible();
    }
  });

  test("F13-08: Model config command returns data", async ({ page }) => {
    await waitForMainWindow(page);

    const config = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("get_model_config", {});
    });
    expect(config).toHaveProperty("provider");
    expect(config).toHaveProperty("model");
  });

  test("F13-09: List models command returns array", async ({ page }) => {
    await waitForMainWindow(page);

    const models = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("list_models", {});
    });
    expect(Array.isArray(models)).toBe(true);
  });

  test("F13-10: Test model command returns result", async ({ page }) => {
    await waitForMainWindow(page);

    const result = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("test_model", { provider: "mock", model: "test" });
    });
    expect(result).toHaveProperty("success");
  });

  test("F13-11: Shortcut config commands work", async ({ page }) => {
    await waitForMainWindow(page);

    const config = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("get_shortcut_config", {});
    });
    expect(Array.isArray(config)).toBe(true);
  });

  test("F13-12: Developer mode toggle works", async ({ page }) => {
    await waitForMainWindow(page);

    // 获取当前开发者模式
    const initial = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("get_developer_mode", {});
    });
    expect(typeof initial).toBe("boolean");

    // 切换开发者模式
    await page.evaluate(async () => {
      await (window as any).__TAURI_INTERNALS__.invoke("save_developer_mode", { enabled: true });
    });

    const updated = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("get_developer_mode", {});
    });
    expect(updated).toBe(true);
  });
});
