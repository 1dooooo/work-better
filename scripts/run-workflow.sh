#!/bin/bash
set -euo pipefail

# =============================================================================
# run-workflow.sh
# 执行 dev-test-review workflow (v2 并行版，带自动重试)
# =============================================================================

TASK_ID="${1:?用法: $0 <task_id>}"
ARTIFACTS_DIR=".workflow/artifacts/${TASK_ID}"
DEV_OUTPUT="${ARTIFACTS_DIR}/dev-output.json"
FINAL_REPORT="${ARTIFACTS_DIR}/final-report.json"

# 加载日志库
source "$(dirname "$0")/workflow-logger.sh"
workflow_log_init "$TASK_ID" "$ARTIFACTS_DIR"

# 颜色
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

# =============================================================================
# 检查 dev-output.json
# =============================================================================
log_phase "VALIDATE_INPUT"

if [ ! -f "$DEV_OUTPUT" ]; then
    log_artifact "missing" "$DEV_OUTPUT" "dev-agent 未生成"
    log_error "dev-output.json 不存在: ${DEV_OUTPUT}"
    log_error "请先运行: ./scripts/create-dev-output.sh ${TASK_ID}"
    workflow_log_finish "fail"
    exit 1
fi

log_artifact "read" "$DEV_OUTPUT"

# 读取配置
TEST_LEVEL=$(jq -r '.test_level' "$DEV_OUTPUT")
log_json_read "$DEV_OUTPUT" "test_level" "$TEST_LEVEL"

MAX_RETRIES=$(jq -r '.gate_config.max_retries // 2' "$DEV_OUTPUT")
log_json_read "$DEV_OUTPUT" "gate_config.max_retries" "$MAX_RETRIES"

REQUIRED_AGENTS=$(jq -r '.required_agents | join(",")' "$DEV_OUTPUT" 2>/dev/null || echo "dev-agent")
log_json_read "$DEV_OUTPUT" "required_agents" "$REQUIRED_AGENTS"

log_info "task_id: ${TASK_ID}"
log_info "test_level: ${TEST_LEVEL}"
log_info "max_retries: ${MAX_RETRIES}"
log_info "required_agents: ${REQUIRED_AGENTS}"

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
# 并行审查函数
# =============================================================================
run_parallel_review() {
    log_phase "PARALLEL_REVIEW"

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

    log_agent_call "test-agent" "start" "并行审查"
    log_agent_call "review-agent" "start" "并行审查"
    log_agent_call "product-reviewer" "start" "并行审查"

    log_info "等待所有并行 agent 完成..."

    # 检查所有输出文件
    PARALLEL_DONE=true
    for f in test-report.json review-report.json product-review.json; do
        if [ ! -f "${ARTIFACTS_DIR}/${f}" ]; then
            log_artifact "missing" "${ARTIFACTS_DIR}/${f}" "尚未生成"
            log_warn "${f} 尚未生成"
            PARALLEL_DONE=false
        else
            log_artifact "read" "${ARTIFACTS_DIR}/${f}"
            log_agent_call "${f%.json}" "complete"
            log_info "${f} ✓"
        fi
    done

    if [ "$PARALLEL_DONE" = false ]; then
        log_warn "部分 agent 未完成，请在 Claude Code 中执行上述命令后重新运行此脚本"
        return 1
    fi

    return 0
}

