---
title: 测试文档索引
type: index
domain: testing
created: 2026-06-06
updated: 2026-06-07
status: active
---

# 测试文档索引

## 架构与规范

| 文档 | 说明 |
|------|------|
| [architecture.md](architecture.md) | 总体架构（顶层文档） |
| [conventions.md](conventions.md) | 命名、组织、编写规范 |
| [layers/overview.md](layers/overview.md) | 层级定义与边界划分 |

## 层级指南

| 文档 | 说明 |
|------|------|
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

## 多 Agent 协作

| 文档 | 说明 |
|------|------|
| [../development/multi-agent-collaboration.md](../development/multi-agent-collaboration.md) | 多 Agent 协作开发规范 |
| [../../.workflow/specs/dev-test-review.yaml](../../.workflow/specs/dev-test-review.yaml) | Workflow spec 定义 |

## 测试运行指南

| 门控 | 命令 | 耗时 |
|------|------|------|
| Fast | `cargo nextest run --workspace` | <2min |
| Deep | `pnpm test` | <10min |
| Full | `pnpm test:all` | <15min |
| Nightly | CI scheduled pipeline | <20min |

## 导航

| 资源 | 位置 |
|------|------|
| 测试 Codemap | [CODEMAPS/testing.codemap.md](CODEMAPS/testing.codemap.md) |
| CI 配置 | `.github/workflows/test.yml` |
| Runbook | `.claude/plans/testing-redesign.md` |
| Workflow Spec | `.workflow/specs/dev-test-review.yaml` |
| Artifact Schemas | `.workflow/templates/` |

## 当前状态（2026-06-07 验证）

- 总测试函数数: 1383 通过 / 0 失败 / 11 跳过
- Rust 侧: 1204 个测试函数（wb-ai:122, wb-collector:73, wb-core:48, wb-processor:387, wb-scheduler:71, wb-storage:131, 集成测试:~372）
- TypeScript 侧: 164 个测试函数（vitest 18 个测试文件）
- E2E (Playwright): 15 通过 / 11 跳过（F3 处理管线 UI + F5 调度器接入未实现）
- H1-H2 安全扫描: 通过（npm audit 零漏洞，无敏感数据泄露）
- H3-H5 安全测试: 设计完成，待实现
- G 层验收场景: 182 个场景已定义（scenarios/catalog.md），cucumber-rs 实现有 regex 冲突待修复，详见 #fix-cucumber-regex

## 废弃文档

`*.deprecated.md` 文件保留供参考，详见各目录。
