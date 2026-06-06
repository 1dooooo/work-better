# Work Better - Agent Guide

## 核心思想

**Work Better 是一个以 Obsidian 为中心的 AI 工作观察者。**

被动采集工作信息，智能处理与分析，自动维护任务与报告，帮助使用者做到更好工作而不必刻意记录。

### 三大原则

1. **观察者姿态**——被动采集、主动整理，用户不需要改变工作习惯
2. **Obsidian 为中心**——所有数据的最终状态以 Obsidian 中的记录为准，数据归用户所有
3. **自主但可干预**——AI 在私有数据空间有完全自主权，涉及共享数据的操作需人工确认

### 核心公式

```
高信道密度采集 → 分级智能处理 → 多维持久化 → 多形态呈现
```

## 文档索引

| 文档 | 路径 | 说明 |
|------|------|------|
| 产品文档 | [docs/product/overview.md](docs/product/overview.md) | 产品理念、功能方向、演进指导 |
| 功能索引 | [docs/features/index.md](docs/features/index.md) | 全功能点分层索引，指导测试 |
| 架构总览 | [docs/architecture/overview.md](docs/architecture/overview.md) | 高层架构设计与模块索引 |
| 采集层 | [docs/architecture/modules/collection.md](docs/architecture/modules/collection.md) | 采集层架构细节 |
| 处理层 | [docs/architecture/modules/processing.md](docs/architecture/modules/processing.md) | 处理层架构细节 |
| 存储层 | [docs/architecture/modules/storage.md](docs/architecture/modules/storage.md) | 存储层架构细节 |
| 呈现层 | [docs/architecture/modules/presentation.md](docs/architecture/modules/presentation.md) | 呈现层架构细节 |
| 定时任务 | [docs/architecture/modules/scheduler.md](docs/architecture/modules/scheduler.md) | 定时任务系统架构 |
| 事件模型 | [docs/architecture/modules/event-model.md](docs/architecture/modules/event-model.md) | 事件与数据模型定义 |
| 开发指引 | [docs/development/setup.md](docs/development/setup.md) | 开发环境搭建与工作流指引 |
| 测试策略 | [docs/testing/strategy.md](docs/testing/strategy.md) | 测试策略总览、分层定义、技术栈 |
| 测试规范 | [docs/testing/conventions.md](docs/testing/conventions.md) | 命名、组织、编写规范 |
| Harness 设计 | [docs/testing/infrastructure/harness.md](docs/testing/infrastructure/harness.md) | 测试夹具系统架构 |
| Mock 系统 | [docs/testing/infrastructure/mocking.md](docs/testing/infrastructure/mocking.md) | AI/飞书 Mock 策略 |
| 测试数据 | [docs/testing/infrastructure/fixtures.md](docs/testing/infrastructure/fixtures.md) | 工厂模式、种子数据 |
| 单元测试 | [docs/testing/layers/unit.md](docs/testing/layers/unit.md) | 单元测试编写指南 |
| 集成测试 | [docs/testing/layers/integration.md](docs/testing/layers/integration.md) | 集成测试编写指南 |
| E2E 测试 | [docs/testing/layers/e2e.md](docs/testing/layers/e2e.md) | E2E 测试编写指南 |
| CI 集成 | [docs/testing/ci.md](docs/testing/ci.md) | 流水线中的测试阶段 |

## 文档自维护规范

所有文档遵循以下规范以保持新鲜度：

### 触发条件

| 变更类型 | 需要更新的文档 |
|---------|--------------|
| 新增/移除功能 | 功能索引 + 对应模块文档 + 产品文档（如涉及方向变化） |
| 架构变更 | 架构总览 + 对应模块文档 |
| 新增/移除模块 | 架构总览 + agent.md 文档索引 |
| 产品方向调整 | 产品文档 + agent.md 核心思想（如涉及原则变化） |
| 数据模型变更 | 事件模型文档 + 对应模块文档 |

### 维护流程

1. 代码变更时，检查是否触发上述条件
2. 先更新文档，再提交代码（文档与代码同 PR）
3. 文档中的架构图、数据模型必须与代码实现一致
4. 模块文档中的接口定义必须与实际代码同步

### 文档健康检查

定期检查：
- [ ] 所有文档链接有效
- [ ] 架构图与代码结构一致
- [ ] 功能索引覆盖所有已实现功能
- [ ] 模块文档中的接口与代码同步
- [ ] 无过时的 TODO 或 TBD 标记