# =============================================================================
# 评估结果函数
# =============================================================================
evaluate_results() {
    OVERALL_RESULT="pass"
    ISSUES=()
    NEED_RETRY=false
    RETRY_REASON=""

    # 检查 product-review
    PRODUCT_VERDICT=$(jq -r '.verdict // "unknown"' "${ARTIFACTS_DIR}/product-review.json" 2>/dev/null || echo "unknown")
    PRODUCT_CATEGORY=$(jq -r '.category // "unknown"' "${ARTIFACTS_DIR}/product-review.json" 2>/dev/null || echo "unknown")
    log_json_read "product-review.json" "verdict" "$PRODUCT_VERDICT"
    log_json_read "product-review.json" "category" "$PRODUCT_CATEGORY"
    log_gate_result "product-review" "$PRODUCT_VERDICT" "category=${PRODUCT_CATEGORY}"

    if [ "$PRODUCT_VERDICT" = "fail" ] && [ "$PRODUCT_CATEGORY" = "bug" ]; then
        OVERALL_RESULT="fail"
        ISSUE_SUMMARY=$(jq -r '.summary // "未提供摘要"' "${ARTIFACTS_DIR}/product-review.json" 2>/dev/null)
        ISSUES+=("产品审查发现 bug: ${ISSUE_SUMMARY}")
        NEED_RETRY=true
        RETRY_REASON="product-review 发现 bug"
        log_decision "NEED_RETRY=true" "产品审查发现 bug，需要修复"
    fi

    # 检查 test-report
    TEST_RESULT=$(jq -r '.result // "unknown"' "${ARTIFACTS_DIR}/test-report.json" 2>/dev/null || echo "unknown")
    log_json_read "test-report.json" "result" "$TEST_RESULT"
    log_gate_result "test-report" "$TEST_RESULT"

    if [ "$TEST_RESULT" = "fail" ]; then
        OVERALL_RESULT="fail"
        FAIL_COUNT=$(jq '.failures | length' "${ARTIFACTS_DIR}/test-report.json" 2>/dev/null || echo "0")
        ISSUES+=("测试失败: ${FAIL_COUNT} 个用例")
        NEED_RETRY=true
        RETRY_REASON="测试失败"
        log_decision "NEED_RETRY=true" "测试失败 ${FAIL_COUNT} 个用例"

        # 记录每个失败的详情
        if [ "$FAIL_COUNT" -gt 0 ]; then
            jq -r '.failures[] | "  - \(.source_location // "unknown"): \(.error // "no message")"' "${ARTIFACTS_DIR}/test-report.json" 2>/dev/null | while IFS= read -r line; do
                log_debug "FAILURE: ${line}"
            done
        fi
    fi

    # 检查 review-report
    REVIEW_VERDICT=$(jq -r '.verdict // "unknown"' "${ARTIFACTS_DIR}/review-report.json" 2>/dev/null || echo "unknown")
    log_json_read "review-report.json" "verdict" "$REVIEW_VERDICT"
    log_gate_result "review-report" "$REVIEW_VERDICT"

    if [ "$REVIEW_VERDICT" = "request_changes" ] || [ "$REVIEW_VERDICT" = "block" ]; then
        OVERALL_RESULT="fail"
        ISSUES+=("代码审查未通过: ${REVIEW_VERDICT}")
        NEED_RETRY=true
        RETRY_REASON="代码审查未通过"
        log_decision "NEED_RETRY=true" "代码审查未通过: ${REVIEW_VERDICT}"
    fi

    # 导出变量供调用方使用
    export OVERALL_RESULT
    export ISSUES
    export NEED_RETRY
    export RETRY_REASON
    export TEST_RESULT
    export REVIEW_VERDICT
    export PRODUCT_VERDICT
    export PRODUCT_CATEGORY
}

# =============================================================================
# 触发 dev-agent 修复
# =============================================================================
trigger_dev_fix() {
    local retry_count=$1
    local reason=$2

    log_phase "DEV_FIX"
    log_retry "$retry_count" "$MAX_RETRIES" "$reason"

    # 构建修复 prompt
    FIX_PROMPT="修复以下问题（重试 ${retry_count}/${MAX_RETRIES}）：\n"

    # 添加 product-review 问题
    if [ -f "${ARTIFACTS_DIR}/product-review.json" ]; then
        PRODUCT_SUMMARY=$(jq -r '.summary // ""' "${ARTIFACTS_DIR}/product-review.json" 2>/dev/null)
        if [ -n "$PRODUCT_SUMMARY" ]; then
            FIX_PROMPT+="\n产品审查问题：${PRODUCT_SUMMARY}\n"
            # 添加 suggestions
            SUGGESTIONS=$(jq -r '.suggestions[]? // empty' "${ARTIFACTS_DIR}/product-review.json" 2>/dev/null)
            if [ -n "$SUGGESTIONS" ]; then
                FIX_PROMPT+="修复建议：\n${SUGGESTIONS}\n"
            fi
        fi
    fi

    # 添加 test-report 问题
    if [ -f "${ARTIFACTS_DIR}/test-report.json" ]; then
        TEST_FAIL_COUNT=$(jq '.failures | length' "${ARTIFACTS_DIR}/test-report.json" 2>/dev/null || echo "0")
        if [ "$TEST_FAIL_COUNT" -gt 0 ]; then
            FIX_PROMPT+="\n测试失败详情：\n"
            FIX_PROMPT+=$(jq -r '.failures[] | "- \(.source_location): \(.error)"' "${ARTIFACTS_DIR}/test-report.json" 2>/dev/null)
            FIX_PROMPT+="\n"
        fi
    fi

    # 添加 review-report 问题
    if [ -f "${ARTIFACTS_DIR}/review-report.json" ]; then
        REVIEW_FINDINGS=$(jq -r '.findings[]? | "- [\(.severity)] \(.title): \(.description)"' "${ARTIFACTS_DIR}/review-report.json" 2>/dev/null)
        if [ -n "$REVIEW_FINDINGS" ]; then
            FIX_PROMPT+="\n代码审查问题：\n${REVIEW_FINDINGS}\n"
        fi
        # 添加 code_reuse_issues
        REUSE_ISSUES=$(jq -r '.code_reuse_issues[]? | "- [\(.severity)] \(.title): \(.description)"' "${ARTIFACTS_DIR}/review-report.json" 2>/dev/null)
        if [ -n "$REUSE_ISSUES" ]; then
            FIX_PROMPT+="\n代码复用问题：\n${REUSE_ISSUES}\n"
        fi
    fi

    FIX_PROMPT+="\n请读取上述问题，修复代码并更新 dev-output.json。"

    echo ""
    echo -e "${RED}============================================================${NC}"
    echo -e "${RED}需要修复（重试 ${retry_count}/${MAX_RETRIES}）${NC}"
    echo -e "${RED}原因: ${reason}${NC}"
    echo -e "${RED}============================================================${NC}"

    print_agent_prompt "dev-agent" "$FIX_PROMPT" "$DEV_OUTPUT" "$BLUE"

    log_agent_call "dev-agent" "start" "修复: ${reason}"
    log_info "等待 dev-agent 完成修复..."

    # 检查 dev-output.json 是否已更新
    if [ ! -f "$DEV_OUTPUT" ]; then
        log_agent_call "dev-agent" "fail" "未生成 dev-output.json"
        log_artifact "missing" "$DEV_OUTPUT" "dev-agent 未生成"
        return 1
    fi

    log_agent_call "dev-agent" "complete" "修复完成"
    log_artifact "read" "$DEV_OUTPUT" "dev-agent 修复后"
    return 0
}

