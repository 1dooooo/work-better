#!/bin/bash
# 文档健康检查脚本
# 用法: bash scripts/check-docs.sh

set -e

ERRORS=0
WARNINGS=0

echo "=== 文档健康检查 ==="
echo ""

# 1. Frontmatter 检查
echo "▶ 检查 frontmatter..."
for f in $(find docs/ README.md CONTRIBUTING.md -name "*.md" -not -path "*/superpowers/*" -not -path "*/deprecated/*" 2>/dev/null); do
  if ! head -1 "$f" | grep -q "^---"; then
    echo "  ❌ 缺少 frontmatter: $f"
    ERRORS=$((ERRORS + 1))
  fi
done
echo ""

# 2. 索引链接一致性
echo "▶ 检查索引链接..."
for index in $(find docs/ -name "_index.md" -o -name "index.md" | grep -v superpowers | grep -v decisions/index.md); do
  dir=$(dirname "$index")
  grep -oE '\[.*?\]\(([^)]+)\)' "$index" | grep -oE '\(([^)]+)\)' | tr -d '()' | while read link; do
    target="$dir/$link"
    if [ ! -f "$target" ]; then
      echo "  ❌ 死链接: $index → $link"
      ERRORS=$((ERRORS + 1))
    fi
  done
done
echo ""

# 3. 文件长度检查
echo "▶ 检查文件长度..."
for f in $(find docs/ -name "*.md" -not -path "*/superpowers/*" -not -path "*/deprecated/*" 2>/dev/null); do
  lines=$(wc -l < "$f")
  if [ "$lines" -gt 300 ]; then
    echo "  ⚠️  文件过长 ($lines 行): $f"
    WARNINGS=$((WARNINGS + 1))
  fi
done
echo ""

# 4. deprecated 一致性
echo "▶ 检查 deprecated 一致性..."
for f in $(find docs/ -path "*/deprecated/*" -name "*.md" 2>/dev/null); do
  basename_f=$(basename "$f")
  for index in $(find docs/ -name "_index.md" -o -name "index.md" | grep -v superpowers | grep -v deprecated); do
    if grep -q "$basename_f" "$index"; then
      echo "  ⚠️  deprecated 文件仍在索引中: $f → $index"
      WARNINGS=$((WARNINGS + 1))
    fi
  done
done
echo ""

# 总结
echo "=== 检查完成 ==="
echo "错误: $ERRORS | 警告: $WARNINGS"

if [ "$ERRORS" -gt 0 ]; then
  exit 1
fi
