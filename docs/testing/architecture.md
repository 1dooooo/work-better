---
title: 测试体系总体架构
type: structural
domain: testing
created: 2026-06-06
updated: 2026-06-28
status: active
---

# 测试体系总体架构

> **维护说明**：本文档是测试体系的顶层架构文档。当测试层级划分、框架选型、Agent 可解析性策略、多 Agent 协作模式发生变更时更新本文档。

## 设计目标

本测试体系围绕一个核心假设构建：**开发 Agent 完成任务后，测试通过即代表产品可用**。

### 99.99% 置信度

测试通过 = 产品可用。这不是覆盖率数字，而是行为覆盖的完备性：

- 所有关键路径必须 100% 分支覆盖
- 所有非法状态转换必须被拦截验证
- 测试失败必须意味着真实的产品缺陷，而非环境问题或测试脆弱性
- 182 个产品级场景已定义（验收测试 cucumber-rs 实现待修复 regex 冲突后重新引入）
- 20 个安全测试场景覆盖依赖漏洞、注入攻击、权限越界

### 多 Agent 协作

开发流程采用多 Agent 协作模式，每个 Agent 职责单一：

| Agent | 职责 | 负责的测试层 |
|-------|------|------------|
| workflow-advisor | 任务分析 + 执行计划 | 无（规划角色） |
| dev-agent | 功能开发 | L1-L2（单元/集成） |
| test-agent | 测试执行与生成 | L4-L5（E2E/验收）+ H1-H2（安全扫描） |
| review-agent | 代码审查 | H3-H5（安全测试生成） |
| product-reviewer | 产品符合性审查 | 无（产品视角验证） |
| validator | Schema + 数据完整性验证 | 无（验证角色） |
| system-inspector | 系统健康 + 执行效率 | 无（巡检角色） |
| optimizer | Agent prompt + workflow 优化 | 无（优化建议） |

> 详见 [多 Agent 协作开发规范](../development/multi-agent-collaboration.md)

### 分级触发

不同变更规模触发不同深度的测试门禁：

| 门禁级别 | 触发条件 | 耗时目标 | 覆盖范围 |
|---------|---------|---------|---------|
| L1 快速门 | 任何代码变更 | <2min | 受影响的纯单元测试 + H1-H2 安全扫描 |
| L2 PR 门 | Pull Request | <10min | L1 + 集成 + H3-H5 + E2E + 受影响的验收场景 |
| L3 发布门 | Release / merge to main | <15min | 全量 A-G + H 层 |
| L4 夜间门 | 每日定时 (cron) | <60min | 全量 + 契约回归 + 真实集成 |

### 并行执行

- 同一层级内的功能测试互不依赖，必须可并行执行
- Rust 侧：cargo-nextest 进程级隔离并行
- TypeScript 侧：Vitest 线程池并行
- G 层验收：按 product domain 分 7 组并行
- L2 通过后：review-agent 与 test-agent (L4/L5) 并行执行

### 变更关联

所有被修改代码涉及的功能都必须被测试覆盖：

- 通过变更影响分析映射文件 → 测试场景
- 未被覆盖的变更路径标记为 blocking warning
- Agent 测试报告包含"未测试的变更路径"清单

### 执行路径验证

测试覆盖的代码路径必须与生产执行的代码路径一致：

- **验证时机**：test-agent 执行 L4（系统级）测试时验证
- **验证方法**：对比测试覆盖的函数与实际调用的函数
- **验证输出**：写入 test-report.json 的 `execution_path_verification` 字段
- **问题类型**：
  - 未覆盖路径：生产路径无测试覆盖 → 添加针对性测试
  - 重复实现：多处独立实现相同逻辑 → 合并或复用
  - 死测试路径：测试覆盖了生产不会执行的代码 → 移除或标记为回归防护

> 详见 [执行路径验证指南](../guides/execution-path-verification.md)

## 测试金字塔

```
                    ┌─────────────────────┐
                    │  L5: 黑盒验收测试    │  182 tests · 待实现 |
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
            ├─────────────────────────────────────┤
            │  H:  安全测试                        │  ~20 tests · 秒级（与 L1 并行）
            └─────────────────────────────────────┘
```

### 各层概览

