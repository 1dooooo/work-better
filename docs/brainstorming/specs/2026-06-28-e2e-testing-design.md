---
title: E2E 测试体系改进设计
type: design
domain: testing
created: 2026-06-28
status: draft
---

# E2E 测试体系改进设计

## 背景

当前项目的 E2E 测试体系存在核心问题：**测试无法在真实 Tauri 环境中运行**。

### 现状分析

| 方面 | 现状 | 问题 |
|------|------|------|
| TypeScript E2E | 6 个 spec 文件（F1-F6） | Mock 在浏览器侧注入，无法测试真实 Tauri IPC |
| F3 处理管线 | 4 个测试 | 全部跳过（UI 未实现） |
| F5 调度器 | 4 个测试 | 全部跳过（TasksView 用硬编码数据） |
| Rust E2E | wb-real-backend-tests | 编译错误待修复 |
| G 层验收 | 182 个场景已定义 | cucumber-rs regex 冲突，未实现 |

### 核心问题

1. **Mock 层与真实环境脱节**：`helpers.ts` 通过 `page.addInitScript()` 注入 mock 的 `__TAURI_INTERNALS__`，只能测试 React 组件的渲染和交互逻辑，无法验证真实的 Tauri IPC 通信和 Rust 后端行为。

2. **测试与实现脱节**：F3（处理管线）、F5（调度器）的测试是"规格说明"而非可执行测试，因为对应的 UI 功能未实现。

3. **缺少真实环境验证**：没有 CI/CD 中运行 E2E 测试的机制，没有 nightly build + test 的流程。

4. **验收测试完全缺失**：182 个场景已定义但未实现，cucumber-rs 的 regex 冲突未修复。

## 设计目标

让 F1（手动捕获）和 F2（飞书采集）两个场景在真实 Tauri app 中运行，验证完整的端到端链路。

### 成功标准

- F1-01、F1-02：能在真实 Tauri app 中运行，验证手动捕获流程
- F2-01、F2-02、F2-03：能在真实 Tauri app 中运行，验证飞书采集流程
- 测试时间：< 30 秒（两个场景）
- 提供 `pnpm test:e2e:dev` 命令供开发者使用

## 技术方案

### 方案选择

| 方案 | 优点 | 缺点 | 选择 |
|------|------|------|------|
| A. WebDriver 协议 | 官方推荐 | 配置复杂，macOS 有限制 | ❌ |
| B. cargo tauri dev | 最简单直接 | 需要管理进程生命周期 | ✅ |
| C. 混合方案 | 最完整 | 维护成本高 | ❌ |

**选择方案 B**：使用 `cargo tauri dev` 启动真实的 Tauri app，Playwright 连接到运行中的 app 进行测试。

### 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│  测试环境                                                    │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │  Playwright  │───→│  Tauri App  │───→│  Rust 后端  │     │
│  │  (测试驱动)  │    │  (WebView)  │    │  (真实 IPC) │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│         │                   │                   │           │
│         ▼                   ▼                   ▼           │
│    测试脚本            React UI            SQLite/FS        │
│                            │                   │           │
│                            ▼                   ▼           │
│                       事件展示              数据持久化        │
└─────────────────────────────────────────────────────────────┘
```

### 关键变更

| 组件 | 当前 | 变更后 |
|------|------|--------|
| Playwright 配置 | `vite preview` 服务器 | `cargo tauri dev` 进程 |
| Mock 层 | 浏览器侧注入 `__TAURI_INTERNALS__` | 移除 mock，使用真实 IPC |
| 外部依赖 | 无隔离 | 飞书 API 使用 mock server |
| 测试数据 | 浏览器内存 | 真实 SQLite（测试后清理） |

## 详细设计

### 1. Playwright 配置变更

**文件**：`playwright.config.ts`

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

### 2. 测试代码变更

**文件**：`tests/ts/e2e/helpers.ts`

移除 `injectTauriMock` 函数，改为使用真实的 Tauri IPC：

```typescript
import { test as base, expect, type Page } from "@playwright/test";

// 移除所有 mock 相关代码
// 保留辅助函数：waitForMainWindow, navigateToView, createMockEvent

export async function waitForMainWindow(page: Page): Promise<void> {
  await page.waitForSelector(".sidebar", { timeout: 30000 });
}

