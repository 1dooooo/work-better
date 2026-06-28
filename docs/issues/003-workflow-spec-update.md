# Issue 003: 更新 Workflow Spec 为 v5

## Parent

PRD: docs/prd/multi-agent-refactor-v5.md

## What to build

更新 workflow spec 为 v5 统一调度版，定义新的执行流程。

**核心变更**：
1. 更新执行流程为 Phase 1-4：
   - Phase 1: 并行启动（dev + test-plan + review-criteria）
   - Phase 2: 执行验证（test + review + product-review）
   - Phase 3: 验证（validator）
   - Phase 4: 监督（system-inspector）
2. 添加 parallel_group 标记
3. 移除未使用的配置：
   - chaos-testing_framework
   - circuit_breaker
   - quality_evaluation
   - distributed_tracing
   - checkpoint_recovery
4. 保留 cost_limits

**当前状态**：
- `.workflow/specs/dev-test-review.yaml` 已更新为 v5
- 需要添加 parallel_group 标记

## Acceptance criteria

- [ ] workflow spec 定义 Phase 1-4 执行流程
- [ ] parallel_group 标记正确
- [ ] 未使用的配置已移除
- [ ] cost_limits 保留
- [ ] 所有 8 个 agent 在 spec 中定义

## Blocked by

- 002: Schema 文件更新

## User Stories

- 1: As a developer, I want workflow-runner to orchestrate all agents, so that I have a single entry point for the entire workflow
- 2: As a developer, I want dev-agent, test-agent, and product-reviewer to start in parallel, so that I can reduce total workflow time
- 3: As a developer, I want test-agent to design test methods based on user requirements (not dev output), so that test targets are independent of implementation
- 4: As a developer, I want product-reviewer to define review criteria based on user requirements (not dev output), so that acceptance standards are independent of implementation
- 5: As a developer, I want review-agent to start only after dev-agent completes, so that code review has actual code to review
- 6: As a developer, I want validator to run after Phase 2, so that all artifacts are validated before final report
- 7: As a developer, I want system-inspector to run after validation, so that system health is monitored
- 8: As a developer, I want optimizer to be triggered only when system-inspector finds issues, so that optimization is on-demand
- 13: As a developer, I want to use Agent tool's parallel calls, so that parallel execution is actually implemented
- 15: As a developer, I want to remove unused infrastructure (chaos-testing, circuit_breaker, etc.), so that workflow spec is simplified
- 17: As a developer, I want parallel_group markers in YAML, so that workflow-runner knows which steps to run in parallel
