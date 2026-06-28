# E2E 测试体系改进实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 F1（手动捕获）和 F2（飞书采集）两个 E2E 测试场景在真实 Tauri 环境中运行，验证完整的端到端链路。

**Architecture:** 使用 `cargo tauri dev` 启动真实的 Tauri app，Playwright 连接到运行中的 app 进行测试。移除浏览器侧的 mock 层，改用真实的 Tauri IPC 通信。飞书 API 使用 MSW mock server 隔离外部依赖。

**Tech Stack:** Playwright, Tauri, MSW (Mock Service Worker), TypeScript

## Global Constraints

- 测试必须在真实 Tauri app 中运行，不能使用浏览器侧 mock
- 飞书 API 必须使用 mock server 隔离，不能调用真实 API
- 测试数据必须使用临时目录，测试后清理
- 测试时间目标：< 30 秒（两个场景）
- 提供 `pnpm test:e2e:dev` 命令供开发者使用

---

### Task 1: 配置 Playwright 使用真实 Tauri 环境

**Files:**
- Modify: `playwright.config.ts`

**Interfaces:**
- Produces: Playwright 配置，使用 `cargo tauri dev` 作为 webServer

- [ ] **Step 1: 修改 Playwright 配置**

修改 `playwright.config.ts`，将 webServer 从 `vite preview` 改为 `cargo tauri dev`：

```typescript
import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests/ts/e2e",
  timeout: 60000, // 增加超时时间，Tauri 启动需要时间
  projects: [
    {
      name: "chromium",
      use: {
        browserName: "chromium",
      },
    },
  ],
  use: {
    baseURL: "http://localhost:1420", // Tauri dev server 默认端口
  },
  webServer: {
    command: "cargo tauri dev", // 启动真实 Tauri app
    port: 1420,
    reuseExistingServer: !process.env.CI,
    timeout: 120000, // Tauri 编译需要较长时间
  },
});
```

- [ ] **Step 2: 验证配置**

运行以下命令验证 Playwright 能连接到 Tauri app：

```bash
pnpm test:e2e --list
```

Expected: 列出所有 E2E 测试文件，无错误

- [ ] **Step 3: Commit**

```bash
git add playwright.config.ts
git commit -m "config: 配置 Playwright 使用真实 Tauri 环境"
```

---

### Task 2: 创建飞书 API Mock Server

**Files:**
- Create: `tests/ts/e2e/mock-feishu-server.ts`

**Interfaces:**
- Produces: `feishuServer` - MSW server 实例，模拟飞书 API

- [ ] **Step 1: 创建飞书 API Mock Server**

创建 `tests/ts/e2e/mock-feishu-server.ts`，模拟飞书 API 响应：

```typescript
/**
 * 飞书 API Mock Server
 *
 * 使用 MSW 模拟飞书 API 响应，用于 E2E 测试隔离外部依赖。
 */
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";

/**
 * 飞书消息 API mock handlers
 */
const handlers = [
  // 获取消息列表
  http.post("https://open.feishu.cn/open-apis/im/v1/messages", () => {
    return HttpResponse.json({
      code: 0,
      msg: "success",
      data: {
        items: [
          {
            message_id: "mock-msg-001",
            msg_type: "text",
            create_time: Date.now().toString(),
            body: {
              content: JSON.stringify({ text: "飞书消息 1" }),
            },
            sender: {
              sender_id: {
                open_id: "mock-user-001",
              },
              sender_type: "user",
            },
          },
          {
            message_id: "mock-msg-002",
            msg_type: "text",
            create_time: Date.now().toString(),
            body: {
              content: JSON.stringify({ text: "飞书消息 2" }),
            },
            sender: {
              sender_id: {
                open_id: "mock-user-002",
              },
              sender_type: "user",
            },
          },
        ],
        has_more: false,
        page_token: null,
      },
    });
  }),

  // 获取会话信息
  http.get(
    "https://open.feishu.cn/open-apis/im/v1/chats/:chatId",
    ({ params }) => {
      return HttpResponse.json({
        code: 0,
        msg: "success",
        data: {
          chat_id: params.chatId,
          name: "测试会话",
          chat_type: "group",
        },
      });
    },
  ),
];

/**
 * 飞书 API Mock Server 实例
 *
 * 使用方法：
 * - 测试前：feishuServer.listen()
 * - 测试后：feishuServer.close()
 * - 重置 handlers：feishuServer.resetHandlers()
 */
export const feishuServer = setupServer(...handlers);
```

