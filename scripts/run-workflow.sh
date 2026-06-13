#!/bin/bash
set -euo pipefail

# =============================================================================
# run-workflow.sh
# 执行 dev-test-review workflow
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
NC='\033[0m'

log_info()    { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn()    { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error()   { echo -e "${RED}[ERROR]${NC} $1"; }
log_section() { echo -e "\n${BLUE}========================================${NC}"; echo -e "${BLUE}$1${NC}"; echo -e "${BLUE}========================================${NC}"; }

# 检查 dev-output.json 是否存在
if [ ! -f "$DEV_OUTPUT" ]; then
    log_error "dev-output.json 不存在: ${DEV_OUTPUT}"
    log_error "请先运行: ./scripts/create-dev-output.sh ${TASK_ID}"
    exit 1
fi

# 读取配置
TEST_LEVEL=$(jq -r '.test_level' "$DEV_OUTPUT")
REQUIRED_AGENTS=$(jq -r '.required_agents | join(",")' "$DEV_OUTPUT")
MAX_RETRIES=$(jq -r '.gate_config.max_retries // 2' "$DEV_OUTPUT")
TIMEOUT=$(jq -r '.gate_config.timeout_seconds // 300' "$DEV_OUTPUT")

log_info "task_id: ${TASK_ID}"
log_info "test_level: ${TEST_LEVEL}"
log_info "required_agents: ${REQUIRED_AGENTS}"
log_info "max_retries: ${MAX_RETRIES}"
log_info "timeout: ${TIMEOUT}s"

# Gate 执行函数
run_gate() {
    local gate_num=$1
    local gate_name=$2
    local agent=$3
    local prompt=$4
    local output_file=$5
    local retry=0

    log_section "Gate ${gate_num}: ${gate_name}"

    while [ $retry -lt $MAX_RETRIES ]; do
        log_info "执行 ${agent} (尝试 $((retry + 1))/${MAX_RETRIES})..."

        # 生成 agent 执行提示
        echo ""
        echo "------------------------------------------------------------"
        echo -e "${YELLOW}请在 Claude Code 中执行以下命令：${NC}"
        echo "------------------------------------------------------------"
        echo ""
        echo "使用 Agent 工具执行:"
        echo ""
        echo "  agent_type: ${agent}"
        echo "  prompt: \"${prompt}\""
        echo ""
        echo "输出文件: ${output_file}"
        echo "------------------------------------------------------------"
        echo ""

        # 检查输出文件是否已存在
        if [ -f "$output_file" ]; then
            log_info "输出文件已存在: ${output_file}"
            local result
            result=$(jq -r '.result // "unknown"' "$output_file" 2>/dev/null || echo "unknown")

            if [ "$result" = "pass" ]; then
                log_info "Gate ${gate_num} 通过 ✓"
                return 0
            elif [ "$result" = "fail" ]; then
                log_warn "Gate ${gate_num} 失败，重试中..."
                retry=$((retry + 1))
                continue
            else
                log_warn "Gate ${gate_num} 结果未知: ${result}"
                return 1
            fi
        else
            log_warn "输出文件不存在，请先执行 agent"
            log_warn "等待用户手动执行或跳过..."
            return 1
        fi
    done

    log_error "Gate ${gate_num} 达到最大重试次数"
    return 1
}

# 根据 test_level 执行 gates
OVERALL_RESULT="pass"

# Gate 1: 总是执行
GATE1_PROMPT="读取 ${DEV_OUTPUT}，执行 L1 单元测试（cargo test --lib）和 H1-H2 安全扫描（cargo audit）。输出到 ${ARTIFACTS_DIR}/test-report.json。"

if ! run_gate 1 "L1 单元测试 + H1-H2 安全扫描" "test-agent" "$GATE1_PROMPT" "${ARTIFACTS_DIR}/test-report.json"; then
    OVERALL_RESULT="fail"
fi

# Gate 2: Level >= 2 时执行
if [ "$TEST_LEVEL" -ge 2 ] && [ "$OVERALL_RESULT" = "pass" ]; then
    GATE2_PROMPT="读取 ${DEV_OUTPUT}，执行 L2 集成测试（cargo test --features integration）。输出到 ${ARTIFACTS_DIR}/test-report.json。"

    if ! run_gate 2 "L2 集成测试" "test-agent" "$GATE2_PROMPT" "${ARTIFACTS_DIR}/test-report.json"; then
        OVERALL_RESULT="fail"
    fi
fi

# Gate 3: Level >= 3 时执行
if [ "$TEST_LEVEL" -ge 3 ] && [ "$OVERALL_RESULT" = "pass" ]; then
    GATE3_PROMPT="读取 ${DEV_OUTPUT}，执行 E2E 测试和 H3-H5 安全测试。输出到 ${ARTIFACTS_DIR}/test-report.json。"

    if ! run_gate 3 "E2E 测试 + H3-H5 安全测试" "test-agent" "$GATE3_PROMPT" "${ARTIFACTS_DIR}/test-report.json"; then
        OVERALL_RESULT="fail"
    fi
fi

# 生成最终报告
log_section "生成最终报告"

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

cat > "$FINAL_REPORT" << EOF
{
  "task_id": "${TASK_ID}",
  "timestamp": "${TIMESTAMP}",
  "gates_completed": $([ "$OVERALL_RESULT" = "pass" ] && echo "$TEST_LEVEL" || echo "0"),
  "test_report": "${ARTIFACTS_DIR}/test-report.json",
  "review_report": "${ARTIFACTS_DIR}/review-report.json",
  "result": "${OVERALL_RESULT}",
  "next_steps": []
}
EOF

log_info "最终报告: ${FINAL_REPORT}"

# 显示结果
echo ""
log_section "Workflow 执行结果"

if [ "$OVERALL_RESULT" = "pass" ]; then
    log_info "✅ Workflow 完成，所有 gate 通过"
else
    log_error "❌ Workflow 失败"
    echo ""
    echo "请检查:"
    echo "  1. ${ARTIFACTS_DIR}/test-report.json - 测试报告"
    echo "  2. ${ARTIFACTS_DIR}/review-report.json - 审查报告"
    echo "  3. ${FINAL_REPORT} - 最终报告"
    echo ""
    echo "修复后重新运行: ./scripts/run-workflow.sh ${TASK_ID}"
fi
