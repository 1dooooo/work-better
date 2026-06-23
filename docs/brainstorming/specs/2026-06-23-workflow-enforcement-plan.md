# 实施计划：Workflow 强制执行机制

基于 [PRD](./2026-06-23-workflow-enforcement-prd.md)，分 4 个阶段实施。

---

## Phase 1: workflow-runner 内嵌流程（最高优先级）

**目标**：workflow-runner 不再依赖读取外部 YAML，内部 hardcode 完整流程。

### Task 1.1: 重写 workflow-runner.md

**文件**：`.claude/agents/workflow-runner.md`

**改动**：
- 删除"读取 .workflow/specs/dev-test-review.yaml"的指令
- 内嵌固定流程：Phase 1(dev) → Phase 2(test+review+product 并行) → Phase 3(validator) → Phase 4(汇总)
- 内嵌任务路由表（代码变更/Bug修复/重构/文档/配置）
- 内嵌重试策略（L1:3/L2:2/L4:1/L5:0）和熔断器逻辑
- 内嵌 artifact 通信契约（哪个 agent 写哪个文件）
- 保留日志记录机制

**验收标准**：
- workflow-runner.md 中不包含"读取 YAML"的指令
- 固定流程的 4 个 Phase 明确定义
- 任务路由表覆盖 5 种任务类型

### Task 1.2: 验证 workflow-runner 执行

**方法**：用一个简单的代码变更任务测试 workflow-runner 是否按固定流程执行。

**验收标准**：
- workflow-runner 按 Phase 1→2→3→4 顺序执行
- 每个 Phase 的 agent 被正确调用
- final-report.json 正确生成

---

## Phase 2: Hook 自动触发机制

**目标**：PostToolUse hook 在检测到代码变更时自动创建 workflow artifact。

### Task 2.1: 修改 workflow-trigger.cjs

**文件**：`scripts/hooks/workflow-trigger.cjs`

**改动**：
- 在 Edit/Write 操作后，检查目标文件是否在受保护目录（crates/、src/、src-tauri/）
- 排除测试文件（*.test.ts、*.spec.ts）、文档（*.md）、配置（*.json、*.yaml）、样式（*.css）
- 如果匹配且无活跃 workflow：
  1. 创建 `.workflow/artifacts/{task_id}/` 目录（task_id 从时间戳生成）
  2. 生成初始 `dev-output.json`（包含变更文件列表）
  3. 通过 stdout 输出提示："已创建 workflow artifact，请调用 workflow-runner"
- 幂等性：已有活跃 workflow 时不重复创建

**验收标准**：
- 修改 crates/ 下的文件后，artifact 目录自动创建
- 修改 docs/ 下的文件后，不触发
- 重复修改同一文件时，不重复创建 artifact

### Task 2.2: 更新 workflow-check.cjs

**文件**：`scripts/hooks/workflow-check.cjs`

**改动**：
- 确保 PreToolUse hook 与新的自动触发机制兼容
- 检查逻辑不变：有活跃 workflow 时允许编辑，无 workflow 时阻止

**验收标准**：
- 有活跃 workflow 时，Edit/Write 操作被允许
- 无活跃 workflow 时，Edit/Write 操作被阻止（exit 2）

### Task 2.3: Hook 测试

**文件**：`scripts/hooks/__tests__/workflow-trigger.test.cjs`

**改动**：
- 测试代码变更后自动创建 artifact
- 测试非代码变更不触发
- 测试幂等性

**验收标准**：
- 所有测试通过

---

## Phase 3: 规则瘦身

**目标**：CLAUDE.md 简化为一条硬规则，冗余规则文件合并到 agent 定义中。

### Task 3.1: 简化 CLAUDE.md

**文件**：`CLAUDE.md`

**改动**：
- 删除"多 Agent 协作"章节中的详细规则
- 替换为一条硬规则："所有代码变更必须通过 workflow-runner"
- 保留"代码导航"、"准则"等非流程规则
- 保留对 workflow-runner 的引用

**验收标准**：
- CLAUDE.md 中不再包含 dev-agent/test-agent/review-agent 的详细职责描述
- 保留一条明确的硬规则

### Task 3.2: 合并规则文件

**合并方案**：

| 源文件 | 目标 | 操作 |
|--------|------|------|
| `common/agents.md` | `workflow-runner.md` | Agent 编排规则合入 workflow-runner |
| `common/development-workflow.md` | `workflow-runner.md` | 开发流程合入 workflow-runner |
| `common/code-review.md` | `.claude/agents/review-agent.md` | 审查标准合入 review-agent 定义 |
| `common/testing.md` | `.claude/agents/test-agent.md` | 测试要求合入 test-agent 定义 |
| `zh/*` | 删除 | 与 common 重复 |
| `web/*` | 保留 | 前端专项规则 |
| `common/coding-style.md` | 保留 | 通用编码规范 |
| `common/git-workflow.md` | 保留 | Git 规范 |
| `common/security.md` | 保留 | 安全规范 |
| `common/patterns.md` | 保留 | 设计模式 |
| `common/hooks.md` | 保留 | Hook 规范 |
| `common/performance.md` | 保留 | 性能规范 |

**验收标准**：
- 规则文件数量从 20+ 减少到 10 以内
- 无信息丢失（合并前逐条审查）

### Task 3.3: 更新 rules/README.md

**文件**：`.claude/rules/README.md`

**改动**：
- 更新目录结构说明
- 更新安装说明

---

## Phase 4: 验证与清理

**目标**：验证完整流程，清理遗留文件。

### Task 4.1: 端到端测试

**方法**：用一个实际的代码变更任务验证完整流程：

1. 修改 crates/ 下的一个文件
2. 验证 hook 自动创建 artifact
3. 验证主 Agent 调用 workflow-runner
4. 验证 workflow-runner 按固定流程执行（dev → test+review+product → validator）
5. 验证 final-report.json 正确生成

**验收标准**：
- 完整流程自动执行
- 用户不需要手动干预
- 所有 agent 被正确调用

### Task 4.2: 清理遗留文件

**删除**：
- `.claude/rules/zh/` 目录（如果确认与 common 重复）

**保留**：
- `.workflow/specs/dev-test-review.yaml`（保留作为参考文档，但 workflow-runner 不再依赖它）

---

## 执行顺序

```
Phase 1 (workflow-runner 内嵌流程)
  → Task 1.1 → Task 1.2
    ↓
Phase 2 (Hook 自动触发)
  → Task 2.1 → Task 2.2 → Task 2.3
    ↓
Phase 3 (规则瘦身)
  → Task 3.1 → Task 3.2 → Task 3.3
    ↓
Phase 4 (验证与清理)
  → Task 4.1 → Task 4.2
```

每个 Phase 完成后用一个实际任务验证，确保不回退。
