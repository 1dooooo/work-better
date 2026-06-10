---
title: 测试文档索引
type: structural
domain: testing
created: 2026-06-08
updated: 2026-06-08
status: active
---

# 测试文档索引

> **维护说明**：当新增测试文档、修改文档结构、或调整文档关系时更新本文档。

## 文档结构

```
docs/testing/
├── index.md                    # 本文档：测试文档索引
├── architecture.md             # 测试体系总体架构
├── implementation-progress.md  # 测试实施进度
├── test-effectiveness-audit.md # 测试有效性审计报告
├── conventions.md              # 测试编写规范
├── layers/                     # 测试层级定义
│   ├── overview.md             # 层级总览
│   └── security.md             # H 层安全测试定义
├── scenarios/                  # 测试场景目录
│   └── catalog.md              # 场景完整目录
└── execution/                  # 执行相关
    ├── triggering.md           # 分级触发策略
    └── migration.md            # 迁移路线图
```

## 文档说明

### 核心文档

| 文档 | 说明 | 维护频率 |
|------|------|---------|
| [architecture.md](architecture.md) | 测试体系总体架构、框架选型、设计目标 | 架构变更时 |
| [implementation-progress.md](implementation-progress.md) | 测试实施进度、已完成工作、待完成工作 | 每次测试实现后 |
| [conventions.md](conventions.md) | 测试编写规范、命名规则、组织方式 | 规范变更时 |
| [test-effectiveness-audit.md](test-effectiveness-audit.md) | 测试有效性审计——问题分析与修复路线图 | 审计/修复后 |

### 层级文档

| 文档 | 说明 | 维护频率 |
|------|------|---------|
| [layers/overview.md](layers/overview.md) | 测试层级定义、边界划分、Mock 策略 | 层级变更时 |
| [layers/security.md](layers/security.md) | H 层安全测试定义、子层分类、执行策略 | 安全测试变更时 |

### 场景文档

| 文档 | 说明 | 维护频率 |
|------|------|---------|
| [scenarios/catalog.md](scenarios/catalog.md) | 测试场景完整目录、场景 ID、功能点映射 | 新增/修改场景时 |

### 执行文档

| 文档 | 说明 | 维护频率 |
|------|------|---------|
| [execution/triggering.md](execution/triggering.md) | 分级触发策略、变更影响分析、并行执行 | 触发策略变更时 |
| [execution/migration.md](execution/migration.md) | 文档迁移、实施路线图、验收标准 | 迁移计划变更时 |

## 快速参考

### 测试层级

| 层级 | 代号 | 场景数 | 状态 |
|------|------|--------|------|
| L1 纯单元测试 | A+D | 158 | ✅ 完成 |
| L2 集成测试 | B+E | 63 | ✅ 完成 |
| L3 契约测试 | C | 9 | ✅ 完成 |
| L4 跨层 E2E | F | 20 | ⚠️ 8 通过，11 跳过 |
| L5 黑盒验收 | G | 182 | ⏳ 待实现 |
| H 安全测试 | H | 20 | ✅ 完成 |

### 测试框架

| 侧 | 框架 | 用途 |
|------|------|------|
| Rust | rstest | 参数化测试 |
| Rust | insta | 快照/契约测试 |
| Rust | cucumber-rs | BDD 验收测试 |
| TypeScript | Vitest | 单元/集成测试 |
| TypeScript | Playwright | E2E 测试 |
| TypeScript | @testing-library/react | 组件测试 |

### 执行命令

```bash
# 运行所有 Rust 测试
cargo test --workspace

# 运行所有 TypeScript 测试
pnpm test:unit

# 运行 E2E 测试
pnpm test:e2e

# 运行安全测试
cargo test --package wb-core --test security

# 运行验收测试
cargo test --package wb-acceptance
```

## 相关文档

| 文档 | 位置 | 说明 |
|------|------|------|
| 测试有效性 ADR | [../decisions/001-test-effectiveness-gap.md](../decisions/001-test-effectiveness-gap.md) | 决策依据 |
| 多 Agent 协作规范 | [../development/multi-agent-collaboration.md](../development/multi-agent-collaboration.md) | Agent 协作流程 |
| Workflow Spec | [../../.workflow/specs/dev-test-review.yaml](../../.workflow/specs/dev-test-review.yaml) | Workflow 定义 |
| 文档规范 | [../conventions.md](../conventions.md) | 文档编写规范 |
