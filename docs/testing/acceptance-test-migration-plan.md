---
title: 验收测试迁移计划
type: structural
domain: testing
created: 2026-06-11
status: active
---

# 验收测试迁移计划

> **维护说明**：当完成阶段性修复后更新本文档的对应章节和状态标记。

## 背景

根据 [测试有效性审计报告](test-effectiveness-audit.md)，验收测试（L5/G）的 step 实现是空壳：
- 182 个 Gherkin 场景已定义
- step 实现仅在 AcceptanceWorld 的 HashMap 中存字符串
- 没有调用真实的系统（EventLog、Classifier、Storage、Scheduler 等）
- 没有真实的断言

## 现状分析

### 验收测试结构

```
tests/acceptance/
├── features/                    # 7 个 feature 文件，182 个场景
│   ├── g1_information_collection.feature  # 37 个场景
│   ├── g2_intelligent_processing.feature  # 34 个场景
│   ├── g3_data_storage.feature            # 28 个场景
│   ├── g4_task_management.feature         # 20 个场景
│   ├── g5_report_generation.feature       # 12 个场景
│   ├── g6_system_capabilities.feature     # 38 个场景
│   └── g7_cross_cutting.feature           # 13 个场景
└── src/steps/                   # 8 个 step 文件，369 个函数
    ├── g1_collection.rs         # 88 个函数
    ├── g2_processing.rs         # 62 个函数
    ├── g3_storage.rs            # 52 个函数
    ├── g4_task.rs               # 32 个函数
    ├── g5_report.rs             # 30 个函数
    ├── g6_system.rs             # 67 个函数
    ├── g7_cross_cutting.rs      # 38 个函数
    └── mod.rs
```

### 当前 step 实现的问题

1. **Given 步骤**：仅设置 AcceptanceWorld 的字段（event_type、state 等）
2. **When 步骤**：仅设置 processing_result 字符串
3. **Then 步骤**：仅断言 AcceptanceWorld 的字段值

**核心问题**：没有调用真实的系统，断言的是 mock 状态而非真实行为。

## 迁移策略

### 方案：增强 AcceptanceWorld

将 AcceptanceWorld 从简单的 state holder 改为持有真实系统实例：

```rust
pub struct AcceptanceWorld {
    // 真实系统实例
    event_log: SqliteEventLog,
    collector_manager: CollectorManager,
    scheduler: Scheduler,
    task_manager: TaskManager,
    classifier: Classifier,
    pipeline: ProcessingPipeline,

    // 测试状态
    event_type: Option<String>,
    processing_result: Option<String>,
    last_error: Option<String>,
    // ...
}
```

### 迁移优先级

根据 [功能重评估报告](../features/reevaluation-summary.md)，优先迁移核心路径：

#### P0 核心路径（G1 → G2 → G3）

1. **G1 信息采集**（37 个场景）
   - 优先：F1.3.1 文本输入（✅ 已完成）
   - 优先：F1.1.1 消息采集（✅ 已完成）
   - 其他：F1.1.2~F1.1.12（🟡 开发中）

2. **G2 智能处理**（34 个场景）
   - 优先：F2.1.1 分类器（✅ 已完成）
   - 优先：F2.1.2 即时处理路由（✅ 已完成）
   - 优先：F2.1.5 直接归档路由（✅ 已完成）

3. **G3 数据存储**（28 个场景）
   - 优先：F3.1.1 日记生成（🧪 集成验证）
   - 优先：F3.1.2 项目目录（🧪 集成验证）
   - 优先：F3.1.5 模板系统（🧪 集成验证）

#### P1 重要功能（G4 → G6）

4. **G4 任务管理**（20 个场景）
   - 优先：F4.1.1 任务创建（✅ 已完成）
   - 优先：F4.1.2 状态流转（✅ 已完成）

5. **G6 系统能力**（38 个场景）
   - 优先：F6.2.6 全局暂停/恢复（✅ 已完成）
   - 优先：F6.1.4 菜单栏常驻（🧪 集成验证）

#### P2 辅助功能（G5 → G7）

6. **G5 报告生成**（12 个场景）
   - 全部为 🟡 开发中，优先级最低

7. **G7 跨切面**（13 个场景）
   - 依赖其他模块完成

## 实施计划

### 阶段一：改造 AcceptanceWorld（2 小时）

- [ ] 修改 AcceptanceWorld 结构，添加真实系统实例
- [ ] 实现 World 初始化逻辑（创建内存数据库、注册采集器等）
- [ ] 实现 World 清理逻辑（测试结束后清理资源）

### 阶段二：迁移 G1 采集步骤（4 小时）

- [ ] 修改 Given 步骤：创建真实的事件源
- [ ] 修改 When 步骤：调用真实的采集器
- [ ] 修改 Then 步骤：查询 EventLog 验证结果
- [ ] 运行 G1 feature 文件，验证 37 个场景

### 阶段三：迁移 G2 处理步骤（4 小时）

- [ ] 修改 Given 步骤：创建真实的事件
- [ ] 修改 When 步骤：调用真实的 Classifier 和 Pipeline
- [ ] 修改 Then 步骤：查询处理结果验证
- [ ] 运行 G2 feature 文件，验证 34 个场景

### 阶段四：迁移 G3 存储步骤（4 小时）

- [ ] 修改 Given 步骤：创建真实的 WorkRecord
- [ ] 修改 When 步骤：调用真实的 ObsidianWriter
- [ ] 修改 Then 步骤：验证文件系统输出
- [ ] 运行 G3 feature 文件，验证 28 个场景

### 阶段五：迁移 G4/G6 步骤（4 小时）

- [ ] 迁移 G4 任务管理步骤
- [ ] 迁移 G6 系统能力步骤
- [ ] 运行验证

### 阶段六：迁移 G5/G7 步骤（2 小时）

- [ ] 迁移 G5 报告生成步骤
- [ ] 迁移 G7 跨切面步骤
- [ ] 运行验证

## 验收标准

- [ ] 182 个验收场景全部可执行
- [ ] 所有 Then 步骤有真实的断言
- [ ] 核心路径（G1→G2→G3）场景全部通过
- [ ] 测试覆盖率达到 80%

## 相关文档

| 文档 | 关系 |
|------|------|
| [测试有效性审计报告](test-effectiveness-audit.md) | 本文档基于的审计发现 |
| [功能重评估报告](../features/reevaluation-summary.md) | 功能完成状态 |
| [测试体系总体架构](architecture.md) | 测试层级定义 |
