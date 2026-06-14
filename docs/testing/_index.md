---
title: 测试文档索引
type: index
domain: testing
created: 2026-06-06
updated: 2026-06-13
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
| [product-acceptance.md](product-acceptance.md) | 产品验收报告（AI Pipeline 阶段三） |
| [phase3-test-report.md](phase3-test-report.md) | 阶段三测试报告（1425/1425 通过） |
| [e2e-test-report.md](e2e-test-report.md) | L5 E2E 测试验证报告（30/30 通过） |
| [acceptance-criteria-phase2.md](acceptance-criteria-phase2.md) | Phase 2 验收标准（F2.1.1/F2.4.2/F2.4.3） |
| [acceptance-criteria-template.md](acceptance-criteria-template.md) | 验收标准模板 |

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

## 验收测试

| 文档 | 说明 |
|------|------|
| [acceptance-test-migration-plan.md](acceptance-test-migration-plan.md) | 验收测试迁移计划 |
| [acceptance-test-migration-summary.md](acceptance-test-migration-summary.md) | 验收测试迁移总结 |
| [real-backend-test-plan.md](real-backend-test-plan.md) | 真实后端测试计划 |
| [real-backend-test-summary.md](real-backend-test-summary.md) | 真实后端测试总结 |
| [f1-f5-completion-plan.md](f1-f5-completion-plan.md) | F1-F5 功能完成计划 |
| [f1-f5-completion-summary.md](f1-f5-completion-summary.md) | F1-F5 功能完成总结 |

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
