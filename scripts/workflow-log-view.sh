#!/bin/bash
set -euo pipefail

# =============================================================================
# workflow-log-view.sh
# 查看 workflow 日志文件
# =============================================================================
# 用法:
#   ./scripts/workflow-log-view.sh <task_id>              # 查看完整日志
#   ./scripts/workflow-log-view.sh <task_id> --tail 50    # 查看最后 50 行
#   ./scripts/workflow-log-view.sh <task_id> --errors     # 只看 ERROR
#   ./scripts/workflow-log-view.sh <task_id> --phase DEV  # 只看特定阶段
#   ./scripts/workflow-log-view.sh <task_id> --agent      # 只看 agent 相关
#   ./scripts/workflow-log-view.sh <task_id> --decision   # 只看决策点
# =============================================================================

TASK_ID="${1:?用法: $0 <task_id> [--tail N] [--errors] [--phase PHASE] [--agent] [--decision]}"
LOG_FILE=".workflow/artifacts/${TASK_ID}/workflow.log"

# 颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

if [ ! -f "$LOG_FILE" ]; then
    echo -e "${RED}[ERROR]${NC} 日志文件不存在: ${LOG_FILE}"
    echo "可用的 task_id:"
    ls -1 .workflow/artifacts/ 2>/dev/null | sed 's/^/  - /'
    exit 1
fi

# 解析参数
MODE="all"
TAIL_COUNT=""
FILTER_PHASE=""
FILTER_LEVEL=""

shift || true
while [ $# -gt 0 ]; do
    case "$1" in
        --tail)
            MODE="tail"
            TAIL_COUNT="${2:?--tail 需要指定行数}"
            shift 2
            ;;
        --errors)
            MODE="filter"
            FILTER_LEVEL="ERROR"
            shift
            ;;
        --phase)
            MODE="filter_phase"
            FILTER_PHASE="${2:?--phase 需要指定阶段名}"
            shift 2
            ;;
        --agent)
            MODE="filter_agent"
            shift
            ;;
        --decision)
            MODE="filter_decision"
            shift
            ;;
        *)
            echo "未知参数: $1"
            exit 1
            ;;
    esac
done

# 输出函数（带颜色高亮）
colorize_line() {
    local line="$1"
    if echo "$line" | grep -q '\[ERROR\]'; then
        echo -e "${RED}${line}${NC}"
    elif echo "$line" | grep -q '\[WARN\]'; then
        echo -e "${YELLOW}${line}${NC}"
    elif echo "$line" | grep -q '\[INFO\].*PHASE'; then
        echo -e "${PURPLE}${line}${NC}"
    elif echo "$line" | grep -q '\[INFO\].*AGENT'; then
        echo -e "${CYAN}${line}${NC}"
    elif echo "$line" | grep -q '\[INFO\].*DECISION'; then
        echo -e "${GREEN}${line}${NC}"
    elif echo "$line" | grep -q '\[INFO\].*GATE'; then
        echo -e "${BLUE}${line}${NC}"
    else
        echo "$line"
    fi
}

# 显示日志
echo -e "${CYAN}============================================================${NC}"
echo -e "${CYAN} Workflow 日志: ${TASK_ID}${NC}"
echo -e "${CYAN} 文件: ${LOG_FILE}${NC}"
echo -e "${CYAN} 大小: $(wc -l < "$LOG_FILE" | tr -d ' ') 行${NC}"
echo -e "${CYAN}============================================================${NC}"
echo ""

case "$MODE" in
    all)
        while IFS= read -r line; do
            colorize_line "$line"
        done < "$LOG_FILE"
        ;;
    tail)
        tail -n "$TAIL_COUNT" "$LOG_FILE" | while IFS= read -r line; do
            colorize_line "$line"
        done
        ;;
    filter)
        grep "\[${FILTER_LEVEL}\]" "$LOG_FILE" | while IFS= read -r line; do
            colorize_line "$line"
        done || echo "(无匹配行)"
        ;;
    filter_phase)
        grep "\[${FILTER_PHASE}\]" "$LOG_FILE" | while IFS= read -r line; do
            colorize_line "$line"
        done || echo "(无匹配阶段: ${FILTER_PHASE})"
        ;;
    filter_agent)
        grep "AGENT" "$LOG_FILE" | while IFS= read -r line; do
            colorize_line "$line"
        done || echo "(无 agent 日志)"
        ;;
    filter_decision)
        grep -E "DECISION|GATE|RETRY" "$LOG_FILE" | while IFS= read -r line; do
            colorize_line "$line"
        done || echo "(无决策日志)"
        ;;
esac
