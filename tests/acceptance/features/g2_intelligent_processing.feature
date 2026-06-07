@G2
Feature: G2 智能处理 (34 scenarios)
  产品文档: F2.1-F2.5 智能处理层

  # ── 分类路由 ─────────────────────────────────────
  Scenario: G2-01 P0/P1 事件即时处理
    Given task_update 事件
    When 分类
    Then 即时处理

  Scenario: G2-02 @mention/手动笔记即时处理
    Given message @mention 事件
    When 分类
    Then 即时处理

  Scenario: G2-03 一般消息聚合处理
    Given message 一般 事件
    When 分类
    Then 聚合处理

  Scenario: G2-04 长期分析事件模式分析
    Given 需要长期分析的事件
    When 分类
    Then 模式分析

  Scenario: G2-05 低置信度直接归档
    Given 低置信度 事件
    When 分类
    Then 直接归档

  # ── 模型升级 ─────────────────────────────────────
  Scenario: G2-06 实体提取低置信度升级
    Given 实体提取+小模型置信度<0.7
    When 处理
    Then 升级大模型

  Scenario: G2-07 任务识别低置信度升级
    Given 任务识别+置信度<0.6
    When 处理
    Then 升级大模型

  Scenario: G2-08 摘要生成低置信度升级
    Given 摘要生成+置信度<0.6 或>500字
    When 处理
    Then 升级大模型

  Scenario: G2-09 情感判断低置信度升级
    Given 情感判断+置信度<0.8
    When 处理
    Then 升级大模型

  Scenario: G2-10 关联分析低置信度升级
    Given 关联分析+置信度<0.7
    When 处理
    Then 升级大模型

  Scenario: G2-11 模式识别直接用大模型
    Given 模式识别任务
    When 处理
    Then 直接用大模型

  Scenario: G2-12 小模型达标进入 ReviewAgent
    Given 小模型置信度达标
    When 完成
    Then 进入 ReviewAgent

  Scenario: G2-13 双模型失败标记手动处理
    Given 小模型失败+大模型也失败
    When 处理
    Then 标记"需手动处理"并通知

  # ── 预算管理 ─────────────────────────────────────
  Scenario: G2-14 日预算可用时正常调用
    Given 日预算未耗尽
    When 需大模型
    Then 可用

  Scenario: G2-15 日预算耗尽非紧急排队
    Given 日预算耗尽+非紧急
    When 需大模型
    Then 排队明天

  Scenario: G2-16 日预算耗尽紧急溢出
    Given 日预算耗尽+紧急(P0/P1)
    When 需大模型
    Then 允许溢出并通知

  Scenario: G2-17 预算耗尽降级小模型
    Given 日预算耗尽+策略 degrade_to_small
    When 需大模型
    Then 小模型降级

  Scenario: G2-18 Token 使用审计可见
    Given Token 使用被跟踪
    When 查看审计
    Then 可见每日消耗

  # ── SLA 管理 ─────────────────────────────────────
  Scenario: G2-19 P0 超5分钟强制升级
    Given P0 事件+超5分钟未处理
    When 超时
    Then 强制升级并通知

  Scenario: G2-20 P1 超30分钟升级大模型
    Given P1 事件+超30分钟未处理
    When 超时
    Then 升级大模型

  Scenario: G2-21 P2 超4小时继续正常
    Given P2 事件+超4小时未处理
    When 超时
    Then 继续正常流程

  Scenario: G2-22 P3 排入每日批处理
    Given P3 事件
    When 未处理
    Then 排入每日批处理

  Scenario: G2-23 SLA 扫描自动提升
    Given 处理队列
    When SLA 扫描
    Then 超时事件自动提升

  Scenario: G2-24 SLA 日报效率统计
    Given 一天结束
    When SLA 报告生成
    Then 显示效率统计

  # ── ReviewAgent ──────────────────────────────────
  Scenario: G2-25 直接归档输出直接通过
    Given 直接归档输出
    When 返回
    Then 直接通过

  Scenario: G2-26 低置信度输出仅规则验证
    Given 低置信度提取输出
    When 返回
    Then 仅规则层验证

  Scenario: G2-27 高置信度输出抽样审查
    Given 高置信度提取输出
    When 返回
    Then 10%抽样审查

  Scenario: G2-28 任务状态变更需确认
    Given 任务状态变更输出
    When 返回
    Then 规则验证+共享数据需确认

  Scenario: G2-29 报告/摘要一致性检查
    Given 报告/摘要输出
    When 返回
    Then 规则+小模型一致性检查

  Scenario: G2-30 涉及他人信息需用户确认
    Given 涉及他人信息输出
    When 返回
    Then 规则+小模型+用户确认

  Scenario: G2-31 needs_fix 回处理层
    Given ReviewAgent needs_fix
    When 返回
    Then 回处理层重新处理

  Scenario: G2-32 needs_review 推送通知
    Given ReviewAgent needs_review
    When 返回
    Then 推送通知

  Scenario: G2-33 approved 进入存储层
    Given ReviewAgent approved
    When 返回
    Then 进入存储层

  Scenario: G2-34 同类问题频繁自动调整
    Given 同类问题频繁
    When 阈值达到
    Then 自动调整提示或阈值
