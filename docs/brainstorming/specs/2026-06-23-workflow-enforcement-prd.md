# PRD: Workflow 强制执行机制 — 消除 Agent "遗忘"问题

## Problem Statement

主 Agent 在执行代码变更任务时，经常"忘记"调用专业子 Agent（review-agent、test-agent、product-reviewer、validator 等），导致质量把控流程被跳过。

根因分析：
1. **触发断裂**：主 Agent → workflow-runner 的触发是 prompt 级软约束，hook 只阻止"没有 workflow 时直接写代码"，但不阻止"有 workflow 但跳过某些 agent"
2. **流程断裂**：workflow-runner 需要"读取 YAML 理解流程"，但可能不读、读错、或读了不遵循
3. **规则稀释**：20+ 规则文件中，关键的流程强制规则和风格建议混在一起，agent 无法区分优先级

## Solution

建立三层加固机制，确保所有代码变更都经过完整的质量把控流程：

1. **L1 Hook 自动触发**：workflow-trigger.cjs 检测到代码变更时自动调用 workflow-runner，不依赖主 Agent 主动调用
2. **L2 workflow-runner 内嵌流程**：workflow-runner 内部 hardcode 固定流程，不需要读取外部 YAML
3. **L3 规则瘦身**：CLAUDE.md 只保留一条硬规则，其他合并到 agent 定义中

## User Stories

1. As a 主 Agent，我想要在收到代码变更任务时自动触发 workflow-runner，so that 我不需要记住调用流程
2. As a 主 Agent，我想要被 hook 阻止直接调用 dev-agent/test-agent/review-agent，so that 所有调度必须经过 workflow-runner
3. As a workflow-runner，我想要内部 hardcode 完整流程，so that 我不需要读取外部 YAML 文件来理解流程
4. As a workflow-runner，我想要按固定顺序执行 dev → test+review+product-review → validator，so that 每个代码变更都经过完整质量把控
5. As a workflow-runner，我想要根据任务类型自动决定需要调用哪些 agent，so that 简单任务不浪费资源，复杂任务不遗漏检查
6. As a dev-agent，我想要接收明确的任务描述和输入文件，so that 我能高效完成开发任务
7. As a test-agent，我想要在 dev-agent 完成后自动被调用，so that 测试不依赖主 Agent 记忆
8. As a review-agent，我想要在 dev-agent 完成后自动被调用，so that 代码审查不被跳过
9. As a product-reviewer，我想要在 dev-agent 完成后自动被调用，so that 产品审查不被跳过
10. As a validator，我想要在所有审查完成后自动被调用，so that 管道交叉点验证不被遗漏
11. As a guardian-agent，我想要在任务完成后被调用，so that 系统级问题能被及时发现
12. As a cost-tracker-agent，我想要在每个 agent 完成后记录 token 使用，so that 成本能被追踪
13. As a 用户，我想要看到每次代码变更的完整质量报告，so that 我能信任变更的质量
14. As a 用户，我想要在 workflow 失败时收到清晰的错误信息，so that 我能快速定位问题
15. As a 用户，我想要 workflow 自动重试失败的 agent（在限制内），so that 临时问题不需要我手动干预
16. As a 用户，我想要 workflow 超过重试次数时自动上报，so that 持续失败不会被忽略
17. As a hook 系统，我想要在检测到代码文件变更时自动创建 workflow artifact，so that 主 Agent 不需要手动创建
18. As a hook 系统，我想要在 workflow 完成后自动触发后续流程，so that 审查链不被中断
19. As a 规则系统，我想要将所有流程规则集中到一个文件，so that agent 不需要在 20+ 文件中寻找规则
20. As a CLAUDE.md，我想要只保留一条硬规则（"代码变更必须通过 workflow-runner"），so that 关键规则不被稀释
21. As a workflow-runner，我想要在 agent 失败时合并多个失败报告一起修复，so that dev-agent 不需要多次修复同一问题
22. As a workflow-runner，我想要跟踪同一位置的连续失败次数（trend_stop），so that 无限重试循环被阻止
23. As a workflow-runner，我想要在熔断器打开时停止重试，so that 级联故障被防止
24. As a workflow-runner，我想要为每个 workflow 生成结构化日志，so that 执行过程可审计
25. As a workflow-runner，我想要在 workflow 完成后生成 final-report.json，so that 用户能快速了解结果

## Implementation Decisions

### 1. Hook 自动触发机制

修改 `workflow-trigger.cjs`，在检测到代码文件变更时自动调用 workflow-runner：

- **触发条件**：Edit/Write 操作修改了 `crates/`、`src/`、`src-tauri/` 下的文件（排除测试、文档、配置、样式文件）
- **触发动作**：自动创建 workflow artifact 目录 + 生成初始 dev-output.json
- **幂等性**：如果已有活跃 workflow（dev-output.json 存在且无 final-report.json），不重复创建
- **与现有 hook 的关系**：`workflow-check.cjs` 继续作为 PreToolUse 阻止性 hook，`workflow-trigger.cjs` 改为 PostToolUse 自动触发 hook

**重要限制**：hook 是 Node.js 脚本，不能直接调用 Claude Code Agent。hook 的职责是**创建 artifact + 通过 stdout 提示主 Agent**，真正的 agent 调用仍然由主 Agent 发起。但 hook 确保了 artifact 已存在，主 Agent 只需要调用 workflow-runner 而不需要自己创建 workflow。

### 2. workflow-runner 内嵌流程

重写 `workflow-runner.md`，将流程 hardcode 在 agent 定义中：

