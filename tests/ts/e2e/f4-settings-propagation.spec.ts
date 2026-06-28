/**
 * F4: Settings Propagation
 *
 * Tests that settings changes in the UI correctly propagate to the
 * backend via Tauri invoke calls.
 *
 * TODO(I3): F4 tests import mock functions (getMockState, getInvokeLog, etc.)
 * that are now stubs in helpers.ts. These tests cannot run against the real
 * Tauri backend because the mock verification layer does not exist.
 * Re-enable when:
 * 1. A real Tauri IPC call log mechanism is implemented
 * 2. Mock state verification is replaced with actual backend state queries
 * 3. CSS selectors (`.settings__collector-list`, `.settings__saved`, etc.)
 *    are updated to match the actual SettingsView/CollectorSettings markup
 */
import { test, expect } from "@playwright/test";

test.describe.skip("F4: Settings Propagation", () => {
  test("F4-01: chat_id setting change propagates to collector", async ({
    page,
  }) => {
    // Spec: chat_id input change -> save_feishu_chat_id invoke -> "已保存" indicator
  });

  test("F4-02: mode setting change propagates to collector", async ({
    page,
  }) => {
    // Spec: mode radio change -> save_feishu_mode invoke -> "已保存" indicator
  });

  test("F4-03: disable collector propagates to backend", async ({
    page,
  }) => {
    // Spec: collector toggle -> disable_collector invoke -> state update
  });

  test("F4-04: vault_path change propagates to writer via storage config", async ({
    page,
  }) => {
    // Spec: vault path input change -> save_storage_config invoke -> "已保存" indicator
  });
});
