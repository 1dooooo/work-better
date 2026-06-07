import { test as base, expect, type Page } from "@playwright/test";

// ─── Tauri Mock Types ─────────────────────────────────────────────

interface MockEvent {
  id: string;
  timestamp: string;
  collected_at: string;
  source: string;
  source_confidence: string;
  type: string;
  content: unknown;
  raw_payload: string;
  tags: string[];
  related_ids: string[];
  attachments: unknown[];
}

interface CollectorStatus {
  id: string;
  name: string;
  enabled: boolean;
  healthy: boolean;
}

interface StorageConfig {
  vault_path: string;
  db_path: string;
}

interface ModelConfig {
  api_endpoint: string;
  api_key: string;
  token_budget: number;
}

interface TaskInfo {
  id: string;
  name: string;
  layer: string;
  cron: string;
  sla_ms: number;
}

// ─── Default Mock Data ────────────────────────────────────────────

const DEFAULT_EVENTS: MockEvent[] = [
  {
    id: "evt-001",
    timestamp: "2026-06-06T10:00:00Z",
    collected_at: "2026-06-06T10:01:00Z",
    source: "feishu",
    source_confidence: "high",
    type: "message",
    content: "项目进度讨论",
    raw_payload: "{}",
    tags: ["work", "meeting"],
    related_ids: [],
    attachments: [],
  },
  {
    id: "evt-002",
    timestamp: "2026-06-06T11:00:00Z",
    collected_at: "2026-06-06T11:01:00Z",
    source: "manual",
    source_confidence: "high",
    type: "note",
    content: "手动记录的笔记",
    raw_payload: "{}",
    tags: ["note"],
    related_ids: [],
    attachments: [],
  },
];

const DEFAULT_COLLECTORS: CollectorStatus[] = [
  { id: "feishu", name: "飞书采集器", enabled: true, healthy: true },
];

const DEFAULT_STORAGE_CONFIG: StorageConfig = {
  vault_path: "~/Documents/Obsidian",
  db_path: "~/.work-better/data.db",
};

const DEFAULT_MODEL_CONFIG: ModelConfig = {
  api_endpoint: "https://api.openai.com/v1",
  api_key: "sk-test-key",
  token_budget: 4096,
};

const DEFAULT_TASKS: TaskInfo[] = [
  {
    id: "task-001",
    name: "feishu-collect",
    layer: "collector",
    cron: "*/15 * * * *",
    sla_ms: 30000,
  },
  {
    id: "task-002",
    name: "daily-report",
    layer: "report",
    cron: "0 18 * * 1-5",
    sla_ms: 60000,
  },
];

// ─── Mock State ───────────────────────────────────────────────────

export interface MockState {
  events: MockEvent[];
  collectors: CollectorStatus[];
  storageConfig: StorageConfig;
  modelConfig: ModelConfig;
  tasks: TaskInfo[];
  feishuMode: string;
  feishuChatId: string;
  unprocessedCount: number;
  schedulerPaused: boolean;
  collectedCount: number;
  invokeLog: Array<{ cmd: string; args: Record<string, unknown> }>;
}

export function createDefaultMockState(): MockState {
  return {
    events: [...DEFAULT_EVENTS],
    collectors: DEFAULT_COLLECTORS.map((c) => ({ ...c })),
    storageConfig: { ...DEFAULT_STORAGE_CONFIG },
    modelConfig: { ...DEFAULT_MODEL_CONFIG },
    tasks: DEFAULT_TASKS.map((t) => ({ ...t })),
    feishuMode: "cli",
    feishuChatId: "oc_test_chat_id",
    unprocessedCount: 3,
    schedulerPaused: false,
    collectedCount: 0,
    invokeLog: [],
  };
}

// ─── Tauri Invoke Mock Script ─────────────────────────────────────

/**
 * Injects a mock `__TAURI__` and `__TAURI_INTERNALS__` into the page
 * so that `@tauri-apps/api/core` invoke calls are intercepted.
 *
 * Must be called BEFORE page.goto() — uses addInitScript.
 */
