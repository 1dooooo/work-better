# 采集层架构

> **维护说明**：当新增/移除采集器、修改采集策略或事件格式时更新本文档。
> 采集层的接口变更需同步更新 [事件模型](event-model.md) 和 [架构总览](../overview.md)。

## 概述

采集层是系统的信息入口。它从不同来源获取原始信息，转换为标准 Event，写入 EventLog。

**职责边界**：采集层不做任何智能处理，只负责提取和转换。

## 架构

```
┌─────────────────────────────────────────────┐
│              CollectorManager                │
│  采集器生命周期管理 · 开关控制 · 健康监控      │
└──────────┬──────────────────────────────────┘
           │
    ┌──────┴──────┬──────────────┐
    ▼             ▼              ▼
FeishuCollector  SystemCollector  UserCaptureCollector
 (可热插拔)       (可热插拔)       (始终开启)
```

## CollectorManager

采集器管理器，负责采集器的注册、注销、开关控制和健康监控。

### 接口

```
interface CollectorManager {
  // 生命周期
  register(collector: Collector): void
  unregister(collectorId: string): void

  // 开关控制
  enable(collectorId: string, path?: string): void
  disable(collectorId: string, path?: string): void
  isEnabled(collectorId: string, path?: string): boolean
  getEnabledTree(): ConfigTree

  // 健康监控
  healthCheck(collectorId: string): HealthStatus
  healthCheckAll(): HealthStatus[]

  // 采集器发现
  discover(): Collector[]  // 从插件目录自动发现
}
```

### 开关联动规则

- 关闭父级 → 所有子级停用
- 开启父级 → 子级恢复到各自独立的状态
- 开关状态持久化到本地配置

## Collector 接口

每个采集器必须实现统一接口：

```
interface Collector {
  // 元信息
  id: string                // 唯一标识
  name: string              // 显示名称
  version: string           // 版本
  configSchema: ConfigSchema // 配置项定义

  // 生命周期
  init(config: object): Promise<void>   // 初始化
  start(): Promise<void>                // 开始采集
  stop(): Promise<void>                 // 停止采集
  teardown(): Promise<void>             // 清理资源

  // 状态
  healthCheck(): HealthStatus
  isEnabled(): boolean
}
```

### HealthStatus

```
HealthStatus {
  status: 'healthy' | 'degraded' | 'unhealthy'
  message: string | null
  lastCheck: datetime
  lastSuccess: datetime | null
  errorCount: number
}
```

## 采集器详情

### FeishuCollector

飞书采集器，通过飞书开放平台 API 或飞书 CLI 获取数据。

**接入方式**：
- 飞书开放平台 API（推荐）：更稳定、能力更全
- 飞书 CLI（备选）：降低接入门槛

**子采集器与采集策略**：

| 子采集器 | 采集内容 | 过滤规则 | 采集频率 |
|---------|---------|---------|---------|
| messages | @提及、回复、私聊、关键词命中 | 只采集与我相关的 | 实时 |
| docs | 创建、编辑、评论 | 我创建/编辑/被提及的 | 变更时触发 |
| projects | 任务创建、状态变更、评论 | 我负责或参与的项目 | 变更时触发 |
| calendar | 日程事件、会议安排 | 我的日程 | 每小时 |
| meetings | 会议纪要、逐字稿、待办 | 我参与的会议 | 会议结束后 |
| email | 发送、接收、回复 | 我的邮件 | 每 30 分钟 |
| approvals | 发起、处理的审批 | 我的审批实例 | 变更时触发 |
| okr | 目标、关键结果、进度 | 我的 OKR | 每日 |
| bitable | 记录、字段、视图变更 | 我操作的表 | 变更时触发 |
| sheets | 单元格变更 | 我操作的表格 | 变更时触发 |
| wiki | 节点变更 | 我贡献/关注的 | 变更时触发 |
| jianji | 录音总结、待办、章节 | 我参与的 | 会议结束后 |

**飞书 API 能力映射**：

```
飞书业务域          →  子采集器    →  事件类型
────────────────────────────────────────────
消息与群组          →  messages   →  message
云文档              →  docs       →  document_change
云空间              →  (docs)     →  document_change
电子表格            →  sheets     →  document_change
多维表格            →  bitable    →  document_change
日历                →  calendar   →  calendar_event
视频会议            →  meetings   →  meeting
妙记                →  jianji     →  meeting
邮箱                →  email      →  email
任务                →  projects   →  task_update
知识库              →  wiki       →  document_change
审批                →  approvals  →  approval
OKR                 →  okr        →  okr_update
```

### SystemCollector

系统行为采集器，监听用户桌面活动。

| 子采集器 | 采集内容 | 过滤规则 | 采集频率 |
|---------|---------|---------|---------|
| app_switch | 前台应用变化、停留时长 | 停留 > 30 秒才记录，排除系统内部切换 | 采样 |
| browser | URL + 页面标题 | 排除搜索引擎结果页 | 采样 |

### UserCaptureCollector

用户手动采集器，始终开启不可关闭。

| 子采集器 | 采集内容 | 说明 |
|---------|---------|------|
| text_input | 文本 | 通过速记窗口输入 |
| image_paste | 图片 | 粘贴或拖拽到速记窗口 |
| screenshot | 截图 | 快捷键触发 |

## EventLog

事件日志，有序、不可变的事件存储。

### 职责

- 保证事件的时序完整性（物理时间 + 逻辑顺序）
- 处理层消费后标记已处理
- 未消费的事件不丢失（应用重启后可恢复）
- 支持重放——处理逻辑变更后可重新处理历史事件

### 存储

- 底层使用 SQLite 存储
- 事件按 `timestamp` 排序
- 索引：`id`、`source`、`type`、`collected_at`

### 数据结构

```
EventLog {
  append(event: Event): void
  get(eventId: string): Event | null
  query(filter: EventFilter): Event[]
  markProcessed(eventId: string): void
  getUnprocessed(): Event[]
  replay(filter: EventFilter): Event[]  // 重新处理历史事件
}
```

### EventFilter

```
EventFilter {
  source?: Source
  type?: EventType
  confidence?: Confidence
  timeRange?: { start: datetime, end: datetime }
  processed?: boolean
  tags?: string[]
  limit?: number
  offset?: number
}
```
