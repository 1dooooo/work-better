---
title: 测试文档索引
type: index
domain: testing
created: 2026-06-06
updated: 2026-06-12
status: active
---

# 测试文档索引

## 核心文档

| 文档 | 说明 |
|------|------|
| [architecture.md](architecture.md) | 总体架构、框架选型、设计目标 |
| [conventions.md](conventions.md) | 命名、组织、编写规范 |
| [implementation-progress.md](implementation-progress.md) | 测试实施进度 |
| [test-effectiveness-audit.md](test-effectiveness-audit.md) | 测试有效性审计报告 |

## 层级指南

| 文档 | 说明 |
|------|------|
| [layers/overview.md](layers/overview.md) | 层级定义与边界划分 |
| [layers/unit-rust.md](layers/unit-rust.md) | Rust 单元测试 |
| [layers/unit-frontend.md](layers/unit-frontend.md) | 前端单元测试 |
| [layers/integration-rust.md](layers/integration-rust.md) | Rust 集成测试 |
| [layers/integration-frontend.md](layers/integration-frontend.md) | 前端集成测试 |
| [layers/contract.md](layers/contract.md) | 契约测试 |
| [layers/e2e.md](layers/e2e.md) | E2E 测试 |
| [layers/acceptance.md](layers/acceptance.md) | 验收测试 |
| [layers/security.md](layers/security.md) | 安全测试 (H 层) |

## 场景与执行

| 文档 | 说明 |
|------|------|
| [scenarios/catalog.md](scenarios/catalog.md) | 场景目录与覆盖映射 |
| [execution/triggering.md](execution/triggering.md) | 分级触发与执行策略 |
| [execution/parallelization.md](execution/parallelization.md) | 并行执行策略 |
| [execution/migration.md](execution/migration.md) | 迁移与实施路线图 |

## 测试运行

| 门控 | 命令 | 耗时 |
|------|------|------|
| Fast | `cargo nextest run --workspace` | <2min |
| Deep | `pnpm test` | <10min |
| Full | `pnpm test:all` | <15min |

## 测试框架

| 侧 | 框架 | 用途 |
|------|------|------|
| Rust | rstest | 参数化测试 |
| Rust | insta | 快照/契约测试 |
| Rust | cucumber-rs | BDD 验收测试 |
| TypeScript | Vitest | 单元/集成测试 |
| TypeScript | Playwright | E2E 测试 |
| TypeScript | @testing-library/react | 组件测试 |

## 相关文档

| 文档 | 位置 |
|------|------|
| 测试 Codemap | [CODEMAPS/testing.codemap.md](CODEMAPS/testing.codemap.md) |
| 多 Agent 协作 | [../development/multi-agent-collaboration.md](../development/multi-agent-collaboration.md) |
| Workflow Spec | [../../.workflow/specs/dev-test-review.yaml](../../.workflow/specs/dev-test-review.yaml) |

## 废弃文档

`*.deprecated.md` 文件已移入各目录的 `deprecated/` 子目录，保留供参考，不再维护。
