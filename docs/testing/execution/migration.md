---
title: 文档迁移与实施路线图
type: structural
domain: testing
created: 2026-06-06
updated: 2026-06-06
status: draft
---

# 文档迁移与实施路线图

> **维护说明**：当阶段完成、时间线调整、或新增验收标准时更新本文档。

## 现状分析

### 当前文档问题

| 文件 | 问题 |
|------|------|
| strategy.md | 仅 TypeScript/Vitest，未考虑 Rust |
| conventions.md | 仅 TS 命名，缺 Rust |
| ci.md | GitHub Actions 模板未实现 |
| layers/unit.md | 仅 TS 示例 |
| layers/integration.md | 仅 TS 示例 |
| layers/e2e.md | 仅 TS 示例 |
| infrastructure/harness.md | 描述不存在的 Fake/Factories |
| infrastructure/mocking.md | 仅 MSW |
| infrastructure/fixtures.md | 仅 fishery/faker |

### 当前代码测试状态

| 维度 | 状态 |
|------|------|
| Rust inline tests | 81 模块有 #[cfg(test)] |
| Rust integration tests | 1 个 (scheduler_tests.rs) |
| TypeScript tests | 0 个 |
| Vitest 配置 | 不存在 |
| 测试依赖 | 未安装 |

## 新文档结构

```
docs/testing/
├── _index.md
├── architecture.md              # ← 01
├── layers/
│   ├── unit-rust.md             # A 层
│   ├── unit-frontend.md        # D 层
│   ├── integration-rust.md     # B 层
│   ├── integration-frontend.md # E 层
│   ├── contract.md             # C 层
│   ├── e2e.md                  # F 层
│   └── acceptance.md           # G 层
├── scenarios/
│   ├── catalog.md              # ← 03
│   ├── product-scenarios.md    # 182 个产品场景
│   └── code-scenarios.md       # 250 个代码级场景
├── execution/
│   ├── triggering.md           # ← 04
│   ├── parallelization.md
│   └── ci-pipeline.md
├── infrastructure/
│   ├── frameworks.md
│   ├── fixtures.md
│   ├── mocking.md
│   └── fakes.md
└── conventions.md
```

### 文档映射

| 旧文档 | 新文档 | 变更 |
|--------|--------|------|
| strategy.md | architecture.md | 重写 |
| conventions.md | conventions.md | 扩展 |
| ci.md | execution/ci-pipeline.md | 重写 |
| layers/unit.md | unit-rust.md + unit-frontend.md | 拆分+重写 |
| layers/integration.md | integration-rust.md + integration-frontend.md | 拆分+重写 |
| layers/e2e.md | e2e.md + acceptance.md | 拆分+重写 |
| infrastructure/harness.md | frameworks.md + fakes.md | 拆分+重写 |
| infrastructure/mocking.md | mocking.md | 扩展 |
| infrastructure/fixtures.md | fixtures.md | 扩展 |
| (新增) | scenarios/catalog.md | 新增 |
| (新增) | execution/triggering.md | 新增 |

## 实施路线图

### Phase 0: 文档迁移 (Week 1)

- [ ] 将 5 个设计文档整理为正式文档
- [ ] 创建新目录结构
- [ ] 更新 _index.md
- [ ] 标记旧文档 deprecated

### Phase 1: 基础设施搭建 (Week 1-2)

**Rust：**
- [ ] `cargo install cargo-nextest`
- [ ] 添加 dev-deps: rstest, insta, httpmock, cucumber-rs
- [ ] 创建 `.config/nextest.toml`
- [ ] 创建 fixtures/snapshots 目录

**TypeScript：**
- [ ] 安装 vitest, @testing-library/react, playwright, msw
- [ ] 创建 vitest.config.ts / playwright.config.ts
- [ ] 创建 test/ 目录结构
- [ ] 添加 package.json scripts

### Phase 2: A 层测试补充 (Week 2-3)

- [ ] 为 81 个模块补充 rstest 参数化测试
- [ ] 重点: wb-collector, wb-storage 缺少的测试
- [ ] 补充 insta 快照测试 (lark-cli 输出)
- [ ] 验证 `cargo nextest run` 全部通过

### Phase 3: B+E 层 (Week 3-4)

- [ ] B1: SQLite EventLog (in-memory)
- [ ] B2-B6: Tauri 命令层
- [ ] B7: Obsidian writer (tempdir)
- [ ] B8-B9: Freshness + Vector
- [ ] E1: 前端 invoke()

### Phase 4: C 层契约 (Week 4)

- [ ] 录制 lark-cli 真实输出
- [ ] 创建 insta snapshots
- [ ] 配置 nightly CI

### Phase 5: F 层 E2E (Week 5)

- [ ] Playwright Tauri app 测试
- [ ] 实现 20 个场景
- [ ] 并行执行

### Phase 6: G 层验收 (Week 5-8)

- [ ] 搭建 cucumber-rs 框架
- [ ] 实现 TestWorld + 基础 steps
- [ ] 分批实现 182 场景:
  - Week 5: G1 (采集, 37)
  - Week 6: G2 (处理, 34) + G4 (任务, 20)
  - Week 7: G3 (存储, 28) + G5 (报告, 12)
  - Week 8: G6 (系统, 38) + G7 (横切, 13)

### Phase 7: CI/CD (Week 8-9)

- [ ] GitHub Actions 4 gates
- [ ] 变更影响分析脚本
- [ ] 测试报告生成
- [ ] 失败通知

### Phase 8: 文档完善 (Week 9-10)

- [ ] 所有层代码示例
- [ ] 测试编写指南
- [ ] Mock/Fake 使用指南
- [ ] 更新 CLAUDE.md 测试规则

## 验收标准

### 每个 Phase

- [ ] 所有新增测试通过
- [ ] 覆盖率 ≥80% lines, ≥70% branches
- [ ] 运行时间在预算内
- [ ] 文档已更新
- [ ] CI 通过

### 最终验收

- [ ] 432 个测试场景全部可执行
- [ ] 分级触发正常工作
- [ ] 并行执行 <15 min
- [ ] Agent 能理解失败原因
- [ ] 测试通过 = 99.99% 产品可用
