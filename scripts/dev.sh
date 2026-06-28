#!/usr/bin/env bash
# 开发模式运行 Work Better（热重载）
#
# 数据隔离：所有用户数据写入项目本地 .dev-data/，不影响真实用户数据。
#   .dev-data/home/     ← 替代 $HOME（config.json、vault 等）
#   .dev-data/app-data/ ← 替代 Tauri app_data_dir（SQLite 数据库）
set -euo pipefail

cd "$(dirname "$0")/.."
PROJECT_DIR="$(pwd)"

# ── 数据隔离 ────────────────────────────────────────────────────
DEV_HOME="$PROJECT_DIR/.dev-data/home"
DEV_APP_DATA="$PROJECT_DIR/.dev-data/app-data"

# 创建 dev 专用 HOME 目录结构
mkdir -p "$DEV_HOME/.work-better"
mkdir -p "$DEV_APP_DATA"
mkdir -p "$DEV_HOME/Documents/Obsidian"

# 如果 dev config 不存在，从用户 config 复制一份（保留 API key 等配置）
if [ ! -f "$DEV_HOME/.work-better/config.json" ] && [ -f "$HOME/.work-better/config.json" ]; then
  cp "$HOME/.work-better/config.json" "$DEV_HOME/.work-better/config.json"
  echo "📋 已从用户配置复制 config.json 到 .dev-data/"
fi

# 将 HOME 重定向到 dev 目录（影响 config_path、vault ~ 展开等所有 $HOME 依赖）
# 先保存真实的 Rust 工具链路径（HOME 被重定向后 rustup/cargo 会找不到工具链）
REAL_HOME="$HOME"
export HOME="$DEV_HOME"
export WORK_BETTER_HOME="$DEV_HOME"
export RUSTUP_HOME="${RUSTUP_HOME:-$REAL_HOME/.rustup}"
export CARGO_HOME="${CARGO_HOME:-$REAL_HOME/.cargo}"

# Tauri app_data_dir symlink（SQLite 数据库位置）
# macOS: ~/Library/Application Support/{identifier}/
TAURI_APP_SUPPORT="$HOME/Library/Application Support"
TAURI_IDENTIFIER="com.work-better.app"
mkdir -p "$TAURI_APP_SUPPORT"

# 如果目标已存在但不是 symlink（真实目录），先备份
if [ -d "$TAURI_APP_SUPPORT/$TAURI_IDENTIFIER" ] && [ ! -L "$TAURI_APP_SUPPORT/$TAURI_IDENTIFIER" ]; then
  mv "$TAURI_APP_SUPPORT/$TAURI_IDENTIFIER" "$TAURI_APP_SUPPORT/${TAURI_IDENTIFIER}.bak"
  echo "⚠️  已备份原有 app data 目录为 ${TAURI_IDENTIFIER}.bak"
fi

# 创建 symlink 指向 dev app-data
ln -sfn "$DEV_APP_DATA" "$TAURI_APP_SUPPORT/$TAURI_IDENTIFIER"

echo "🔒 Dev 数据隔离已启用"
echo "   HOME=$HOME"
echo "   app_data → $DEV_APP_DATA"
echo ""

# ── 清除构建缓存 ───────────────────────────────────────────────
rm -rf node_modules/.vite dist

# 清除 Tauri WebKit 缓存（仅开发模式需要）
rm -rf "$HOME/Library/Caches/$TAURI_IDENTIFIER/WebKit" 2>/dev/null || true
rm -rf "$HOME/Library/Caches/$TAURI_IDENTIFIER/WebsiteData" 2>/dev/null || true

# 确保 pnpm 等工具在 PATH 中（HOME 重定向可能影响 shell profile 加载）
export PATH="/opt/homebrew/bin:/usr/local/bin:$PATH"

echo "🚀 启动开发模式..."
pnpm exec tauri dev
