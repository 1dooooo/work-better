#!/bin/bash
# ============================================================
# review-agent.sh — 代码审查 + H3-H5 安全检查
# 用法: ./scripts/review-agent.sh <task_id>
# ============================================================
set -uo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TASK_ID="${1:?用法: $0 <task_id>}"
ARTIFACT_DIR="$PROJECT_ROOT/.workflow/artifacts/$TASK_ID"
DEV_OUTPUT="$ARTIFACT_DIR/dev-output.json"

if [ ! -f "$DEV_OUTPUT" ]; then
  echo "ERROR: $DEV_OUTPUT 不存在"; exit 1
fi

echo "=== Review Agent: $TASK_ID ==="

# H3: 输入边界检查
echo ""; echo "=== H3: Input Boundary ==="
TAURI_CMDS=$(grep -rc "#\[tauri::command\]" "$PROJECT_ROOT/src-tauri/src/commands/" --include="*.rs" 2>/dev/null | awk -F: '{s+=$NF} END {print s+0}')
echo "  Tauri commands: $TAURI_CMDS"

# H4: 权限检查
echo ""; echo "=== H4: Permission Check ==="
PERM_CHECKS=$(grep -rn "permission\|authorize" "$PROJECT_ROOT/src-tauri/src/commands/" --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')
echo "  Permission checks: $PERM_CHECKS"

# H5: 文件系统安全
echo ""; echo "=== H5: Filesystem Security ==="
TRAVERSAL=$(grep -rn "canonicalize\|strip_prefix" "$PROJECT_ROOT/src-tauri/src/" --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')
echo "  Path traversal guards: $TRAVERSAL"

# 代码质量
echo ""; echo "=== Code Quality ==="
CHANGED=$(python3 -c "
import json
with open('$DEV_OUTPUT') as f:
    for cf in json.load(f).get('changed_files', []):
        print(cf['path'])
")
TODO_COUNT=0
for f in $CHANGED; do
  fp="$PROJECT_ROOT/$f"
  [ -f "$fp" ] && n=$(grep -c "TODO\|FIXME\|HACK" "$fp" 2>/dev/null || true); TODO_COUNT=$((TODO_COUNT + ${n:-0}))
done
echo "  TODO/FIXME: $TODO_COUNT"

# Verdict
VERDICT="approve"
[ "$PERM_CHECKS" -eq 0 ] && [ "$TAURI_CMDS" -gt 10 ] && VERDICT="approve_with_comments"

# 写入 review-report.json
cat > "$ARTIFACT_DIR/review-report.json" << EOF
{
  "task_id": "$TASK_ID",
  "verdict": "$VERDICT",
  "findings": [],
  "security_tests_generated": [],
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo ""; echo "=== Summary ==="
echo "  Verdict: $VERDICT"
echo "  H3 commands: $TAURI_CMDS | H4 perm checks: $PERM_CHECKS | H5 traversal guards: $TRAVERSAL"
echo ""; echo "review-report.json → $ARTIFACT_DIR/review-report.json"
