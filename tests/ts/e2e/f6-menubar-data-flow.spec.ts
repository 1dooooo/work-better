/**
 * F6: MenuBar Data Flow
 *
 * Tests the MenuBar component (tray view): event count, health status,
 * and scheduler status flow from backend to the menu bar display.
 *
 * TODO(I3): F6 tests import mock functions (getMockState, getInvokeLog,
 * injectTauriMock, overrideInvoke, etc.) that are now stubs in helpers.ts.
 * The MenuBar/Tray view is not part of the current main window UI and
 * requires a separate Tauri window context.
 * Re-enable when:
 * 1. MenuBar/Tray view is implemented as a Tauri window
 * 2. Mock functions are replaced with real Tauri IPC calls
 * 3. CSS selectors (`.menu-bar`, `.menu-bar__badge`, etc.)
 *    are updated to match the actual MenuBar markup
 */
import { test, expect } from "@playwright/test";

test.describe.skip("F6: MenuBar Data Flow", () => {
  test("F6-01: Event count flows to MenuBar badge", async ({ page }) => {
    // Spec: unprocessed count -> badge display
  });

  test("F6-01b: Event count shows sync status when zero", async ({ page }) => {
    // Spec: zero count -> "已同步" badge
  });

  test("F6-02: Health status flows to MenuBar footer", async ({ page }) => {
    // Spec: health status -> footer indicator
  });

  test("F6-02b: Error status flows to MenuBar footer", async ({ page }) => {
    // Spec: error state -> error indicator
  });

  test("F6-03: Scheduler status flows to MenuBar via event count refresh", async ({
    page,
  }) => {
    // Spec: periodic refresh mechanism
  });

  test("F6-03b: MenuBar capture flow updates count after success", async ({
    page,
  }) => {
    // Spec: capture -> count update
  });
});
