---
title: 多 Agent 架构重构 PRD
type: prd
domain: development
created: 2026-06-21
status: ready-for-agent
---

# 多 Agent 架构重构 PRD

## Problem Statement

当前项目使用多 Agent 协作开发，但存在以下问题：

1. **调度机制不合理**：主 Agent 在调用 `run-workflow.sh` 后失去控制权，Bash 脚本成为真正的指挥官
2. **缺少系统监督**：没有专门的 Agent 监督整个系统的健康状况
3. **缺少自动优化**：Agent 完成任务后无法自动发现和执行优化点
4. **缺少质量保障**：没有管道交叉点验证、成本追踪、混沌测试等机制
5. **缺少容错能力**：没有熔断器、检查点恢复等机制

## Solution

重构为**主 Agent 作为指挥官**的架构，使用 Workflow tool 直接编排 subagent，并新增多个专业 Agent 来监督、优化和保障系统质量。

核心变化：
- 主 Agent 成为指挥官，使用 Workflow tool 编排 subagent
- 新增 guardian Agent（守护者）监督整个系统
- 新增 optimizer Agent（优化者）执行具体优化任务
- 新增 orchestrator-agent（监督者）监督所有 Agent
- 新增 validator-agent（验证者）在管道交叉点验证
- 新增 cost-tracker（成本追踪者）追踪 token 使用
- 新增 chaos-tester（混沌测试者）随机故障注入
- 新增 checkpoint-manager（检查点管理者）管理恢复点
- 实现分布式追踪、熔断器、成本限制、质量评估等机制

## User Stories

1. As a developer, I want the main agent to be the commander, so that it can directly orchestrate subagents without losing control
2. As a developer, I want the main agent to use Workflow tool to orchestrate, so that I can have fine-grained control over the workflow
3. As a developer, I want a guardian agent to monitor the system, so that I can be alerted to system-level issues
4. As a developer, I want an optimizer agent to execute optimizations, so that the system can continuously improve
5. As a developer, I want an orchestrator-agent to supervise all agents, so that I can ensure all agents are working correctly
6. As a developer, I want a validator-agent to verify outputs at pipeline junctions, so that errors don't propagate downstream
7. As a developer, I want a cost-tracker to track token usage, so that I can monitor and control costs
8. As a developer, I want a chaos-tester to inject random failures, so that I can test system resilience
9. As a developer, I want a checkpoint-manager to save checkpoints, so that I can resume from failures without starting over
10. As a developer, I want distributed tracing, so that I can trace the full execution path
11. As a developer, I want circuit breakers, so that cascading failures are prevented
12. As a developer, I want cost limits, so that I can control token usage per workflow
13. As a developer, I want quality evaluation, so that I can measure agent performance
14. As a developer, I want a chaos testing framework, so that I can systematically test resilience
15. As a developer, I want checkpoint recovery, so that I can resume from mid-workflow failures
16. As a developer, I want the workflow spec to be updated, so that it reflects the new architecture
17. As a developer, I want CLAUDE.md to be updated, so that it has the new constraint rules
18. As a developer, I want each agent to have clear responsibilities, so that there is no overlap or confusion
19. As a developer, I want each agent to have clear input/output, so that the communication is predictable
20. As a developer, I want the system to be self-improving, so that it gets better over time
21. As a developer, I want the guardian agent to review after each task, so that issues are caught early
22. As a developer, I want the optimizer agent to search for better skills, so that agents can become more professional
23. As a developer, I want the orchestrator-agent to generate reports, so that I can understand system performance
24. As a developer, I want the validator-agent to check schema compliance, so that data integrity is maintained
25. As a developer, I want the cost-tracker to generate cost reports, so that I can analyze cost patterns
26. As a developer, I want the chaos-tester to run chaos tests, so that I can identify weaknesses
27. As a developer, I want the checkpoint-manager to manage checkpoints, so that I can recover from failures
28. As a developer, I want the main agent to read workflow spec, so that it understands the flow
29. As a developer, I want the main agent to judge task complexity, so that it can choose the right orchestration method
30. As a developer, I want the main agent to call predefined workflows for complex tasks, so that the flow is reusable

