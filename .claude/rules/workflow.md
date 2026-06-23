# Multi-Agent Workflow Rule

本项目使用多 Agent 协作开发。当完成一个开发任务（写完代码、修复 bug、重构模块）时，
你必须执行以下 workflow。

## ⚠️ 强制约束

**禁止主 Agent 直接派发子 Agent**（dev-agent、test-agent、review-agent、product-reviewer）。
所有代码变更的子 Agent 调度必须通过 `workflow-runner` 进行。

```
✅ 正确：主 Agent → workflow-runner → workflow-runner 派发子 Agent
❌ 禁止：主 Agent → dev-agent / test-agent / review-agent（直接调用）
```

workflow-runner 收到任务后自行判断是否需要多 Agent：
- 简单变更 → workflow-runner 直接完成
- 复杂变更 → 按 spec 派发子 Agent

## 触发条件

当你的工作涉及以下任一情况时触发 workflow：
- 修改了 `crates/` 下的 Rust 代码
- 修改了 `src/` 下的 TypeScript 代码
- 修改了 `src-tauri/` 下的 Tauri 命令代码
- 新增或删除了功能文件

**不触发**：仅修改文档 (`docs/`)、配置文件 (`.config/`)、脚本 (`scripts/`)

## Hook 自动检查机制（强制执行）

为了确保 workflow 规则被正确执行，项目配置了两个 hook。**这些 hook 是强制性的，违反将导致操作被阻止。**

### PreToolUse Hook: workflow-check（阻止性）

**文件**: `scripts/hooks/workflow-check.cjs`
**触发时机**: 每次 Edit/Write/MultiEdit/Bash 操作前
**行为**:
1. **Edit/Write**: 检查目标文件路径是否在 `crates/`、`src/`、`src-tauri/` 下
2. **MultiEdit**: 检查 `file_paths` 数组中的每个文件路径
3. **Bash**: 检查命令是否包含对受保护目录的文件写入操作（`sed -i`、`tee`、重定向 `>`、`cp`/`mv` 到受保护目录等）
4. 如果匹配，检查是否有活跃的 workflow（dev-output.json 存在且未完成）
5. **如果没有，强制阻止操作（exit 2）并要求创建 workflow**

**排除的文件**（不需要 workflow）：
- 测试文件（*.test.ts, *.spec.ts）
- 文档文件（*.md）
- 配置文件（*.json, *.yaml, *.yml）
- 样式文件（*.css）

**重要**：此 hook 是阻止性的，不是警告。主 Agent 在编辑代码前必须确保 workflow 存在。即使通过 Bash 命令（如 `sed -i`、`echo >`）修改代码文件也会被阻止。

### PostToolUse Hook: workflow-trigger

**文件**: `scripts/hooks/workflow-trigger.cjs`
**触发时机**: 每次 Write/Edit 操作后
**功能**:
1. 检测 `dev-output.json` 被写入后自动触发 `run-workflow.sh`
2. 检测 `git commit` 执行后自动触发 workflow（如果有活跃的任务）

### 配置位置

两个 hook 都配置在 `.claude/hooks/hooks.json` 中：
- `pre:edit-write:workflow-check` (PreToolUse, matcher: `Edit|Write|MultiEdit|Bash`)
- `post:edit-write:workflow-trigger` (PostToolUse)

## 主 Agent 检查点（强制）

**在编辑代码文件前，主 Agent 必须：**

1. **检查目标文件路径**：是否在 `crates/`、`src/`、`src-tauri/` 下
2. **检查排除列表**：是否为测试、文档、配置、样式文件
3. **检查 workflow 状态**：
   - `.workflow/artifacts/` 目录是否存在
   - 是否有包含 `dev-output.json` 的活跃任务
   - 任务是否已完成（有 `final-report.json`）
4. **如果缺少 workflow**：必须先创建，不能直接编辑代码

**违反此检查点将导致 PreToolUse hook 阻止操作（exit 2）。** 通过 Bash 命令（sed -i、echo > 等）修改代码文件同样会被阻止。

## 执行步骤

### Step 1: 生成 dev-output.json

```bash
./scripts/create-dev-output.sh <task_id>
```

task_id 格式：`feat-xxx`、`fix-xxx`、`refactor-xxx`

### Step 2: 运行测试

```bash
./scripts/run-workflow.sh <task_id>
```

这会自动执行：
- Gate Inference: 从 changed_files 推断 gate level
- Gate 1: L1 单元测试 + H1-H2 安全扫描
- Gate 2: L2 集成测试（如果是核心模块变更）
- Gate 3: E2E 测试

### Step 3: 检查结果

读取 `.workflow/artifacts/<task_id>/test-report.json`：
- `result: "pass"` → 任务完成
- `result: "partial_pass"` → 检查 skipped 测试，确认是否需要处理
- `result: "fail"` → 修复失败的测试，回到 Step 2

### Step 4: 失败处理

如果测试失败：
1. 读取 test-report.json 中的 `failures` 数组
2. 分析每个失败的 `source_location` 和 `error`
3. 判定 `failure_type`：
   - `code_bug` → 修复代码
   - `test_bug` → 修复测试
   - `env_issue` → 记录，不阻塞
4. 修复后重新运行 `./scripts/run-workflow.sh <task_id>`

## Eval 自动运行

代码变更（crates/、src/、src-tauri/）会自动触发 eval，无需手动执行。

- **Hook**: `post:edit-write:eval-trigger`（每次 Edit/Write 自动触发，60s 冷却）
- **Eval 定义**: `.claude/evals/*.md`（每个功能一个 eval 文件）
- **运行脚本**: `scripts/run-evals.sh [eval-name]`
- **结果**: `.claude/evals/results/eval-report-*.md`

手动运行：
```bash
./scripts/run-evals.sh              # 运行所有 eval
./scripts/run-evals.sh manual-capture  # 运行指定 eval
```

## 参考文档

- Workflow Spec: `.workflow/specs/dev-test-review.yaml`
- Artifact Schemas: `.workflow/templates/`
- 测试架构: `docs/testing/architecture.md`
- 协作规范: `docs/development/multi-agent-collaboration.md`
