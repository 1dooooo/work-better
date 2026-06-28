# Multi-Agent Workflow Rule

本项目使用多 Agent 协作开发。

## ⚠️ 强制约束

**所有代码变更必须通过 workflow-advisor 规划，由主 Agent 执行。**

```
✅ 正确：主 Agent → workflow-advisor（规划）→ 主 Agent 调用各 Agent
❌ 禁止：主 Agent → 直接调用 dev-agent / test-agent / review-agent
❌ 禁止：workflow-advisor → 直接调用其他 Agent
```

## 触发条件

当你的工作涉及以下任一情况时触发 workflow：
- 修改了 `crates/` 下的 Rust 代码
- 修改了 `src/` 下的 TypeScript 代码
- 修改了 `src-tauri/` 下的 Tauri 命令代码

**不触发**：仅修改文档 (`docs/`)、配置文件 (`.config/`)、脚本 (`scripts/`)

## Hook 自动保障机制

项目配置了 PreToolUse hook (`workflow-check.cjs`)，提供自动保障：

1. **自动创建 artifact**：当检测到代码文件变更且无活跃 workflow 时，自动创建 `.workflow/artifacts/{task_id}/dev-output.json`
2. **无需手动创建**：不需要运行 `./scripts/create-dev-output.sh`
3. **只需调用 workflow-advisor**：artifact 创建后，主 Agent 调用 workflow-advisor 获取执行计划

## 主 Agent 检查点

**在编辑代码文件前，主 Agent 必须：**

1. 调用 workflow-advisor，传入任务描述
2. 等待 workflow-advisor 返回执行计划
3. 按执行计划依次调用相应 agent
4. 将结果汇报给用户

## workflow-advisor 流程

workflow-advisor 作为顾问，协助主 Agent 规划和监督流程：

1. **分析任务**：识别涉及的模块和功能
2. **制定计划**：确定需要哪些 agent，按什么顺序执行
3. **监督执行**：确保主 Agent 按照流程执行
4. **汇总结果**：生成最终报告

### 典型执行流程

```
Phase 1: 开发 → dev-agent
Phase 2: 验证 → validator
Phase 3: 并行 → test-agent + review-agent + product-reviewer
Phase 4: 系统监督 → system-inspector
Phase 5: 优化建议 → optimizer（可选，需用户审批）
```

→ [workflow-advisor 定义](../../.claude/agents/workflow-advisor.md)

## 参考文档

- Workflow Spec: `.workflow/specs/dev-test-review.yaml`（参考文档）
- Artifact Schemas: `.workflow/templates/`
- 测试架构: `docs/testing/architecture.md`
