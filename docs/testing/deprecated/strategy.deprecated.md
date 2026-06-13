---
title: 测试策略
type: structural
domain: testing
created: 2026-06-06
updated: 2026-06-06
status: deprecated
---

# 测试策略总览

## 核心理念

**EventLog 是测试的锚点。**

整个系统的测试围绕 EventLog 展开——它是采集层和处理层的解耦边界，也是测试的天然断言点。

```
采集层测试 ──输出──→ Event ──验证格式──→ EventLog
处理层测试 ──读取──→ EventLog ──验证处理──→ WorkRecord
集成测试   ──端到端──→ Event → WorkRecord → Obsidian 文档
E2E 测试   ──场景──→ 飞书消息 → 处理 → Obsidian → 报告
```

每一层的输入和输出都是明确的、可断言的、可独立验证的。

## 测试分层

| 层级 | 职责 | 速度目标 | 外部依赖 | 文件后缀 |
|------|------|---------|---------|---------|
| **单元测试** | 函数/类的逻辑正确性 | <100ms/用例 | 全 Mock | `*.test.ts` |
| **集成测试** | 层间数据流、接口契约 | <500ms/用例 | Mock 外部服务，真实内部模块 | `*.int.test.ts` |
| **E2E 测试** | 完整用户场景 | <30s/场景 | Mock 飞书/AI，真实文件系统 | `*.e2e.test.ts` |

### 各层职责边界

**单元测试验证什么：**
- Classifier 路由规则是否正确
- ModelRouter 升级阈值判定是否准确
- Task 状态机转换是否合法
- TokenBudget 预算分配逻辑
- SLA 超时判定

**集成测试验证什么：**
- Event 经 Classifier 路由后进入正确的处理路径
- 处理结果经 ReviewAgent 审查后正确写入存储层
- 存储层三层一致性（Obsidian ↔ SQLite ↔ VectorDB）
- 调度器依赖链执行顺序

**E2E 测试验证什么：**
- 飞书消息 → Event → WorkRecord → Obsidian 文档的完整流转
- 快记窗口输入 → EventLog → 自动分类 → 归档
- 定时任务触发 → 聚合 → 报告生成

## 覆盖率标准

| 指标 | 目标 | 说明 |
|------|------|------|
| 行覆盖率 | ≥80% | 所有模块统一标准 |
| 分支覆盖率 | ≥70% | 重点覆盖决策分支 |
| 关键路径 | 100% | Classifier 路由、Task 状态机、SLA 判定 |

### 关键路径（必须 100% 覆盖）

```
Classifier 路由决策表 ── 14 种 Source × 11 种 EventType 的路由结果
Task 状态机转换 ── 合法路径 + 非法拦截
SLA 优先级矩阵 ── P0/P1/P2/P3 的超时判定
模型升级阈值 ── 各任务类型的置信度判定
ReviewAgent 审查判定 ── approved/needs_fix/needs_review
```

## 技术栈

| 用途 | 工具 | 版本 | 说明 |
|------|------|------|------|
| 测试运行器 | Vitest | ^3.x | 单元 + 集成 + E2E（Node.js 端） |
| API Mock | MSW | ^2.x | 飞书 API 契约测试 |
| 数据工厂 | fishery | ^2.x | 类型安全的测试数据生成 |
| 随机数据 | @faker-js/faker | ^9.x | 随机文本、名称等 |
| Schema 验证 | zod | ^3.x | Event/WorkRecord 结构校验 |

> **关于 E2E 测试框架选择：** 本项目的 E2E 场景（采集→处理→存储→报告）主要在 Node.js 端执行，不涉及浏览器 UI 交互，因此统一使用 Vitest。未来若需要浏览器端 UI 测试（如菜单栏、快记窗口的交互），再引入 Playwright。

## 目录结构

```
test/
├── setup.ts                        # 全局 setup（dotenv、polyfills、自定义 matcher）
│
├── _foundation/                    # 测试基础设施（harness 核心）
│   ├── fakes/                      # Fake 实现
│   │   ├── fake-event-log.ts       # EventLog 的内存实现
│   │   ├── fake-model-router.ts    # AI 模型路由 Mock
│   │   ├── fake-review-agent.ts    # 审查代理 Mock
│   │   └── fake-token-budget.ts    # Token 预算 Mock
│   ├── factories/                  # 数据工厂
│   │   ├── event.factory.ts
│   │   ├── work-record.factory.ts
│   │   ├── task.factory.ts
│   │   └── audit.factory.ts
│   ├── matchers/                   # 自定义断言
│   │   └── index.ts
│   ├── handlers/                   # MSW handlers
│   │   └── feishu/                 # 飞书 API handlers
│   │       ├── messages.ts
│   │       ├── calendar.ts
│   │       ├── tasks.ts
│   │       ├── docs.ts
│   │       └── index.ts
│   └── helpers/                    # 工具函数
│       ├── run-pipeline.ts
│       └── load-fixture.ts
│
├── fixtures/                       # 测试数据
│   ├── events/                     # Event 快照（JSONL）
│   ├── api-responses/              # 飞书 API 录制响应
│   ├── models/                     # AI 模型输出快照
│   └── scenarios/                  # 完整业务场景
│
├── collection/                     # 采集层测试
├── processing/                     # 处理层测试
├── storage/                        # 存储层测试
├── presentation/                   # 呈现层测试
├── scheduler/                      # 调度器测试
└── e2e/                            # E2E 测试
```

## 定时任务测试归属

定时任务涉及多个层，测试按"执行逻辑归属"原则分配：

| 测试内容 | 归属目录 |
|---------|---------|
| 任务调度逻辑（cron 解析、依赖链、重试策略、全局控制） | `test/scheduler/` |
| C-01 飞书日历同步的执行逻辑 | `test/collection/` |
| P-01 小时聚合的执行逻辑 | `test/processing/` |
| S-01 任务状态同步的执行逻辑 | `test/storage/` |
| R-01 日报生成的执行逻辑 | `test/presentation/` |

## 文档索引

| 文档 | 路径 | 说明 |
|------|------|------|
| 测试规范 | [conventions.md](conventions.md) | 命名、组织、编写规范 |
| Harness 设计 | [infrastructure/harness.md](infrastructure/harness.md) | 测试夹具系统架构 |
| Mock 系统 | [infrastructure/mocking.md](infrastructure/mocking.md) | AI/飞书 Mock 策略 |
| 测试数据 | [infrastructure/fixtures.md](infrastructure/fixtures.md) | 工厂模式、种子数据 |
| 单元测试 | [layers/unit.md](layers/unit.md) | 单元测试编写指南 |
| 集成测试 | [layers/integration.md](layers/integration.md) | 集成测试编写指南 |
| E2E 测试 | [layers/e2e.md](layers/e2e.md) | E2E 测试编写指南 |
| CI 集成 | [ci.md](ci.md) | 流水线中的测试阶段 |