- [ ] **Step 2: 验证 mock server 能正常工作**

创建临时测试文件验证 mock server：

```bash
cat > /tmp/test-mock-server.ts << 'EOF'
import { feishuServer } from "./tests/ts/e2e/mock-feishu-server";

feishuServer.listen();

fetch("https://open.feishu.cn/open-apis/im/v1/messages", {
  method: "POST",
})
  .then((res) => res.json())
  .then((data) => {
    console.log("Mock server response:", data);
    feishuServer.close();
  });
EOF
```

Expected: 输出包含 mock 消息数据的 JSON

- [ ] **Step 3: Commit**

```bash
git add tests/ts/e2e/mock-feishu-server.ts
git commit -m "test: 创建飞书 API Mock Server"
```

---

### Task 3: 重写 helpers.ts，移除 mock 层

**Files:**
- Modify: `tests/ts/e2e/helpers.ts`

**Interfaces:**
- Produces: `waitForMainWindow`, `navigateToView`, `setupTestEnvironment`, `cleanupTestEnvironment`

- [ ] **Step 1: 重写 helpers.ts**

完全重写 `tests/ts/e2e/helpers.ts`，移除所有 mock 相关代码，保留辅助函数：

```typescript
/**
 * E2E 测试辅助函数
 *
 * 提供真实 Tauri 环境下的测试辅助功能。
 * 移除了所有 mock 相关代码，使用真实的 Tauri IPC。
 */
import { test as base, expect, type Page } from "@playwright/test";

// ─── 测试环境配置 ─────────────────────────────────────────────

/**
 * 设置测试环境
 *
 * 通过 Tauri IPC 设置测试模式，使用临时数据目录。
 * 必须在 page.goto() 之后调用。
 */
export async function setupTestEnvironment(page: Page): Promise<string> {
  const testDir = `/tmp/work-better-test-${Date.now()}`;

  await page.evaluate((dataDir) => {
    return (window as any).__TAURI__.core.invoke("set_test_mode", {
      enabled: true,
      data_dir: dataDir,
    });
  }, testDir);

  return testDir;
}

/**
 * 清理测试环境
 *
 * 清理测试数据，必须在每个测试结束后调用。
 */
export async function cleanupTestEnvironment(page: Page): Promise<void> {
  await page.evaluate(() => {
    return (window as any).__TAURI__.core.invoke("cleanup_test_data");
  });
}

// ─── 窗口导航 ─────────────────────────────────────────────────

/**
 * 等待主窗口加载完成
 */
export async function waitForMainWindow(page: Page): Promise<void> {
  await page.waitForSelector(".sidebar", { timeout: 30000 });
}

/**
 * 导航到指定视图
 *
 * @param page - Playwright page 对象
 * @param viewLabel - 视图标签文本（如 "事件"、"设置"）
 */
export async function navigateToView(
  page: Page,
  viewLabel: string,
): Promise<void> {
  await waitForMainWindow(page);
  await page.click(`.sidebar__item:has-text("${viewLabel}")`);
}

// ─── 事件查询 ─────────────────────────────────────────────────

/**
 * 通过 Tauri IPC 查询事件列表
 */
export async function getEvents(
  page: Page,
  limit: number = 50,
  offset: number = 0,
): Promise<any[]> {
  return page.evaluate(
    ({ limit, offset }) => {
      return (window as any).__TAURI__.core.invoke("get_events", {
        limit,
        offset,
      });
    },
    { limit, offset },
  );
}

/**
 * 通过 Tauri IPC 查询未处理事件数量
 */
export async function getUnprocessedCount(page: Page): Promise<number> {
  return page.evaluate(() => {
    return (window as any).__TAURI__.core.invoke("get_unprocessed_count");
  });
}

// ─── 导出测试框架 ─────────────────────────────────────────────

export { expect };
export { test as base };
```

- [ ] **Step 2: 验证辅助函数**

运行现有测试，确保辅助函数能正常工作：

```bash
pnpm test:e2e --grep "F1-01"
```

Expected: 测试可能失败（因为还没迁移），但辅助函数不应报错

- [ ] **Step 3: Commit**

```bash
git add tests/ts/e2e/helpers.ts
git commit -m "refactor: 重写 helpers.ts，移除 mock 层"
```

---

### Task 4: 迁移 F1 测试到真实 IPC

