---
title: 真实后端测试实施计划
type: structural
domain: testing
created: 2026-06-11
status: active
---

# 真实后端测试实施计划

> **维护说明**：当完成阶段性修复后更新本文档的对应章节和状态标记。

## 背景

根据 [测试有效性审计报告](test-effectiveness-audit.md)，当前 E2E 测试使用 mock 拦截所有 Tauri 后端调用，导致"测试全绿 ≠ 产品可用"。本计划旨在建立真实后端测试基础设施，逐步替换 mock 测试。

## 阶段一：建立真实后端测试基础设施

### 目标

创建可以启动真实 Tauri 应用并执行端到端测试的基础设施。

### 方案

#### 方案 A：Tauri 集成测试模式（推荐）

Tauri 2.x 支持集成测试模式，可以在测试中启动真实的 Tauri 应用：

```rust
// tests/integration/setup.rs
use tauri::test::MockRuntime;

fn create_test_app() -> tauri::App<MockRuntime> {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 初始化测试环境
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to build test app")
}
```

#### 方案 B：直接调用 Rust 命令

绕过 Tauri 运行时，直接调用 Rust 命令函数：

```rust
// tests/integration/commands.rs
use work_better_lib::commands;

#[tokio::test]
async fn test_manual_capture() {
    let event = commands::collect::trigger_manual_capture_inner("test note").await;
    assert!(event.is_ok());
}
```

#### 方案 C：Playwright + 真实 Tauri 应用

启动真实的 Tauri 应用，让 Playwright 连接到真实后端：

```typescript
// test/e2e-real/setup.ts
import { test as base } from '@playwright/test';

export const test = base.extend({
  tauriApp: async ({}, use) => {
    // 启动真实的 Tauri 应用
    const app = await spawnTauriApp();
    await use(app);
    await app.kill();
  },
});
```

### 推荐方案

**阶段一使用方案 B（直接调用 Rust 命令）**：
- 最简单，无需启动完整 Tauri 运行时
- 可以测试核心业务逻辑
- 为后续方案 A/C 打下基础

**阶段二使用方案 A（Tauri 集成测试模式）**：
- 测试完整的 Tauri 生命周期
- 验证插件和窗口管理

**阶段三使用方案 C（Playwright + 真实 Tauri 应用）**：
- 完整的端到端测试
- 验证 UI 和后端交互

## 阶段二：实现 5 个核心功能的真实后端测试

### 功能列表

| 编号 | 功能 | 测试场景 | 预期结果 |
|------|------|---------|---------|
| 1 | 手动捕获 | 输入文本 → 调用 trigger_manual_capture | 事件出现在 EventLog |
| 2 | 创建任务 | 调用 create_task | 任务出现在任务列表 |
| 3 | 采集器开关 | 调用 enable/disable_collector | 采集器状态正确变化 |
| 4 | 调度器暂停/恢复 | 调用 pause/resume_scheduler | 状态正确切换 |
| 5 | 事件标记处理 | 调用 mark_event_processed | 事件状态正确更新 |

### 实现步骤

#### 步骤 1：创建测试基础设施

```rust
// crates/wb-core/tests/real_backend_setup.rs
use std::sync::Once;
use tempfile::TempDir;

static INIT: Once = Once::new();

pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub db_path: std::path::PathBuf,
}

impl TestEnvironment {
    pub fn new() -> Self {
        INIT.call_once(|| {
            // 初始化日志等
        });

        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        Self { temp_dir, db_path }
    }
}
```

#### 步骤 2：实现手动捕获测试

```rust
// crates/wb-core/tests/real_backend_manual_capture.rs
mod real_backend_setup;

#[tokio::test]
async fn test_manual_capture_creates_event_in_log() {
    let env = real_backend_setup::TestEnvironment::new();

    // 初始化 EventLog
    let event_log = EventLog::new(&env.db_path).expect("failed to create event log");

    // 执行手动捕获
    let event = trigger_manual_capture_inner("测试笔记内容")
        .await
        .expect("failed to trigger manual capture");

    // 验证事件被持久化
    let events = event_log.get_events(10).expect("failed to get events");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].content, "测试笔记内容");
    assert_eq!(events[0].source, "manual");
}
```

#### 步骤 3：实现其他 4 个功能的测试

（类似步骤 2，针对每个功能实现）

## 阶段三：迁移现有 E2E 测试

### 迁移策略

1. **保留 mock 测试**：用于验证 UI 交互逻辑
2. **新增真实后端测试**：验证功能可用性
3. **逐步替换**：当真实后端测试稳定后，移除对应的 mock 测试

### 迁移顺序

1. F1 手动捕获（最简单，作为试点）
2. F5 调度器集成（核心功能）
3. F2 飞书采集（依赖外部服务，需要 mock 外部 API）
4. F4 设置传播（配置相关）
5. F6 菜单栏数据流（UI 集成）

## 实施时间表

| 阶段 | 任务 | 预计时间 | 状态 |
|------|------|---------|------|
| 1.1 | 创建测试基础设施 | 2 小时 | ✅ 已完成 |
| 1.2 | 实现手动捕获测试 | 1 小时 | ✅ 已完成 |
| 1.3 | 实现任务创建测试 | 1 小时 | ✅ 已完成 |
| 1.4 | 实现采集器开关测试 | 1 小时 | ✅ 已完成 |
| 1.5 | 实现调度器测试 | 1 小时 | ✅ 已完成 |
| 1.6 | 实现事件处理测试 | 1 小时 | ✅ 已完成 |
| 2.1 | 迁移 F1 测试 | 2 小时 | ⬜ 未开始 |
| 2.2 | 迁移 F5 测试 | 2 小时 | ⬜ 未开始 |
| 2.3 | 迁移其他测试 | 4 小时 | ⬜ 未开始 |

## 验收标准

- [x] 5 个核心功能各有 1 条真实后端 happy-path 测试
- [x] 所有真实后端测试通过
- [ ] 测试覆盖率达到 80%
- [x] 文档更新完成

## 相关文档

| 文档 | 关系 |
|------|------|
| [测试有效性审计报告](test-effectiveness-audit.md) | 本文档基于的审计发现 |
| [ADR-001: 测试有效性差距](../decisions/001-test-effectiveness-gap.md) | 决策依据 |
| [测试体系总体架构](architecture.md) | 现有架构 |