## Implementation Decisions

### 1. Agent 职责矩阵

| Agent | 职责 | 输入 | 输出 | 写代码 | 写测试 | 审查代码 | 编排调度 | 监督优化 |
|-------|------|------|------|--------|--------|---------|---------|---------|
| 主 Agent | 指挥官，编排 subagent | 用户任务 | 任务结果 | ❌ | ❌ | ❌ | ✅ | ❌ |
| dev-agent | 开发者，写代码 + L1-L2 测试 | 任务描述 | dev-output.json | ✅ | ✅ | ❌ | ❌ | ❌ |
| test-agent | 测试者，执行测试 + 安全扫描 | dev-output.json | test-report.json | ❌ | ✅ | ❌ | ❌ | ❌ |
| review-agent | 审查者，代码审查 + 安全测试 | dev-output.json | review-report.json | ❌ | ✅ | ✅ | ❌ | ❌ |
| product-reviewer | 产品审查者，产品符合性审查 | dev-output.json | product-review.json | ❌ | ❌ | ✅ | ❌ | ❌ |
| validator-agent | 验证者，管道交叉点验证 | 所有 artifact | validation-report.json | ❌ | ❌ | ✅ | ❌ | ❌ |
| cost-tracker | 成本追踪者，追踪 token 使用 | 所有 artifact | cost-report.json | ❌ | ❌ | ❌ | ❌ | ✅ |
| chaos-tester | 混沌测试者，随机故障注入 | workflow 配置 | chaos-test-report.json | ❌ | ✅ | ❌ | ❌ | ❌ |
| checkpoint-manager | 检查点管理者，管理恢复点 | workflow 状态 | checkpoint.json | ❌ | ❌ | ❌ | ❌ | ✅ |
| guardian Agent | 守护者，监督整个系统 | 所有 artifact | system-health-report.json | ❌ | ❌ | ❌ | ❌ | ✅ |
| optimizer Agent | 优化者，执行具体优化任务 | guardian Agent 输出 | optimization-plan.json | ✅ | ❌ | ❌ | ❌ | ✅ |
| orchestrator-agent | 监督者，监督所有 Agent | 所有 Agent 输出 | orchestration-report.json | ❌ | ❌ | ❌ | ✅ | ✅ |
| workflow-runner | 可选的编排辅助工具 | 主 Agent 调用 | final-report.json | ❌ | ❌ | ❌ | ✅ | ❌ |

### 2. 职责边界规则

- **主 Agent vs orchestrator-agent**：主 Agent 编排任务，orchestrator-agent 监督执行
- **guardian Agent vs optimizer Agent**：guardian 发现问题，optimizer 执行优化
- **validator-agent vs review-agent**：validator 验证数据，review-agent 审查代码
- **cost-tracker vs orchestrator-agent**：cost-tracker 追踪成本，orchestrator-agent 监督整体

### 3. 编排方式

- **简单任务**：主 Agent 内联编排，直接调用 dev-agent
- **复杂任务**：主 Agent 调用预定义 workflow
- **任务复杂度**：由专门的 agent 判断

### 4. 通信机制

- **文件契约通信**：通过 `.workflow/artifacts/` 下的 JSON 文件传递信息
- **Schema 验证**：所有 artifact 必须符合对应的 schema
- **Handoff skill**：用于会话传递

### 5. 自迭代机制

- 每个 agent 完成任务后记录 improvements
- guardian Agent 审查并生成优化计划
- 用户审批后，optimizer Agent 执行优化
- optimizer Agent 自己验证优化效果

### 6. 约束机制

