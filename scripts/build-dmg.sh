#!/usr/bin/env bash
# 打包 macOS .dmg 安装镜像
set -euo pipefail

cd "$(dirname "$0")/.."

echo "📦 打包 .dmg..."
pnpm exec tauri build --bundles dmg

DMG_PATH="target/release/bundle/dmg/Work Better_0.1.0_aarch64.dmg"
if [ -f "$DMG_PATH" ]; then
    echo "✅ 打包完成: $DMG_PATH"
else
    echo "✅ 打包完成，请查看 target/release/bundle/dmg/"
fi
