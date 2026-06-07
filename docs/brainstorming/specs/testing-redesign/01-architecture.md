---
title: 测试体系总体架构
type: structural
domain: testing
created: 2026-06-06
updated: 2026-06-06
status: draft
---

# 测试体系总体架构

> **维护说明**：本文档是测试重设计方案的顶层架构文档。当测试层级划分、框架选型、Agent 可解析性策略发生变更时更新本文档。各层级的详细实现见 `02-test-layers.md`。

## 设计目标

本测试体系围绕一个核心假设构建：**开发 Agent 完成任务后，测试通过即代表产品可用**。

### 99.99% 置信度

测试通过 = 产品可用。这不是覆盖率数字，而是行为覆盖的完备性：

- 所有关键路径（Classifier 路由、Task 状态机、SLA 判定、模型升级阈值、ReviewAgent verdict）必须 100% 分支覆盖
- 所有 Source × EventType 组合的路由结果必须有对应测试
- 所有非法状态转换必须被拦截验证
- 测试失败必须意味着真实的产品缺陷，而非环境问题或测试脆弱性
- 182 个产品级场景全部有可执行的验收测试

### 分级触发

不同变更规模触发不同深度的测试门禁：

| 门禁级别 | 触发条件 | 耗时目标 | 覆盖范围 |
|---------|---------|---------|---------|
| L1 快速门 | 任何代码变更 | <2min | 受影响的纯单元测试 |
| L2 PR 门 | Pull Request | <10min | L1 + 集成 + E2E + 受影响的验收场景 |
| L3 发布门 | Release / merge to main | <15min | 全量 A-G 层 (432 场景) |
| L4 夜间门 | 每日定时 (cron) | <60min | 全量 + 契约回归 + 真实集成 |

### 并行执行

- 同一层级内的功能测试互不依赖，必须可并行执行
- Rust 侧：cargo-nextest 进程级隔离并行
- TypeScript 侧：Vitest 线程池并行
- G 层验收：按 product domain 分 7 组并行

### 变更关联

所有被修改代码涉及的功能都必须被测试覆盖：

- 通过变更影响分析映射文件 → 测试场景
- 未被覆盖的变更路径标记为 blocking warning
- Agent 测试报告包含"未测试的变更路径"清单

## 测试金字塔

```
                    ┌─────────────────────┐
                    │  L5: 黑盒验收测试    │  182 tests · 分钟级
                    │  (G: Acceptance)     │
                  ┌─┴─────────────────────┴─┐
                  │  L4: 跨层 E2E 测试      │  ~20 tests · 十秒级
                  │  (F: Cross-Layer E2E)    │
                ┌─┴─────────────────────────┴─┐
                │  L3: 契约测试               │  ~9 tests · 秒级
                │  (C: Contract Tests)        │
              ┌─┴─────────────────────────────┴─┐
              │  L2: 集成测试                    │  ~63 tests · 秒级
              │  (B+E: Integration)              │
            ┌─┴─────────────────────────────────┴─┐
            │  L1: 纯单元测试                      │  ~158 tests · 毫秒级
            │  (A+D: Pure Unit)                    │
            └─────────────────────────────────────┘
```

### 各层概览

| 层级 | 标识 | 测试数量 | 执行速度 | 外部依赖 |
|------|------|---------|---------|---------|
| L1 纯单元测试 | A (Rust) + D (TS) | ~158 | 毫秒级 | 无 |
| L2 集成测试 | B (Rust) + E (TS) | ~63 | 秒级 | Mock 外部，真实内部 |
| L3 契约测试 | C | ~9 | 秒级 | 录制/回放的外部 API |
| L4 跨层 E2E | F | ~20 | 十秒级 | Mock 飞书/AI，真实 FS |
| L5 黑盒验收 | G | 182 | 分钟级 | 真实或沙箱环境 |

> 各层详细定义见 [02-test-layers.md](02-test-layers.md)。

## 框架选型

### Rust 侧技术栈

| 用途 | 框架 | 选择理由 |
|------|------|----------|
| 参数化测试 | rstest (90M+ downloads) | fixtures、`#[context]` Agent 可解析、`#[trace]` 失败输出变量 |
| 快照/契约测试 | insta (70M+ downloads) | `assert_json_snapshot!`、redaction 处理动态字段、`.snap` 版本控制 |
| HTTP Mock (契约) | httpmock (23M downloads) | record/replay、standalone Docker、YAML 配置 |
| HTTP Mock (通用) | wiremock (55M downloads) | 最佳 async 支持、最活跃社区 |
| 测试运行器 | cargo-nextest (3k stars) | NDJSON 输出、进程级隔离、filtersets 标签过滤、profiles |
| BDD 验收 | cucumber-rs (15M downloads) | 纯 Rust API (无需 .feature)、World 状态管理、Agent 可 AST 解析 |

### TypeScript 侧技术栈

| 用途 | 框架 | 选择理由 |
|------|------|----------|
| 单元测试 | Vitest 3.x | Vite 原生集成、TypeScript 原生支持 |
| 组件测试 | @testing-library/react | 用户行为驱动、不依赖实现细节 |
| E2E 测试 | Playwright | 跨浏览器、Tauri app 支持、trace 录制 |
| API Mock | MSW 2.x | Service Worker 级拦截、类型安全 handlers |

### 选型原则

