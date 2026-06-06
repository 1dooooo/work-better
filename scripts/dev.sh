#!/usr/bin/env bash
# 开发模式运行 Work Better（热重载）
set -euo pipefail

cd "$(dirname "$0")/.."

echo "🚀 启动开发模式..."
pnpm exec tauri dev