- **决策级约束**：CLAUDE.md（软约束，强烈推荐）
- **流程级约束**：.claude/rules/（软约束）
- **编排级约束**：.workflow/specs/（软约束）
- **角色级约束**：.claude/agents/*.md（软约束）
- **执行级约束**：.claude/hooks/hooks.json（硬约束，代码级强制）

## Testing Decisions

### 测试原则

- **只测试外部行为**：不测试实现细节
- **测试 seams**：在模块边界测试
- **优先现有 seams**：使用已有的测试基础设施

### 测试模块

| 模块 | 测试类型 | 测试位置 |
|------|---------|---------|
| Agent 定义 | 单元测试 | `.claude/agents/*.md` 的 prompt 验证 |
| Workflow Spec | 集成测试 | `.workflow/specs/dev-test-review.yaml` 的流程验证 |
| Artifact Schema | 集成测试 | `.workflow/templates/*.schema.json` 的 schema 验证 |
| Hooks | 集成测试 | `.claude/hooks/hooks.json` 的 hook 执行验证 |
| 端到端流程 | E2E 测试 | 完整的 workflow 执行验证 |

### 测试 Prior Art

- **现有测试**：`crates/` 下的 Rust 测试、`src/` 下的 TypeScript 测试
- **测试框架**：Rust 使用 `rstest`、`insta`；TypeScript 使用 `vitest`、`@playwright/test`
- **测试模式**：AAA（Arrange-Act-Assert）模式

## Out of Scope

1. **A2A 协议实现**：当前使用文件契约通信，不实现 A2A 协议
2. **OpenTelemetry 集成**：当前使用文件日志，不实现分布式追踪
3. **混沌测试框架**：当前只实现基础的混沌测试，不实现完整框架
4. **检查点恢复**：当前只实现基础的检查点管理，不实现完整恢复机制
5. **质量评估框架**：当前只实现基础的质量评估，不实现完整框架

## Further Notes

### 实现阶段

**Phase 1（核心架构）**：
1. 重构主 Agent 为指挥官
2. 新增 guardian Agent
3. 新增 optimizer Agent
4. 新增 orchestrator-agent

**Phase 2（关键 Agent）**：
5. 新增 validator-agent
6. 新增 cost-tracker
7. 更新 workflow spec
8. 更新 CLAUDE.md

**Phase 3（高级功能）**：
9. 新增 chaos-tester
10. 新增 checkpoint-manager
11. 实现分布式追踪
12. 实现熔断器
13. 实现成本限制

**Phase 4（完善功能）**：
14. 实现质量评估
15. 实现混沌测试框架
16. 实现检查点恢复

### Issue 拆分

| Issue | 描述 | 优先级 | 预估工作量 |
|-------|------|--------|-----------|
| 1. 重构主 Agent 为指挥官 | 修改 CLAUDE.md，让主 Agent 成为指挥官，直接调度 subagent | 高 | 中 |
| 2. 新增 guardian Agent | 创建守护者 Agent，负责监督整个系统 | 高 | 小 |
| 3. 新增 optimizer Agent | 创建优化者 Agent，执行具体优化任务 | 高 | 小 |
| 4. 新增 orchestrator-agent | 创建监督者 Agent，监督所有 Agent | 高 | 小 |
| 5. 新增 validator-agent | 创建验证者 Agent，管道交叉点验证 | 高 | 中 |
| 6. 新增 cost-tracker | 创建成本追踪者 Agent | 高 | 小 |
| 7. 新增 chaos-tester | 创建混沌测试者 Agent | 中 | 中 |
| 8. 新增 checkpoint-manager | 创建检查点管理者 Agent | 中 | 中 |
| 9. 实现分布式追踪 | 使用 OpenTelemetry 结构化追踪 | 高 | 大 |
| 10. 实现熔断器 | 防止级联故障 | 高 | 中 |
| 11. 实现成本限制 | 设置每个 workflow 的 token 上限 | 高 | 中 |
| 12. 实现质量评估 | 5 个核心评估维度 | 中 | 大 |
| 13. 实现混沌测试框架 | 随机故障注入测试 | 中 | 大 |
| 14. 实现检查点恢复 | 从中断点恢复 | 中 | 大 |
| 15. 更新 workflow spec | 更新 dev-test-review.yaml | 高 | 中 |
| 16. 更新 CLAUDE.md | 更新约束规则 | 高 | 小 |
