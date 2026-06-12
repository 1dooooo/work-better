---
title: wb-processor CODEMAP
type: codemap
domain: architecture
crate: wb-processor
created: 2026-06-12
updated: 2026-06-12
status: active
---

# wb-processor CODEMAP

> **职责**：事件处理层。消费 EventLog 中的事件，进行分类、提取、审核，输出 WorkRecord。
> **对应文档**：[处理层架构](../architecture/modules/processing.md)

## 文件导航

### 核心流水线

| 文件 | 职责 | 关键类型 |
|------|------|---------|
| `lib.rs` | 模块导出 + pub use | — |
| `pipeline.rs` | 处理流水线（串联所有步骤） | `ProcessingPipeline`, `ProcessedResult`, `StepTimings` |
| `classifier.rs` | 事件分类器（规则引擎） | `Classifier`, `ProcessingRoute` (Instant/Aggregate/Pattern/Archive) |
| `extraction.rs` | 实体提取 | `EntityExtractor`, `ExtractedData` |
| `persist.rs` | 持久化步骤 | `PersistStep` — 生成 Obsidian 路径、写入文件 |

### 审核子系统

| 文件 | 职责 | 关键类型 |
|------|------|---------|
| `review.rs` | 审核模型定义 | `TieredReview`, `SmallModelReview`, `LargeModelReview`, `OutputType`, `DataScope` |
| `review_rules.rs` | 规则层审核 | 格式校验、必填字段、状态流转合法性 |
| `reviewer.rs` | 审核代理（组合规则+模型审核） | `ReviewAgent` |

### SLA 管理

| 文件 | 职责 | 关键类型 |
|------|------|---------|
| `sla.rs` | SLA 优先级和超时管理 | `SlaManager`, `SlaConfig`, `Priority`, `TimelinessReport` |

### 审计

| 文件 | 职责 | 关键类型 |
|------|------|---------|
| `audit_pipeline.rs` | 审计数据处理管道 | `AuditPipeline`, `AuditFilter`, `OptimizationSuggestion` |

### 任务发现 (`task/`)

| 文件 | 职责 | 关键类型 |
|------|------|---------|
| `mod.rs` | 任务子模块导出 | — |
| `model.rs` | 任务发现模型 | 任务发现的数据结构 |
| `create.rs` | 任务创建 | 从提取结果创建 Task |
| `discovery.rs` | 任务自动发现入口 | 通用发现逻辑 |
| `discovery_confirm.rs` | 发现确认流程 | 用户确认机制 |
| `discovery_email.rs` | 邮件任务识别 | 从邮件中提取任务 |
| `discovery_meeting.rs` | 会议待办提取 | 从会议纪要中提取任务 |
| `discovery_message.rs` | 消息任务识别 | 从聊天消息中识别任务 |
| `hierarchy.rs` | 任务层级 | 父子任务关系 |
| `lifecycle.rs` | 任务生命周期 | 状态流转管理 |
| `sync.rs` | 飞书任务同步 | 双向同步逻辑 |

### 报告生成 (`report/`)

| 文件 | 职责 | 关键类型 |
|------|------|---------|
| `mod.rs` | 报告子模块导出 | `Report`, `ReportGenerator`, `ReportType` |
| `template.rs` | 报告模板引擎 | 模板渲染 |
| `daily.rs` | 日报生成 | — |
| `weekly.rs` | 周报生成 | — |
| `monthly.rs` | 月报生成 | — |
| `quarterly.rs` | 季报生成 | — |
| `semi_annual.rs` | 半年报生成 | — |
| `annual.rs` | 年报生成 | — |
| `confirm.rs` | 报告确认流程 | 用户确认、编辑 |
| `export.rs` | 报告导出 | Markdown / PDF |
| `sync_feishu.rs` | 报告同步飞书 | 确认后同步到飞书文档 |

### 测试文件 (`tests/`)

| 文件 | 职责 |
|------|------|
| `report_generation_l2.rs` | 报告生成 L2 集成测试 |
| `real_backend_task_creation.rs` | 真实后端任务创建测试 |

## 数据流

```
Event
  → Classifier::classify()        → ProcessingRoute
  → TaskRunner::run_extract()     → ExtractedData
  → EntityExtractor::to_work_record() → WorkRecord
  → ReviewAgent::review()         → ReviewResult
  → PersistStep::persist()        → 写入文件系统
```

## 关键设计

- **ProcessingPipeline** 串联所有步骤，Archive 路由跳过模型调用
- **Classifier** 是纯规则引擎，不调用 AI
- **ReviewAgent** 组合规则层审核 + 模型层审核
- **PersistStep** 生成 Obsidian 路径并写入文件

## 修改指引

| 你想改什么 | 先读 | 再改 |
|-----------|------|------|
| 修改事件分类规则 | `classifier.rs` 的 `classify()` | 修改路由逻辑 |
| 修改审核规则 | `review_rules.rs` | 添加/修改规则 |
| 新增报告类型 | `report/mod.rs` | 新建文件 + 注册 |
| 修改任务发现逻辑 | `task/discovery*.rs` | 对应的发现器 |
| 修改流水线步骤 | `pipeline.rs` 的 `process()` | 调整步骤顺序 |
