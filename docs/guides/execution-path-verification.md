---
title: 执行路径验证指南
type: guide
domain: testing
created: 2026-06-14
updated: 2026-06-14
status: active
---

# 执行路径验证指南

> **维护说明**：当测试覆盖工具变更、execution_path_verification schema 调整、或验证流程改进时更新本文档。

## 什么是执行路径验证

执行路径验证（Execution Path Verification）是一种确保**测试覆盖的代码路径与生产环境实际执行的代码路径一致**的验证方法。

核心问题：测试通过不代表功能可用。测试可能覆盖了一条路径，但生产环境实际走的是另一条路径。执行路径验证就是要发现这类不一致。

### 与传统覆盖率的区别

| 维度 | 传统覆盖率 | 执行路径验证 |
|------|-----------|------------|
| 度量对象 | 代码行/分支是否被执行 | 执行的路径是否与生产一致 |
| 关注点 | "测试跑了哪些代码" | "测试跑的代码是不是生产跑的代码" |
| 发现的问题 | 未覆盖的代码 | 覆盖了但覆盖错的代码 |
| 典型场景 | 忘记写测试 | 测试用 mock 绕过了真实逻辑 |

### 为什么需要验证

测试通过但功能不工作，通常源于以下模式：

1. **Mock 偏差**：测试中 mock 了某组件，绕过了实际执行逻辑
2. **条件分支差异**：测试环境的条件（环境变量、配置）与生产不同，走了不同分支
3. **调用链断裂**：测试覆盖了函数 A，但生产中 A 是通过另一条调用链触发的
4. **数据路径差异**：测试用构造数据，生产用真实数据，触发不同的序列化/反序列化路径

## 验证流程

```
识别生产执行路径 → 对比测试覆盖路径 → 标记不一致 → 修复或记录
```

### Step 1: 识别执行路径

通过两种方式识别生产环境的执行路径：

**静态分析（script）**：基于代码结构推断调用链
- 分析入口点（Tauri command、API handler、事件监听器）
- 追踪函数调用图（call graph）
- 识别条件分支和错误处理路径

**动态分析（llm）**：基于代码语义推断执行路径
- 分析业务逻辑的实际执行流程
- 识别 mock 与真实实现的差异
- 推断环境条件对路径选择的影响

### Step 2: 对比测试覆盖

将识别出的执行路径与测试覆盖情况进行对比：

- 每条执行路径是否有对应的测试覆盖
- 覆盖该路径的测试是否使用了真实的组件（而非 mock）
- 测试的断言是否验证了路径中的关键状态转换

### Step 3: 标记不一致

不一致的路径记录在 `execution_path_verification.mismatches` 中，每条不一致包含：

| 字段 | 说明 |
|------|------|
| `path` | 不一致的代码路径标识 |
| `test_coverage` | 测试中的执行路径描述 |
| `production_path` | 生产环境的实际执行路径 |
| `description` | 不一致原因说明 |

### Step 4: 修复或记录

- **可修复的不一致**：修改测试使其覆盖正确的路径
- **环境差异导致的**：在 test-report.json 中记录，标注为 env_issue
- **设计决策**：如刻意使用 mock 隔离外部依赖，在 details 中说明理由

## 验证工具

### 覆盖率工具

| 工具 | 用途 | 输出 |
|------|------|------|
| cargo-tarpaulin | Rust 覆盖率采集 | 行覆盖、分支覆盖数据 |
| cargo-nextest | Rust 测试运行 | NDJSON 事件流，含 source_location |
| vitest --coverage | TypeScript 覆盖率采集 | Istanbul 格式覆盖率报告 |
| cargo-audit | Rust 依赖审计 | 漏洞列表（H1 安全扫描） |

### LLM 分析

静态覆盖率工具只能告诉你"哪些行被执行了"，无法判断"执行的路径是否与生产一致"。LLM 分析补充这一能力：

1. 解析 coverage 报告，提取被覆盖的函数和分支
2. 对比生产入口点的调用链，识别覆盖偏差
3. 生成 `execution_path_verification` 字段的内容

### 工具链整合

```
cargo-tarpaulin / vitest --coverage
        ↓ 覆盖率数据
cargo-nextest NDJSON 事件
        ↓ 测试执行结果
LLM 分析（静态 + 语义）
        ↓ 执行路径验证结果
test-report.json → execution_path_verification
```

## 输出格式

验证结果写入 `test-report.json` 的 `execution_path_verification` 字段。

### schema 定义

