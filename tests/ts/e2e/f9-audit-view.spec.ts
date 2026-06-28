/**
 * F9: Audit View
 *
 * Tests the AuditView: audit log display, summary stats.
 */
import { test, expect } from "@playwright/test";
import { createMainWindowMockScript, waitForMainWindow } from "./helpers";

test.describe("F9: Audit View", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(createMainWindowMockScript());
    await page.goto("/");
  });

  test("F9-01: Audit commands return data", async ({ page }) => {
    await waitForMainWindow(page);

    // 验证审计相关命令可调用
    const audits = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("get_processing_audits", {});
    });
    expect(Array.isArray(audits)).toBe(true);

    const logs = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("get_execution_logs", {});
    });
    expect(Array.isArray(logs)).toBe(true);

    const summary = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("get_audit_summary", {});
    });
    expect(summary).toHaveProperty("total");
    expect(summary).toHaveProperty("passed");
    expect(summary).toHaveProperty("failed");
  });

  test("F9-02: Audit summary has correct structure", async ({ page }) => {
    await waitForMainWindow(page);

    const summary = await page.evaluate(async () => {
      return await (window as any).__TAURI_INTERNALS__.invoke("get_audit_summary", {});
    });

    expect(typeof summary.total).toBe("number");
    expect(typeof summary.passed).toBe("number");
    expect(typeof summary.failed).toBe("number");
  });
});
