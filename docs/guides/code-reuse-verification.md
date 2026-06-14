---
title: 代码复用验证指南
type: guide
domain: development
created: 2026-06-14
updated: 2026-06-14
status: active
---

# 代码复用验证指南

> **维护说明**：当代码复用检查工具、验证流程、或输出格式变更时更新本文档。

## 什么是代码复用验证

代码复用验证是检查新增代码是否复用了已有核心函数的过程，而非重新实现相同功能。

**核心问题**：当项目中已存在功能等价的函数时，dev-agent 是否复用了它？

**与传统代码审查的区别**：
- 传统审查关注代码质量（命名、长度、嵌套）
- 代码复用验证关注功能等价性（是否重复实现）

## 为什么需要验证

重复实现会导致：
- **维护成本上升**：同一功能多处修改
- **行为不一致风险**：不同实现可能有微妙差异
- **测试覆盖分散**：同一逻辑多处测试
- **技术债务累积**：代码库逐渐臃肿

**典型案例**：
- `events.rs::process_single_event()` 独立实现了任务发现逻辑，未复用 `discovery_ai::discover_with_ai()`
- 单元测试覆盖了正确的 `discover_with_ai`，但实际执行使用的是 `process_single_event`
- 结果：测试通过，但功能不工作

## 验证流程

### Step 1: 识别已有实现

在实现新功能前，搜索项目中是否已存在功能等价的函数：

```bash
# 搜索函数定义
grep -r "fn |function" crates/ --include="*.rs"
grep -r "function " src/ --include="*.ts" --include="*.tsx"

# 搜索特定功能
grep -r "discover_with_ai" crates/
grep -r "process_event" crates/

# 使用 ripgrep 搜索
rg "fn discover" --type rust
rg "function processEvent" --type ts
```

### Step 2: 对比功能等价性

找到已有实现后，对比新代码与已有实现的功能等价性：

| 对比维度 | 检查点 |
|---------|--------|
| 函数签名 | 参数类型、返回类型是否一致 |
| 核心逻辑 | 算法、数据流、状态转换是否一致 |
| 边界处理 | 错误处理、空值处理、边界条件是否一致 |
| 副作用 | 文件操作、网络请求、状态变更是否一致 |

### Step 3: 标记不一致

如果发现功能等价但未复用的实现，标记为问题：

| 问题类型 | 严重程度 | 说明 |
|---------|---------|------|
| 核心函数未复用 | HIGH | 已有核心函数可用但未使用 |
| 重复实现 | HIGH | 多处独立实现相同逻辑 |
| 执行路径不一致 | MEDIUM | 测试覆盖的代码路径与生产执行不一致 |

### Step 4: 修复或记录

根据问题类型采取相应行动：

- **核心函数未复用**：重构为调用已有核心函数
- **重复实现**：合并为单一实现，或创建共享工具函数
- **执行路径不一致**：补充测试或修正测试覆盖

## 验证工具

### 静态分析工具

| 工具 | 用途 | 命令 |
|------|------|------|
| grep/ripgrep | 搜索函数定义 | `rg "fn function_name" --type rust` |
| cargo clippy | Rust 代码检查 | `cargo clippy -- -W clippy::duplicate_implementation` |
| eslint | TypeScript 检查 | `eslint --rule no-duplicate-functions` |

### 动态分析工具

| 工具 | 用途 | 命令 |
|------|------|------|
| cargo tarpaulin | Rust 覆盖率 | `cargo tarpaulin --out Html` |
| vitest coverage | TypeScript 覆盖率 | `pnpm test:coverage` |
| LLM 分析 | 功能等价性判断 | test-agent 执行 |

### 工具链整合

```
代码变更
    │
    ▼
静态分析（grep/ripgrep）
    │
    ├── 发现潜在重复 → 标记警告
    │
    ▼
功能等价性分析（LLM）
    │
    ├── 确认重复 → 标记 HIGH issue
    │
    ▼
执行路径验证（coverage + LLM）
    │
    └── 路径不一致 → 标记 MEDIUM issue
```

## 验证输出

验证结果写入 review-report.json 的 `code_reuse_issues` 字段：

```json
{
  "code_reuse_issues": [
    {
      "severity": "high",
      "title": "核心函数未复用",
      "file": "src-tauri/src/commands/events.rs",
      "line": 347,
      "description": "process_single_event() 独立实现了任务发现逻辑，未复用 discovery_ai::discover_with_ai()",
      "existing_function": "discover_with_ai"
    }
  ]
}
```

## 示例：Task Discovery 验证

### 场景

dev-agent 实现了 `process_single_event()` 函数，用于从事件中发现任务。

### 验证过程

**Step 1: 识别已有实现**

```bash
$ rg "fn discover" --type rust
crates/wb-processor/src/task/discovery.rs:    pub async fn discover_with_ai(...)
crates/wb-processor/src/task/discovery_ai.rs:    pub async fn discover_with_ai(...)
```

**Step 2: 对比功能等价性**

| 对比维度 | discovery_ai::discover_with_ai | events.rs::process_single_event |
|---------|-------------------------------|--------------------------------|
| 参数 | Event + existing_tasks | Event |
| 返回 | Vec<Task> | 单个 Task |
| is_status_update | ✅ 检查 | ❌ 不检查 |
| 重复任务检测 | ✅ 检查 | ❌ 不检查 |

**Step 3: 标记不一致**

- 核心函数未复用：HIGH
- 缺少 is_status_update 检查：HIGH
- 缺少重复任务检测：HIGH

**Step 4: 修复建议**

重构 `process_single_event()` 为调用 `discover_with_ai()`，确保行为一致。

## 参考文档

- [代码复用规范](../development/multi-agent-collaboration.md#代码复用规范)
- [执行路径验证指南](./execution-path-verification.md)
- [测试架构](../testing/architecture.md#变更关联)