**Files:**
- Modify: `tests/ts/e2e/f1-manual-capture.spec.ts`

**Interfaces:**
- Consumes: `waitForMainWindow`, `navigateToView`, `setupTestEnvironment`, `cleanupTestEnvironment`, `getEvents` from helpers
- Consumes: `feishuServer` from mock-feishu-server

- [ ] **Step 1: 修改 F1 测试文件**

修改 `tests/ts/e2e/f1-manual-capture.spec.ts`，使用真实 IPC：

```typescript
/**
 * F1: Manual Capture Flow
 *
 * Tests the manual capture functionality across CaptureWindow and MenuBar.
 * Verifies that typed text triggers Tauri invoke and events appear in state.
 *
 * 运行在真实 Tauri 环境中，使用真实的 Tauri IPC。
 */
import { test, expect } from "@playwright/test";
import {
  waitForMainWindow,
  navigateToView,
  setupTestEnvironment,
  cleanupTestEnvironment,
  getEvents,
} from "./helpers";

test.describe("F1: Manual Capture Flow", () => {
  test("F1-01: Type text and manual capture creates event in state", async ({
    page,
  }) => {
    // 导航到速记窗口
    await page.goto("/?view=capture");

    // 等待速记窗口加载
    const textarea = page.locator(".capture__input");
    await expect(textarea).toBeVisible();

    // 输入笔记内容
    await textarea.fill("This is a test capture note");

    // 点击提交按钮
    const submitBtn = page.locator(".capture__submit");
    await expect(submitBtn).toBeEnabled();
    await submitBtn.click();

    // 验证成功提示
    await expect(page.locator(".capture__toast--success")).toBeVisible();
    await expect(page.locator(".capture__toast--success")).toHaveText("已捕获");

    // 等待后端处理完成
    await page.waitForTimeout(1000);

    // 通过 Tauri IPC 查询事件
    const events = await getEvents(page, 10, 0);

    // 验证事件已创建
    const captured = events.find(
      (e: any) =>
        e.content === "This is a test capture note" && e.source === "manual",
    );
    expect(captured).toBeDefined();
    expect(captured.type).toBe("note");
  });

  test("F1-02: Capture with image attachment records attachment metadata", async ({
    page,
  }) => {
    // 导航到速记窗口
    await page.goto("/?view=capture");

    // 等待速记窗口加载
    const textarea = page.locator(".capture__input");
    await expect(textarea).toBeVisible();

    // 输入笔记内容
    await textarea.fill("Note with potential image");

    // 验证图片预览区域不可见（没有粘贴图片）
    await expect(page.locator(".capture__image-preview")).not.toBeVisible();

    // 点击提交按钮
    const submitBtn = page.locator(".capture__submit");
    await submitBtn.click();

    // 验证成功提示
    await expect(page.locator(".capture__toast--success")).toBeVisible({
      timeout: 5000,
    });

    // 等待后端处理完成
    await page.waitForTimeout(1000);

    // 通过 Tauri IPC 查询事件
    const events = await getEvents(page, 10, 0);

    // 验证事件已创建
    const captured = events.find(
      (e: any) => e.content === "Note with potential image",
    );
    expect(captured).toBeDefined();

    // 验证图片预览结构存在（CSS class）
    const previewContainer = page.locator(".capture__image-preview");
    await expect(previewContainer).not.toBeVisible();
  });
});
```

- [ ] **Step 2: 运行 F1 测试**

运行 F1 测试，验证能在真实 Tauri 环境中运行：

```bash
pnpm test:e2e --grep "F1"
```

Expected: 测试通过，或因 UI 元素选择器问题失败（非 mock 问题）

- [ ] **Step 3: 修复测试问题（如有）**

如果测试失败，检查错误原因：
- 如果是选择器问题：更新 `.capture__input`、`.capture__submit` 等选择器
- 如果是超时问题：增加等待时间
- 如果是 IPC 问题：检查 Tauri 命令是否正确

- [ ] **Step 4: Commit**

```bash
git add tests/ts/e2e/f1-manual-capture.spec.ts
git commit -m "test: 迁移 F1 测试到真实 Tauri IPC"
```

---

### Task 5: 迁移 F2 测试到真实 IPC + mock server

**Files:**
- Modify: `tests/ts/e2e/f2-feishu-collection.spec.ts`

