# Testing Requirements

## Minimum Test Coverage: 80%

Test Types (ALL required):
1. **Unit Tests** - Individual functions, utilities, components
2. **Integration Tests** - API endpoints, database operations
3. **E2E Tests** - Critical user flows (framework chosen per language)

## Test-Driven Development

MANDATORY workflow:
1. Write test first (RED)
2. Run test - it should FAIL
3. Write minimal implementation (GREEN)
4. Run test - it should PASS
5. Refactor (IMPROVE)
6. Verify coverage (80%+)

## Troubleshooting Test Failures

1. Use **tdd-guide** agent
2. Check test isolation
3. Verify mocks are correct
4. Fix implementation, not tests (unless tests are wrong)

## Execution Path Verification

Code paths covered by tests **must match** the paths exercised in production. Tests that cover dead code or exercise different call chains than production provide false confidence.

### Principle

Test coverage metrics (line, branch, function) measure *what* the tests touch, not whether those paths are the ones production actually runs. Execution path verification closes this gap by comparing tested paths against real call traces.

### Verification Method

1. **Collect production call traces** — instrument or profile the production flow to record the functions and branches actually invoked.
2. **Collect test call traces** — run the test suite under coverage instrumentation and record the functions and branches exercised.
3. **Diff the two sets** — anything in production but not in tests is an untested path; anything in tests but not in production is dead or redundant test code.

### When to Verify

The `test-agent` performs execution path verification during **L4 (system-level) testing**. At this level the full integration surface is available, making call-trace comparison meaningful.

### Verification Output

| Output | Meaning | Action |
|--------|---------|--------|
| **Untested path** | Production path not covered by any test | Add targeted test |
| **Duplicate implementation** | Same logic exists in multiple locations | Consolidate or alias |
| **Dead test path** | Test exercises code no production path reaches | Remove or annotate as regression guard |

> **Reference:** For the full verification procedure, trace-collection setup, and tooling details, see [Testing Architecture](../../../docs/testing/architecture.md).

## Agent Support

- **tdd-guide** - Use PROACTIVELY for new features, enforces write-tests-first

## Test Structure (AAA Pattern)

Prefer Arrange-Act-Assert structure for tests:

```typescript
test('calculates similarity correctly', () => {
  // Arrange
  const vector1 = [1, 0, 0]
  const vector2 = [0, 1, 0]

  // Act
  const similarity = calculateCosineSimilarity(vector1, vector2)

  // Assert
  expect(similarity).toBe(0)
})
```

### Test Naming

Use descriptive names that explain the behavior under test:

```typescript
test('returns empty array when no markets match query', () => {})
test('throws error when API key is missing', () => {})
test('falls back to substring search when Redis is unavailable', () => {})
```
