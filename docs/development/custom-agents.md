---
title: 自定义 Agent 注册指南
created: 2026-06-12
status: active
---

# 自定义 Agent 注册指南

## 概述

本项目使用多 Agent 协作开发模式，需要注册 8 个自定义 agent：
- **workflow-advisor** — 流程顾问（任务分析、执行计划、流程监督）
- **dev-agent** — 开发 agent（代码实现 + L1-L2 测试）
- **test-agent** — 测试 agent（L4-L5 测试生成与执行）
- **review-agent** — 代码审查 agent（代码质量 + H3-H5 安全测试）
- **product-reviewer** — 产品审查 agent（产品定义符合性审查）
- **validator** — 验证 agent（Schema + 数据完整性验证）
- **system-inspector** — 系统巡检 agent（系统健康 + 执行效率监督）
- **optimizer** — 优化 agent（Agent prompt + workflow 优化建议）

角色定义见 [Agent Guide](/agent.md)。

## 注册方式

### 方式 1：使用启动脚本（推荐）

```bash
./scripts/start-claude-with-agents.sh
```

这个脚本会自动读取 `~/.claude/agents.json` 并传入 Claude Code。

### 方式 2：手动传入

```bash
claude --agents "$(cat ~/.claude/agents.json)"
```

### 方式 3：使用通用 agent 替代

当自定义 agent 不可用时，使用 `general-purpose` agent 并在 prompt 中指定角色：

```
Agent type: general-purpose
prompt: "你是 [agent 角色]。职责：[具体职责]..."
```

## Agent 定义文件

### 位置

- `.claude/agents/` — 项目级 agent 定义（本项目使用）
- `~/.claude/agents/` — 全局 agent 定义

### 格式

Agent 定义文件使用 **Markdown + frontmatter 格式**：

```markdown
---
name: agent-name
description: >
  简短描述。
type: agent
domain: 领域
created: YYYY-MM-DD
updated: YYYY-MM-DD
status: active
tools:
  - Read
  - Write
model: sonnet
---

# Agent Name

职责描述。

## 输入

- 输入文件或数据

## 工作流程

1. 步骤 1
2. 步骤 2
...

## 输出

输出文件或数据

## 约束

- 约束条件
```

**⚠️ 重要**：文件必须包含 frontmatter（`---` 分隔符），`name` 和 `description` 字段为必填。

## agents.json 配置

`.claude/agents.json` 文件定义了自定义 agent 的配置：

```json
{
  "dev-agent": {
    "description": "功能开发 + L1/L2 测试",
    "prompt": "你是开发者 agent。职责：编写代码 + L1/L2 测试。"
  },
  "product-reviewer": {
    "description": "产品审查，判断功能是否符合预期",
    "prompt": "你是产品审查者。职责：从产品定义角度审查功能实现。"
  }
}
```

## Workflow 执行

### 自动触发

当 Claude 检测到代码变更时，会根据 agent.md 中的强制触发规则提醒执行 workflow。

### 手动执行

```bash
# 1. 生成 dev-output.json
./scripts/create-dev-output.sh <task_id>

# 2. 执行 workflow
./scripts/run-workflow.sh <task_id>
```

### 使用 agent 执行

```bash
# 使用 dev-agent
Agent type: dev-agent
prompt: "读取 .workflow/artifacts/{task_id}/dev-output.json，执行开发任务。"

# 使用 product-reviewer
Agent type: product-reviewer
prompt: "读取 .workflow/artifacts/{task_id}/dev-output.json，从产品角度审查。"

# 使用 test-agent
Agent type: test-agent
prompt: "读取 .workflow/artifacts/{task_id}/dev-output.json，执行测试。"

# 使用 review-agent
Agent type: review-agent
prompt: "读取 .workflow/artifacts/{task_id}/dev-output.json，执行代码审查。"

# 使用 workflow-advisor
Agent type: workflow-advisor
prompt: "读取任务描述，制定执行计划。"
```

## 故障排查

### Agent 不可用

如果自定义 agent 不可用：

1. 检查 `.claude/agents.json` 是否存在
2. 检查 agent 定义文件是否在 `.claude/agents/` 目录下
3. **检查文件格式**：必须以 `# Agent Name` 开头，不要使用 frontmatter
4. 使用启动脚本重新启动 Claude Code
5. 或使用 `general-purpose` agent 替代

### 常见问题

**问题**：agent 文件存在但 Claude Code 不识别

**原因**：使用了 frontmatter 格式（`---\nname: ...\n---`）

**解决**：改为简单 Markdown 格式，以 `# Agent Name` 开头

### Workflow 执行失败

1. 检查 `.workflow/artifacts/{task_id}/` 目录下的文件
2. 查看 `final-report.json` 中的错误信息
3. 根据错误类型进行修复：
   - `code_bug` → 修复代码
   - `test_bug` → 修复测试
   - `env_issue` → 检查环境配置

## 参考文档

- [Workflow Spec](/.workflow/specs/dev-test-review.yaml)
- [Agent Guide](/agent.md)
- [Multi-Agent Collaboration](/docs/development/multi-agent-collaboration.md)