export async function injectTauriMock(
  page: Page,
  state: MockState = createDefaultMockState(),
): Promise<void> {
  await page.addInitScript((s: MockState) => {
    // Store state on window for later access
    (window as any).__mockState = s;

    // Mock the Tauri internals invoke function
    (window as any).__TAURI_INTERNALS__ = {
      transformCallback: (cb: Function, once?: boolean) => {
        const id = Math.random().toString(36).slice(2);
        (window as any).__callbacks = (window as any).__callbacks || {};
        (window as any).__callbacks[id] = cb;
        return id;
      },
      invoke: async (cmd: string, args?: Record<string, unknown>) => {
        const st = (window as any).__mockState as MockState;
        st.invokeLog.push({ cmd, args: args ?? {} });

        // Add a small delay so UI loading states are observable
        await new Promise((r) => setTimeout(r, 30));

        switch (cmd) {
          // ── Events ──
          case "get_events":
            return st.events.slice(0, (args?.limit as number) ?? 50);
          case "get_unprocessed_count":
            return st.unprocessedCount;
          case "mark_event_processed":
            st.events = st.events.map((e) =>
              e.id === args?.eventId ? { ...e, processed: true } : e,
            );
            return null;
          case "trigger_manual_capture": {
            const newEvent: MockEvent = {
              id: `evt-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
              timestamp: new Date().toISOString(),
              collected_at: new Date().toISOString(),
              source: "manual",
              source_confidence: "high",
              type: "note",
              content: args?.text ?? "",
              raw_payload: "{}",
              tags: [],
              related_ids: [],
              attachments: (args as any)?.image_data
                ? [{ type: "image", data: "base64..." }]
                : [],
            };
            st.events.unshift(newEvent);
            st.unprocessedCount += 1;
            return newEvent;
          }
          case "hide_capture_window":
            return null;

          // ── Feishu Collection ──
          case "trigger_feishu_collect": {
            const chatId = args?.chatId as string;
            const limit = (args?.limit as number) ?? 20;
            const count = limit <= 5 ? limit : 5;
            for (let i = 0; i < count; i++) {
              st.events.unshift({
                id: `feishu-${Date.now()}-${i}`,
                timestamp: new Date().toISOString(),
                collected_at: new Date().toISOString(),
                source: "feishu",
                source_confidence: "high",
                type: "message",
                content: chatId
                  ? `来自 ${chatId} 的消息 ${i}`
                  : `飞书消息 ${i}`,
                raw_payload: "{}",
                tags: ["feishu"],
                related_ids: [],
                attachments: [],
              });
            }
            st.unprocessedCount += count;
            st.collectedCount = count;
            // Fire feishu:collect-complete event
            setTimeout(() => {
              const cbs = (window as any).__callbacks ?? {};
              Object.values(cbs).forEach((cb: any) => {
                try {
                  cb({ payload: count });
                } catch {}
              });
            }, 50);
            return count;
          }

          // ── Collector Management ──
          case "get_collector_statuses":
            return st.collectors;
          case "list_collectors":
            return st.collectors.map((c) => c.id);
          case "enable_collector":
            st.collectors = st.collectors.map((c) =>
              c.id === args?.id ? { ...c, enabled: true } : c,
            );
            return null;
          case "disable_collector":
            st.collectors = st.collectors.map((c) =>
              c.id === args?.id ? { ...c, enabled: false } : c,
            );
            return null;
          case "check_collector_health": {
            const col = st.collectors.find((c) => c.id === args?.id);
            return {
              level: col?.healthy ? "ok" : "error",
              message: col?.healthy ? null : "Collector unhealthy",
              error_count: col?.healthy ? 0 : 1,
            };
          }

          // ── Feishu Config ──
          case "get_feishu_mode":
            return st.feishuMode;
          case "save_feishu_mode":
            st.feishuMode = args?.mode as string;
            return null;
          case "get_feishu_chat_id":
            return st.feishuChatId;
          case "save_feishu_chat_id":
            st.feishuChatId = args?.chatId as string;
            return null;

          // ── Storage Config ──
          case "get_storage_config":
            return st.storageConfig;
          case "save_storage_config":
            st.storageConfig = args?.config as StorageConfig;
            return null;

          // ── Model Config ──
          case "get_model_config":
            return st.modelConfig;
          case "save_model_config":
            st.modelConfig = args?.config as ModelConfig;
            return null;

          // ── Scheduler ──
          case "list_scheduled_tasks":
            return st.tasks;
          case "pause_scheduler":
            st.schedulerPaused = true;
            return null;
          case "resume_scheduler":
            st.schedulerPaused = false;
            return null;
          case "is_scheduler_paused":
            return st.schedulerPaused;

          default:
            console.warn(`[TauriMock] Unhandled command: ${cmd}`);
            return null;
        }
      },
    };

    // Also mock the event listen system
    (window as any).__TAURI__ = {
      event: {
        listen: async (
          eventName: string,
          handler: (event: { payload: any }) => void,
        ) => {
          (window as any).__eventListeners =
            (window as any).__eventListeners || {};
          (window as any).__eventListeners[eventName] =
            (window as any).__eventListeners[eventName] || [];
          (window as any).__eventListeners[eventName].push(handler);
          return () => {
            const listeners =
              (window as any).__eventListeners[eventName] ?? [];
            const idx = listeners.indexOf(handler);
            if (idx >= 0) listeners.splice(idx, 1);
          };
        },
      },
    };
  }, state);
}

// ─── Browser-Side State Access ────────────────────────────────────

/**
 * Read the mock state back from the browser.
 */
export async function getMockState(page: Page): Promise<MockState> {
  return page.evaluate(() => (window as any).__mockState as MockState);
}

/**
 * Get the invoke log to verify command chains.
 */
export async function getInvokeLog(
  page: Page,
): Promise<Array<{ cmd: string; args: Record<string, unknown> }>> {
  const state = await getMockState(page);
  return state.invokeLog;
}

/**
 * Update the browser-side mock state. Use this when you need to change
 * state AFTER page.goto() has already been called (since addInitScript
 * serializes the state at injection time).
 */
export async function updateBrowserState(
  page: Page,
  updater: (state: MockState) => void,
): Promise<void> {
  await page.evaluate(
    ({ fn }) => {
      const st = (window as any).__mockState as MockState;
      const updater = new Function("state", `return (${fn})(state)`) as (
        s: MockState,
      ) => void;
      updater(st);
    },
    { fn: updater.toString() },
  );
}

/**
 * Override the browser-side invoke handler for a specific command.
 * Useful for simulating errors or custom behavior.
 */
export async function overrideInvoke(
  page: Page,
  handler: (cmd: string, args: Record<string, unknown>) => any,
): Promise<void> {
  await page.evaluate((fn) => {
    const originalInvoke = (window as any).__TAURI_INTERNALS__.invoke;
    const customHandler = new Function(
      "cmd",
      "args",
      "originalInvoke",
      `return (${fn})(cmd, args, originalInvoke)`,
    ) as (
      cmd: string,
      args: Record<string, unknown>,
      orig: Function,
    ) => any;
    (window as any).__TAURI_INTERNALS__.invoke = async (
      cmd: string,
      args?: Record<string, unknown>,
    ) => customHandler(cmd, args ?? {}, originalInvoke);
  }, handler.toString());
}

// ─── Extended Test Fixture ─────────────────────────────────────────

interface TestFixtures {
  mockState: MockState;
}

export const test = base.extend<TestFixtures>({
  mockState: async ({ page }, use) => {
    const state = createDefaultMockState();
    await injectTauriMock(page, state);

    // Navigate to the app after mock injection
    await page.goto("/");

    await use(state);
  },
});

export { expect };

// ─── Common Helpers ────────────────────────────────────────────────

/**
 * Wait for the main window sidebar to be visible.
 */
export async function waitForMainWindow(page: Page): Promise<void> {
  await page.waitForSelector(".sidebar", { timeout: 10000 });
}

/**
 * Navigate to a specific view via sidebar.
 */
export async function navigateToView(
  page: Page,
  viewLabel: string,
): Promise<void> {
  await waitForMainWindow(page);
  await page.click(`.sidebar__item:has-text("${viewLabel}")`);
}

/**
 * Create a mock event with overrides.
 */
export function createMockEvent(
  overrides: Partial<MockEvent> = {},
): MockEvent {
  return {
    id: `evt-${Date.now()}`,
    timestamp: new Date().toISOString(),
    collected_at: new Date().toISOString(),
    source: "manual",
    source_confidence: "high",
    type: "note",
    content: "test content",
    raw_payload: "{}",
    tags: [],
    related_ids: [],
    attachments: [],
    ...overrides,
  };
}
