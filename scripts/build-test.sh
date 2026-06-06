#!/usr/bin/env bash
# 构建测试 — 编译前端和 Rust 后端，不打包
set -euo pipefail

cd "$(dirname "$0")/.."

echo "🔨 构建前端..."
pnpm build

echo "🦀 编译 Rust..."
cargo build --workspace

echo "✅ 构建成功，无编译错误"
