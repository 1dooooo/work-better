---
title: 事件模型
type: structural
domain: architecture
created: 2026-06-06
updated: 2026-06-06
status: active
---

# 事件模型

> **维护说明**：当数据结构字段变更时更新本文档。事件模型是所有模块的基础，变更影响面大，需同步更新相关模块文档。

## 概述

事件模型定义了系统中所有核心数据结构。事件是系统的原子单位，所有信息以事件形式进入系统。

## Event（事件）

事件是不可变的，一旦写入 EventLog 不可修改。

```
Event {
  // 标识
  id: string              // 唯一标识，UUID v4

  // 时间
  timestamp: datetime     // 事件发生的精确时间
  collected_at: datetime  // 采集时间（可能晚于发生时间）

  // 来源
  source: Source          // 来源枚举
  source_confidence: Confidence  // 来源置信度

  // 内容
  type: EventType         // 事件类型枚举
  content: object         // 结构化内容（schema 因 type 而异）
  raw_payload: string     // 原始数据（JSON 序列化，用于重新处理）

  // 元数据
  tags: string[]          // 自动或手动标签
  related_ids: string[]   // 关联事件 ID（形成上下文链）
  attachments: Attachment[]  // 附件（图片、文件）
}
```

### Source 枚举

```
Source {
  feishu_message        // 飞书消息
  feishu_doc            // 飞书文档
  feishu_project        // 飞书项目
  feishu_calendar       // 飞书日历
  feishu_meeting        // 飞书视频会议/妙记
  feishu_email          // 飞书邮箱
  feishu_approval       // 飞书审批
  feishu_okr            // 飞书 OKR
  feishu_bitable        // 飞书多维表格
  feishu_sheet          // 飞书电子表格
  feishu_wiki           // 飞书知识库
  system_app_switch     // 应用切换
  system_browser        // 浏览器历史
  user_capture          // 用户手动捕获
}
```

### Confidence 枚举

```
Confidence {
  high    // 高置信度：飞书文档变更、会议纪要、任务状态流转、审批
  medium  // 中置信度：飞书聊天消息、浏览器访问
  low     // 低置信度：应用切换、碎片化行为
}
```

### EventType 枚举

```
EventType {
  message               // 消息
  document_change       // 文档变更
  task_update           // 任务状态变更
  meeting               // 会议
  calendar_event        // 日历事件
  email                 // 邮件
  approval              // 审批
  okr_update            // OKR 变更
  browsing              // 浏览器访问
  app_activity          // 应用活动
  manual_note           // 手动笔记
}
```

### Attachment 结构

```
Attachment {
  id: string            // 附件唯一标识
  type: AttachmentType  // image | file
  filename: string      // 文件名
  path: string          // 存储路径
  mime_type: string     // MIME 类型
  size_bytes: number    // 文件大小
}
```

## WorkRecord（工作记录）

处理层的输出，是事件经过智能处理后的结构化产物。

```
WorkRecord {
  // 标识
  id: string              // 唯一标识
  created_at: datetime    // 生成时间

  // 来源追溯
  source_event_ids: string[]  // 来源事件 ID 列表（可追溯）

  // 内容
  title: string           // 标题
  summary: string         // 摘要
  detail: string          // 详细内容（markdown）
  category: Category      // 分类

  // 关联
  project: string | null  // 关联项目 ID
  people: string[]        // 涉及的人
  tags: string[]          // 标签

  // 任务相关（category 为 task 时有值）
  task_status: TaskStatus | null
  task_due: datetime | null
  task_progress: string | null

  // 元数据
  model_used: string      // 使用的模型标识
  confidence: number      // 处理置信度（0-1）
  needs_review: boolean   // 是否需要人工确认

  // 存储
  obsidian_path: string   // Obsidian 中的文件路径
}
```

### Category 枚举

```
Category {
  task              // 任务
  meeting           // 会议
  communication     // 沟通
  research          // 调研
  review            // 审查
  planning          // 规划
  document          // 文档
  decision          // 决策
}
```

## Task（任务）

