#!/bin/bash
set -euo pipefail

# =============================================================================
# create-dev-output.sh
# 生成 dev-output.json，供主 Agent 读取
# =============================================================================

TASK_ID="${1:?用法: $0 <task_id>}"
ARTIFACTS_DIR=".workflow/artifacts/${TASK_ID}"

# 加载日志库
source "$(dirname "$0")/workflow-logger.sh"
workflow_log_init "$TASK_ID" "$ARTIFACTS_DIR"
log_phase "INIT"

# 检查是否在 git 仓库中
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    log_error "不在 git 仓库中"
    workflow_log_finish "fail"
    exit 1
fi
log_debug "Git 仓库检查通过"

# 创建 artifacts 目录
mkdir -p "$ARTIFACTS_DIR"
log_debug "Artifacts 目录: ${ARTIFACTS_DIR}"

# =============================================================================
# 获取变更的文件列表
# =============================================================================
log_phase "DETECT_CHANGES"

# 优先使用 staged files，其次使用 unstaged files，最后使用 HEAD~1
if git diff --cached --name-only --diff-filter=ACMR | grep -q .; then
    CHANGED_FILES=$(git diff --cached --name-only --diff-filter=ACMR)
    log_info "使用 staged files"
    log_debug "变更来源: git diff --cached"
elif git diff --name-only --diff-filter=ACMR | grep -q .; then
    CHANGED_FILES=$(git diff --name-only --diff-filter=ACMR)
    log_info "使用 unstaged files"
    log_debug "变更来源: git diff"
else
    CHANGED_FILES=$(git diff --name-only --diff-filter=ACMR HEAD~1 2>/dev/null || echo "")
    if [ -z "$CHANGED_FILES" ]; then
        log_warn "未检测到变更文件，生成空 dev-output.json"
        CHANGED_FILES=""
    else
        log_info "使用 HEAD~1 的变更"
        log_debug "变更来源: git diff HEAD~1"
    fi
fi

# 记录每个变更文件
if [ -n "$CHANGED_FILES" ]; then
    log_info "变更文件列表:"
    echo "$CHANGED_FILES" | while IFS= read -r f; do
        log_debug "  - ${f}"
    done
fi

# 过滤出代码文件（排除 docs/、.config/、scripts/）
CODE_FILES=$(echo "$CHANGED_FILES" | grep -E '^(crates/|src/|src-tauri/)' || true)

if [ -z "$CODE_FILES" ]; then
    log_warn "没有 crates/、src/、src-tauri/ 下的代码变更"
    log_warn "workflow 通常不需要执行，但继续生成 dev-output.json"
else
    log_info "代码变更文件:"
    echo "$CODE_FILES" | while IFS= read -r f; do
        log_debug "  - ${f}"
    done
fi

# =============================================================================
# 推断 test_level
# =============================================================================
log_phase "INFER_GATE"

infer_test_level() {
    local files="$1"
    local level=1
    local reasons=()

    # 检查是否修改了核心模块（wb-core、公共类型、配置）
    if echo "$files" | grep -qE '^crates/wb-core/|^crates/.*/config\.rs$|^Cargo\.toml$'; then
        level=3
        reasons+=("核心模块变更")
    fi

    # 检查是否修改了多个 crate
    local crate_count
    crate_count=$(echo "$files" | grep -E '^crates/' | cut -d'/' -f2 | sort -u | wc -l | tr -d ' ')
    if [ "$crate_count" -gt 1 ] && [ "$level" -lt 3 ]; then
        level=2
        reasons+=("多 crate 变更(${crate_count})")
    fi

    # 检查是否修改了公共接口（lib.rs、mod.rs）
    if echo "$files" | grep -qE 'lib\.rs$|mod\.rs$'; then
        [ "$level" -lt 2 ] && level=2
        reasons+=("公共接口变更")
    fi

    # 检查是否涉及安全敏感代码
    if echo "$files" | grep -qE 'auth|token|secret|password|crypto'; then
        [ "$level" -lt 3 ] && level=3
        reasons+=("安全敏感代码")
    fi

    # 检查是否涉及 E2E（前端 + 后端同时修改）
    local has_frontend=false
    local has_backend=false
    echo "$files" | grep -qE '^src/|^src-tauri/' && has_frontend=true
    echo "$files" | grep -qE '^crates/' && has_backend=true
    if $has_frontend && $has_backend; then
        level=3
        reasons+=("前后端同时变更")
    fi

    # 输出推断原因
    if [ ${#reasons[@]} -gt 0 ]; then
        echo "${level}|$(IFS=','; echo "${reasons[*]}")"
    else
        echo "${level}|默认"
    fi
}

INFER_RESULT=$(infer_test_level "$CODE_FILES")
TEST_LEVEL=$(echo "$INFER_RESULT" | cut -d'|' -f1)
INFER_REASON=$(echo "$INFER_RESULT" | cut -d'|' -f2)

log_decision "test_level=${TEST_LEVEL}" "推断原因: ${INFER_REASON}"

# 推断 required_agents
infer_required_agents() {
    local level="$1"
    local agents='["dev-agent"]'

    if [ "$level" -ge 2 ]; then
        agents='["dev-agent","test-agent","product-reviewer"]'
    fi
    if [ "$level" -ge 3 ]; then
        agents='["dev-agent","test-agent","review-agent","product-reviewer"]'
    fi

    echo "$agents"
}

REQUIRED_AGENTS=$(infer_required_agents "$TEST_LEVEL")
log_decision "required_agents=${REQUIRED_AGENTS}" "基于 test_level=${TEST_LEVEL}"

# 将变更文件转为 JSON 数组
files_to_json_array() {
    local files="$1"
    if [ -z "$files" ]; then
        echo "[]"
        return
    fi
    echo "$files" | jq -R -s 'split("\n") | map(select(length > 0))'
}

CHANGED_FILES_JSON=$(files_to_json_array "$CHANGED_FILES")
CODE_FILES_JSON=$(files_to_json_array "$CODE_FILES")

# =============================================================================
# 写入 dev-output.json
# =============================================================================
log_phase "WRITE_OUTPUT"

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

cat > "${ARTIFACTS_DIR}/dev-output.json" << EOF
{
  "task_id": "${TASK_ID}",
  "timestamp": "${TIMESTAMP}",
  "changed_files": ${CHANGED_FILES_JSON},
  "code_changed_files": ${CODE_FILES_JSON},
  "test_level": ${TEST_LEVEL},
  "summary": "Auto-generated by create-dev-output.sh",
  "required_agents": ${REQUIRED_AGENTS},
  "gate_config": {
    "max_retries": 2,
    "timeout_seconds": 300,
    "fail_fast": false
  }
}
EOF

log_artifact "write" "${ARTIFACTS_DIR}/dev-output.json" "task_id=${TASK_ID}"
log_info "task_id: ${TASK_ID}"
log_info "changed_files: $(echo "$CHANGED_FILES" | wc -l | tr -d ' ') 个文件"
log_info "code_changed_files: $(echo "$CODE_FILES" | wc -l | tr -d ' ') 个文件"
log_info "test_level: ${TEST_LEVEL}"
log_info "required_agents: ${REQUIRED_AGENTS}"

workflow_log_finish "done"

echo ""
echo "下一步: ./scripts/run-workflow.sh ${TASK_ID}"
