/**
 * F3: Processing Pipeline
 *
 * Tests the processing pipeline: classifier routing, confidence-based
 * model selection, review approval/rejection, and output persistence.
 *
 * NOTE: These scenarios test backend processing logic that is not directly
 * exposed in the current frontend UI. They are marked as skip with TODO
 * comments indicating what UI surface would need to exist to run them.
 * The tests are written as specifications for when the processing UI
 * is implemented.
 *
 * TODO(I3): F3 tests import mock functions (getMockState, getInvokeLog, etc.)
 * that are now stubs in helpers.ts. These tests cannot run against the real
 * Tauri backend because the required commands (process_event, submit_for_review,
 * persist_event) do not exist yet. Re-enable when:
 * 1. Processing UI is implemented in the frontend
 * 2. Corresponding Tauri commands are added to the Rust backend
 * 3. Mock functions are replaced with real Tauri IPC calls
 */
import { test, expect } from "@playwright/test";

test.describe.skip("F3: Processing Pipeline", () => {
  // F3-01: Event -> classifier -> correct processing path
  test("F3-01: Event triggers classifier and routes to correct processing path", async ({
    page,
  }) => {
    // Spec: When a user triggers processing on an event, the UI should:
    // 1. Call invoke("process_event", { eventId })
    // 2. Show a loading state while processing
    // 3. Display the classification result (category, confidence)
    // 4. Route to the correct processing path display
    //
    // Current gap: No process_event command or processing UI exists.
  });

  // F3-02: Low confidence triggers upgrade to large model
  test("F3-02: Low confidence event gets upgraded to large model call", async ({
    page,
  }) => {
    // Spec: When processing returns low confidence (< 0.5):
    // 1. UI should show "low confidence" indicator
    // 2. Automatically retry with a larger model
    // 3. Show "upgraded" status
    // 4. Display the improved result
  });

  // F3-03: Processing output goes to ReviewAgent for approval
  test("F3-03: Processing output is sent to ReviewAgent for approval", async ({
    page,
  }) => {
    // Spec: After processing completes:
    // 1. UI calls invoke("submit_for_review", { eventId, result })
    // 2. Shows "pending review" status
    // 3. ReviewAgent returns approved or rejected
    // 4. UI updates to show final status
  });

  // F3-04: Approved output is persisted to Obsidian, VectorDB, and SQLite
  test("F3-04: Approved event persists to Obsidian, VectorDB, and SQLite", async ({
    page,
  }) => {
    // Spec: When an event is approved:
    // 1. invoke("persist_event", { eventId }) is called
    // 2. Backend writes to Obsidian vault (markdown file)
    // 3. Backend writes to VectorDB (embedding)
    // 4. Backend writes to SQLite (processed flag + metadata)
    // 5. UI shows "persisted" status with targets
  });

  // Integration test: Verify that the "标记已处理" button works end-to-end
  test("F3-01-integration: Mark event as processed via UI", async ({
    page,
  }) => {
    // This tests the existing "mark processed" functionality
    // Requires real Tauri environment with events loaded
  });
});
