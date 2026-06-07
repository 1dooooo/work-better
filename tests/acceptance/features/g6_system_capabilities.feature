@G6
Feature: G6 系统能力 (38 scenarios)
  产品文档: F6.1-F6.3 系统能力层

  # ── 快捷键 ───────────────────────────────────────
  Scenario: G6-01 Cmd+Shift+Space 打开快捷记录
    Given 用户在任何应用
    When 按 Cmd+Shift+Space
    Then 快捷记录窗口出现

  Scenario: G6-02 再按快捷键隐藏窗口
    Given 窗口可见
    When 再按快捷键
    Then 窗口隐藏

  Scenario: G6-03 截图键打开截图窗口
    Given 用户在任何应用
    When 按截图键
    Then 截图并打开窗口

  # ── 通知 ─────────────────────────────────────────
  Scenario: G6-04 确认项通知
    Given 有待确认项
    When 通知触发
    Then 显示通知含描述

  Scenario: G6-05 常规信息轻提醒
    Given 有常规信息
    When 轻提醒触发
    Then 非侵入通知

  # ── 菜单栏 ───────────────────────────────────────
  Scenario: G6-06 菜单栏显示状态
    Given 用户查看状态
    When 点击菜单栏
    Then 显示待确认/任务/摘要等

  Scenario: G6-07 两次点击快速操作
    Given 用户要快速操作
    When 交互菜单栏
    Then 两次点击内完成

  Scenario: G6-08 通知中心一屏可见
    Given 打开菜单栏
    When 查看通知中心
    Then 一屏可见所有可操作项

  Scenario: G6-09 复杂操作重定向主窗口
    Given 需要复杂操作
    When 从菜单栏选择
    Then 重定向主窗口

  # ── 主窗口 ───────────────────────────────────────
  Scenario: G6-10 时间线视图
    Given 打开主窗口
    When 查看时间线
    Then 时间轴+缩放+过滤

  Scenario: G6-11 时间线项详情链接
    Given 点击时间线项
    When 展开详情
    Then 有 Obsidian 原文链接

  Scenario: G6-12 任务板分组
    Given 打开主窗口
    When 查看任务板
    Then 按状态列分组

  Scenario: G6-13 拖拽任务卡片
    Given 拖拽任务卡片
    When 拖到不同列
    Then 状态更新并同步

  Scenario: G6-14 双路搜索
    Given 用户搜索
    When 执行
    Then RAG+结构化双路搜索

  Scenario: G6-15 数据探索统计
    Given 打开主窗口
    When 查看数据探索
    Then 时间/任务/会议/模式图表

  Scenario: G6-16 搜索结果打开原文
    Given 搜索结果
    When 点击
    Then 打开 Obsidian 原文

  # ── 设置 ─────────────────────────────────────────
  Scenario: G6-17 模型配置
    Given 打开设置
    When 配置模型
    Then API 端点/参数/预算

  Scenario: G6-18 收集器配置
    Given 打开设置
    When 配置收集器
    Then 飞书凭据/开关

  Scenario: G6-19 存储配置
    Given 打开设置
    When 配置存储
    Then Obsidian 路径/向量DB/备份

  Scenario: G6-20 快捷键配置
    Given 打开设置
    When 配置快捷键
    Then 自定义组合键

  Scenario: G6-21 新鲜度规则配置
    Given 打开设置
    When 配置新鲜度规则
    Then 频率和策略

  Scenario: G6-22 审计查询
    Given 打开设置
    When 查看审计
    Then 按维度查询并导出

  # ── 调度器 ───────────────────────────────────────
  Scenario: G6-23 cron 偏移窗口执行
    Given 调度器运行中
    When cron 触发
    Then 在偏移窗口内执行

  Scenario: G6-24 依赖任务
    Given A 依赖 B
    When B 未完成
    Then A 不启动

  Scenario: G6-25 超 SLA 自动终止
    Given 任务超 SLA
    When 超时检测
    Then 自动终止

  Scenario: G6-26 失败重试递增
    Given 任务失败
    When 失败
    Then 重试最多3次递增间隔

  Scenario: G6-27 重试全失败标记 failed
    Given 3次重试全失败
    When 最终失败
    Then 标记 failed

  Scenario: G6-28 预算不足推迟低优先级
    Given 日预算不足
    When 低优先级需执行
    Then 推迟

  Scenario: G6-29 暂停所有定时任务
    Given 用户激活暂停
    When 触发
    Then 所有定时任务停止

  Scenario: G6-30 紧急停止
    Given 用户激活紧急停止
    When 触发
    Then 执行中任务立即终止

  Scenario: G6-31 恢复后积压执行
    Given 暂停中
    When 恢复
    Then 积压任务按优先级执行

  Scenario: G6-32 调度 UI 任务列表
    Given 查看调度 UI
    When 检查任务
    Then 显示 ID/名称/计划/状态/开关

  Scenario: G6-33 执行日志详情
    Given 查看执行日志
    When 检查
    Then 显示状态/时长/摘要/错误/重试

  Scenario: G6-34 同类任务不并行
    Given 同类任务执行中
    When 另一个触发
    Then 不并行

  Scenario: G6-35 采集层整点后执行
    Given 采集层定时任务
    When 触发
    Then 整点后0-5分钟执行

  Scenario: G6-36 处理层延迟执行
    Given 处理层定时任务
    When 触发
    Then 整点后5-30分钟执行

  Scenario: G6-37 存储层低峰期执行
    Given 存储层定时任务
    When 触发
    Then 低峰期(02:00-05:00)执行

  Scenario: G6-38 报告层用户配置时间
    Given 报告层定时任务
    When 触发
    Then 用户配置时间执行
