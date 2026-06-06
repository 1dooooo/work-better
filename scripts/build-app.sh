#!/usr/bin/env bash
# 打包 macOS .app 应用程序包
set -euo pipefail

cd "$(dirname "$0")/.."

echo "📦 打包 .app..."
pnpm exec tauri build --bundles app

APP_PATH="target/release/bundle/macos/Work Better.app"
if [ -d "$APP_PATH" ]; then
    echo "✅ 打包完成: $APP_PATH"
else
    echo "✅ 打包完成，请查看 target/release/bundle/macos/"
fi
