---
title: F1/F5 模块补全计划
type: structural
domain: testing
created: 2026-06-11
status: active
---

# F1/F5 模块补全计划

> **维护说明**：当完成阶段性修复后更新本文档的对应章节和状态标记。

## 补全目标

### F1 信息采集（当前 38% → 目标 80%）

需要补全 13 个功能：
- F1.1.2~F1.1.12：飞书采集器（11 个）
- F1.3.2~F1.3.3：手动采集器（2 个）

### F5 报告生成（当前 45% → 目标 80%）

需要补全 6 个功能：
- F5.1.1~F5.1.6：报告类型生成（6 个）

## 补全策略

### F1 飞书采集器

**当前状态**：仅有 L1 单元测试（convert 函数测试）

**补全方案**：
1. 为每个采集器补充 L2 集成测试
2. 使用 httpmock 模拟飞书 API 响应
3. 验证完整的采集链路：调用 lark-cli → 解析响应 → 生成 Event

**测试结构**：
```rust
#[tokio::test]
async fn test_docs_collector_l2() {
    // 1. Mock 飞书 API 响应
    let mock_server = MockServer::start();
    mock_server.mock(|when, then| {
        when.method(POST).path("/docs/search");
        then.status(200).json_body(json!({
            "data": {
                "results": [{
                    "entity_type": "docx",
                    "result_meta": {"token": "doc-001", "owner_name": "user-001"},
                    "title_highlighted": "设计文档"
                }]
            }
        }));
    });

    // 2. 调用采集器
    let events = FeishuDocsCollector::collect(10).unwrap();

    // 3. 验证结果
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].source, Source::FeishuDoc);
}
```

### F5 报告生成

**当前状态**：仅有 L1 单元测试（数据聚合测试）

**补全方案**：
1. 为每种报告类型补充 L2 集成测试
2. 验证完整的报告生成链路：获取事件 → 聚合数据 → 生成 Markdown
3. 将 ReportGenerator 接入前端和调度器

**测试结构**：
```rust
#[tokio::test]
async fn test_daily_report_l2() {
    // 1. 准备测试数据
    let event_log = SqliteEventLog::new_in_memory().unwrap();
    // 添加测试事件...

    // 2. 生成报告
    let generator = ReportGenerator::new(event_log);
    let report = generator.generate_daily_report().await.unwrap();

    // 3. 验证报告内容
    assert!(report.contains("今日完成事项"));
    assert!(report.contains("明日计划"));
}
```

## 实施计划

### 阶段一：F1 飞书采集器 L2 测试（4 小时）

- [ ] F1.1.2 文档采集
- [ ] F1.1.3 项目采集
- [ ] F1.1.4 日历采集
- [ ] F1.1.5 会议采集
- [ ] F1.1.6 邮箱采集
- [ ] F1.1.7 审批采集
- [ ] F1.1.8 OKR 采集
- [ ] F1.1.9 多维表格采集
- [ ] F1.1.10 电子表格采集
- [ ] F1.1.11 知识库采集
- [ ] F1.1.12 妙记采集

### 阶段二：F1 手动采集器补全（2 小时）

- [ ] F1.3.2 图片粘贴
- [ ] F1.3.3 截图捕获

### 阶段三：F5 报告生成 L2 测试（4 小时）

- [ ] F5.1.1 日报生成
- [ ] F5.1.2 周报生成
- [ ] F5.1.3 月报生成
- [ ] F5.1.4 季报生成
- [ ] F5.1.5 半年报生成
- [ ] F5.1.6 年报生成

### 阶段四：集成验证（2 小时）

- [ ] 运行完整测试套件
- [ ] 更新功能索引状态
- [ ] 创建补全总结报告

## 验收标准

- [ ] F1 模块完成度达到 80%（17/21 功能）
- [ ] F5 模块完成度达到 80%（9/11 功能）
- [ ] 所有新增测试通过
- [ ] 测试覆盖率达到 80%

## 相关文档

| 文档 | 关系 |
|------|------|
| [测试有效性审计报告](test-effectiveness-audit.md) | 本文档基于的审计发现 |
| [功能重评估报告](../features/reevaluation-summary.md) | 功能完成状态 |
| [测试体系总体架构](architecture.md) | 测试层级定义 |
