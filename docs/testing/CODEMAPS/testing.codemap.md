---
title: Testing System Codemap
type: codemap
domain: testing
created: 2026-06-06
updated: 2026-06-06
---

# Testing System Codemap

## Quick Navigation

### Rust Tests (cargo test --workspace)

| Crate | Unit Tests | Integration Tests | Location |
|-------|-----------|-------------------|----------|
| wb-core | task.rs, event.rs | - | crates/wb-core/src/ |
| wb-collector | manager.rs, config.rs | contract.rs | crates/wb-collector/{src,tests}/ |
| wb-processor | classifier.rs, sla.rs, review*.rs | - | crates/wb-processor/src/ |
| wb-storage | - | 9 integration files | crates/wb-storage/tests/ |
| wb-ai | router.rs, budget.rs | - | crates/wb-ai/src/ |
| wb-scheduler | dependency.rs, resource.rs, scheduler.rs | - | crates/wb-scheduler/src/ |

### Frontend Tests (pnpm test:unit)

| Type | Files | Location |
|------|-------|----------|
| Component | 14 files | src/components/**/*.test.tsx |
| Utility | utils.test.ts | src/lib/ |
| State | store.test.tsx | src/lib/ |
| Integration | 2 files | test/integration/ |

### E2E Tests (pnpm test:e2e)

| Suite | Files | Location |
|-------|-------|----------|
| F1-F6 | 6 spec files | test/e2e/ |

### Acceptance Tests (G-layer)

| Suite | Scenarios | Location |
|-------|-----------|----------|
| G1-G7 | 182 | crates/wb-storage/tests/g*.rs |

### Test Infrastructure

| Item | Location |
|------|----------|
| Test helpers (Rust) | crates/wb-core/src/test_helpers.rs |
| Acceptance helpers | crates/wb-storage/tests/acceptance_helpers.rs |
| E2E helpers | test/e2e/helpers.ts |
| Vitest config | vitest.config.ts |
| Playwright config | playwright.config.ts |
| Nextest config | .config/nextest.toml |
| CI pipeline | .github/workflows/test.yml |