export async function navigateToView(
  page: Page,
  viewLabel: string,
): Promise<void> {
  await waitForMainWindow(page);
  await page.click(`.sidebar__item:has-text("${viewLabel}")`);
}
```

**文件**：`tests/ts/e2e/f1-manual-capture.spec.ts`

修改为使用真实 IPC：

```typescript
import { test, expect } from "@playwright/test";
import { waitForMainWindow, navigateToView } from "./helpers";

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

    // 验证事件已创建（通过查询真实数据库）
    // 注意：这里需要等待后端处理完成
    await page.waitForTimeout(1000);

    // 通过 Tauri IPC 查询事件
    const events = await page.evaluate(() => {
      return (window as any).__TAURI__.core.invoke("get_events", {
        limit: 10,
        offset: 0,
      });
    });

    const captured = events.find(
      (e: any) =>
        e.content === "This is a test capture note" && e.source === "manual",
    );
    expect(captured).toBeDefined();
    expect(captured.type).toBe("note");
  });
});
```

### 3. 外部依赖隔离

**飞书 API Mock Server**

使用 wiremock 或 MSW 创建 mock server，模拟飞书 API 响应：

```typescript
// tests/ts/e2e/mock-feishu-server.ts
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";

export const feishuServer = setupServer(
  http.post("https://open.feishu.cn/open-apis/im/v1/messages", () => {
    return HttpResponse.json({
      code: 0,
      msg: "success",
      data: {
        message_id: "mock-message-id",
        create_time: Date.now().toString(),
      },
    });
  }),
);

// 在测试前启动，测试后关闭
```

**测试数据隔离**

使用临时目录和数据库：

```typescript
// tests/ts/e2e/helpers.ts
export async function setupTestEnvironment(page: Page): Promise<void> {
  // 通过 Tauri IPC 设置测试环境
  await page.evaluate(() => {
    return (window as any).__TAURI__.core.invoke("set_test_mode", {
      enabled: true,
      data_dir: "/tmp/work-better-test-" + Date.now(),
    });
  });
}

export async function cleanupTestEnvironment(page: Page): Promise<void> {
  // 清理测试数据
  await page.evaluate(() => {
    return (window as any).__TAURI__.core.invoke("cleanup_test_data");
  });
}
```

### 4. 测试脚本

**package.json**

```json
{
  "scripts": {
    "test:e2e:dev": "playwright test --project=chromium",
    "test:e2e:headed": "playwright test --project=chromium --headed",
    "test:e2e:debug": "playwright test --project=chromium --debug"
  }
}
```

## 实施计划

### Phase 1：基础设施（1-2 小时）

1. 修改 `playwright.config.ts`，配置 `cargo tauri dev` 作为 webServer
2. 创建测试环境配置（临时数据库、mock 飞书 API）
3. 验证 Playwright 能连接到 Tauri app

### Phase 2：测试迁移（2-3 小时）

1. 修改 F1 测试：移除 mock，使用真实 IPC
2. 修改 F2 测试：连接 mock 飞书 server，验证采集流程
3. 添加测试数据清理逻辑

### Phase 3：验证与优化（1 小时）

1. 运行测试，修复问题
2. 添加 `pnpm test:e2e:dev` 脚本
3. 文档更新

## 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| Tauri 编译时间长 | 测试启动慢 | 使用 `reuseExistingServer`，开发时复用已启动的 app |
| 测试数据残留 | 测试不稳定 | 使用临时目录，测试后清理 |
| 飞书 API 限制 | 无法测试真实采集 | 使用 mock server，验证 IPC 调用 |
| macOS 权限问题 | 文件系统操作失败 | 使用临时目录，避免系统目录 |

## 验收标准

- [ ] F1-01、F1-02 能在真实 Tauri app 中运行
- [ ] F2-01、F2-02、F2-03 能在真实 Tauri app 中运行
- [ ] 测试时间 < 30 秒（两个场景）
- [ ] 提供 `pnpm test:e2e:dev` 命令
- [ ] 文档更新

## 参考资料

- [Tauri Testing 官方文档](https://v2.tauri.app/develop/tests/)
- [Playwright 配置文档](https://playwright.dev/docs/test-configuration)
- [项目测试架构文档](../testing/architecture.md)
