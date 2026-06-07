---
title: CI 集成
type: guide
domain: testing
created: 2026-06-06
updated: 2026-06-06
status: deprecated
---

# CI/CD 集成

## 测试流水线阶段

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Lint      │ ──→ │  Unit       │ ──→ │ Integration │ ──→ │   E2E       │
│   Check     │     │  Tests      │     │ Tests       │     │ Tests       │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
     ↓                   ↓                   ↓                   ↓
  格式检查           快速反馈            层间验证            场景验证
  类型检查           <2min               <5min              <10min
```

### 阶段说明

| 阶段 | 命令 | 阶段超时 | 单用例超时 | 失败处理 |
|------|------|---------|-----------|---------|
| Lint | `pnpm lint && pnpm typecheck` | 2min | — | 阻断，不运行后续测试 |
| Unit + Coverage | `pnpm test:coverage` | 10min | 10s | 阻断，低于覆盖率阈值也失败 |
| Integration | `pnpm test:int` | 10min | 60s | 阻断 |
| E2E | `pnpm test:e2e` | 15min | 120s | 阻断 |

## GitHub Actions 配置

```yaml
# .github/workflows/test.yml
name: Test

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  lint:
    name: Lint & Type Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v2
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'pnpm'
      - run: pnpm install --frozen-lockfile
      - run: pnpm lint
      - run: pnpm typecheck

  unit:
    name: Unit Tests + Coverage
    runs-on: ubuntu-latest
    needs: lint
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v2
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'pnpm'
      - run: pnpm install --frozen-lockfile
      - run: pnpm test:coverage
      - name: Upload coverage
        uses: actions/upload-artifact@v4
        with:
          name: coverage-report
          path: coverage/

  integration:
    name: Integration Tests
    runs-on: ubuntu-latest
    needs: unit
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v2
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'pnpm'
      - run: pnpm install --frozen-lockfile
      - run: pnpm test:int

  e2e:
    name: E2E Tests
    runs-on: ubuntu-latest
    needs: integration
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v2
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'pnpm'
      - run: pnpm install --frozen-lockfile
      - run: pnpm test:e2e
```

## 覆盖率配置

```typescript
// vitest.config.ts
export default defineConfig({
  test: {
    coverage: {
      provider: 'v8',
      include: ['src/**'],
      exclude: [
        'src/**/*.d.ts',
        'src/**/*.test.ts',
        'src/**/index.ts',  // 纯导出文件
      ],
      thresholds: {
        lines: 80,
        branches: 70,
        functions: 80,
        statements: 80,
      },
      reporter: ['text', 'json', 'html', 'lcov'],
    },
  },
});
```

## Pre-commit Hook

```json
// package.json
{
  "lint-staged": {
    "*.ts": [
      "eslint --fix",
      "prettier --write"
    ],
    "*.test.ts": [
      "vitest related --run"
    ]
  }
}
```

```json
// .husky/pre-commit
{
  "hooks": {
    "pre-commit": "lint-staged"
  }
}
```

## 本地开发命令

```json
{
  "scripts": {
    "test": "vitest",
    "test:unit": "vitest run",
    "test:int": "vitest run --config vitest.config.int.ts",
    "test:e2e": "vitest run --config vitest.config.e2e.ts",
    "test:all": "vitest run && vitest run --config vitest.config.int.ts && vitest run --config vitest.config.e2e.ts",
    "test:coverage": "vitest run --coverage",
    "test:watch": "vitest watch",
    "test:ui": "vitest --ui",
    "test:related": "vitest related --run"
  }
}
```

## 测试报告

### 控制台输出

```bash
pnpm test:unit --reporter=verbose
```

### HTML 报告

```bash
pnpm test:unit --reporter=html
# 生成 coverage/index.html
```

### CI 中的报告

使用 `actions/upload-artifact` 上传覆盖率报告，PR 中可查看。

## 环境变量

测试环境不需要真实的服务凭证：

```bash
# .env.test
FEISHU_APP_ID=test_app_id
FEISHU_APP_SECRET=test_app_secret
OBSIDIAN_VAULT_PATH=/tmp/test-vault
```

Vitest 自动加载 `.env.test`：

```typescript
// vitest.config.ts
export default defineConfig({
  test: {
    env: {
      NODE_ENV: 'test',
    },
  },
});
```