1. 社区活跃度优先：下载量、更新频率、issue 响应
2. Agent 可解析性优先：输出格式结构化
3. 进程隔离优先：测试间零共享状态
4. 快照测试优先：结构变更自动暴露

## Agent 可解析性设计

### nextest NDJSON

每行一个 JSON 事件，Agent 逐行解析：

```json
{"type":"test","event":"failed","name":"classifier::route_unknown","duration":{"secs":0,"nanos":85000},"stderr":"..."}
{"type":"suite","event":"ok","passed":156,"failed":1,"ignored":1}
```

Agent 提取：失败用例名 + stderr、慢测试 (duration)、crate/module 定位。

### insta 快照文件

`.snap` 文件纯文本，unified diff 格式：

```text
---
source: crates/wb-processor/src/classifier.rs
expression: classify(&event)
---
{ "route": "immediate", "priority": "P0" }
```

Agent 解析：source 定位、expression 理解断言意图、diff 自动 review。

### cucumber-rs 纯 Rust API

```rust
#[given(regex = r"^一个 (.+) 类型的事件$")]
fn given_event(world: &mut TestWorld, event_type: String) { ... }

#[when("该事件被分类器处理")]
fn when_classify(world: &mut TestWorld) { ... }

#[then(regex = r"^路由结果应为 (.+)$")]
fn then_route(world: &mut TestWorld, expected: String) { ... }
```

Agent 解析：given/when/then 字符串是自然语言、World 字段变更可追踪。

### Playwright JSON Reporter

```json
{ "suites": [{ "specs": [{ "title": "飞书消息写入 Obsidian", "ok": true }] }] }
```

Agent 解析：status 判断、trace 附件回放、duration 性能基线。

## 数据流与测试锚点

EventLog 是系统核心锚点，所有测试围绕数据链路展开：

```
信息源 ──采集──→ Event ──写入──→ EventLog ──消费──→ WorkRecord ──持久化──→ Obsidian
                     │              │                   │                    │
                     ▼              ▼                   ▼                    ▼
                 L1 格式验证    L2 流转验证         L2 处理验证          L4 端到端验证
```

| 锚点位置 | 验证内容 | 测试层级 |
|---------|---------|---------|
| Event 构造 | 字段完整性、类型正确性 | L1 |
| EventLog 写入 | 顺序性、不可变性、查询正确性 | L1 |
| Classifier 输出 | 路由决策表每条规则 | L1 |
| ModelRouter 决策 | 升级阈值、Token 预算 | L1 |
| Task 状态机 | 合法/非法转换 | L1 |
| Event → EventLog → 处理层 | 事件流转路径 | L2 |
| 处理层 → 存储层 | WorkRecord 三层写入 | L2 |
| 飞书 API 请求/响应 | 格式、解析、错误处理 | L3 |
| 飞书消息 → Obsidian | 完整业务管道 | L4 |
| 用户场景端到端 | 信息输入到最终产出 | L5 |

## 目录结构

### Rust 侧

```
crates/
├── wb-core/src/              # A 层: inline #[cfg(test)]
├── wb-processor/
│   ├── src/                  # A 层: inline tests (rstest)
│   ├── tests/                # B 层: 集成测试
│   │   ├── contract/         # C 层: insta/httpmock 契约测试
│   │   └── acceptance/       # G 层: cucumber-rs 验收测试
├── wb-collector/src/         # A 层 + 契约快照
├── wb-storage/
│   ├── src/                  # A 层
│   └── tests/                # B 层: SQLite/Obsidian/Vector 集成
├── wb-ai/src/                # A 层
└── wb-scheduler/
    ├── src/                  # A 层
    └── tests/                # B 层: scheduler_tests.rs
```

### TypeScript 侧

```
src/                          # D 层: 组件/工具单元测试 (Vitest)
test/
├── integration/              # E 层: invoke() 集成测试
└── e2e/                      # F 层: Playwright 跨层 E2E
```

### 配置文件

```
.config/nextest.toml          # nextest profiles + filtersets
vitest.config.ts              # Vitest 主配置
vitest.config.int.ts          # Vitest 集成测试配置
playwright.config.ts          # Playwright E2E 配置
```

## 测试数量总览

| 层级 | Rust 侧 | TypeScript 侧 | 合计 | 依据 |
|------|---------|---------------|------|------|
| L1 纯单元 | 136 (A) | 22 (D) | 158 | 6 crate 核心逻辑 + 前端组件 |
| L2 集成 | 46 (B) | 17 (E) | 63 | 层间接口 + 存储操作 |
| L3 契约 | 9 (C) | 0 | 9 | lark-cli + 飞书 API + FS 行为 |
| L4 E2E | 0 | 20 (F) | 20 | 完整业务管道 |
| L5 验收 | 182 (G) | 0 | 182 | 产品级 Given/When/Then |
| **合计** | **373** | **59** | **432** | |

## 文档索引

| 文档 | 内容 |
|------|------|
| [01-architecture.md](01-architecture.md) | 本文档：总体架构、框架选型、设计目标 |
| [02-test-layers.md](02-test-layers.md) | 各层定义、边界、编写模式、代码示例 |
| [03-scenario-catalog.md](03-scenario-catalog.md) | 432 个测试场景完整目录 |
| [04-triggering-execution.md](04-triggering-execution.md) | 分级触发、变更影响分析、并行执行、CI 配置 |
| [05-migration-rollout.md](05-migration-rollout.md) | 文档迁移、实施路线图、验收标准 |
