#!/bin/bash
# ============================================================
# Eval Runner - 运行所有 eval 定义中的 Code Grader
# ============================================================
# 用法: ./scripts/run-evals.sh [eval-name]
# 无参数时运行所有 eval，有参数时运行指定 eval
# ============================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
EVALS_DIR="$PROJECT_ROOT/.claude/evals"
RESULTS_DIR="$PROJECT_ROOT/.claude/evals/results"

mkdir -p "$RESULTS_DIR"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="$RESULTS_DIR/eval-report-${TIMESTAMP}.md"

# 颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

TOTAL=0
PASSED=0
FAILED=0

run_grader() {
    local eval_name="$1"
    local grader_script="$2"
    local description="$3"

    TOTAL=$((TOTAL + 1))
    echo -n "  [$eval_name] $description ... "

    if cd "$PROJECT_ROOT" && eval "$grader_script" > /dev/null 2>&1; then
        echo -e "${GREEN}PASS${NC}"
        PASSED=$((PASSED + 1))
        echo "  - [$eval_name] $description: PASS" >> "$REPORT_FILE"
    else
        echo -e "${RED}FAIL${NC}"
        FAILED=$((FAILED + 1))
        echo "  - [$eval_name] $description: FAIL" >> "$REPORT_FILE"
    fi
}

echo "# Eval Report - $TIMESTAMP" > "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 确定要运行的 eval 文件
if [ -n "${1:-}" ]; then
    EVAL_FILES="$EVALS_DIR/$1.md"
    if [ ! -f "$EVAL_FILES" ]; then
        echo -e "${RED}Error: Eval '$1' not found${NC}"
        exit 1
    fi
else
    EVAL_FILES="$EVALS_DIR"/*.md
fi

for eval_file in $EVAL_FILES; do
    eval_name=$(basename "$eval_file" .md)

    # 跳过非 eval 文件
    if [[ "$eval_name" == "README" ]] || [[ "$eval_name" == "baseline" ]]; then
        continue
    fi

    echo ""
    echo -e "${YELLOW}Running eval: $eval_name${NC}"
    echo "## $eval_name" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"

    # 提取 Code Grader 脚本（在 ```bash 和 ``` 之间的内容，在 ### Code Grader 之后）
    in_grader=false
    grader_script=""

    while IFS= read -r line; do
        if [[ "$line" == "### Code Grader" ]]; then
            in_grader=true
            continue
        fi

        if [[ "$in_grader" == true ]]; then
            if [[ "$line" == '```bash' ]]; then
                grader_script=""
                continue
            elif [[ "$line" == '```' ]]; then
                if [ -n "$grader_script" ]; then
                    # 提取注释作为描述
                    description=$(echo "$grader_script" | grep "^#" | head -1 | sed 's/^# //')
                    if [ -z "$description" ]; then
                        description="Code grader check"
                    fi
                    run_grader "$eval_name" "$grader_script" "$description"
                    grader_script=""
                fi
                in_grader=false
                continue
            fi
            grader_script="${grader_script}${line}"$'\n'
        fi
    done < "$eval_file"
done

echo ""
echo "=============================="
echo -e "Total: $TOTAL | ${GREEN}Passed: $PASSED${NC} | ${RED}Failed: $FAILED${NC}"

if [ $TOTAL -gt 0 ]; then
    PASS_RATE=$((PASSED * 100 / TOTAL))
    echo "Pass Rate: ${PASS_RATE}%"
    echo "" >> "$REPORT_FILE"
    echo "**Pass Rate: ${PASS_RATE}%** ($PASSED/$TOTAL)" >> "$REPORT_FILE"
fi

echo ""
echo "Report saved to: $REPORT_FILE"

if [ $FAILED -gt 0 ]; then
    exit 1
fi
