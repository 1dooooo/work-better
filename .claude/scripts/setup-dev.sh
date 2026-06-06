#!/usr/bin/env bash
# Work Better 开发环境 Setup 脚本
# 用法: bash .claude/scripts/setup-dev.sh [--lang typescript,python,...]
#
# 功能:
#   1. 检查前置工具
#   2. 安装项目依赖
#   3. 按需安装语言规则到 ~/.claude/rules/
#   4. 验证 hooks 配置

set -euo pipefail

# ── 颜色 ──
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info()  { echo -e "${BLUE}[INFO]${NC}  $*"; }
ok()    { echo -e "${GREEN}[OK]${NC}    $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
fail()  { echo -e "${RED}[FAIL]${NC}  $*"; }

PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
RULES_SOURCE="$PROJECT_ROOT/.claude/rules"

# ── 参数解析 ──
LANGS=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --lang) LANGS="$2"; shift 2 ;;
    --help|-h)
      echo "用法: bash .claude/scripts/setup-dev.sh [--lang typescript,python,...]"
      echo ""
      echo "选项:"
      echo "  --lang <langs>  逗号分隔的语言列表，安装对应的编码规则"
      echo "                  可选: typescript, python, golang, web, swift, php, ruby, arkts,"
      echo "                        angular, react, dart, cpp, csharp, fsharp, java, kotlin, perl, rust"
      echo "  --help          显示帮助"
      exit 0
      ;;
    *) fail "未知参数: $1"; exit 1 ;;
  esac
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Work Better — 开发环境 Setup"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# ── Step 1: 检查前置工具 ──
info "检查前置工具..."

check_cmd() {
  if command -v "$1" &>/dev/null; then
    ok "$1 $(command "$1" --version 2>/dev/null | head -1 || echo '(found)')"
    return 0
  else
    fail "$1 未安装"
    return 1
  fi
}

MISSING=0
check_cmd node   || MISSING=1
check_cmd git    || MISSING=1
check_cmd claude || { warn "Claude Code CLI 未安装（可选，也可使用 IDE 插件）"; }

# 检查包管理器
if command -v pnpm &>/dev/null; then
  PKG_MGR="pnpm"
  ok "包管理器: pnpm"
elif command -v yarn &>/dev/null; then
  PKG_MGR="yarn"
  ok "包管理器: yarn"
elif command -v npm &>/dev/null; then
  PKG_MGR="npm"
  ok "包管理器: npm"
else
  fail "未找到包管理器（pnpm / yarn / npm）"
  MISSING=1
fi

if [[ $MISSING -eq 1 ]]; then
  echo ""
  fail "缺少必要工具，请先安装后重试。"
  exit 1
fi

# ── Step 2: 安装项目依赖 ──
echo ""
info "安装项目依赖..."

cd "$PROJECT_ROOT"
if [[ -f "package.json" ]]; then
  $PKG_MGR install 2>/dev/null || warn "依赖安装跳过或失败（非关键）"
  ok "项目依赖已安装"
else
  warn "未找到 package.json，跳过依赖安装"
fi

# ── Step 3: 生成 settings.json ──
echo ""
SETTINGS_FILE="$PROJECT_ROOT/.claude/settings.json"
SETTINGS_TEMPLATE="$PROJECT_ROOT/.claude/settings.template.json"

if [[ -f "$SETTINGS_FILE" ]]; then
  ok "settings.json 已存在，跳过生成"
else
  if [[ -f "$SETTINGS_TEMPLATE" ]]; then
    # 用实际路径替换模板占位符
    sed "s|<你的项目路径>|$PROJECT_ROOT|g" "$SETTINGS_TEMPLATE" > "$SETTINGS_FILE"
    ok "settings.json 已从模板生成（路径已自动填充）"
  else
    warn "settings.template.json 不存在，跳过"
  fi
fi

# 检查 ECC 插件
echo ""
info "检查 ECC 插件..."
if command -v claude &>/dev/null; then
  if claude plugins list 2>/dev/null | grep -q "ecc"; then
    ok "ECC 插件已安装"
  else
    warn "ECC 插件未安装"
    echo "  ECC 提供 agents、skills、hooks 等开发工具。"
    echo "  安装方式：在 Claude Code 中执行 /install-plugin ecc"
    echo "  或参考：https://github.com/anthropics/ecc"
  fi