| 层级 | 标识 | 测试数量 | 执行速度 | 外部依赖 | 生成方 |
|------|------|---------|---------|---------|--------|
| L1 纯单元测试 | A (Rust) + D (TS) | ~158 | 毫秒级 | 无 | dev-agent |
| L2 集成测试 | B (Rust) + E (TS) | ~63 | 秒级 | Mock 外部，真实内部 | dev-agent |
| L3 契约测试 | C | ~9 | 秒级 | 录制/回放的外部 API | 手动/工具 |
| L4 跨层 E2E | F | ~20 | 十秒级 | Mock 飞书/AI，真实 FS | test-agent |
| L5 黑盒验收 | G | 182 (已定义) | — | cucumber-rs (待修复) | test-agent |
| H 安全测试 | H | ~20 | 秒级 | 无 (H1-H2) / Mock (H3-H5) | test-agent (H1-H2) / review-agent (H3-H5) |

> 各层详细定义见 [layers/overview.md](layers/overview.md) 和 [layers/security.md](layers/security.md)

## 框架选型

### Rust 侧技术栈

| 用途 | 框架 | 选择理由 |
|------|------|----------|
| 参数化测试 | rstest | fixtures、`#[context]` Agent 可解析、`#[trace]` 失败输出变量 |
| 快照/契约测试 | insta | `assert_json_snapshot!`、redaction 处理动态字段 |
| HTTP Mock (契约) | httpmock | record/replay、standalone Docker、YAML 配置 |
| HTTP Mock (通用) | wiremock | 最佳 async 支持、最活跃社区 |
| 测试运行器 | cargo-nextest | NDJSON 输出、进程级隔离、filtersets 标签过滤 |
| BDD 验收 | cucumber-rs | 纯 Rust API、World 状态管理、Agent 可 AST 解析 |
| 依赖审计 | cargo-audit | Rust 依赖漏洞扫描 (H1) |

### TypeScript 侧技术栈

| 用途 | 框架 | 选择理由 |
|------|------|----------|
| 单元测试 | Vitest 3.x | Vite 原生集成、TypeScript 原生支持 |
| 组件测试 | @testing-library/react | 用户行为驱动、不依赖实现细节 |
| E2E 测试 | Playwright | 跨浏览器、Tauri app 支持、trace 录制 |
| API Mock | MSW 2.x | Service Worker 级拦截、类型安全 handlers |

## Agent 可解析性设计

### nextest NDJSON

每行一个 JSON 事件，Agent 逐行解析：

```json
{"type":"test","event":"failed","name":"classifier::route_unknown","duration":{"secs":0,"nanos":85000},"stderr":"..."}
{"type":"suite","event":"ok","passed":156,"failed":1,"ignored":1}
```

### insta 快照文件

`.snap` 文件纯文本，unified diff 格式，Agent 解析 source 定位和 expression 理解断言意图。

### cucumber-rs 纯 Rust API

given/when/then 字符串是自然语言，World 字段变更可追踪。

### Playwright JSON Reporter

status 判断、trace 附件回放、duration 性能基线。

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
| Task 状态机 | 合法/非法转换 | L1 |
| Event → EventLog → 处理层 | 事件流转路径 | L2 |
| 处理层 → 存储层 | WorkRecord 三层写入 | L2 |
| 飞书 API 请求/响应 | 格式、解析、错误处理 | L3 |
| 飞书消息 → Obsidian | 完整业务管道 | L4 |
| 用户场景端到端 | 信息输入到最终产出 | L5 |
| Tauri command 输入 | 注入攻击、路径遍历 | H3 |
| Tauri command 权限 | 权限越界 | H4 |
| 文件系统操作 | 符号链接逃逸、沙箱突破 | H5 |

## 多 Agent 协作与 Workflow

### 协作流程

```
用户下达任务 → workflow-advisor 分析 + 制定计划
        │
        ▼
Phase 1: dev-agent 写代码 + L1-L2 测试 → dev-output.json
        │
        ▼
Phase 2 (并行):
        ├── test-agent: 测试 + 安全扫描
        ├── review-agent: 代码审查 + H3-H5
        ├── product-reviewer: 产品符合性审查
        ├── validator: Schema 验证
        └── system-inspector: 系统巡检
        │
        ▼
Phase 3: 主 Agent 汇总 → final-report.json
        │
        └── 有失败 → dev-agent 修复 → 重跑 Phase 2
```

### A2A 通信

Agent 之间通过 `.workflow/artifacts/{task_id}/` 下的文件通信：

| 文件 | 写入方 | 读取方 |
|------|--------|--------|
| dev-output.json | dev-agent | workflow, test, review, product |
| test-report.json | test-agent | workflow, dev |
| test-plan.json | test-agent | workflow |
| review-report.json | review-agent | workflow |
| review-criteria.json | product-reviewer | workflow |
| product-review.json | product-reviewer | workflow, dev |
| validation-report.json | validator | workflow |
| system-inspector-report.json | system-inspector | workflow |
| optimization-plan.json | optimizer | workflow, 用户 |
| error-response.json | workflow | dev |
| final-report.json | 主 Agent | 用户 |

