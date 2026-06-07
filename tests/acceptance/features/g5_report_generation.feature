@G5
Feature: G5 报告生成 (12 scenarios)
  产品文档: F5.1-F5.2 报告生成

  Scenario: G5-01 日报生成
    Given 工作日18:00
    When 日报触发
    Then 生成含完成/计划/阻塞的日报

  Scenario: G5-02 周报生成
    Given 周五17:00
    When 周报触发
    Then 生成含进展/成果/计划/风险的周报

  Scenario: G5-03 月报生成
    Given 月末
    When 月报触发
    Then 生成含目标/时间/效率的月报

  Scenario: G5-04 季报生成
    Given 季末
    When 季报触发
    Then 生成含 OKR/里程碑/能力的季报

  Scenario: G5-05 半年报/年报
    Given 12/31
    When 半年报/年报触发
    Then 生成对应报告

  Scenario: G5-06 月末+季末同时触发
    Given 月末同时季末
    When 两者同时到期
    Then 各自按 SLA 生成

  Scenario: G5-07 完成后通知审查
    Given 报告生成完成
    When 完毕
    Then 通知用户审查确认

  Scenario: G5-08 审查编辑后为最终版
    Given 用户审查编辑
    When 完成
    Then 编辑版本为最终版本

  Scenario: G5-09 导出 Markdown/PDF
    Given 用户确认
    When 选择导出
    Then 可导出 Markdown 或 PDF

  Scenario: G5-10 同步飞书文档
    Given 用户确认
    When 选择同步飞书
    Then 推送到飞书文档

  Scenario: G5-11 自定义报告模板
    Given 用户自定义格式
    When 修改模板
    Then 后续遵循模板

  Scenario: G5-12 自定义生成时间
    Given 用户改生成时间
    When 在新时间生成
    Then 后续在新时间生成
