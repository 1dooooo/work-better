@G4
Feature: G4 任务管理 (20 scenarios)
  产品文档: F4.1-F4.3 任务管理

  Scenario: G4-01 手动创建任务
    Given 用户手动创建任务
    When 保存
    Then Task(status=todo, source=obsidian)

  Scenario: G4-02 AI 从会议提取任务
    Given 系统从会议发现任务
    When AI 提取
    Then needs_review=true

  Scenario: G4-03 飞书任务同步
    Given 飞书项目任务变更
    When 同步
    Then Obsidian 更新 source=feishu

  Scenario: G4-04 todo→in_progress 合法
    Given todo
    When todo→in_progress
    Then 合法并持久化

  Scenario: G4-05 →blocked 合法
    Given in_progress
    When →blocked
    Then 合法

  Scenario: G4-06 blocked→in_progress 合法
    Given blocked
    When →in_progress
    Then 合法

  Scenario: G4-07 →cancelled 合法
    Given in_progress
    When →cancelled
    Then 合法

  Scenario: G4-08 todo→done 非法
    Given todo
    When 直接→done
    Then 拒绝并解释

  Scenario: G4-09 done→in_progress 非法
    Given done
    When →in_progress
    Then 拒绝

  Scenario: G4-10 blocked→done 非法
    Given blocked
    When 直接→done
    Then 拒绝

  Scenario: G4-11 标记 done 设置 completed_at
    Given 标记 done
    When 完成
    Then 设置 completed_at

  Scenario: G4-12 子任务关联
    Given 有子任务
    When 完成
    Then 通过 parent_task 关联

  Scenario: G4-13 会议待办识别
    Given 会议结束有待办
    When 分析
    Then 识别并创建 needs_review 任务

  Scenario: G4-14 聊天承诺识别
    Given 聊天消息含承诺
    When 分析
    Then 识别并创建 needs_review 任务

  Scenario: G4-15 邮件请求识别
    Given 邮件含请求
    When 分析
    Then 识别并创建 needs_review 任务

  Scenario: G4-16 文档评论待办
    Given 文档评论含待办
    When 分析
    Then 识别并创建 needs_review 任务

  Scenario: G4-17 自动发现任务需确认
    Given 自动发现的任务
    When 呈现
    Then 必须确认或拒绝才激活

  Scenario: G4-18 飞书状态变更自动同步
    Given 飞书任务状态变更
    When 捕获
    Then Obsidian 自动更新(无需确认)

  Scenario: G4-19 Obsidian→飞书需确认
    Given Obsidian 任务修改
    When 同步飞书
    Then 需用户确认

  Scenario: G4-20 冲突解决机制
    Given 两端同时修改
    When 检测冲突
    Then 呈现解决机制
