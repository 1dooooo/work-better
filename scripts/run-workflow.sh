#!/bin/bash
set -euo pipefail

# =============================================================================
# run-workflow.sh
# 执行 dev-test-review workflow (v2 并行版)
# =============================================================================

TASK_ID="${1:?用法: $0 <task_id>}"
ARTIFACTS_DIR=".workflow/artifacts/${TASK_ID}"
DEV_OUTPUT="${ARTIFACTS_DIR}/dev-output.json"
FINAL_REPORT="${ARTIFACTS_DIR}/final-report.json"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

log_info()    { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn()    { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error()   { echo -e "${RED}[ERROR]${NC} $1"; }
log_phase()   { echo -e "\n${PURPLE}>>> $1${NC}"; }
log_section() { echo -e "\n${BLUE}========================================${NC}"; echo -e "${BLUE}$1${NC}"; echo -e "${BLUE}========================================${NC}"; }

# 检查 dev-output.json 是否存在
if [ ! -f "$DEV_OUTPUT" ]; then
    log_error "dev-output.json 不存在: ${DEV_OUTPUT}"
    log_error "请先运行: ./scripts/create-dev-output.sh ${TASK_ID}"
    exit 1
fi

# 读取配置
TEST_LEVEL=$(jq -r '.test_level' "$DEV_OUTPUT")
MAX_RETRIES=$(jq -r '.gate_config.max_retries // 2' "$DEV_OUTPUT")

log_info "task_id: ${TASK_ID}"
log_info "test_level: ${TEST_LEVEL}"
log_info "max_retries: ${MAX_RETRIES}"

# =============================================================================
# Agent 执行提示函数
# =============================================================================

print_agent_prompt() {
    local agent=$1
    local prompt=$2
    local output_file=$3
    local color=$4

    echo ""
    echo -e "${color}------------------------------------------------------------${NC}"
    echo -e "${color}请在 Claude Code 中执行以下命令：${NC}"
    echo -e "${color}------------------------------------------------------------${NC}"
    echo ""
    echo "  agent_type: ${agent}"
    echo "  prompt: \"${prompt}\""
    echo ""
    echo "  输出文件: ${output_file}"
    echo -e "${color}------------------------------------------------------------${NC}"
    echo ""
}

# =============================================================================
# 阶段 1：开发（顺序）
# =============================================================================

log_phase "阶段 1：开发"

DEV_PROMPT="读取 ${DEV_OUTPUT}，理解需求，编写代码和 L1/L2 测试。输出到 ${DEV_OUTPUT}。"

print_agent_prompt "dev-agent" "$DEV_PROMPT" "$DEV_OUTPUT" "$BLUE"

log_info "等待 dev-agent 完成..."
log_info "检查 ${DEV_OUTPUT} 是否已更新..."

if [ ! -f "$DEV_OUTPUT" ]; then
    log_error "dev-agent 未生成 dev-output.json"
    exit 1
fi

log_info "dev-agent 完成 ✓"

# =============================================================================
# 阶段 2：并行审查
# =============================================================================

log_phase "阶段 2：并行审查（test-agent + review-agent + product-reviewer）"

# 生成各 agent 的 prompt
TEST_PROMPT="读取 ${DEV_OUTPUT}，根据 test_level 执行测试和安全扫描。输出到 ${ARTIFACTS_DIR}/test-report.json。"
REVIEW_PROMPT="读取 ${DEV_OUTPUT}，逐文件审查代码质量、安全性和性能。输出到 ${ARTIFACTS_DIR}/review-report.json。"
PRODUCT_PROMPT="读取 ${DEV_OUTPUT}，从产品定义角度审查功能实现。输出到 ${ARTIFACTS_DIR}/product-review.json。"

# 打印三个 agent 的执行提示（并行）
echo ""
echo -e "${YELLOW}============================================================${NC}"
echo -e "${YELLOW}并行启动 3 个 agent（在 Claude Code 中同时执行）：${NC}"
echo -e "${YELLOW}============================================================${NC}"

print_agent_prompt "test-agent" "$TEST_PROMPT" "${ARTIFACTS_DIR}/test-report.json" "$GREEN"
print_agent_prompt "review-agent" "$REVIEW_PROMPT" "${ARTIFACTS_DIR}/review-report.json" "$RED"
print_agent_prompt "product-reviewer" "$PRODUCT_PROMPT" "${ARTIFACTS_DIR}/product-review.json" "$YELLOW"

log_info "等待所有并行 agent 完成..."

# 检查所有输出文件
PARALLEL_DONE=true
for f in test-report.json review-report.json product-review.json; do
    if [ ! -f "${ARTIFACTS_DIR}/${f}" ]; then
        log_warn "${f} 尚未生成"
        PARALLEL_DONE=false
    else
        log_info "${f} ✓"
    fi
done

if [ "$PARALLEL_DONE" = false ]; then
    log_warn "部分 agent 未完成，请在 Claude Code 中执行上述命令后重新运行此脚本"
    exit 1
fi

# =============================================================================
# 阶段 3：汇总结果
# =============================================================================

log_section "阶段 3：汇总结果"

OVERALL_RESULT="pass"
ISSUES=()

# 检查 product-review
PRODUCT_VERDICT=$(jq -r '.verdict // "unknown"' "${ARTIFACTS_DIR}/product-review.json" 2>/dev/null || echo "unknown")
PRODUCT_CATEGORY=$(jq -r '.category // "unknown"' "${ARTIFACTS_DIR}/product-review.json" 2>/dev/null || echo "unknown")
log_info "产品审查: verdict=${PRODUCT_VERDICT}, category=${PRODUCT_CATEGORY}"

if [ "$PRODUCT_VERDICT" = "fail" ] && [ "$PRODUCT_CATEGORY" = "bug" ]; then
    OVERALL_RESULT="fail"
    ISSUES+=("产品审查发现 bug: $(jq -r '.summary // "未提供摘要"' "${ARTIFACTS_DIR}/product-review.json" 2>/dev/null)")
fi

# 检查 test-report
TEST_RESULT=$(jq -r '.result // "unknown"' "${ARTIFACTS_DIR}/test-report.json" 2>/dev/null || echo "unknown")
log_info "测试结果: ${TEST_RESULT}"

if [ "$TEST_RESULT" = "fail" ]; then
    OVERALL_RESULT="fail"
    FAIL_COUNT=$(jq '.failures | length' "${ARTIFACTS_DIR}/test-report.json" 2>/dev/null || echo "0")
    ISSUES+=("测试失败: ${FAIL_COUNT} 个用例")
fi

# 检查 review-report
REVIEW_VERDICT=$(jq -r '.verdict // "unknown"' "${ARTIFACTS_DIR}/review-report.json" 2>/dev/null || echo "unknown")
log_info "代码审查: verdict=${REVIEW_VERDICT}"

if [ "$REVIEW_VERDICT" = "request_changes" ] || [ "$REVIEW_VERDICT" = "block" ]; then
    OVERALL_RESULT="fail"
    ISSUES+=("代码审查未通过: ${REVIEW_VERDICT}")
fi

# =============================================================================
# 生成最终报告
# =============================================================================

log_section "生成最终报告"

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# 构建 issues JSON 数组
ISSUES_JSON="[]"
if [ ${#ISSUES[@]} -gt 0 ]; then
    ISSUES_JSON=$(printf '%s\n' "${ISSUES[@]}" | jq -R -s 'split("\n") | map(select(length > 0))')
fi

cat > "$FINAL_REPORT" << EOF
{
  "task_id": "${TASK_ID}",
  "timestamp": "${TIMESTAMP}",
  "test_level": ${TEST_LEVEL},
  "result": "${OVERALL_RESULT}",
  "gates_completed": {
    "test": "${TEST_RESULT}",
    "review": "${REVIEW_VERDICT}",
    "product_review": "${PRODUCT_VERDICT}"
  },
  "issues": ${ISSUES_JSON},
  "artifacts": {
    "dev_output": "${DEV_OUTPUT}",
    "test_report": "${ARTIFACTS_DIR}/test-report.json",
    "review_report": "${ARTIFACTS_DIR}/review-report.json",
    "product_review": "${ARTIFACTS_DIR}/product-review.json"
  }
}
EOF

log_info "最终报告: ${FINAL_REPORT}"

# 显示结果
echo ""
log_section "Workflow 执行结果"

if [ "$OVERALL_RESULT" = "pass" ]; then
    log_info "✅ Workflow 完成，所有审查通过"
else
    log_error "❌ Workflow 失败"
    echo ""
    echo "问题列表:"
    for issue in "${ISSUES[@]}"; do
        echo "  - ${issue}"
    done
    echo ""
    echo "修复后重新运行: ./scripts/run-workflow.sh ${TASK_ID}"
fi
