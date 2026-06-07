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
 */
import {
  test,
  expect,
  getMockState,
  getInvokeLog,
  injectTauriMock,
  createDefaultMockState,
  navigateToView,
  createMockEvent,
} from "./helpers";

test.describe("F3: Processing Pipeline", () => {
  // F3-01: Event -> classifier -> correct processing path
  // This requires a processing UI or at minimum a "process" button on events.
  // The current EventsView only has "标记已处理" (mark processed), not actual
  // processing. When the processing pipeline UI is added, unskip this test.
  test.skip(
    true,
    "TODO F3-01: Requires processing pipeline UI — classifier routing is a backend concern not yet exposed in frontend",
  );
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
    // When implemented, the test should:
    // - Create an event with known content
    // - Mock invoke("process_event") to return { category: "meeting", confidence: 0.95, path: "direct" }
    // - Verify the UI shows the classification result
    // - Verify the correct path indicator is displayed
  });

  // F3-02: Low confidence triggers upgrade to large model
  test.skip(
    true,
    "TODO F3-02: Requires confidence display and model upgrade UI",
  );
  test("F3-02: Low confidence event gets upgraded to large model call", async ({
    page,
  }) => {
    // Spec: When processing returns low confidence (< 0.5):
    // 1. UI should show "low confidence" indicator
    // 2. Automatically retry with a larger model
    // 3. Show "upgraded" status
    // 4. Display the improved result
    //
    // Mock: invoke("process_event") returns { confidence: 0.3 }
    // Then: invoke("process_event", { model: "large" }) returns { confidence: 0.9 }
    // Verify: UI shows both attempts and final result
  });

  // F3-03: Processing output goes to ReviewAgent for approval
  test.skip(
    true,
    "TODO F3-03: Requires review/approval UI in the frontend",
  );
  test("F3-03: Processing output is sent to ReviewAgent for approval", async ({
    page,
  }) => {
    // Spec: After processing completes:
    // 1. UI calls invoke("submit_for_review", { eventId, result })
    // 2. Shows "pending review" status
    // 3. ReviewAgent returns approved or rejected
    // 4. UI updates to show final status
    //
    // Mock: invoke("submit_for_review") returns { status: "approved", reviewer: "ReviewAgent" }
    // Verify: UI shows approved badge
    // Mock: invoke("submit_for_review") returns { status: "rejected", reason: "low quality" }
    // Verify: UI shows rejected badge with reason
  });

  // F3-04: Approved output is persisted to Obsidian, VectorDB, and SQLite
  test.skip(
    true,
    "TODO F3-04: Requires output persistence verification UI",
  );
  test("F3-04: Approved event persists to Obsidian, VectorDB, and SQLite", async ({
    page,
  }) => {
    // Spec: When an event is approved:
    // 1. invoke("persist_event", { eventId }) is called
    // 2. Backend writes to Obsidian vault (markdown file)
    // 3. Backend writes to VectorDB (embedding)
    // 4. Backend writes to SQLite (processed flag + metadata)
    // 5. UI shows "persisted" status with targets
    //
    // Mock: invoke("persist_event") returns {
    //   obsidian: { path: "2026-06-06-note.md", success: true },
    //   vector: { id: "vec-123", success: true },
    //   sqlite: { success: true }
    // }
    // Verify: UI shows all three persistence targets as successful
  });

  // Integration test: Verify that the "标记已处理" button works end-to-end
  test("F3-01-integration: Mark event as processed via UI", async ({
    page,
    mockState,
  }) => {
    // This tests the existing "mark processed" functionality
    await navigateToView(page, "事件");

    // Wait for events to load
    await page.waitForSelector(".events-view__list", { timeout: 5000 });

    // Count initial events
    const eventCards = page.locator(".events-view__card");
    const initialCount = await eventCards.count();
    expect(initialCount).toBeGreaterThan(0);

    // Click "标记已处理" on the first event
    const firstMarkBtn = eventCards.first().locator(".events-view__action-btn");
    await expect(firstMarkBtn).toHaveText("标记已处理");
    await firstMarkBtn.click();

    // Verify the invoke was called
    const log = await getInvokeLog(page);
    const markCall = log.find((l) => l.cmd === "mark_event_processed");
    expect(markCall).toBeDefined();
    expect(markCall!.args.eventId).toBe(mockState.events[0].id);
  });
});
