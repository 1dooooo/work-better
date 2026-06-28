# Issue 004: 更新 CLAUDE.md 和 Rules

## Parent

PRD: docs/prd/multi-agent-refactor-v5.md

## What to build

更新 CLAUDE.md 和 workflow.md 规则，使其反映 v5 流程。

**核心变更**：
1. 简化 CLAUDE.md 流程描述：
   - 只写核心流程，不详细描述每个 Phase
   - 包含所有 8 个 agent
2. 更新 workflow.md 规则：
   - 更新 workflow-runner 流程描述
   - 添加 Phase 1-4 说明

**当前状态**：
- CLAUDE.md 第 54 行仍写旧流程
- workflow.md 需要更新

## Acceptance criteria

- [ ] CLAUDE.md 包含 v5 流程描述
- [ ] workflow.md 包含 v5 流程描述
- [ ] 所有 8 个 agent 在文档中提及
- [ ] 无过时引用

## Blocked by

- 003: Workflow Spec 更新

## User Stories

- 12: As a developer, I want CLAUDE.md to have simplified flow description, so that it's easy to understand
