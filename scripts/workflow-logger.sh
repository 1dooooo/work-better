#!/bin/bash
# =============================================================================
# workflow-logger.sh
# Workflow 日志工具库 — 提供统一的文件 + 终端日志能力
# =============================================================================
# 用法：在其他脚本中 source 此文件
#   source "$(dirname "$0")/workflow-logger.sh"
#   workflow_log_init <task_id>
#   log_info "消息"
# =============================================================================

# 日志文件路径（由 workflow_log_init 设置）
_WORKFLOW_LOG_FILE=""
_WORKFLOW_TASK_ID=""
_WORKFLOW_START_TIME=""
_CURRENT_PHASE="INIT"

# 颜色输出
_RED='\033[0;31m'
_GREEN='\033[0;32m'
_YELLOW='\033[1;33m'
_BLUE='\033[0;34m'
_PURPLE='\033[0;35m'
_CYAN='\033[0;36m'
_NC='\033[0m'

# =============================================================================
# 初始化
# =============================================================================

workflow_log_init() {
    local task_id="$1"
    local artifacts_dir="${2:-.workflow/artifacts/${task_id}}"

    _WORKFLOW_TASK_ID="$task_id"
    _WORKFLOW_LOG_FILE="${artifacts_dir}/workflow.log"
    _WORKFLOW_START_TIME=$(date +%s)

    # 确保目录存在
    mkdir -p "$(dirname "$_WORKFLOW_LOG_FILE")"

    # 写入日志头
    {
        echo "============================================================"
        echo " Workflow Log"
        echo " Task ID: ${task_id}"
        echo " Started: $(date '+%Y-%m-%d %H:%M:%S %Z')"
        echo " Host:    $(hostname)"
        echo " PID:     $$"
        echo "============================================================"
        echo ""
    } > "$_WORKFLOW_LOG_FILE"

    echo -e "${_CYAN}[LOG]${_NC} 日志文件: ${_WORKFLOW_LOG_FILE}"
}

# =============================================================================
# 核心日志函数
# =============================================================================

_write_log() {
    local level="$1"
    local phase="$2"
    local message="$3"
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S.%3N')

    # 写入文件
    if [ -n "$_WORKFLOW_LOG_FILE" ]; then
        echo "[${timestamp}] [${level}] [${phase}] ${message}" >> "$_WORKFLOW_LOG_FILE"
    fi
}

log_debug() {
    _write_log "DEBUG" "$_CURRENT_PHASE" "$1"
    if [ "${WORKFLOW_VERBOSE:-0}" = "1" ]; then
        echo -e "${_CYAN}[DEBUG]${_NC} $1"
    fi
}

log_info() {
    _write_log "INFO" "$_CURRENT_PHASE" "$1"
    echo -e "${_GREEN}[INFO]${_NC} $1"
}

log_warn() {
    _write_log "WARN" "$_CURRENT_PHASE" "$1"
    echo -e "${_YELLOW}[WARN]${_NC} $1"
}

log_error() {
    _write_log "ERROR" "$_CURRENT_PHASE" "$1"
    echo -e "${_RED}[ERROR]${_NC} $1"
}

# =============================================================================
# 阶段标记
# =============================================================================

log_phase() {
    local phase_name="$1"
    _CURRENT_PHASE="$phase_name"
    _write_log "INFO" "PHASE" "======== 阶段开始: ${phase_name} ========"
    echo ""
    echo -e "${_PURPLE}>>> ${phase_name}${_NC}"
}

log_section() {
    local title="$1"
    _write_log "INFO" "$_CURRENT_PHASE" "---- ${title} ----"
    echo ""
    echo -e "${_BLUE}========================================${_NC}"
    echo -e "${_BLUE}${title}${_NC}"
    echo -e "${_BLUE}========================================${_NC}"
}

# =============================================================================
# 结构化日志
# =============================================================================