> 详见 [多 Agent 协作开发规范](../development/multi-agent-collaboration.md)

### 重试策略

| 层级 | 最大重试 | 趋势停止 | 失败处理 |
|------|---------|---------|---------|
| L1 | 3 次 | 同一 source_location 连续 2 次 | dev-agent 判定 failure_type 后修复 |
| L2 | 2 次 | 同一测试用例连续 2 次 | 同上 |
| L4 | 1 次 | 同一测试连续 1 次 | 同上 |
| L5 | 0 次 | 首次失败即上报 | 回到产品设计层面 |

## 测试数量总览

| 层级 | Rust 侧 | TypeScript 侧 | 合计 | 生成方 |
|------|---------|---------------|------|--------|
| L1 纯单元 | 136 (A) | 22 (D) | 158 | dev-agent |
| L2 集成 | 46 (B) | 17 (E) | 63 | dev-agent |
| L3 契约 | 9 (C) | 0 | 9 | 手动/工具 |
| L4 E2E | 0 | 20 (F) | 20 | test-agent |
| L5 验收 | 182 (G) | 0 | 182 (待实现) | test-agent |
| H 安全 | 13 (H1-H2) | 7 (H3-H5) | 20 | test-agent + review-agent |
| **合计** | **386** | **66** | **452** | |

## 当前状态

> **场景数 vs 测试函数数**：场景数（Scenario）是产品级的行为描述，一个场景可能对应多个测试函数。
> 下表同时列出两个维度，避免混淆。

**实际测试函数数**: 1443 通过 / 0 失败 / 11 跳过（2026-06-08 验证）

| 层级 | 场景数 | 测试函数数 | 通过 | 跳过 | 状态 |
|------|--------|-----------|------|------|------|
| A (Rust 单元) | 136 | ~580 | 580 | 0 | 完成 |
| B (Rust 集成) | 46 | ~470 | 470 | 0 | 完成 |
| C (契约) | 9 | ~50 | 50 | 0 | 完成 |
| D (TS 单元) | 22 | 164 | 164 | 0 | 完成 |
| E (TS 集成) | 17 | 56 | 56 | 0 | 完成 |
| F (E2E) | 20 | 26 | 15 | 11 | 8 场景通过，4 场景 skipped（前端 UI 未实现） |
| G (验收) | 182 | — | — | — | 场景目录已定义，待 cucumber-rs 实现 |
| H1-H2 (安全扫描) | 8 | 2 | 2 | 0 | npm audit + grep 扫描通过 |
| H3-H5 (安全测试) | 12 | 60 | 60 | 0 | 已实现（2026-06-08） |

### E2E 跳过原因

| 跳过测试 | 原因 | 解除条件 |
|---------|------|---------|
| F3-01 ~ F3-04 | 处理管线 UI 未实现（当前只有"标记已处理"，无实际处理入口） | 实现 process_event command + 处理状态展示 UI |
| F5-01 ~ F5-04 | TasksView 使用硬编码数据，未接入 list_scheduled_tasks | TasksView 改为从 scheduler 获取真实数据 |

### 与文档声称的差异

architecture.md 之前声称 94.7% (427/452) 完成度。实际情况：
- **功能测试**（A-F 层）：1299 个测试函数全部通过，覆盖充分
- **验收测试**（G 层）：182 个场景已定义在 scenarios/catalog.md，但未实现为可执行的 cucumber 步骤
- **安全测试**（H 层）：H1-H2 自动扫描可用，H3-H5 需要 Tauri command 级别的参数化测试

## 文档索引

| 文档 | 内容 |
|------|------|
| [architecture.md](architecture.md) | 本文档：总体架构、框架选型、设计目标 |
| [conventions.md](conventions.md) | 命名、组织、编写规范 |
| [layers/overview.md](layers/overview.md) | 层级定义与边界划分 |
| [layers/security.md](layers/security.md) | H 层安全测试定义 |
| [scenarios/catalog.md](scenarios/catalog.md) | 测试场景完整目录 |
| [execution/triggering.md](execution/triggering.md) | 分级触发、变更影响分析、并行执行 |
| [execution/migration.md](execution/migration.md) | 文档迁移、实施路线图、验收标准 |
| [../development/multi-agent-collaboration.md](../development/multi-agent-collaboration.md) | 多 Agent 协作开发规范 |
| [CODEMAPS/testing.codemap.md](CODEMAPS/testing.codemap.md) | 测试系统导航地图 |
