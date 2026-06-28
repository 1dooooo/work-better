# Issue 002: 更新 Artifact Schema 文件

## Parent

PRD: docs/prd/multi-agent-refactor-v5.md

## What to build

更新 artifact schema 文件，添加新 schema，删除旧 schema，并提交到 git。

**核心变更**：
1. 创建新 schema 文件：
   - test-plan.schema.json
   - review-criteria.schema.json
   - system-inspector-report.schema.json
2. 删除旧 schema 文件：
   - checkpoint.schema.json
   - system-health-report.schema.json
   - orchestration-report.schema.json
   - cost-report.schema.json
   - chaos-test-report.schema.json
3. 提交到 git

**当前状态**：
- 新 schema 文件已创建（untracked）
- 旧 schema 文件已删除

## Acceptance criteria

- [ ] `.workflow/templates/` 包含 11 个 schema 文件
- [ ] 新 schema 文件已提交到 git
- [ ] 旧 schema 文件已从 git 删除
- [ ] schema 文件内容正确

## Blocked by

- 001: Agent Registry 统一

## User Stories

- 23: As a developer, I want artifact schemas to be validated, so that data integrity is maintained
