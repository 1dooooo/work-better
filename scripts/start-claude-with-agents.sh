#!/bin/bash
# =============================================================================
# start-claude-with-agents.sh
# 启动 Claude Code 并加载自定义 agent
# =============================================================================

set -euo pipefail

# 颜色输出
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info()  { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }

# 检查 agents.json 是否存在
AGENTS_CONFIG="$HOME/.claude/agents.json"
if [ ! -f "$AGENTS_CONFIG" ]; then
    log_warn "agents.json 不存在: $AGENTS_CONFIG"
    log_warn "请先创建 agents.json 文件"
    exit 1
fi

# 读取 agents 配置
AGENTS_JSON=$(cat "$AGENTS_CONFIG")

log_info "启动 Claude Code 并加载自定义 agent..."
log_info "Agents: dev-agent, product-reviewer"

# 启动 Claude Code，传入自定义 agent 配置
claude --agents "$AGENTS_JSON" "$@"
