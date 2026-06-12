---
title: wb-ai CODEMAP
type: codemap
domain: architecture
crate: wb-ai
created: 2026-06-12
updated: 2026-06-12
status: active
---

# wb-ai CODEMAP

> **职责**：AI 模型适配层。管理大小模型的调用策略、升级机制、Token 预算。
> **对应文档**：[处理层架构 - ModelRouter](../architecture/modules/processing.md)

## 文件导航

| 文件 | 职责 | 关键类型 |
|------|------|---------|
| `lib.rs` | 模块导出 + pub use | — |
| `adapter.rs` | 模型适配器 trait | `ModelAdapter` trait, `MockAdapter`, `Classification`, `Extraction` |
| `router.rs` | 模型路由器（升级决策） | `ModelRouter`, `TaskType`, `UpgradeThreshold` |
| `budget.rs` | Token 预算管理 | `TokenBudget`, `OverloadStrategy` |
| `task_runner.rs` | 任务执行器（组合路由+预算+适配器） | `TaskRunner`, `ModelSize`, `TaskOutput` |
| `config.rs` | AI 模型配置 | `ModelConfig` |
| `anthropic.rs` | Anthropic 适配器 | `AnthropicAdapter` — 实现 `ModelAdapter` |
| `openai.rs` | OpenAI 适配器 | `OpenAIAdapter` — 实现 `ModelAdapter` |
| `prompt/` | Prompt 模板 | — |
| `prompt/mod.rs` | Prompt 子模块导出 | — |
| `prompt/classify.rs` | 分类 prompt | 事件分类的 prompt 模板 |
| `prompt/extract.rs` | 提取 prompt | 实体提取的 prompt 模板 |
| `prompt/summarize.rs` | 摘要 prompt | 内容摘要的 prompt 模板 |
| `prompt/analyze.rs` | 分析 prompt | 深度分析的 prompt 模板 |

## 数据流

```
TaskRunner::run_extract(event, confidence)
  → ModelRouter::should_upgrade(task_type, confidence)  → 决定用小/大模型
  → TokenBudget::check()                                → 检查预算
  → ModelAdapter::classify() / extract()                → 调用模型
  → 返回 TaskOutput
```

## 关键设计

- **ModelAdapter trait**：统一模型接口，支持 MockAdapter 用于测试
- **ModelRouter**：按任务类型 + 置信度决定是否升级到大模型
- **TokenBudget**：每日 token 预算控制，超预算时的降级策略
- **TaskRunner**：组合路由、预算、适配器的高层执行器

## 升级阈值（默认）

| 任务类型 | 阈值 | 说明 |
|---------|------|------|
| EntityExtraction | 0.7 | 小模型较弱 |
| TaskIdentification | 0.6 | 小模型较好 |
| Summarization | 0.6 | 长度 > 500 字也触发升级 |
| SentimentAnalysis | 0.8 | 小模型较弱 |
| RelationAnalysis | 0.7 | 小模型较弱 |
| PatternRecognition | 强制大模型 | 小模型不胜任 |
| Classification | 0.6 | 小模型较好 |

## 修改指引

| 你想改什么 | 先读 | 再改 |
|-----------|------|------|
| 修改升级阈值 | `router.rs` 的 `default_thresholds()` | 修改阈值配置 |
| 新增模型提供商 | `adapter.rs` | 新建文件实现 `ModelAdapter` trait |
| 修改 prompt | `prompt/` 对应文件 | 修改模板内容 |
| 调整预算策略 | `budget.rs` | 修改 `OverloadStrategy` |
| 新增任务类型 | `router.rs` 的 `TaskType` 枚举 | 添加枚举变体 + 阈值配置 |