**Interfaces:**
- Consumes: `waitForMainWindow`, `navigateToView`, `setupTestEnvironment`, `cleanupTestEnvironment`, `getEvents`, `getUnprocessedCount` from helpers
- Consumes: `feishuServer` from mock-feishu-server

- [ ] **Step 1: 修改 F2 测试文件**

修改 `tests/ts/e2e/f2-feishu-collection.spec.ts`，使用真实 IPC + mock server：

```typescript
/**
 * F2: Feishu Collection Flow
 *
 * Tests the Feishu (Lark) collection flow: UI triggers collection,
 * lark-cli mock returns events, UI updates count.
 *
 * 运行在真实 Tauri 环境中，使用真实的 Tauri IPC。
 * 飞书 API 使用 MSW mock server 隔离。
 */
import { test, expect } from "@playwright/test";
import {
  waitForMainWindow,
  navigateToView,
  setupTestEnvironment,
  cleanupTestEnvironment,
  getEvents,
  getUnprocessedCount,
} from "./helpers";
import { feishuServer } from "./mock-feishu-server";

// 启动飞书 API mock server
test.beforeAll(() => {
  feishuServer.listen();
});

test.afterAll(() => {
  feishuServer.close();
});

test.afterEach(() => {
  feishuServer.resetHandlers();
});

test.describe("F2: Feishu Collection Flow", () => {
  test("F2-01: UI triggers collection via lark-cli and updates event count", async ({
    page,
  }) => {
    // 导航到事件视图
    await page.goto("/");
    await navigateToView(page, "事件");

    // 等待事件视图加载
    await expect(page.locator(".events-view")).toBeVisible();
    await expect(page.locator(".view__title")).toHaveText("事件流");

    // 记录初始事件数量
    const initialCount = await getUnprocessedCount(page);

    // 点击 "采集飞书" 按钮
    const collectBtn = page.locator("button:has-text('采集飞书')");
    await expect(collectBtn).toBeVisible();
    await expect(collectBtn).toBeEnabled();

    await collectBtn.click();

    // 等待采集完成（按钮文本恢复）
    await expect(collectBtn).toHaveText("采集飞书", { timeout: 5000 });

    // 等待后端处理完成
    await page.waitForTimeout(1000);

    // 验证新事件出现
    const events = await getEvents(page, 50, 0);
    const feishuEvents = events.filter((e: any) => e.source === "feishu");
    expect(feishuEvents.length).toBeGreaterThan(0);

    // 验证未处理事件数量增加
    const newCount = await getUnprocessedCount(page);
    expect(newCount).toBeGreaterThan(initialCount);
  });

  test("F2-02: Specified chat_id overrides config when collecting", async ({
    page,
  }) => {
    // 导航到事件视图
    await page.goto("/");
    await navigateToView(page, "事件");

    // 等待事件视图加载
    await page.waitForSelector(".events-view", { timeout: 5000 });

    // 点击采集按钮
    const collectBtn = page.locator("button:has-text('采集飞书')");
    await collectBtn.click();

    // 等待采集完成
    await expect(collectBtn).toHaveText("采集飞书", { timeout: 5000 });

    // 导航到设置页面
    await navigateToView(page, "设置");

    // 找到会话 ID 输入框并修改
    const chatIdInput = page.locator('input[placeholder="输入飞书会话 ID"]');
    await expect(chatIdInput).toBeVisible();
    await chatIdInput.fill("oc_new_chat_id");
    await chatIdInput.blur(); // 触发保存

    // 等待保存完成
    await page.waitForTimeout(500);

    // 返回事件视图
    await navigateToView(page, "事件");

    // 再次采集
    await page.locator("button:has-text('采集飞书')").click();
    await page.waitForTimeout(1000);

    // 验证使用了新的会话 ID
    const events = await getEvents(page, 50, 0);
    const newEvents = events.filter(
      (e: any) => e.content && e.content.includes("oc_new_chat_id"),
    );
    expect(newEvents.length).toBeGreaterThan(0);
  });

  test("F2-03: Disabled collector returns error on collection attempt", async ({
    page,
  }) => {
    // 导航到设置页面
    await page.goto("/");
    await navigateToView(page, "设置");

    // 等待设置页面加载
    await page.waitForSelector(".settings-view", { timeout: 5000 });

    // 找到飞书采集器开关并禁用
    const feishuToggle = page.locator(
      '[data-collector="feishu"] .collector-toggle',
    );
    await expect(feishuToggle).toBeVisible();
    await feishuToggle.click();

    // 等待状态更新
    await page.waitForTimeout(500);

    // 导航到事件视图
    await navigateToView(page, "事件");

    // 尝试采集
    const collectBtn = page.locator("button:has-text('采集飞书')");
    await collectBtn.click();

    // 等待错误出现
    await page.waitForTimeout(1000);

    // 验证错误消息
    const errorMsg = page.locator(".events-view__error");
    await expect(errorMsg).toBeVisible();
    await expect(errorMsg).toContainText("采集失败");
  });
});
```

