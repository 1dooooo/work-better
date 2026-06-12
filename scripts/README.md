---
title: Workflow Scripts
created: 2026-06-12
status: active
---

# Workflow Scripts

## 脚本说明

### create-dev-output.sh

生成 `dev-output.json`，包含：
- 变更文件列表
- 推断的 test_level (1-5)
- 推断的 required_agents
- Gate 配置

```bash
./scripts/create-dev-output.sh <task_id>
```

**task_id 格式**: `feat-xxx`、`fix-xxx`、`refactor-xxx`

**输出**: `.workflow/artifacts/<task_id>/dev-output.json`

### run-workflow.sh

执行完整的 dev-test-review workflow：
- Gate 1: L1 单元测试 + H1-H2 安全扫描
- Gate 2: L2 集成测试 (Level >= 2)
- Gate 3: E2E 测试 + H3-H5 安全测试 (Level >= 3)

```bash
./scripts/run-workflow.sh <task_id>
```

**前置条件**: 先运行 `create-dev-output.sh`

**输出**:
- `.workflow/artifacts/<task_id>/test-report.json`
- `.workflow/artifacts/<task_id>/review-report.json`
- `.workflow/artifacts/<task_id>/final-report.json`

## 使用流程

```bash
# 1. 完成代码开发后
./scripts/create-dev-output.sh feat-my-feature

# 2. 检查 dev-output.json
cat .workflow/artifacts/feat-my-feature/dev-output.json

# 3. 执行 workflow
./scripts/run-workflow.sh feat-my-feature

# 4. 检查结果
cat .workflow/artifacts/feat-my-feature/final-report.json
```

## test_level 推断规则

| Level | 条件 |
|-------|------|
| 1 | 单个模块的简单修改 |
| 2 | 多个 crate 或公共接口修改 |
| 3 | 核心模块、安全敏感代码、E2E 场景 |
| 4 | 手动指定 |
| 5 | 手动指定 |

## 与 Claude Code 集成

这些脚本设计为与 Claude Code 的 workflow 规则配合使用：

1. Claude 检测到代码变更时，会提醒你执行 workflow
2. 运行 `create-dev-output.sh` 生成配置
3. 运行 `run-workflow.sh` 执行测试
4. Claude 会读取报告并协助修复问题
