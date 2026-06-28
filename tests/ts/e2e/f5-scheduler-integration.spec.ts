/**
 * F5: Scheduler Integration
 *
 * Tests scheduler task management: listing tasks, pause/resume,
 * dependency handling, and timeout behavior.
 *
 * TODO(I3): F5 tests import mock functions (getMockState, getInvokeLog, etc.)
 * that are now stubs in helpers.ts. The current frontend TasksView uses
 * hardcoded local state and is not wired to the real scheduler backend.
 * Re-enable when:
 * 1. TasksView is connected to real Tauri scheduler commands
 * 2. Mock functions are replaced with real Tauri IPC calls
 * 3. CSS selectors (`.tasks-view`, `.tasks-view__board`, etc.)
 *    are updated to match the actual TasksView markup
 */
import { test, expect } from "@playwright/test";

test.describe.skip("F5: Scheduler Integration", () => {
  test("F5-01: Register a scheduled task and verify it appears in the list", async ({
    page,
  }) => {
    // Spec: list_scheduled_tasks -> UI displays real tasks
  });

  test("F5-02: Pause and resume a scheduled task", async ({ page }) => {
    // Spec: pause/resume buttons -> pause_scheduler/resume_scheduler invoke
  });

  test("F5-03: Task with dependency shows dependency chain", async ({
    page,
  }) => {
    // Spec: depends_on field -> UI shows dependency indicator
  });

  test("F5-04: Task that exceeds SLA shows timeout indicator", async ({
    page,
  }) => {
    // Spec: sla_ms exceeded -> timeout indicator in UI
  });

  test("F5-01-integration: Hardcoded scheduled tasks display correctly", async ({
    page,
  }) => {
    // Spec: TasksView shows hardcoded scheduled tasks
  });

  test("F5-02-integration: Task status toggle works on the board", async ({
    page,
  }) => {
    // Spec: Task board columns and card movement
  });
});
