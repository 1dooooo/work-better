@G3
Feature: G3 数据存储 (28 scenarios)
  产品文档: F3.1-F3.6 三层存储体系

  Scenario: G3-01 WorkRecord 写入正确目录
    Given WorkRecord 持久化
    When 写入 Obsidian
    Then 放入正确目录

  Scenario: G3-02 自动创建双向链接
    Given 引用项目/人/实体
    When 写入
    Then 自动创建双向链接

  Scenario: G3-03 自动应用标签
    Given 被分类
    When 写入
    Then 自动应用标签

  Scenario: G3-04 多维度访问
    Given 多上下文
    When 查看任一位置
    Then 不同维度可访问

  Scenario: G3-05 自定义模板
    Given 自定义模板
    When 配置
    Then 新文件遵循模板

  Scenario: G3-06 异步生成嵌入
    Given 新文档写入
    When 成功
    Then 异步生成嵌入

  Scenario: G3-07 修改后5分钟重新嵌入(防抖)
    Given 文档修改
    When 检测
    Then 5分钟后重新嵌入(防抖)

  Scenario: G3-08 删除后嵌入移除
    Given 文档删除
    When 检测
    Then 嵌入移除

  Scenario: G3-09 语义搜索相似度排序
    Given 语义搜索
    When 执行
    Then 按相似度排序返回

  Scenario: G3-10 RAG 召回相关文档
    Given 大模型需上下文
    When RAG 召回
    Then 检索相关文档

  Scenario: G3-11 结构化数据索引查询
    Given 结构化数据存在
    When 查询
    Then 按索引字段快速查询

  Scenario: G3-12 任务状态变更历史跟踪
    Given 任务状态变更
    When 更新
    Then 跟踪完整转换历史

  Scenario: G3-13 三层写入顺序
    Given WorkRecord 持久化
    When 完成顺序
    Then 顺序: Obsidian→向量DB→结构化DB

  Scenario: G3-14 Obsidian 编辑触发 DB 更新
    Given 用户在 Obsidian 编辑
    When 保存
    Then 向量DB和结构化DB更新

  Scenario: G3-15 一致性检查差异修复
    Given 一致性检查(每周)
    When 发现差异
    Then 标记并触发重建

  Scenario: G3-16 向量DB 数≠文档数
    Given 向量DB数≠文档数
    When 检查
    Then 标记不匹配

  Scenario: G3-17 飞书任务完成 Obsidian 更新
    Given 飞书任务标记完成
    When 新鲜度比对
    Then Obsidian 更新为完成

  Scenario: G3-18 飞书文档过时重生成摘要
    Given 飞书文档已更新
    When 每天检查
    Then 检测过时并重新生成摘要

  Scenario: G3-19 断链检测
    Given 双向链接指向已删除文件
    When 每天检查
    Then 标记断链

  Scenario: G3-20 标签命名规范化
    Given 标签命名不一致
    When 每周规范化
    Then 合并变体

  Scenario: G3-21 重复信息检测
    Given 信息多次记录
    When 每周检测
    Then 标记合并候选

  Scenario: G3-22 过时知识审查
    Given 知识已过时
    When 每月审查
    Then 标记需用户审查

  Scenario: G3-23 检查完成推送通知
    Given 检查完成+有需注意项
    When 完成顺序
    Then 存储推送通知

  Scenario: G3-24 检查完成静默修复
    Given 检查完成+有可修复项
    When 完成顺序
    Then 静默修复

  Scenario: G3-25 新鲜度报告
    Given 检查完成
    When 执行完毕
    Then 生成新鲜度报告

  Scenario: G3-26 重建向量DB
    Given 用户触发重建向量DB
    When 执行
    Then 所有文档重新嵌入

  Scenario: G3-27 重处理历史事件
    Given 用户触发重处理历史
    When 执行
    Then 所有事件重新处理

  Scenario: G3-28 全量一致性检查
    Given 用户触发全量一致性检查
    When 执行
    Then 三层互相验证
