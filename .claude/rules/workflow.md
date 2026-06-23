# Multi-Agent Workflow Rule

本项目使用多 Agent 协作开发。

## ⚠️ 强制约束

**所有代码变更必须通过 workflow-runner。**

```
✅ 正确：主 Agent → workflow-runner → workflow-runner 派发子 Agent
❌ 禁止：主 Agent → dev-agent / test-agent / review-agent（直接调用）
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
3. **只需调用 workflow-runner**：artifact 创建后，主 Agent 调用 workflow-runner 即可

## 主 Agent 检查点

**在编辑代码文件前，主 Agent 必须：**

1. 调用 workflow-runner，传入任务描述
2. 等待 workflow-runner 返回结果
3. 将结果汇报给用户

## workflow-runner 流程

workflow-runner 按固定流程执行（hardcode，不读取外部 YAML）：

1. **Phase 1: 开发** → dev-agent
2. **Phase 2: 并行审查** → test-agent + review-agent + product-reviewer
3. **Phase 3: 验证** → validator
4. **Phase 4: 汇总** → 合并结果，有失败则重试

→ [workflow-runner 定义](../../.claude/agents/workflow-runner.md)

## 参考文档

- Workflow Spec: `.workflow/specs/dev-test-review.yaml`（参考文档，workflow-runner 不依赖）
- Artifact Schemas: `.workflow/templates/`
- 测试架构: `docs/testing/architecture.md`
