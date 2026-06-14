#!/usr/bin/env bash
# 清理开发模式产生的本地数据
set -euo pipefail

cd "$(dirname "$0")/.."

if [ -d ".dev-data" ]; then
  rm -rf .dev-data
  echo "🧹 已清理 .dev-data/"
else
  echo "✅ 没有需要清理的 dev 数据"
fi

# 清理可能残留的 symlink
TAURI_APP_SUPPORT="$HOME/Library/Application Support"
if [ -L "$TAURI_APP_SUPPORT/com.work-better.app" ]; then
  rm "$TAURI_APP_SUPPORT/com.work-better.app"
  echo "🧹 已清理 Tauri app data symlink"
fi
