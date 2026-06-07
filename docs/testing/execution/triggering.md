---
title: 分级触发与执行策略
type: structural
domain: testing
created: 2026-06-06
updated: 2026-06-06
status: active
---

# 分级触发与执行策略

> **维护说明**：当修改触发条件、变更影响分析规则、或 CI 配置时更新本文档。

## 分级触发体系

### Gate 1: 快速门 (<2 min)

**触发条件**：每次代码变更
**包含测试**：受影响模块的 A 层 + D 层

```bash
cargo nextest run -E 'package(wb-processor) + test(unit)' --message-format json
pnpm vitest run --reporter=json --filter="ModelSettings"
```

### Gate 2: 深度门 (<10 min)

**触发条件**：PR 创建/更新
**包含测试**：Gate 1 + B+E 集成 + F E2E + G 受影响子集

```bash
cargo nextest run --message-format json --profile pr
cargo test --test acceptance -- --tags="@processing"
pnpm exec playwright test --workers=4
```

### Gate 3: 全量门 (<15 min)

**触发条件**：发布前 / merge to main
**包含测试**：全部 A-G (432 场景)

执行顺序 (有重叠)：A+D → B+E+C → F → G 全量并行

### Gate 4: 契约验证门 (夜间)

**触发条件**：cron 02:00 / lark-cli 版本变更
**包含测试**：C 层 + G 层外部依赖子集
失败发通知，不阻塞开发。

## 变更影响分析

### Rust 依赖图

```
wb-core ← 所有 crate
wb-processor ← wb-core
wb-ai ← wb-core
wb-storage ← wb-core
wb-collector ← wb-core
wb-scheduler ← wb-core
src-tauri ← 所有 crates
```

### 影响规则表

