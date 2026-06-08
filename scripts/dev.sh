#!/usr/bin/env bash
# 开发模式运行 Work Better（热重载）
set -euo pipefail

cd "$(dirname "$0")/.."

# 清除构建缓存，避免旧代码残留
rm -rf node_modules/.vite dist

# 清除 Tauri WebKit 缓存（仅开发模式需要）
rm -rf ~/Library/Caches/com.work-better.app/WebKit 2>/dev/null || true
rm -rf ~/Library/Caches/com.work-better.app/WebsiteData 2>/dev/null || true

echo "🚀 启动开发模式..."
pnpm exec tauri dev