```json
{
  "execution_path_verification": {
    "verified": true,
    "mismatches": [],
    "details": "所有生产执行路径均有对应测试覆盖，mock 使用合理"
  }
}
```

### 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `verified` | boolean | 测试覆盖的代码路径是否与生产执行一致 |
| `mismatches` | array | 不一致的执行路径列表，为空表示全部一致 |
| `details` | string | 验证的补充说明，包括验证范围和局限性 |

### 不一致记录示例

```json
{
  "execution_path_verification": {
    "verified": false,
    "mismatches": [
      {
        "path": "task_discovery::discover_from_events",
        "test_coverage": "测试中 mock 了 EventRepository，返回预构造的事件列表",
        "production_path": "生产中通过 SQLite 查询真实事件，经过 filtering 和排序",
        "description": "mock 绕过了 SQLite 查询逻辑和事件过滤条件，生产中可能出现空结果但测试不会"
      }
    ],
    "details": "task_discovery 模块的 discover_from_events 路径存在 mock 偏差，建议补充集成测试"
  }
}
```

## 示例：Task Discovery 执行路径验证

以 `crates/wb-processor/src/task/discovery.rs` 为例说明完整的验证流程。

### 1. 识别生产执行路径

Task Discovery 的生产执行路径：

```
入口：pipeline.rs 调用 task::discovery::discover()
  → 查询 EventRepository 获取事件列表
  → 对事件进行 filtering（时间范围、类型过滤）
  → 基于事件内容推断任务
  → 写入 TaskRepository
```

关键路径点：
- SQLite 查询真实事件数据
- 事件过滤条件（时间范围、事件类型）
- 任务推断逻辑（内容分析、重复检测）

### 2. 对比测试覆盖

| 路径点 | 测试覆盖情况 | 是否一致 |
|--------|------------|---------|
| EventRepository 查询 | mock 返回预构造数据 | 不一致 — 绕过了 SQLite 查询和过滤逻辑 |
| 事件过滤条件 | mock 数据已过滤，跳过过滤逻辑 | 不一致 — 过滤逻辑未被测试 |
| 任务推断逻辑 | 直接测试，无 mock | 一致 |
| TaskRepository 写入 | mock 验证调用参数 | 基本一致 |

### 3. 标记不一致

```json
{
  "execution_path_verification": {
    "verified": false,
    "mismatches": [
      {
        "path": "task::discovery::discover → EventRepository::query",
        "test_coverage": "mock EventRepository 返回 vec![event1, event2]",
        "production_path": "SQLite SELECT + WHERE time_range AND event_type",
        "description": "mock 绕过了数据库查询和过滤逻辑"
      },
      {
        "path": "task::discovery::discover → event_filter",
        "test_coverage": "mock 数据已预先过滤，filter 函数未被调用",
        "production_path": "filter() 对每个事件执行时间和类型检查",
        "description": "事件过滤路径完全未被覆盖"
      }
    ],
    "details": "task discovery 的单元测试 mock 过重，建议补充 L2 集成测试覆盖 SQLite 查询和过滤路径"
  }
}
```

### 4. 修复建议

- 补充 L2 集成测试，使用真实 SQLite（内存模式）验证查询和过滤路径
- 保留现有 L1 单元测试中对推断逻辑的覆盖
- 在集成测试中验证空事件列表、边界时间范围等边界情况

## 局限性

1. **静态分析的局限**：无法覆盖运行时动态生成的路径（如宏展开、动态分发）
2. **LLM 推断的局限**：语义分析可能遗漏隐式依赖（如全局状态、线程局部变量）
3. **环境差异**：部分路径差异是设计决策（如 mock 外部 API），不应强制覆盖
4. **成本权衡**：完整验证每条路径的成本可能高于修复 bug 本身，需根据模块重要性分级执行

## 与测试体系的关系

执行路径验证是测试体系的补充机制，不是替代：

- **测试金字塔**（L1-L5）确保"测试覆盖了什么"
- **执行路径验证**确保"覆盖的是正确的什么"
- 两者结合才能实现"测试通过 = 产品可用"的置信度目标

> 测试体系总体架构详见 [testing/architecture.md](../testing/architecture.md)

## 相关文档

- [测试体系总体架构](../testing/architecture.md) — 测试金字塔和分级触发
- [test-report.json schema](../../.workflow/templates/test-report.schema.json) — 完整的报告结构定义
- [Task Discovery 源码](../../crates/wb-processor/src/task/discovery.rs) — 验证示例的实现
