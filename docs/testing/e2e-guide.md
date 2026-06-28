---
title: E2E 测试开发指南
type: guide
domain: testing
created: 2026-06-28
status: active
---

# E2E 测试开发指南

> **维护说明**：本文档是 E2E 测试的开发指南。当测试框架、选择器策略、或测试环境配置变更时更新本文档。

## 概述

E2E 测试验证完整的用户流程，从 UI 交互到 Rust 后端到数据持久化。使用 Playwright 驱动真实的 Tauri app。

## 快速开始

### 前置条件

1. **Rust 工具链**：确保 `cargo` 在 PATH 中
2. **Node.js 依赖**：`pnpm install`
3. **Tauri 开发环境**：参考 [Tauri 开发文档](https://v2.tauri.app/develop/)

### 运行测试

```bash
# 运行所有 E2E 测试（自动启动 Tauri app）
pnpm test:e2e:dev

# 有界面模式（查看浏览器操作）
pnpm test:e2e:headed

# 调试模式（逐步执行）
pnpm test:e2e:debug

# 运行特定测试文件
pnpm test:e2e:dev tests/ts/e2e/f1-manual-capture.spec.ts

# 运行特定测试用例
pnpm test:e2e:dev --grep "F1-01"
```

## 测试架构

### 文件结构

```
tests/ts/e2e/
├── helpers.ts                    # 测试辅助函数
├── mock-feishu-server.ts         # 飞书 API Mock Server
├── f1-manual-capture.spec.ts     # F1: 手动捕获流程
├── f2-feishu-collection.spec.ts  # F2: 飞书采集流程
├── f3-processing-pipeline.spec.ts # F3: 处理管线（跳过）
├── f4-settings-propagation.spec.ts # F4: 设置传播（跳过）
├── f5-scheduler-integration.spec.ts # F5: 调度器集成（跳过）
└── f6-menubar-data-flow.spec.ts  # F6: 菜单栏数据流（跳过）
```

### 核心辅助函数

```typescript
// helpers.ts 导出的辅助函数

// 等待主窗口加载
await waitForMainWindow(page);

// 导航到指定视图
await navigateToView(page, "事件");
await navigateToView(page, "设置");

// 查询真实 Tauri 数据
const events = await getEvents(page, 50);
const count = await getUnprocessedCount(page);

// 测试环境管理（TODO: 待实现）
const testDir = await setupTestEnvironment(page);
await cleanupTestEnvironment(page);
```

## 编写测试

### 选择器策略

**优先使用语义选择器**：

```typescript
// ✅ 推荐：语义选择器
await page.getByRole("button", { name: "提交" }).click();
await page.getByText("已捕获").waitFor();
await page.getByPlaceholder("输入飞书会话 ID").fill("oc_test");

// ❌ 避免：CSS 类选择器（容易因样式变更失效）
await page.locator(".capture__submit").click();
await page.locator(".events-view__card").first().click();
```

**选择器优先级**：

1. `getByRole()` - 最稳定，基于无障碍角色
2. `getByText()` - 基于文本内容
3. `getByPlaceholder()` - 基于占位符
4. `getByTestId()` - 基于 `data-testid` 属性（需要在组件中添加）
5. `locator()` - CSS 选择器（最后选择）

### 测试模式

```typescript
import { test, expect } from "@playwright/test";

test.describe("功能名称", () => {
  // 每个测试前的设置
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await waitForMainWindow(page);
  });

  // 每个测试后的清理
  test.afterEach(async ({ page }) => {
    // 清理测试数据
    await cleanupTestEnvironment(page);
  });

  test("测试用例名称", async ({ page }) => {
    // Arrange - 准备
    await navigateToView(page, "事件");

    // Act - 执行
    await page.getByRole("button", { name: "采集" }).click();

    // Assert - 验证
    await expect(page.getByText("采集成功")).toBeVisible();
  });
});
```

### 异步操作

```typescript
// ❌ 避免：固定等待时间
await page.waitForTimeout(1000);

// ✅ 推荐：等待特定条件
await expect(page.getByText("已捕获")).toBeVisible({ timeout: 5000 });

// ✅ 推荐：等待元素消失
await expect(page.getByText("加载中...")).toBeHidden();

// ✅ 推荐：轮询等待
await expect.poll(async () => {
  const events = await getEvents(page, 10);
  return events.length;
}, { timeout: 5000 }).toBeGreaterThan(0);
```

## 已知限制

### ⚠️ MSW 无法拦截 Rust HTTP 调用

**问题**：`msw/node` 只能拦截 Node.js 进程的 HTTP 请求，无法拦截 Rust 后端的 `reqwest` 调用。

**影响**：飞书 API 未真正隔离，测试可能调用真实 API。

**解决方案**（TODO）：

1. 在 Rust 后端添加 `FEISHU_API_BASE` 环境变量支持
2. E2E 测试启动时设置该变量指向本地 mock server
3. 或使用 `lark-cli` 的 mock 模式

```rust
// TODO: 在 src-tauri/src/commands/collect.rs 中添加
let api_base = std::env::var("FEISHU_API_BASE")
    .unwrap_or_else(|_| "https://open.feishu.cn".to_string());
```

### ⚠️ 测试数据隔离待实现

**问题**：`setupTestEnvironment()` 和 `cleanupTestEnvironment()` 已定义但未接入 Rust 后端。

**影响**：测试之间可能共享状态，导致测试不稳定。

**解决方案**（TODO）：

1. 在 Rust 后端实现 `set_test_mode` 命令
2. 在 Rust 后端实现 `cleanup_test_data` 命令
3. 在测试中调用这些命令

```rust
// TODO: 在 src-tauri/src/commands/ 中添加
#[tauri::command]
pub async fn set_test_mode(enabled: bool, data_dir: String) -> Result<(), String> {
    // 实现测试模式切换
}

#[tauri::command]
pub async fn cleanup_test_data() -> Result<(), String> {
    // 实现测试数据清理
}
```

### ⚠️ trigger_manual_capture 需要 AI 模型

**问题**：手动捕获命令需要配置 AI 模型 API key。

**影响**：F1 测试在未配置 API key 的环境中会失败。

**解决方案**（TODO）：

1. 实现 `set_test_mode` 命令，跳过 AI 处理
2. 或在测试环境中配置测试 API key
3. 或让 AI 步骤可选（测试模式下跳过）

## 添加新测试

### 步骤

1. **创建测试文件**：`tests/ts/e2e/f{N}-{feature-name}.spec.ts`

2. **导入辅助函数**：
   ```typescript
   import { test, expect } from "@playwright/test";
   import { waitForMainWindow, navigateToView, getEvents } from "./helpers";
   ```

3. **编写测试**：
   - 使用语义选择器
   - 遵循 AAA 模式（Arrange-Act-Assert）
   - 添加适当的等待和断言

4. **运行测试**：
   ```bash
   pnpm test:e2e:dev tests/ts/e2e/f{N}-{feature-name}.spec.ts
   ```

5. **更新文档**：在本文档和 `architecture.md` 中添加测试说明

### 测试命名规范

```
F{N}-{NN}: {测试描述}
```

- `F{N}`: 功能模块编号（1-6）
- `{NN}`: 测试用例编号（01, 02, ...）
- `{测试描述}`: 简洁描述测试目的

示例：
- `F1-01: Type text and manual capture creates event`
- `F2-01: UI triggers collection via lark-cli and updates event count`

## 调试技巧

### 查看测试录制

```bash
# 运行测试并录制 trace
pnpm test:e2e:dev --trace on

# 查看录制结果
npx playwright show-trace test-results/*/trace.zip
```

### 截图和视频

```bash
# 失败时自动截图
pnpm test:e2e:dev --screenshot on

# 录制视频
pnpm test:e2e:dev --video on
```

### 交互式调试

```bash
# 启动调试模式
pnpm test:e2e:debug

# 在代码中添加断点
await page.pause();
```

## 常见问题

### Q: 测试超时怎么办？

**A**: 检查以下几点：
1. Tauri app 是否正常启动（`cargo tauri dev`）
2. 选择器是否正确（使用 `page.pause()` 检查 DOM）
3. 是否有异步操作未等待
4. 增加超时时间：`{ timeout: 30000 }`

### Q: 如何添加 data-testid？

**A**: 在 React 组件中添加 `data-testid` 属性：

```tsx
// src/components/views/EventsView.tsx
<div data-testid="events-view" className="...">
  <button data-testid="collect-feishu-btn" onClick={handleCollect}>
    采集
  </button>
</div>
```

然后在测试中使用：

```typescript
await page.getByTestId("collect-feishu-btn").click();
```

### Q: 如何测试错误场景？

**A**: 使用 `expect().rejects` 或检查错误消息：

```typescript
// 方法 1: 检查错误消息
await page.getByRole("button", { name: "采集" }).click();
await expect(page.getByText("采集失败")).toBeVisible();

// 方法 2: 使用 MSW 模拟错误
feishuServer.use(
  http.post("https://open.feishu.cn/open-apis/im/v1/messages", () => {
    return HttpResponse.json({ code: 500, msg: "Internal Server Error" }, { status: 500 });
  })
);
```

## 参考资料

- [Playwright 官方文档](https://playwright.dev/docs/intro)
- [Tauri 测试指南](https://v2.tauri.app/develop/tests/)
- [Testing Library 文档](https://testing-library.com/docs/)
- [项目测试架构](./architecture.md)
