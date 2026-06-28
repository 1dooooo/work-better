# PRD: 多 Agent 体系重构 v5

## Problem Statement

当前多 agent 体系存在以下问题：
1. **成功率低**：30 个 workflow 实例中只有 7 个成功（35%），主要原因是并行审查阶段无法真正并行调用 agent
2. **agent 膨胀**：42 个 agent 定义中只有 5 个真正产出过输出，37 个从未运行
3. **监督层不可达**：post_task_phase 在 workflow-runner 中不可达，guardian/orchestrator/cost-tracker/validator 从未运行
4. **职责重叠**：guardian 和 orchestrator 功能高度重叠，test-agent 和 review-agent 在安全测试上重叠
5. **双源冲突**：agents.json 和 .claude/agents/*.md 定义矛盾，导致 agent 行为不一致

## Solution

重构为 8 个核心 agent 的统一调度体系：
- 所有 agent 平等对待，workflow-runner 统一调度
- Phase 1 并行启动（dev + test-plan + review-criteria）
- Phase 2 执行验证（test + review + product-review）
- 合并 guardian + orchestrator 为 system-inspector
- 删除从未运行的 checkpoint-manager

## User Stories

1. As a developer, I want workflow-runner to orchestrate all agents, so that I have a single entry point for the entire workflow
2. As a developer, I want dev-agent, test-agent, and product-reviewer to start in parallel, so that I can reduce total workflow time
3. As a developer, I want test-agent to design test methods based on user requirements (not dev output), so that test targets are independent of implementation
4. As a developer, I want product-reviewer to define review criteria based on user requirements (not dev output), so that acceptance standards are independent of implementation
5. As a developer, I want review-agent to start only after dev-agent completes, so that code review has actual code to review
6. As a developer, I want validator to run after Phase 2, so that all artifacts are validated before final report
7. As a developer, I want system-inspector to run after validation, so that system health is monitored
8. As a developer, I want optimizer to be triggered only when system-inspector finds issues, so that optimization is on-demand
9. As a developer, I want all agent definitions in agents.json, so that there is a single source of truth
10. As a developer, I want .md files to be updated to match agents.json, so that there are no contradictions
11. As a developer, I want all schema files committed to git, so that validator can perform schema validation
12. As a developer, I want CLAUDE.md to have simplified flow description, so that it's easy to understand
13. As a developer, I want to use Agent tool's parallel calls, so that parallel execution is actually implemented
14. As a developer, I want to keep cost_limits in workflow spec, so that cost control is maintained
15. As a developer, I want to remove unused infrastructure (chaos-testing, circuit_breaker, etc.), so that workflow spec is simplified
16. As a developer, I want consistent agent naming (short names without -agent suffix), so that references are clear
17. As a developer, I want parallel_group markers in YAML, so that workflow-runner knows which steps to run in parallel
18. As a developer, I want each agent to have a clear responsibility boundary, so that there is no overlap or confusion
19. As a developer, I want workflow-runner to handle retry logic, so that failures are properly managed
20. As a developer, I want final-report.json to include all agent summaries, so that I have a complete view of the workflow
21. As a developer, I want agent prompts to be layered (base + task), so that agents are both stable and flexible
22. As a developer, I want workflow spec to define clear trigger conditions, so that agents are called at the right time
23. As a developer, I want artifact schemas to be validated, so that data integrity is maintained
24. As a developer, I want to remove checkpoint-manager, so that workflow complexity is reduced
25. As a developer, I want system-inspector to combine guardian and orchestrator responsibilities, so that supervision is unified

## Implementation Decisions

### 1. Agent Registry

All 8 agents defined in `.claude/agents.json`:
- workflow-runner, dev-agent, test-agent, review-agent, product-reviewer
- system-inspector, validator, optimizer

`.claude/agents/*.md` files updated to match agents.json.

### 2. Workflow Phases

**Phase 1: Parallel Start**
- dev-agent: write code + L1/L2 tests → dev-output.json
- test-agent: design test methods → test-plan.json
- product-reviewer: define review criteria → review-criteria.json
- Implementation: Agent tool parallel calls (multiple Agent invocations in single response)

**Phase 2: Execute Validation**
- test-agent: execute tests → test-report.json
- product-reviewer: execute review → product-review.json
- review-agent: code review + H1-H5 security → review-report.json
- Implementation: Agent tool parallel calls

**Phase 3: Validation**
- validator: schema + data integrity → validation-report.json
- Trigger: after Phase 2

**Phase 4: Supervision**
- system-inspector: system health + execution efficiency → system-inspector-report.json
- Trigger: after validation

**Optional: Optimization**
- optimizer: generate optimization plan → optimization-plan.json
- Trigger: system-inspector finds agent-level issues
- Requires user approval

### 3. Parallel Execution

Use Agent tool's parallel calls (multiple Agent invocations in single response), not Workflow tool's parallel() API.

YAML uses `parallel_group` markers to indicate which steps should run in parallel.

### 4. Naming Convention

Short names without `-agent` suffix where applicable:
- workflow-runner, dev-agent, test-agent, review-agent, product-reviewer
- system-inspector, validator, optimizer

### 5. Configuration

Keep in workflow spec:
- cost_limits (cost control)

Remove from workflow spec:
- chaos-testing_framework (never used)
- circuit_breaker (never used)
- quality_evaluation (never used)
- distributed_tracing (never used)
- checkpoint_recovery (removed with checkpoint-manager)

### 6. Artifact Schemas

New schemas:
- test-plan.schema.json
- review-criteria.schema.json
- system-inspector-report.schema.json

Deleted schemas:
- checkpoint.schema.json
- system-health-report.schema.json
- orchestration-report.schema.json
- cost-report.schema.json
- chaos-test-report.schema.json

### 7. Responsibility Boundaries

| Agent | Does | Does NOT |
|-------|------|----------|
| workflow-runner | Orchestrate, retry, report | Write code, test, review |
| dev-agent | Write code + L1/L2 tests | Write L4/L5 tests, review |
| test-agent | Design tests (Phase 1), execute tests (Phase 2) | Write code, review, security tests |
| review-agent | Code review + H1-H5 security | Write code, write tests, product decisions |
| product-reviewer | Define criteria (Phase 1), execute review (Phase 2) | Code quality review |
| system-inspector | System health + execution efficiency | Write code, execute optimization |
| validator | Schema + data integrity | Code quality, product review |
| optimizer | Generate optimization plan | Execute without approval |

## Testing Decisions

### Test Seams

1. **Agent invocation**: Test that workflow-runner correctly invokes agents in parallel
2. **Artifact validation**: Test that validator correctly validates schemas
3. **Retry logic**: Test that workflow-runner handles failures and retries correctly
4. **Phase transitions**: Test that workflow progresses through phases correctly

### Test Types

- Unit tests for individual agent logic
- Integration tests for agent communication via artifacts
- E2E tests for complete workflow execution

### Prior Art

- Existing workflow tests in `.workflow/artifacts/` directories
- Schema validation tests in validator agent

## Out of Scope

1. **Prompt layering**: Base prompt + task prompt mechanism not implemented in this phase
2. **Workflow tool integration**: Using Agent tool parallel calls instead of Workflow tool
3. **Performance optimization**: Focus on correctness first, performance later
4. **Agent memory**: Not addressed in this refactor
5. **Cross-project workflows**: Only single-project workflows considered

## Further Notes

### Migration Path

1. Update `.claude/agents/*.md` to match agents.json
2. Commit new schema files to git
3. Update CLAUDE.md with simplified flow description
4. Update workflow spec with parallel_group markers
5. Test workflow execution with new configuration

### Success Metrics

- Workflow success rate: target >80% (currently 35%)
- Agent utilization: all 8 agents should produce output
- Workflow duration: target <10 minutes for typical tasks

### Risks

1. **Parallel execution may fail**: Agent tool parallel calls may not work as expected
2. **Schema validation may be strict**: New schemas may reject valid artifacts
3. **Agent prompts may need tuning**: New prompts may produce unexpected results
