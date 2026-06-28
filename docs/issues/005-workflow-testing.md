# Issue 005: 测试新工作流

## Parent

PRD: docs/prd/multi-agent-refactor-v5.md

## What to build

执行一个完整的 workflow，验证新配置是否正常工作。

**核心变更**：
1. 执行一个完整的 workflow（使用新的 Phase 1-4 流程）
2. 验证并行执行是否正常
3. 验证 schema 验证是否正常
4. 验证重试逻辑是否正常
5. 验证 system-inspector 和 optimizer 是否正常触发

**当前状态**：
- 所有配置已更新
- 需要实际测试验证

## Acceptance criteria

- [ ] workflow 成功执行
- [ ] Phase 1 并行启动正常
- [ ] Phase 2 执行验证正常
- [ ] Phase 3 验证正常
- [ ] Phase 4 监督正常
- [ ] 所有 artifact 生成正确
- [ ] final-report.json 包含所有 agent 摘要

## Blocked by

- 004: 文档更新

## User Stories

- 18: As a developer, I want each agent to have a clear responsibility boundary, so that there is no overlap or confusion
- 19: As a developer, I want workflow-runner to handle retry logic, so that failures are properly managed
- 20: As a developer, I want final-report.json to include all agent summaries, so that I have a complete view of the workflow
