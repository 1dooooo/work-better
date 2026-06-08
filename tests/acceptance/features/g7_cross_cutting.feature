@G7
Feature: G7 横切关注 (13 scenarios)
  产品文档: F2.4.5/F2.5/F3.4 横切关注点

  Scenario: G7-01 AI 处理个人数据自主执行
    Given AI 处理个人数据
    When 数据处理
    Then 自主执行无需确认

  Scenario: G7-02 AI 修改共享数据需确认
    Given AI 要修改共享数据
    When 即将执行
    Then 必须用户确认

  Scenario: G7-03 用户确认后执行同步
    Given 用户确认共享操作
    When 确认
    Then 执行并同步飞书

  Scenario: G7-04 事件采集不可变记录
    Given 事件被采集
    When 进入系统
    Then EventLog 不可变记录

  Scenario: G7-05 事件消费标记 processed
    Given 事件被消费
    When 消费完成
    Then 标记为 processed

  Scenario: G7-06 三层写入顺序
    Given WorkRecord 产出
    When 三层写入
    Then 三层写入完成

  Scenario: G7-07 表示层联合查询
    Given 表示层读取
    When 审计查询
    Then 三层联合查询接口

  Scenario: G7-08 Obsidian 编辑双 DB 一致
    Given 用户在 Obsidian 编辑
    When 编辑保存
    Then 两 DB 更新保持一致

  Scenario: G7-09 处理审计记录
    Given 事件进入处理
    When 每步执行
    Then 生成 ProcessingAudit

  Scenario: G7-10 trace_id 完整链路
    Given 同一事件有审计记录
    When 审计查看
    Then trace_id 链接完整链路

  Scenario: G7-11 多维度审计查询
    Given 审计数据存在
    When 审计查询
    Then 可按多维度查询

  Scenario: G7-12 月度审计聚合
    Given 审计数据积累
    When 月度聚合
    Then 聚合为统计摘要

  Scenario: G7-13 模式检测产生建议
    Given 检测到模式(同类错误频繁)
    When 生成建议
    Then 产生改进建议
