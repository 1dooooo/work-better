@G1
Feature: G1 信息采集 (37 scenarios)
  产品文档: F1.1-F1.4 信息采集层

  # ── 飞书消息 ─────────────────────────────────────
  Scenario: G1-01 飞书消息@提及用户捕获
    Given 飞书消息@提及用户
    When 消息到达
    Then 捕获为 message 且 confidence 为 high

  Scenario: G1-02 飞书消息回复用户参与的线程
    Given 飞书消息是回复用户参与的线程
    When 消息到达
    Then 捕获并关联线程

  Scenario: G1-03 飞书私信捕获
    Given 飞书私信
    When 消息到达
    Then 捕获为 message

  Scenario: G1-04 关键词规则匹配捕获
    Given 消息匹配关键词规则
    When 评估
    Then 捕获为 message

  Scenario: G1-05 无关消息不捕获
    Given 消息与用户无关
    When 评估
    Then 不捕获

  # ── 飞书文档 ─────────────────────────────────────
  Scenario: G1-06 飞书文档变更捕获
    Given 飞书文档被用户创建/编辑/评论
    When 检测
    Then 捕获 document_change

  Scenario: G1-07 文档中用户被提及
    Given 文档中用户被提及
    When 检测
    Then 捕获 document_change

  # ── 项目任务 ─────────────────────────────────────
  Scenario: G1-08 飞书项目任务变更
    Given 飞书项目任务变更
    When 捕获
    Then 捕获 task_update

  # ── 日历 ─────────────────────────────────────────
  Scenario: G1-09 日历事件同步
    Given 日历有即将到来事件
    When 每小时同步
    Then 捕获 calendar_event

  # ── 会议 ─────────────────────────────────────────
  Scenario: G1-10 视频会议结束捕获
    Given 用户参加视频会议
    When 结束
    Then 捕获 meeting

  Scenario: G1-11 妙记摘要捕获
    Given 飞书妙记有录制摘要
    When 结束
    Then 捕获摘要/待办/章节

  # ── 邮件 ─────────────────────────────────────────
  Scenario: G1-12 飞书邮件同步
    Given 用户通过飞书邮件操作
    When 每30分钟同步
    Then 捕获 email

  # ── 审批 ─────────────────────────────────────────
  Scenario: G1-13 审批状态变更
    Given 飞书审批状态变更
    When 检测
    Then 捕获 approval

  # ── OKR ──────────────────────────────────────────
  Scenario: G1-14 OKR 同步
    Given 用户有 OKR
    When 每天同步
    Then 捕获 okr_update

  # ── 多维表格/电子表格/知识库 ─────────────────────
  Scenario: G1-15 多维表格记录变更
    Given 多维表格记录变更
    When 检测
    Then 捕获 document_change

  Scenario: G1-16 电子表格单元格变更
    Given 电子表格单元格变更
    When 检测
    Then 捕获 document_change

  Scenario: G1-17 知识库节点变更
    Given 知识库节点变更
    When 检测
    Then 捕获 document_change

  # ── 应用活动 ─────────────────────────────────────
  Scenario: G1-18 应用切换停留>30秒记录
    Given 用户切换应用停留>30秒
    When 检测
    Then 记录 app_activity

  Scenario: G1-19 应用切换停留<30秒防抖
    Given 用户切换应用停留<30秒
    When 检测
    Then 不记录

  # ── 浏览记录 ─────────────────────────────────────
  Scenario: G1-20 非搜索页 URL 记录
    Given 用户访问非搜索页 URL
    When 检测
    Then 记录 browsing

  Scenario: G1-21 搜索结果页不记录
    Given 用户访问搜索结果页
    When 检测
    Then 不记录

  # ── 快捷记录 ─────────────────────────────────────
  Scenario: G1-22 全局快捷键聚焦输入
    Given 用户按全局快捷键
    When 窗口打开
    Then 聚焦输入区

  Scenario: G1-23 手动输入创建 manual_note
    Given 窗口打开
    When 输入并提交
    Then 创建 manual_note

  Scenario: G1-24 粘贴图片附件
    Given 窗口打开
    When 粘贴图片
    Then 接受为附件

  Scenario: G1-25 拖放文件附件
    Given 窗口打开
    When 拖放文件
    Then 接受为附件

  Scenario: G1-26 截图预载
    Given 用户按截图键
    When 截图完成
    Then 打开窗口并预载截图

  Scenario: G1-27 提交后窗口自动隐藏
    Given 窗口打开
    When 提交完成
    Then 窗口自动隐藏

  # ── 收集器管理 ───────────────────────────────────
  Scenario: G1-28 收集器禁用级联
    Given 收集器运行中
    When 禁用
    Then 停止且子收集器也禁用

  Scenario: G1-29 父收集器重启用恢复子状态
    Given 父收集器禁用
    When 重新启用
    Then 子收集器恢复各自状态

  Scenario: G1-30 收集器故障自动禁用
    Given 收集器故障
    When 健康检查
    Then 自动禁用并通知

  Scenario: G1-31 unhealthy 状态显示
    Given 健康状态 unhealthy
    When 查看
    Then 显示错误指示

  Scenario: G1-32 运行时注册新收集器
    Given 运行时注册新收集器
    When 注册
    Then 立即开始采集

  # ── 飞书连接配置 ─────────────────────────────────
  Scenario: G1-33 飞书 API 模式
    Given 配置飞书连接
    When 选 API
    Then 用飞书开放平台 API

  Scenario: G1-34 飞书 CLI 降级
    Given 配置飞书连接
    When 选 CLI
    Then 用 lark-cli 降级

  # ── 事件可靠性 ───────────────────────────────────
  Scenario: G1-35 多事件严格时间顺序
    Given 多事件同时到达
    When 写入 EventLog
    Then 严格时间顺序

  Scenario: G1-36 系统重启事件恢复
    Given 系统重启未处理事件
    When 有未处理事件
    Then 事件恢复(无数据丢失)

  Scenario: G1-37 处理逻辑变更重放
    Given 处理逻辑变更
    When 触发重放
    Then 历史事件重新处理