- [ ] **Step 2: 运行 F2 测试**

运行 F2 测试，验证能在真实 Tauri 环境中运行：

```bash
pnpm test:e2e --grep "F2"
```

Expected: 测试通过，或因 UI 元素选择器问题失败（非 mock 问题）

- [ ] **Step 3: 修复测试问题（如有）**

如果测试失败，检查错误原因：
- 如果是选择器问题：更新 `.events-view`、`button:has-text('采集飞书')` 等选择器
- 如果是超时问题：增加等待时间
- 如果是 mock server 问题：检查 MSW handlers 是否正确

- [ ] **Step 4: Commit**

```bash
git add tests/ts/e2e/f2-feishu-collection.spec.ts
git commit -m "test: 迁移 F2 测试到真实 Tauri IPC + mock server"
```

---

### Task 6: 添加测试脚本和文档更新

**Files:**
- Modify: `package.json`
- Modify: `docs/testing/architecture.md`

**Interfaces:**
- Produces: `pnpm test:e2e:dev`, `pnpm test:e2e:headed`, `pnpm test:e2e:debug` 命令

- [ ] **Step 1: 添加测试脚本**

修改 `package.json`，添加 E2E 测试脚本：

```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "test": "vitest run",
    "test:unit": "vitest run",
    "test:unit:watch": "vitest",
    "test:int": "vitest run --config vitest.integration.config.ts",
    "test:e2e": "playwright test",
    "test:e2e:dev": "playwright test --project=chromium",
    "test:e2e:headed": "playwright test --project=chromium --headed",
    "test:e2e:debug": "playwright test --project=chromium --debug",
    "test:all": "pnpm test:unit && pnpm test:int && pnpm test:e2e",
    "test:coverage": "vitest run --coverage",
    "test:rust": "cargo nextest run --workspace",
    "test:rust:ci": "cargo nextest run --workspace --profile ci"
  }
}
```

- [ ] **Step 2: 更新测试架构文档**

更新 `docs/testing/architecture.md`，添加 E2E 测试环境说明：

在 "框架选型" 部分添加：

```markdown
### E2E 测试环境

| 方面 | 配置 | 说明 |
|------|------|------|
| Tauri 环境 | `cargo tauri dev` | 启动真实的 Tauri app |
| Playwright 配置 | `playwright.config.ts` | baseURL: http://localhost:1420 |
| 飞书 API 隔离 | MSW mock server | 模拟飞书 API 响应 |
| 测试数据 | 临时目录 | `/tmp/work-better-test-{timestamp}` |
| 测试命令 | `pnpm test:e2e:dev` | 运行 E2E 测试 |
```

- [ ] **Step 3: 验证所有测试能正常运行**

运行所有 E2E 测试：

```bash
pnpm test:e2e:dev
```

Expected: F1 和 F2 测试通过，其他测试可能因功能未实现而跳过

- [ ] **Step 4: Commit**

```bash
git add package.json docs/testing/architecture.md
git commit -m "docs: 添加 E2E 测试脚本和文档更新"
```

---

## 验收检查清单

- [ ] F1-01、F1-02 能在真实 Tauri app 中运行
- [ ] F2-01、F2-02、F2-03 能在真实 Tauri app 中运行
- [ ] 测试时间 < 30 秒（两个场景）
- [ ] 提供 `pnpm test:e2e:dev` 命令
- [ ] 文档更新

## 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| Tauri 编译时间长 | 测试启动慢 | 使用 `reuseExistingServer`，开发时复用已启动的 app |
| 测试数据残留 | 测试不稳定 | 使用临时目录，测试后清理 |
| 飞书 API 限制 | 无法测试真实采集 | 使用 mock server，验证 IPC 调用 |
| macOS 权限问题 | 文件系统操作失败 | 使用临时目录，避免系统目录 |