```
Task {
  // 标识
  id: string              // 唯一标识
  created_at: datetime    // 创建时间
  updated_at: datetime    // 最后更新时间

  // 内容
  title: string           // 任务标题
  description: string     // 任务描述（markdown）

  // 状态
  status: TaskStatus      // 状态
  priority: Priority      // 优先级
  due_date: datetime | null     // 截止时间
  completed_at: datetime | null // 完成时间

  // 关联
  project: string | null  // 所属项目 ID
  parent_task: string | null    // 父任务 ID
  assignee: string        // 负责人
  collaborators: string[] // 协作人

  // 来源追溯
  source_event_ids: string[]    // 触发创建/更新的事件 ID
  source_platform: string       // 来源平台（obsidian | feishu | manual）
  feishu_task_id: string | null // 飞书任务 ID（用于双向同步）

  // AI 增强
  ai_summary: string | null     // AI 生成的任务摘要
  ai_progress: string | null    // AI 推断的进度描述
  ai_risk: string | null        // AI 识别的风险

  // 元数据
  tags: string[]
  confidence: number
  needs_review: boolean

  // 存储
  obsidian_path: string
}
```

### TaskStatus 枚举

```
TaskStatus {
  todo          // 待办
  in_progress   // 进行中
  blocked       // 阻塞
  done          // 已完成
  cancelled     // 已取消
}
```

### 合法状态流转

```
todo → in_progress → done
todo → in_progress → blocked → in_progress → done
todo → cancelled
in_progress → cancelled
blocked → cancelled
```

非法流转（需审核拦截）：`todo → done`、`done → in_progress`、`blocked → done`

### Priority 枚举

```
Priority {
  P0    // 紧急
  P1    // 高
  P2    // 中
  P3    // 低
}
```

## Project（项目）

```
Project {
  id: string
  name: string
  description: string

  // 关联
  feishu_project_id: string | null
  obsidian_path: string

  // 成员
  owner: string
  members: string[]

  // 状态
  status: ProjectStatus
  start_date: datetime | null
  target_date: datetime | null

  // AI 增强
  ai_progress: string | null
  ai_health: string | null
  ai_summary: string | null
}
```

### ProjectStatus 枚举

```
ProjectStatus {
  active      // 活跃
  paused      // 暂停
  completed   // 已完成
  archived    // 已归档
}
```

## ProcessingAudit（处理审计）

```
ProcessingAudit {
  // 标识
  event_id: string        // 关联的原始事件 ID
  record_id: string | null // 最终生成的 WorkRecord ID
  trace_id: string        // 串联同一事件所有审计记录的链路 ID

  // 步骤信息
  step: AuditStep         // 处理步骤
  timestamp: datetime     // 审计记录时间
  duration_ms: number     // 步骤耗时

  // 模型信息
  model: string           // 使用的模型标识
  model_version: string   // 模型版本
  prompt_id: string       // prompt 模板 ID
  prompt_params: object   // prompt 的动态参数

  // 输入输出
  input_summary: string   // 输入的摘要
  output: object          // 模型的原始输出

  // 质量指标
  confidence: number      // 置信度（0-1）
  token_input: number     // 输入 token 数
  token_output: number    // 输出 token 数
  cost_estimate: number   // 预估成本（美元）

  // 升级相关
  upgrade_reason: string | null   // 升级原因
  previous_model: string | null   // 升级前使用的模型

  // 审核相关
  review_verdict: ReviewVerdict | null
  review_issues: string[] | null

  // 用户干预
  user_action: string | null      // 用户操作
  user_correction: string | null  // 用户修正内容
}
```

### AuditStep 枚举

```
AuditStep {
  classifier        // 分类器决策
  extract           // 信息提取
  upgrade           // 模型升级
  review            // 审核
  user_confirm      // 用户确认
  persist           // 持久化
}
```

### ReviewVerdict 枚举

```
ReviewVerdict {
  approved      // 通过
  needs_fix     // 需要修复（返回处理层重新处理）
  needs_review  // 需要用户确认
}
```
