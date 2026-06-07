# Multi-Agent Workflow Rule

本项目使用多 Agent 协作开发。当完成一个开发任务（写完代码、修复 bug、重构模块）时，
你必须执行以下 workflow。

## 触发条件

当你的工作涉及以下任一情况时触发 workflow：
- 修改了 `crates/` 下的 Rust 代码
- 修改了 `src/` 下的 TypeScript 代码
- 修改了 `src-tauri/` 下的 Tauri 命令代码
- 新增或删除了功能文件

**不触发**：仅修改文档 (`docs/`)、配置文件 (`.config/`)、脚本 (`scripts/`)

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

## 参考文档

- Workflow Spec: `.workflow/specs/dev-test-review.yaml`
- Artifact Schemas: `.workflow/templates/`
- 测试架构: `docs/testing/architecture.md`
- 协作规范: `docs/development/multi-agent-collaboration.md`
- 验收测试: `tests/acceptance/features/` (G1-G7, 182 scenarios)
