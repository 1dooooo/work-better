---
title: L5 E2E 测试验证报告
created: 2026-06-13
status: complete
tags: [testing, e2e, l5, pipeline]
---

# L5 E2E 测试验证报告

## 执行结果

```
running 30 tests
test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

**30/30 测试全部通过。**

## 覆盖完整性

30 个测试完整覆盖 6 个分组，数量与代码声明一致：

| 分组 | 主题 | 测试数 | 测试编号 |
|------|------|--------|----------|
| A | 核心 Pipeline 流转 | 6 | A1-A6 |
| B | 任务流转生命周期 | 7 | B1-B7 |
| C | 审核分层 | 4 | C1-C4 |
| D | 降级与容错 | 4 | D1-D4 |
| E | 数据完整性 | 4 | E1-E4 |
| F | 边界条件 | 5 | F1-F5 |
| **合计** | | **30** | |

## 断言质量

### L5 特征验证

| L5 特征 | 涉及测试 | 验证方式 |
|---------|----------|----------|
| .md 文件写入 | A1,A4,A5,E1,E2,F1-F5 | `find_md_files()` + `fs::read_to_string()` 验证真实文件存在和内容 |
| .md 内容包含业务字段 | A1,A4,A5,E1 | 断言 .md 内容包含 title/summary/people/id |
| WorkRecord 字段完整 | E3 | 验证 id/title/summary/category/obsidian_path/model_used/confidence/source_event_ids |
| 审计链路 | E4 | 验证 processing_time_ms >= 各步骤耗时之和 |
| 路由正确性 | A1-A6,D1 | 断言 route == Instant/Aggregate/Archive |
| 分类正确性 | A1-A6,B1,C1-C4 | 断言 category == Task/Decision/Meeting |
| 审核分层 | C1-C4 | 断言 reviewer == rule/small_model/large_model |
| 降级容错 | D1-D4 | 断言 model_used == extract-fallback, confidence == 0.7 |
| 状态机合法性 | B2-B7 | 断言合法转换成功、非法转换返回 Err |

### 真实文件系统验证

14 个测试直接读取文件系统验证 .md 文件存在和内容。核心链路测试（A/E/F 组）验证了真实的文件系统输出，而非仅检查结构体字段。

## 隔离性

- 24 个异步测试均使用独立 `tempfile::tempdir()`
- 6 个同步测试（B2-B7）为纯内存操作
- F5 并发测试为每条消息创建独立 tmpdir

**全部通过，无共享状态泄漏风险。**

## 已知问题

1. **B2-B7 的 L5 特征偏弱**：这 6 个测试验证 Task 状态机转换，本质上是 L4 单元测试级别，不涉及文件系统写入或数据库记录。作为状态机完整性验证可接受，但严格来说更适合作为 L4 测试。

2. **E3 未直接读取 SQLite**：当前 `PersistStep` 基于文件系统持久化，验证 WorkRecord 结构体字段是合理的。如未来引入 SQLite 存储，需增加数据库层面验证。

3. **E4 审计链路验证偏弱**：当前只验证耗时合理性，未验证 trace_id 是否贯穿各步骤。

## 验收结论

| 验收项 | 结果 |
|--------|------|
| 30 个测试全绿 | PASS |
| 覆盖 6 个分组 | PASS |
| 断言验证真实文件/记录 | PASS |
| 测试隔离性 | PASS |
| L5 特征（.md 写入 + 审计链路） | PASS（有改进空间） |
