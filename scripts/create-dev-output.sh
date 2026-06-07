#!/bin/bash
# ============================================================
# create-dev-output.sh
# dev-agent 完成开发后调用，生成 .workflow/artifacts/{task_id}/dev-output.json
# 用法: ./scripts/create-dev-output.sh [task_id]
# ============================================================
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TASK_ID="${1:-$(date +%Y%m%d-%H%M%S)}"
ARTIFACT_DIR="$PROJECT_ROOT/.workflow/artifacts/$TASK_ID"
mkdir -p "$ARTIFACT_DIR"

# 获取变更文件列表（相对于项目根目录）
CHANGED_FILES=$(cd "$PROJECT_ROOT" && git diff --name-only HEAD~1 2>/dev/null || echo "")
if [ -z "$CHANGED_FILES" ]; then
  CHANGED_FILES=$(cd "$PROJECT_ROOT" && git diff --name-only --cached 2>/dev/null || echo "")
fi

# 推断 affected_modules
MODULES=""
for f in $CHANGED_FILES; do
  case "$f" in
    crates/wb-core/*) MODULES="$MODULES wb-core" ;;
    crates/wb-processor/*) MODULES="$MODULES wb-processor" ;;
    crates/wb-ai/*) MODULES="$MODULES wb-ai" ;;
    crates/wb-storage/*) MODULES="$MODULES wb-storage" ;;
    crates/wb-scheduler/*) MODULES="$MODULES wb-scheduler" ;;
    crates/wb-collector/*) MODULES="$MODULES wb-collector" ;;
    src-tauri/*) MODULES="$MODULES src-tauri" ;;
    src/*) MODULES="$MODULES frontend" ;;
  esac
done
MODULES=$(echo "$MODULES" | tr ' ' '\n' | sort -u | tr '\n' ' ' | xargs)

# 构建 changed_files JSON 数组
CHANGED_JSON="["
FIRST=true
for f in $CHANGED_FILES; do
  if [ "$FIRST" = true ]; then FIRST=false; else CHANGED_JSON="$CHANGED_JSON,"; fi
  # 判断 change_type
  if git show HEAD~1:"$f" >/dev/null 2>&1; then
    if [ ! -f "$PROJECT_ROOT/$f" ]; then
      CT="deleted"
    else
      CT="modified"
    fi
  else
    CT="added"
  fi
  CHANGED_JSON="$CHANGED_JSON{\"path\":\"$f\",\"change_type\":\"$CT\"}"
done
CHANGED_JSON="$CHANGED_JSON]"

# 构建 modules JSON 数组
MOD_JSON="["
FIRST=true
for m in $MODULES; do
  if [ "$FIRST" = true ]; then FIRST=false; else MOD_JSON="$MOD_JSON,"; fi
  MOD_JSON="$MOD_JSON\"$m\""
done
MOD_JSON="$MOD_JSON]"

# 写入 dev-output.json
cat > "$ARTIFACT_DIR/dev-output.json" << EOF
{
  "task_id": "$TASK_ID",
  "task_type": "feature",
  "changed_files": $CHANGED_JSON,
  "affected_modules": $MOD_JSON,
  "new_tests": [],
  "acceptance_criteria": [],
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo "dev-output.json written to $ARTIFACT_DIR/dev-output.json"
echo "Task ID: $TASK_ID"