else
  warn "Claude Code CLI 未安装，跳过 ECC 检查"
fi

# ── Step 4: 安装语言规则 ──
echo ""
if [[ -z "$LANGS" ]]; then
  info "跳过语言规则安装（未指定 --lang）"
  info "如需安装，运行: bash .claude/scripts/setup-dev.sh --lang typescript,python"
else
  info "安装语言规则: $LANGS"

  TARGET_BASE="$HOME/.claude/rules/ecc"
  mkdir -p "$TARGET_BASE"

  # 始终安装 common 规则
  if [[ -d "$RULES_SOURCE/common" ]]; then
    cp -r "$RULES_SOURCE/common" "$TARGET_BASE/"
    ok "common 规则已安装"
  fi

  # 安装中文翻译
  if [[ -d "$RULES_SOURCE/zh" ]]; then
    cp -r "$RULES_SOURCE/zh" "$TARGET_BASE/"
    ok "zh 规则已安装"
  fi

  # 按指定语言安装
  IFS=',' read -ra LANG_ARRAY <<< "$LANGS"
  for lang in "${LANG_ARRAY[@]}"; do
    lang=$(echo "$lang" | xargs)  # trim
    if [[ -d "$RULES_SOURCE/$lang" ]]; then
      cp -r "$RULES_SOURCE/$lang" "$TARGET_BASE/"
      ok "$lang 规则已安装"
    else
      warn "$lang 规则不存在，跳过"
    fi
  done

  echo ""
  ok "规则已安装到 $TARGET_BASE"
fi

# ── Step 5: 验证配置 ──
echo ""
info "验证项目配置..."

CHECKS_PASSED=0
CHECKS_TOTAL=0

check_file() {
  CHECKS_TOTAL=$((CHECKS_TOTAL + 1))
  if [[ -f "$PROJECT_ROOT/$1" ]]; then
    ok "$1"
    CHECKS_PASSED=$((CHECKS_PASSED + 1))
  else
    fail "$1 缺失"
  fi
}

check_dir() {
  CHECKS_TOTAL=$((CHECKS_TOTAL + 1))
  if [[ -d "$PROJECT_ROOT/$1" ]]; then
    ok "$1/"
    CHECKS_PASSED=$((CHECKS_PASSED + 1))
  else
    fail "$1/ 缺失"
  fi
}

check_file "CLAUDE.md"
check_file "agent.md"
check_file "package.json"
check_file ".gitignore"
check_file ".claude/settings.json"
check_dir  ".claude/rules/common"
check_dir  ".claude/agents"
check_dir  ".claude/skills"
check_dir  ".claude/hooks"
check_dir  ".claude/scripts"
check_dir  ".claude/commands"
check_dir  ".claude/contexts"
check_dir  "docs"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [[ $CHECKS_PASSED -eq $CHECKS_TOTAL ]]; then
  ok "所有检查通过 ($CHECKS_PASSED/$CHECKS_TOTAL)"

  # 写入 setup 完成标记
  MARKER="$PROJECT_ROOT/.claude/.setup-done"
  cat > "$MARKER" <<EOF
{
  "completed_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "node_version": "$(node --version 2>/dev/null || echo 'unknown')",
  "pkg_manager": "${PKG_MGR:-unknown}",
  "langs": "${LANGS:-none}"
}
EOF
  ok "setup 标记已写入 .claude/.setup-done"

  echo ""
  echo "  下一步:"
  echo "    1. 运行 'claude' 启动 Claude Code"
  echo "    2. 阅读 agent.md 了解项目核心思想"
  echo "    3. 阅读 docs/development/setup.md 了解完整开发流程"
  echo ""
else
  warn "部分检查未通过 ($CHECKS_PASSED/$CHECKS_TOTAL)"
  echo "  请检查上方标记为 [FAIL] 的项目"
  echo ""
fi