| 变更文件 | 影响 A 层 | 影响 B 层 | 影响 G 层 |
|----------|----------|----------|----------|
| wb-core/src/task.rs | A3, A6 | B1 | G4 |
| wb-core/src/event.rs | A1, A11 | B1 | G1, G2 |
| wb-processor/src/classifier.rs | A1 | B2 | G2 |
| wb-processor/src/sla.rs | A4 | - | G2 |
| wb-processor/src/review*.rs | A6 | - | G2 |
| wb-processor/src/task/*.rs | A3 | - | G4 |
| wb-processor/src/report/*.rs | - | - | G5 |
| wb-ai/src/router.rs | A2 | - | G2 |
| wb-ai/src/budget.rs | A5 | - | G2, G6 |
| wb-storage/src/sqlite/*.rs | - | B1 | G3 |
| wb-storage/src/obsidian/*.rs | - | B7 | G3 |
| wb-storage/src/freshness/*.rs | - | B8 | G3 |
| wb-storage/src/vector/*.rs | - | B9 | G3 |
| wb-storage/src/config.rs | - | B3 | G6 |
| wb-collector/src/feishu/*.rs | A11 | - | G1 |
| wb-collector/src/manager.rs | A10 | B4 | G1 |
| wb-scheduler/src/*.rs | A7-A9 | B5 | G6 |
| src-tauri/src/commands/*.rs | A12 | B2-B6 | G6 |
| src/components/*.tsx | D1 | E1 | G6 |
| src/lib/tauri.ts | - | E1 | G6 |
| src/capture/*.tsx | D1 | E1 | G1 |

### 自动化脚本

```bash
#!/bin/bash
# scripts/test-impact.sh
CHANGED_FILES=$(git diff --name-only HEAD~1)
RUST_PKGS=""; G_DOMAINS=""

for f in $CHANGED_FILES; do
  case $f in
    crates/wb-core/src/task.*) RUST_PKGS+="wb-core "; G_DOMAINS+="G4 " ;;
    crates/wb-core/src/event.*) RUST_PKGS+="wb-core "; G_DOMAINS+="G1 G2 " ;;
    crates/wb-processor/src/classifier.*) RUST_PKGS+="wb-processor "; G_DOMAINS+="G2 " ;;
    crates/wb-processor/src/sla.*) RUST_PKGS+="wb-processor "; G_DOMAINS+="G2 " ;;
    crates/wb-processor/src/review*) RUST_PKGS+="wb-processor "; G_DOMAINS+="G2 " ;;
    crates/wb-processor/src/task/*) RUST_PKGS+="wb-processor "; G_DOMAINS+="G4 " ;;
    crates/wb-processor/src/report/*) RUST_PKGS+="wb-processor "; G_DOMAINS+="G5 " ;;
    crates/wb-ai/src/router.*) RUST_PKGS+="wb-ai "; G_DOMAINS+="G2 " ;;
    crates/wb-ai/src/budget.*) RUST_PKGS+="wb-ai "; G_DOMAINS+="G2 G6 " ;;
    crates/wb-storage/src/sqlite/*) RUST_PKGS+="wb-storage "; G_DOMAINS+="G3 " ;;
    crates/wb-storage/src/obsidian/*) RUST_PKGS+="wb-storage "; G_DOMAINS+="G3 " ;;
    crates/wb-storage/src/freshness/*) RUST_PKGS+="wb-storage "; G_DOMAINS+="G3 " ;;
    crates/wb-storage/src/vector/*) RUST_PKGS+="wb-storage "; G_DOMAINS+="G3 " ;;
    crates/wb-collector/src/feishu/*) RUST_PKGS+="wb-collector "; G_DOMAINS+="G1 " ;;
    crates/wb-collector/src/manager.*) RUST_PKGS+="wb-collector "; G_DOMAINS+="G1 " ;;
    crates/wb-scheduler/src/*) RUST_PKGS+="wb-scheduler "; G_DOMAINS+="G6 " ;;
    src-tauri/src/commands/*) RUST_PKGS+="wb-core wb-collector wb-storage wb-scheduler "; G_DOMAINS+="G6 " ;;
    src/components/*.tsx) G_DOMAINS+="G6 " ;;
    src/lib/tauri.ts) G_DOMAINS+="G6 " ;;
    src/capture/*.tsx) G_DOMAINS+="G1 " ;;
  esac
done

RUST_PKGS=$(echo $RUST_PKGS | tr ' ' '\n' | sort -u | tr '\n' ' ')
G_DOMAINS=$(echo $G_DOMAINS | tr ' ' '\n' | sort -u | tr '\n' ' ')

echo "RUST: cargo nextest run -E 'package($(echo $RUST_PKGS | tr ' ' '|'))' --message-format json"
echo "G: $G_DOMAINS"
```

## 并行执行策略

### Rust: nextest 进程级并行

```toml
# .config/nextest.toml
[profile.default]
test-threads = "num-cpus"

[profile.pr]
fail-fast = true
slow-timeout = { period = "30s", terminate-after = 2 }

[profile.release]
fail-fast = false
slow-timeout = { period = "60s", terminate-after = 3 }

[profile.ci]
retries = 2
fail-fast = false
```

### G 层: 7 组 domain 并行

```
G1 (采集) ──┐
G2 (处理) ──┤
G3 (存储) ──┤
G4 (任务) ──├──→ 7 进程并行，每个 <2 min
G5 (报告) ──┤
G6 (系统) ──┤
G7 (横切) ──┘
```

### Playwright: workers 并行

```typescript
// playwright.config.ts
export default defineConfig({
  workers: process.env.CI ? 4 : 2,
  retries: 2,
  reporter: [['json', { outputFile: 'test-results/results.json' }]],
});
```

## CI Pipeline

```yaml
name: Test Pipeline
on:
  push:
    branches: [main, dev]
  pull_request:
  schedule:
    - cron: '0 2 * * *'

jobs:
  fast-gate:
    runs-on: macos-latest
    timeout-minutes: 5
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: taiki-e/install-action@nextest
      - uses: pnpm/action-setup@v4
      - run: pnpm install
      - run: bash scripts/test-impact.sh | bash
      - run: pnpm vitest run --reporter=json

  deep-gate:
    if: github.event_name == 'pull_request'
    needs: fast-gate
    runs-on: macos-latest
    timeout-minutes: 15
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: taiki-e/install-action@nextest
      - uses: pnpm/action-setup@v4
      - run: pnpm install
      - run: cargo nextest run --message-format json --profile pr
      - run: pnpm vitest run
      - run: pnpm exec playwright test
      - run: cargo test --test acceptance
      - uses: actions/upload-artifact@v4
        if: failure()
        with:
          name: test-results
          path: test-results/

  full-gate:
    if: github.ref == 'refs/heads/main'
    needs: deep-gate
    runs-on: macos-latest
    timeout-minutes: 20
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: taiki-e/install-action@nextest
      - uses: pnpm/action-setup@v4
      - run: pnpm install
      - run: cargo nextest run --message-format json --profile release
      - run: pnpm vitest run
      - run: pnpm exec playwright test
      - run: cargo test --test acceptance -- --parallel

  contract-gate:
    if: github.event_name == 'schedule'
    runs-on: macos-latest
    timeout-minutes: 60
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --test contract
      - run: cargo test --test acceptance -- --tags=@external
```

## 测试报告

### Agent 可解析失败输出

```
TEST FAILURE: A2-03
├── 测试: EntityExtraction threshold boundary
├── 关联场景: G2-06 (模型自动升级)
├── 失败原因: confidence 0.69 未触发升级 (expected: trigger upgrade)
├── 代码位置: crates/wb-ai/src/router.rs:42
└── 建议: should_upgrade() 边界条件应包含等于阈值
```
