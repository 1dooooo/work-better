# Issue 001: 统一 Agent 注册表

## Parent

PRD: docs/prd/multi-agent-refactor-v5.md

## What to build

统一所有 agent 定义到单一数据源（agents.json），并更新 .md 文件使其一致。

**核心变更**：
1. 确保 `.claude/agents.json` 包含所有 8 个 agent 定义
2. 更新 `.claude/agents/*.md` 文件，使其与 agents.json 的 prompt 和 description 一致
3. 统一命名：使用短名称（无 -agent 后缀）

**当前状态**：
- agents.json 已包含 8 个 agent 定义
- .md 文件需要更新以匹配 agents.json

## Acceptance criteria

- [ ] `.claude/agents.json` 包含 8 个 agent 定义（workflow-runner, dev-agent, test-agent, review-agent, product-reviewer, system-inspector, validator, optimizer）
- [ ] `.claude/agents/*.md` 文件内容与 agents.json 一致
- [ ] 命名统一使用短名称
- [ ] 无双源冲突

## Blocked by

None - 可以立即开始

## User Stories

- 9: As a developer, I want all agent definitions in agents.json, so that there is a single source of truth
- 10: As a developer, I want .md files to be updated to match agents.json, so that there are no contradictions
- 16: As a developer, I want consistent agent naming (short names without -agent suffix), so that references are clear