**固定流程（不可跳过）：**
```
Phase 1: 开发
  → 调用 dev-agent，传入任务描述
  → 等待 dev-output.json

Phase 2: 并行审查（三个 agent 同时调用）
  → test-agent → test-report.json
  → review-agent → review-report.json
  → product-reviewer → product-review.json

Phase 3: 验证
  → validator → validation-report.json

Phase 4: 汇总
  → 合并所有报告
  → 有失败 → 检查重试次数 → 回到 Phase 1 修复
  → 全部通过 → 写 final-report.json
```

**任务路由表：**
workflow-runner 根据任务描述自动匹配任务类型，决定调用哪些 agent：

| 任务类型 | 必须调用 | 可选 |
|---------|---------|------|
| 代码变更 | dev → test + review + product-review → validator | guardian, orchestrator, cost-tracker |
| Bug 修复 | dev → test + review → validator | product-reviewer |
| 重构 | dev → test + review + product-review → validator | optimizer |
| 文档变更 | 跳过 workflow | — |
| 配置变更 | dev → test → validator | review |

### 3. 规则瘦身

**CLAUDE.md 简化为一条硬规则：**
```markdown
## 编排规则（唯一强制规则）

所有代码变更（crates/、src/、src-tauri/），主 Agent 必须：
1. 调用 workflow-runner，传入任务描述
2. 等待 workflow-runner 返回结果
3. 将结果汇报给用户

主 Agent 禁止：
- 直接调用 dev-agent、test-agent、review-agent、product-reviewer
- 直接写代码（由 workflow-runner 委托 dev-agent 完成）
- 直接运行测试（由 workflow-runner 委托 test-agent 完成）
```

**删除/合并的规则文件：**
- `common/agents.md` → 合入 workflow-runner.md
- `common/development-workflow.md` → 合入 workflow spec
- `common/code-review.md` → 合入 review-agent.md
- `common/testing.md` → 合入 test-agent.md
- `zh/*` → 删除（和 common 重复）
- `web/*` → 保留（前端专项规则）

### 4. Artifact 通信契约

保持现有的文件契约通信机制，所有 agent 通过 `.workflow/artifacts/{task_id}/` 下的 JSON 文件传递信息：

| 文件 | 写入方 | 读取方 |
|------|--------|--------|
| dev-output.json | dev-agent | workflow-runner, test-agent, review-agent, product-reviewer |
| test-report.json | test-agent | workflow-runner, dev-agent |
| review-report.json | review-agent | workflow-runner |
| product-review.json | product-reviewer | workflow-runner, dev-agent |
| validation-report.json | validator | workflow-runner |
| final-report.json | workflow-runner | 用户 |

### 5. 重试与熔断策略

保持现有的重试策略和熔断器机制：

| Gate | 最大重试 | 趋势停止 |
|------|---------|---------|
| L1 | 3 | 2 |
| L2 | 2 | 2 |
| L4 | 1 | 1 |
| L5 | 0 | 0 |

熔断器：同一 agent 连续失败 3 次 → 打开熔断器 → 停止重试 → 上报用户。

### 6. PostToolUse Hook 自动创建 workflow artifact

修改 `workflow-trigger.cjs`，使其在检测到代码文件变更后自动创建 workflow：

```
PostToolUse (Edit/Write)
  → 检查目标文件是否在受保护目录
  → 检查是否已有活跃 workflow
  → 如果没有：
    1. 创建 .workflow/artifacts/{task_id}/ 目录
    2. 生成初始 dev-output.json（从变更文件列表）
    3. 通过 stdout 提示主 Agent 调用 workflow-runner
```

## Testing Decisions

### 测试策略

1. **Hook 测试**：验证 workflow-check.cjs 在有/无 workflow 时正确阻止/允许操作
2. **Hook 测试**：验证 workflow-trigger.cjs 在代码变更后正确创建 artifact
3. **workflow-runner 测试**：验证固定流程的执行顺序
4. **workflow-runner 测试**：验证任务路由表的匹配逻辑
5. **集成测试**：验证完整的 dev → test+review+product → validator 流程
6. **边界测试**：验证重试、熔断、trend_stop 机制

### 测试文件

- `scripts/hooks/__tests__/workflow-check.test.cjs` — 已有，需扩展
- `scripts/hooks/__tests__/workflow-trigger.test.cjs` — 已有，需扩展

### 测试标准

- 每个 hook 行为必须有独立测试用例
- 测试必须覆盖正常路径和异常路径
- 测试必须验证幂等性（重复触发不产生副作用）

## Out of Scope

1. **Cursor/Codex 兼容**：本次只针对 Claude Code 的 hook 机制，其他工具的适配后续处理
2. **混沌测试框架**：chaos-tester-agent 的完善不在本次范围内
3. **检查点恢复**：checkpoint-manager-agent 的完善不在本次范围内
4. **分布式追踪**：tracing 机制的完善不在本次范围内
5. **成本限制**：cost-tracker-agent 的完善不在本次范围内
6. **新 Agent 开发**：本次只调整现有 agent 的编排方式，不新增 agent

## Further Notes

### 迁移策略

1. 先改 workflow-runner.md（内嵌流程）
2. 再改 workflow-trigger.cjs（自动触发）
3. 最后瘦身 CLAUDE.md 和规则文件
4. 每步完成后用一个实际任务验证

### 风险

1. **hook 误触发**：非代码变更的 Edit/Write 操作可能被误判为需要 workflow → 通过排除模式缓解
2. **workflow-runner 过度工程化**：固定流程可能不适合所有任务类型 → 通过任务路由表提供灵活性
3. **规则瘦身遗漏**：删除的规则可能包含重要信息 → 合并前逐条审查

### 成功标准

1. 主 Agent 在收到代码变更任务时，100% 触发 workflow-runner
2. workflow-runner 100% 执行完整流程（dev → test+review+product → validator）
3. 规则文件数量从 20+ 减少到 10 以内
4. 用户不需要手动干预 workflow 执行