# =============================================================================
# 生成最终报告
# =============================================================================
generate_final_report() {
    local status=$1
    local retry_count=$2

    log_phase "FINAL_REPORT"

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
  "result": "${status}",
  "retry_count": ${retry_count},
  "gates_completed": {
    "test": "${TEST_RESULT:-unknown}",
    "review": "${REVIEW_VERDICT:-unknown}",
    "product_review": "${PRODUCT_VERDICT:-unknown}"
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

    log_artifact "write" "$FINAL_REPORT" "result=${status}"
    log_info "最终报告: ${FINAL_REPORT}"
}

# =============================================================================
# 主流程
# =============================================================================

RETRY_COUNT=0

# 阶段 1：开发
log_phase "DEV"

DEV_PROMPT="读取 ${DEV_OUTPUT}，理解需求，编写代码和 L1/L2 测试。输出到 ${DEV_OUTPUT}。"

print_agent_prompt "dev-agent" "$DEV_PROMPT" "$DEV_OUTPUT" "$BLUE"

log_agent_call "dev-agent" "start" "初始开发"
log_info "等待 dev-agent 完成..."

if [ ! -f "$DEV_OUTPUT" ]; then
    log_agent_call "dev-agent" "fail" "未生成 dev-output.json"
    log_artifact "missing" "$DEV_OUTPUT" "dev-agent 未生成"
    workflow_log_finish "fail"
    exit 1
fi

log_agent_call "dev-agent" "complete"
log_artifact "read" "$DEV_OUTPUT" "dev-agent 完成后"

# 阶段 2+3：并行审查 + 评估 + 重试循环
while true; do
    # 运行并行审查
    if ! run_parallel_review; then
        workflow_log_finish "fail"
        exit 1
    fi

    # 评估结果
    evaluate_results

    # 如果全部通过，生成报告并退出
    if [ "$OVERALL_RESULT" = "pass" ]; then
        generate_final_report "done" "$RETRY_COUNT"

        echo ""
        log_section "Workflow 执行结果"
        log_info "Workflow 完成，所有审查通过"
        log_info "重试次数: ${RETRY_COUNT}"
        workflow_log_finish "done"
        exit 0
    fi

    # 检查是否需要重试
    if [ "$NEED_RETRY" = false ]; then
        # 不需要重试（例如 product fail 是 gap/new_feature）
        generate_final_report "done" "$RETRY_COUNT"

        echo ""
        log_section "Workflow 执行结果"
        log_info "Workflow 完成（有非阻塞问题）"
        workflow_log_finish "done"
        exit 0
    fi

    # 检查重试次数
    if [ "$RETRY_COUNT" -ge "$MAX_RETRIES" ]; then
        log_error "已达最大重试次数 (${MAX_RETRIES})，停止重试"
        generate_final_report "fail" "$RETRY_COUNT"

        echo ""
        log_section "Workflow 执行结果"
        log_error "Workflow 失败，已达最大重试次数"
        echo ""
        echo "问题列表:"
        for issue in "${ISSUES[@]}"; do
            echo "  - ${issue}"
            log_error "  - ${issue}"
        done
        echo ""
        echo "请手动检查问题并修复后重新运行: ./scripts/run-workflow.sh ${TASK_ID}"
        workflow_log_finish "fail"
        exit 1
    fi

    # 触发 dev-agent 修复
    RETRY_COUNT=$((RETRY_COUNT + 1))
    if ! trigger_dev_fix "$RETRY_COUNT" "$RETRY_REASON"; then
        log_error "dev-agent 修复失败"
        generate_final_report "fail" "$RETRY_COUNT"
        workflow_log_finish "fail"
        exit 1
    fi

    log_info "修复完成，重新运行并行审查..."
done
