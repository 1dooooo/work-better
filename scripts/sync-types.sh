#!/bin/bash
# 从 wb-core/bindings 复制 ts-rs 生成的类型到 src/generated/
set -euo pipefail

SRC="crates/wb-core/bindings"
DST="src/generated"

if [ ! -d "$SRC" ]; then
    echo "[sync-types] No bindings found. Run: cargo test -p wb-core"
    exit 1
fi

mkdir -p "$DST"
cp -r "$SRC"/* "$DST"/

echo "[sync-types] Copied $(find "$DST" -name "*.ts" | wc -l | tr -d ' ') type files to $DST"