log_agent_call() {
    local agent="$1"
    local action="$2"
    local detail="${3:-}"

    case "$action" in
        start)
            _write_log "INFO" "AGENT" ">>> 调用 ${agent} — ${detail}"
            echo -e "${_CYAN}[AGENT]${_NC} >>> 调用 ${agent}"
            ;;
        complete)
            _write_log "INFO" "AGENT" "<<< ${agent} 完成 — ${detail}"
            echo -e "${_GREEN}[AGENT]${_NC} <<< ${agent} 完成"
            ;;
        fail)
            _write_log "ERROR" "AGENT" "xxx ${agent} 失败 — ${detail}"
            echo -e "${_RED}[AGENT]${_NC} xxx ${agent} 失败"
            ;;
        timeout)
            _write_log "ERROR" "AGENT" "~~~ ${agent} 超时 — ${detail}"
            echo -e "${_RED}[AGENT]${_NC} ~~~ ${agent} 超时"
            ;;
    esac
}

log_artifact() {
    local operation="$1"
    local file="$2"
    local detail="${3:-}"

    case "$operation" in
        read)
            _write_log "DEBUG" "ARTIFACT" "<- 读取 ${file} ${detail}"
            ;;
        write)
            _write_log "INFO" "ARTIFACT" "-> 写入 ${file} ${detail}"
            ;;
        missing)
            _write_log "WARN" "ARTIFACT" "?? 缺失 ${file} ${detail}"
            ;;
        invalid)
            _write_log "ERROR" "ARTIFACT" "!! 无效 ${file} ${detail}"
            ;;
    esac
}

log_decision() {
    local decision="$1"
    local reason="$2"
    _write_log "INFO" "DECISION" "决策: ${decision} — 原因: ${reason}"
    echo -e "${_YELLOW}[DECISION]${_NC} ${decision} — ${reason}"
}

log_retry() {
    local attempt="$1"
    local max="$2"
    local reason="$3"
    _write_log "WARN" "RETRY" "重试 ${attempt}/${max} — ${reason}"
    echo -e "${_YELLOW}[RETRY]${_NC} 重试 ${attempt}/${max} — ${reason}"
}

log_gate_result() {
    local gate="$1"
    local result="$2"
    local detail="${3:-}"

    if [ "$result" = "pass" ]; then
        _write_log "INFO" "GATE" "PASS ${gate} ${detail}"
        echo -e "${_GREEN}[GATE]${_NC} PASS ${gate}"
    else
        _write_log "ERROR" "GATE" "FAIL ${gate} ${detail}"
        echo -e "${_RED}[GATE]${_NC} FAIL ${gate}"
    fi
}

# 记录 jq 解析结果（调试 JSON 读取问题）
log_json_read() {
    local file="$1"
    local field="$2"
    local value="$3"
    _write_log "DEBUG" "JSON" "Read ${file}::${field} = ${value}"
}

# 记录 shellcheck / 命令执行
log_exec() {
    local cmd="$1"
    local exit_code="$2"
    local detail="${3:-}"
    if [ "$exit_code" -eq 0 ]; then
        _write_log "DEBUG" "EXEC" "OK (exit=${exit_code}) ${cmd} ${detail}"
    else
        _write_log "ERROR" "EXEC" "FAIL (exit=${exit_code}) ${cmd} ${detail}"
    fi
}

# =============================================================================
# 结束与统计
# =============================================================================

workflow_log_finish() {
    local status="$1"
    local end_time
    end_time=$(date +%s)
    local duration=$(( end_time - _WORKFLOW_START_TIME ))

    _write_log "INFO" "DONE" "============================================================"
    _write_log "INFO" "DONE" " Workflow 结束"
    _write_log "INFO" "DONE" " Status:   ${status}"
    _write_log "INFO" "DONE" " Duration: ${duration}s"
    _write_log "INFO" "DONE" " Finished: $(date '+%Y-%m-%d %H:%M:%S %Z')"
    _write_log "INFO" "DONE" "============================================================"

    echo ""
    echo -e "${_CYAN}[LOG]${_NC} 总耗时: ${duration}s"
    echo -e "${_CYAN}[LOG]${_NC} 日志文件: ${_WORKFLOW_LOG_FILE}"
}

get_log_file() {
    echo "$_WORKFLOW_LOG_FILE"
}
