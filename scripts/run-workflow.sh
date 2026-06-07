#!/bin/bash
# ============================================================
# run-workflow.sh — 测试执行引擎
# 读取 dev-output.json，执行测试，写入 test-report.json
# 用法: ./scripts/run-workflow.sh <task_id>
# ============================================================
set -uo pipefail
# NOTE: 不用 set -e，改为手动检查关键步骤退出码

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TASK_ID="${1:?用法: $0 <task_id>}"
ARTIFACT_DIR="$PROJECT_ROOT/.workflow/artifacts/$TASK_ID"
DEV_OUTPUT="$ARTIFACT_DIR/dev-output.json"

if [ ! -f "$DEV_OUTPUT" ]; then
  echo "ERROR: $DEV_OUTPUT 不存在。先运行 scripts/create-dev-output.sh"
  exit 1
fi

mkdir -p "$ARTIFACT_DIR"

# ============================================================
# Phase 1: Gate Inference
# ============================================================
GATE=$(python3 -c "
import json, sys
with open('$DEV_OUTPUT') as f:
    data = json.load(f)
for cf in data.get('changed_files', []):
    p = cf['path']
    if any(p.startswith(pfx) for pfx in ['crates/wb-core/', 'crates/wb-processor/', 'crates/wb-ai/', 'crates/wb-storage/', 'src-tauri/src/commands/']):
        print('L2'); sys.exit(0)
print('L1')
")

echo "=== Gate Level: $GATE ==="

# ============================================================
# Phase 2: L1 Unit Tests + H1-H2 Security
# ============================================================
echo ""
echo "=== Gate 1: L1 Unit Tests + H1-H2 Security ==="

L1_TOTAL=0; L1_PASSED=0; L1_FAILED=0

# Rust tests
echo "  [Rust] cargo test --workspace..."
RUST_OUT=$(cd "$PROJECT_ROOT" && cargo test --workspace 2>&1) || true
RUST_PASSED=$(echo "$RUST_OUT" | grep "test result" | awk -F'[; ]' '{s+=$4} END {print s+0}')
RUST_FAILED=$(echo "$RUST_OUT" | grep "test result" | awk -F'[; ]' '{s+=$7} END {print s+0}')
L1_TOTAL=$((L1_TOTAL + RUST_PASSED + RUST_FAILED))
L1_PASSED=$((L1_PASSED + RUST_PASSED))
L1_FAILED=$((L1_FAILED + RUST_FAILED))
echo "  [Rust] $RUST_PASSED passed, $RUST_FAILED failed"

# TS tests
echo "  [TS] pnpm test..."
TS_OUT=$(cd "$PROJECT_ROOT" && pnpm test 2>&1) || true
TS_PASSED=$(echo "$TS_OUT" | grep -o "Tests  *[0-9]* passed" | grep -o "[0-9]*" || echo "0")
L1_TOTAL=$((L1_TOTAL + TS_PASSED))
L1_PASSED=$((L1_PASSED + TS_PASSED))
echo "  [TS] $TS_PASSED passed"

# H1: Dependency audit
echo "  [H1] npm audit..."
H1_VULNS=$(cd "$PROJECT_ROOT" && npm audit --json 2>/dev/null | python3 -c "
import sys, json
try:
    d = json.load(sys.stdin)
    v = d.get('metadata', {}).get('vulnerabilities', {})
    print(v.get('critical', 0) + v.get('high', 0))
except: print(0)
" 2>/dev/null || echo "0")
echo "  [H1] $H1_VULNS critical/high vulnerabilities"

# H2: Secret scan
echo "  [H2] Secret scan..."
H2_SECRETS=$(cd "$PROJECT_ROOT" && grep -rn \
  "sk-[a-zA-Z0-9]\{20,\}\|api_key\s*=\s*\"[a-zA-Z0-9]\{20,\}" \
  crates/ src/ --include="*.rs" --include="*.ts" \
  --exclude-dir=target --exclude-dir=node_modules 2>/dev/null | wc -l | tr -d ' ') || H2_SECRETS=0
echo "  [H2] $H2_SECRETS potential secrets"

# ============================================================
# Phase 3: E2E Tests
# ============================================================
echo ""
echo "=== Gate 3: E2E Tests ==="
E2E_OUT=$(cd "$PROJECT_ROOT" && npx playwright test 2>&1) || true
E2E_PASSED=$(echo "$E2E_OUT" | grep -o "[0-9]* passed" | grep -o "[0-9]*" || echo "0")
E2E_FAILED=$(echo "$E2E_OUT" | grep -o "[0-9]* failed" | grep -o "[0-9]*" || echo "0")
E2E_SKIPPED=$(echo "$E2E_OUT" | grep -o "[0-9]* skipped" | grep -o "[0-9]*" || echo "0")
echo "  [E2E] ${E2E_PASSED} passed, ${E2E_FAILED} failed, ${E2E_SKIPPED} skipped"

# ============================================================
# Phase 4: Write test-report.json
# ============================================================
TOTAL=$((L1_TOTAL + E2E_PASSED + E2E_FAILED + E2E_SKIPPED))
PASSED=$((L1_PASSED + E2E_PASSED))
FAILED=$((L1_FAILED + E2E_FAILED))

if [ "$FAILED" -gt 0 ]; then
  OVERALL="fail"
elif [ "$E2E_SKIPPED" -gt 0 ]; then
  OVERALL="partial_pass"
else
  OVERALL="pass"
fi

cat > "$ARTIFACT_DIR/test-report.json" << EOF
{
  "task_id": "$TASK_ID",
  "gate_level": "$GATE",
  "result": "$OVERALL",
  "summary": {
    "total": $TOTAL,
    "passed": $PASSED,
    "failed": $FAILED,
    "skipped": $E2E_SKIPPED
  },
  "failures": [],
  "uncovered_paths": [],
  "security_scan": {
    "h1_dependency_audit": {
      "vulnerabilities_found": $H1_VULNS,
      "critical": 0,
      "high": $H1_VULNS,
      "details": []
    },
    "h2_secret_scan": {
      "secrets_found": $H2_SECRETS,
      "details": []
    }
  },
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo ""
echo "=== Summary ==="
echo "  Gate: $GATE"
echo "  Result: $OVERALL"
echo "  Total: $TOTAL | Passed: $PASSED | Failed: $FAILED | Skipped: $E2E_SKIPPED"
echo "  H1 vulns: $H1_VULNS | H2 secrets: $H2_SECRETS"
echo ""
echo "test-report.json → $ARTIFACT_DIR/test-report.json"

[ "$FAILED" -gt 0 ] && exit 1
exit 0
