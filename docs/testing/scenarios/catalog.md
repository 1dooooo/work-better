---
title: 测试场景目录
type: structural
domain: testing
created: 2026-06-06
updated: 2026-06-06
status: draft
---

# 测试场景目录

> **维护说明**：当新增功能、修改业务规则、或调整测试覆盖时更新本文档。这是测试体系的场景索引，确保 100% 功能覆盖。

## 使用说明

- 每个场景有唯一 ID：`{层级}-{编号}`
- 场景描述格式：Given / When / Then
- 标注所属模块、关联功能点 (F1.1.1 等)、运行时间估算

## 统计总览

| 层级 | 场景数 | 总运行时间 | 外部依赖 |
|------|--------|----------|----------|
| A | 136 | <5s | 无 |
| B | 46 | <15s | SQLite(in-memory), FS(temp) |
| C | 9 | <10s | lark-cli(快照), 飞书API(httpmock) |
| D | 22 | <5s | 无 |
| E | 17 | <10s | Tauri IPC(mock) |
| F | 20 | <2min | Tauri app(test mode) |
| G | 182 | <10min(并行) | 取决于场景 |
| **合计** | **432** | **<15min** | |

---

## A 层：纯 Rust 单元测试 (136)

### A1: 分类器路由规则 (16) — wb-processor/classifier.rs

| ID | 场景 | 功能点 | 时间 |
|----|------|--------|------|
| A1-01 | 低置信度事件路由到 Archive | F2.1.5 | <1ms |
| A1-02 | TaskUpdate 路由到 Instant | F2.1.2 | <1ms |
| A1-03 | Approval 路由到 Instant | F2.1.2 | <1ms |
| A1-04 | ManualNote 路由到 Instant | F2.1.2 | <1ms |
| A1-05 | Message(@mention) 路由到 Instant | F2.1.2 | <1ms |
| A1-06 | Message(无@mention) 路由到 Aggregate | F2.1.3 | <1ms |
| A1-07 | DocumentChange 路由到 Aggregate | F2.1.3 | <1ms |
| A1-08 | Browsing 路由到 Aggregate | F2.1.3 | <1ms |
| A1-09 | AppActivity 路由到 Aggregate | F2.1.3 | <1ms |
| A1-10 | OkrUpdate 路由到 Pattern | F2.1.4 | <1ms |
| A1-11 | Meeting 路由到 Instant | F2.1.2 | <1ms |
| A1-12 | CalendarEvent 路由到 Instant | F2.1.2 | <1ms |
| A1-13 | Email 路由到 Instant | F2.1.2 | <1ms |
| A1-14 | @mention 检测解析 JSON content | F1.1.1 | <1ms |
| A1-15 | @mention 检测处理 raw string | F1.1.1 | <1ms |
| A1-16 | @mention 检测处理非 JSON 降级 | F1.1.1 | <1ms |

### A2: 模型升级阈值 (16) — wb-ai/router.rs

| ID | 场景 | 功能点 |
|----|------|--------|
| A2-01 | EntityExtraction < 0.7 触发升级 | F2.2.3 |
| A2-02 | EntityExtraction = 0.7 不升级 | F2.2.3 |
| A2-03 | EntityExtraction > 0.7 不升级 | F2.2.3 |
| A2-04 | TaskIdentification < 0.6 触发升级 | F2.2.3 |
| A2-05 | TaskIdentification > 0.6 不升级 | F2.2.3 |
| A2-06 | Summarization < 0.6 触发升级 | F2.2.3 |
| A2-07 | Summarization > 0.6 不升级 | F2.2.3 |
| A2-08 | SentimentAnalysis < 0.8 触发升级 | F2.2.3 |
| A2-09 | SentimentAnalysis > 0.8 不升级 | F2.2.3 |
| A2-10 | RelationAnalysis < 0.7 触发升级 | F2.2.3 |
| A2-11 | RelationAnalysis > 0.7 不升级 | F2.2.3 |
| A2-12 | PatternRecognition 始终用大模型 | F2.2.3 |
| A2-13 | Classification < 0.6 触发升级 | F2.2.3 |
| A2-14 | Classification > 0.6 不升级 | F2.2.3 |
| A2-15 | 自定义阈值覆盖默认值 | F2.2.4 |
| A2-16 | 未知任务类型不升级 | F2.2.3 |

### A3: Task 状态机 (20) — wb-core/task.rs

| ID | 场景 | 功能点 |
|----|------|--------|
| A3-01 | Todo → InProgress 合法 | F4.1.3 |
| A3-02 | Todo → Cancelled 合法 | F4.1.3 |
| A3-03 | InProgress → Done 合法 (设 completed_at) | F4.1.3 |
| A3-04 | InProgress → Blocked 合法 | F4.1.3 |
| A3-05 | InProgress → Cancelled 合法 | F4.1.3 |
| A3-06 | Blocked → InProgress 合法 | F4.1.3 |
| A3-07 | Blocked → Cancelled 合法 | F4.1.3 |
| A3-08 | Todo → Done 非法 | F4.1.3 |
| A3-09 | Done → InProgress 非法 | F4.1.3 |
| A3-10 | Blocked → Done 非法 | F4.1.3 |
| A3-11 | Cancelled → 任何状态非法 | F4.1.3 |
| A3-12 | Done → Todo 非法 | F4.1.3 |
| A3-13 | Done → Blocked 非法 | F4.1.3 |
| A3-14 | Todo → Blocked 非法 | F4.1.3 |
| A3-15 | 同状态转换非法 | F4.1.3 |
| A3-16 | 转换保持不可变性 | F4.1.3 |
| A3-17 | completed_at 仅在 Done 时设置 | F4.1.3 |
| A3-18 | updated_at 每次转换更新 | F4.1.3 |
| A3-19 | 完整生命周期验证 | F4.1.3 |
| A3-20 | Archive 仅从 Done 状态可用 | F4.1.3 |

### A4: SLA 超时 (16) — wb-processor/sla.rs

| ID | 场景 | 功能点 |
|----|------|--------|
| A4-01~08 | P0-P3 在限内/超限判断 | F2.3.2 |
| A4-09~10 | 优先级升级链 P3→P0 + 天花板 | F2.3.3 |
| A4-11~13 | 优先级估算规则 (needs_review/high/medium) | F2.3.3 |
| A4-14~16 | 日报计算 (空/全准时/部分超时) | F2.3.4 |

### A5: Token 预算 (16) — wb-ai/budget.rs

| ID | 场景 | 功能点 |
|----|------|--------|
| A5-01~07 | 预算基础 (初始化/记录/剩余/饱和减法/exhausted/reset) | F2.2.5 |
| A5-08~11 | can_afford + 策略 (AllowOverflow/QueueTomorrow) | F2.2.6 |
| A5-12~16 | resolve_strategy 组合 (充足/耗尽+紧急/非紧急) | F2.2.6 |

### A6: Review Agent (22) — wb-processor/review*.rs

| ID | 场景 | 功能点 |
|----|------|--------|
| A6-01~05 | RequiredFields 通过/失败组合 | F2.4.1 |
| A6-06~08 | ConfidenceThreshold 边界 | F2.4.1 |
| A6-09~10 | ContentLength 通过/失败 | F2.4.1 |
| A6-11~14 | CategoryConsistency 组合 | F2.4.1 |
| A6-15~22 | ReviewAgent verdict 组合 (Approved/NeedsFix/NeedsReview/skip/custom/confidence) | F2.4.2 |

### A7-A12: 调度器与其他 (30)

| ID | 子层 | 场景概要 | 功能点 |
|----|------|---------|--------|
| A7-01~07 | 依赖图 | 无依赖可运行/依赖完成/未完成/拓扑排序/环检测/空图 | F6.2.2 |
| A8-01~06 | 资源推迟 | P0不推迟/P1仅0推迟/P2阈值判断/P3阈值判断 | F6.2.5 |
| A9-01~04 | 重试超时 | 成功首次/重试至limit/超时Timeout/间隔递增 | F6.2.4 |
| A10-01~05 | Manager状态 | 注册列出/启用禁用/is_enabled/健康检查/未注册返回None | F1.4.1 |
| A11-01~04 | 消息转换 | 有效转换/无id返回None/无效JSON降级/幂等id | F1.1.1 |
| A12-01~04 | 配置构建 | feishu enabled/全部禁用/路径验证/manual健康 | F1.4.4 |

---

## B 层：Rust 集成测试 (46)

| ID | 场景 | Mock 策略 | 功能点 |
|----|------|----------|--------|
| B1-01~08 | SQLite EventLog (追加/检索/标记/过滤/查询/往返/并发) | in-memory | F3.3.1 |
| B2-01~03 | Tauri events 命令 | in-memory | F3.3.1 |
| B3-01~09 | Tauri settings 持久化 (读写/验证/默认/降级) | tempdir | F6.3.1 |
| B4-01~04 | Tauri collectors 管理 | Mock Manager | F1.4.1 |
| B5-01~04 | Tauri scheduler 命令 | Mock Scheduler | F6.2.1 |
| B6-01~03 | Tauri manual capture | in-memory | F1.3.1 |
| B7-01~06 | Obsidian writer (日记/项目/模板/链接/标签) | tempdir | F3.1.1 |
| B8-01~05 | Freshness engine (同步/断链/重复/标签/一致性) | in-memory | F3.4.1 |
| B9-01~04 | Vector DB (嵌入/搜索/同步/删除) | InMemory | F3.2.1 |

---

## C 层：契约测试 (9)

| ID | 场景 | 方法 | 功能点 |
|----|------|------|--------|
| C1-01 | LarkMessagesResponse 反序列化 | insta snapshot | F1.1.1 |
| C1-02 | LarkMessage 字段匹配 | insta snapshot | F1.1.1 |
| C1-03 | LarkSender 字段匹配 | insta snapshot | F1.1.1 |
| C1-04 | 空 data.messages 降级 | insta snapshot | F1.1.1 |
| C2-01 | 飞书 API 响应 schema | httpmock record/replay | F1.1.1 |
| C2-02 | 飞书 API 错误响应格式 | httpmock record/replay | F1.1.1 |
| C3-01 | macOS screencapture 退出码 | platform assert | F1.3.3 |
| C3-02 | HOME 下配置目录创建 | filesystem assert | F6.3.1 |
| C3-03 | macOS SQLite 文件锁定 | filesystem assert | F3.3.1 |

---

## D 层：TypeScript 单元测试 (22)

| ID | 场景 | 子层 | 功能点 |
|----|------|------|--------|
| D1-01~14 | 组件渲染 (MenuBar/Sidebar/EventsView/TasksView/TimelineView/ReportsView/SettingsView/CaptureWindow/ModelSettings/CollectorSettings/StorageSettings/ShortcutSettings/FreshnessSettings/ReportSettings) | D1 | F6.1.4 |
| D2-01~05 | 工具函数 (时间格式化/状态颜色/优先级标签/图标映射/文本截断) | D2 | F6.1.4 |
| D3-01~03 | 状态管理 (视图切换/表单状态/窗口开关) | D3 | F6.1.4 |

---

## E 层：TypeScript 集成测试 (17)

| ID | 场景 | 子层 | 功能点 |
|----|------|------|--------|
| E1-01~15 | Tauri invoke (getEvents/getUnprocessedCount/markEventProcessed/triggerManualCapture/triggerFeishuCollect/getCollectorStatuses/listCollectors/enableCollector/disableCollector/checkCollectorHealth/getFeishuMode/saveFeishuMode/getFeishuChatId/saveFeishuChatId/listScheduledTasks/pauseScheduler) | E1 | F3.3.1 |
| E2-01~02 | 事件监听 (onFeishuCollectComplete + 清理) | E2 | F1.1.1 |

---

## F 层：跨层 E2E (20)

| ID | 场景 | 功能点 |
|----|------|--------|
| F1-01 | 输入文字→手动捕获→Event 在 SQLite | F1.3.1 |
| F1-02 | 带图片附件的捕获→Event 有附件 | F1.3.2 |
| F2-01 | UI 触发采集→lark-cli→事件→UI 更新计数 | F1.1.1 |
| F2-02 | 指定 chat_id 覆盖配置 | F1.1.1 |
| F2-03 | 收集器禁用时采集返回错误 | F1.4.1 |
| F3-01 | 事件→分类器→正确处理路径 | F2.1.1 |
| F3-02 | 低置信度→升级→大模型调用 | F2.2.3 |
| F3-03 | 处理输出→ReviewAgent→approved/rejected | F2.4.2 |
| F3-04 | Approved→Obsidian+VectorDB+SQLite | F3.1.1 |
| F4-01~04 | 设置变更传播 (chat_id/mode/禁用/vault_path) | F1.4.4 |
| F5-01~04 | 调度器集成 (注册执行/暂停恢复/依赖/超时) | F6.2.1 |
| F6-01~03 | 菜单栏数据流 (计数/健康/调度状态) | F6.1.4 |

---

## G 层：黑盒验收测试 (182)

> 每条场景 1:1 映射产品文档。完整 Given/When/Then 见下表。

### G1: 信息采集 (37) — 场景 1-37

| ID | 场景描述 | 功能点 |
|----|---------|--------|
| G1-01 | Given 飞书消息@提及用户, When 到达, Then 捕获为 message(confidence=high) | F1.1.1 |
| G1-02 | Given 飞书消息是回复用户参与的线程, When 到达, Then 捕获并关联线程 | F1.1.1 |
| G1-03 | Given 飞书私信, When 到达, Then 捕获为 message | F1.1.1 |
| G1-04 | Given 消息匹配关键词规则, When 评估, Then 即使无@也捕获 | F1.1.1 |
| G1-05 | Given 消息与用户无关, When 评估, Then 不捕获 | F1.1.1 |
| G1-06 | Given 飞书文档被用户创建/编辑/评论, When 检测, Then 捕获 document_change | F1.1.2 |
| G1-07 | Given 文档中用户被提及, When 检测, Then 捕获事件 | F1.1.2 |
| G1-08 | Given 飞书项目任务变更, When 捕获, Then 捕获 task_update | F1.1.3 |
| G1-09 | Given 日历有即将到来事件, When 每小时同步, Then 捕获 calendar_event | F1.1.4 |
| G1-10 | Given 用户参加视频会议, When 结束, Then 捕获 meeting(含纪要和待办) | F1.1.5 |
| G1-11 | Given 飞书妙记有录制摘要, When 结束, Then 捕获摘要/待办/章节 | F1.1.5 |
| G1-12 | Given 用户通过飞书邮件操作, When 每30分钟同步, Then 捕获 email | F1.1.6 |
| G1-13 | Given 飞书审批状态变更, When 变更, Then 捕获 approval | F1.1.7 |
| G1-14 | Given 用户有 OKR, When 每日同步, Then 捕获 okr_update | F1.1.8 |
| G1-15 | Given 多维表格记录变更, When 检测, Then 捕获 document_change | F1.1.9 |
| G1-16 | Given 电子表格单元格变更, When 检测, Then 捕获 document_change | F1.1.10 |
| G1-17 | Given 知识库节点变更, When 检测, Then 捕获 document_change | F1.1.11 |
| G1-18 | Given 用户切换应用停留>30秒, When 检测, Then 记录 app_activity | F1.2.1 |
| G1-19 | Given 用户切换应用停留<30秒, When 检测, Then 不记录(防抖) | F1.2.1 |
| G1-20 | Given 用户访问非搜索页 URL, When 检测, Then 记录 browsing | F1.2.2 |
| G1-21 | Given 用户访问搜索结果页, When 检测, Then 不记录 | F1.2.2 |
| G1-22 | Given 用户按全局快捷键, When 窗口打开, Then 聚焦输入区 | F1.3.1 |
| G1-23 | Given 窗口打开, When 输入并提交, Then 创建 manual_note | F1.3.1 |
| G1-24 | Given 窗口打开, When 粘贴图片, Then 接受为附件 | F1.3.2 |
| G1-25 | Given 窗口打开, When 拖放文件, Then 接受为附件 | F1.3.2 |
| G1-26 | Given 用户按截图键, When 截图完成, Then 打开窗口并预载截图 | F1.3.3 |
| G1-27 | Given 用户提交快捷记录, When 完成, Then 窗口自动隐藏(取决于配置) | F1.3.1 |
| G1-28 | Given 收集器运行中, When 禁用, Then 停止且子收集器也禁用 | F1.4.2 |
| G1-29 | Given 父收集器禁用, When 重新启用, Then 子收集器恢复各自状态 | F1.4.2 |
| G1-30 | Given 收集器故障, When 健康检查, Then 自动禁用并通知 | F1.4.3 |
| G1-31 | Given 健康状态 unhealthy, When 查看, Then 显示错误指示 | F1.4.3 |
| G1-32 | Given 运行时注册新收集器, When 注册, Then 立即开始采集 | F1.4.1 |
| G1-33 | Given 配置飞书连接, When 选 API, Then 用飞书开放平台 API | F1.4.4 |
| G1-34 | Given 配置飞书连接, When 选 CLI, Then 用 lark-cli 降级 | F1.4.4 |
| G1-35 | Given 多事件同时到达, When 写入 EventLog, Then 严格时间顺序 | F1.4.5 |
| G1-36 | Given 系统重启, When 有未处理事件, Then 事件恢复(无数据丢失) | F1.4.5 |
| G1-37 | Given 处理逻辑变更, When 触发重放, Then 历史事件重新处理 | F1.4.5 |

### G2: 智能处理 (34) — 场景 38-71

| ID | 场景描述 | 功能点 |
|----|---------|--------|
| G2-01 | Given task_update/approval/meeting, When 分类, Then 即时处理(P0/P1) | F2.1.2 |
| G2-02 | Given message(@mention)/manual_note, When 分类, Then 即时处理 | F2.1.2 |
| G2-03 | Given message(一般)/doc_change/browsing/app_activity, When 分类, Then 聚合处理(P2) | F2.1.3 |
| G2-04 | Given 需长期分析的事件, When 分类, Then 模式分析(P3) | F2.1.4 |
| G2-05 | Given confidence=low 或纯归档, When 分类, Then 直接归档 | F2.1.5 |
| G2-06 | Given 实体提取+小模型置信度<0.7, When 处理, Then 升级大模型 | F2.2.3 |
| G2-07 | Given 任务识别+置信度<0.6, When 处理, Then 升级大模型 | F2.2.3 |
| G2-08 | Given 摘要生成+置信度<0.6 或>500字, When 处理, Then 升级大模型 | F2.2.3 |
| G2-09 | Given 情感判断+置信度<0.8, When 处理, Then 升级大模型 | F2.2.3 |
| G2-10 | Given 关联分析+置信度<0.7, When 处理, Then 升级大模型 | F2.2.3 |
| G2-11 | Given 模式识别任务, When 处理, Then 直接用大模型 | F2.2.3 |
| G2-12 | Given 小模型置信度达标, When 完成, Then 进入 ReviewAgent | F2.2.3 |
| G2-13 | Given 小模型失败+大模型也失败, When 处理, Then 标记"需手动处理"并通知 | F2.2.3 |
| G2-14 | Given 日预算未耗尽, When 需大模型, Then 可用 | F2.2.5 |
| G2-15 | Given 日预算耗尽+非紧急, When 需大模型, Then 排队明天 | F2.2.6 |
| G2-16 | Given 日预算耗尽+紧急(P0/P1), When 需大模型, Then 允许溢出并通知 | F2.2.6 |
| G2-17 | Given 日预算耗尽+策略 degrade_to_small, When 需大模型, Then 小模型降级 | F2.2.6 |
| G2-18 | Given Token 使用被跟踪, When 查看审计, Then 可见每日消耗 | F2.2.5 |
| G2-19 | Given P0 事件+超5分钟未处理, When 超时, Then 强制升级并通知 | F2.3.1 |
| G2-20 | Given P1 事件+超30分钟未处理, When 超时, Then 升级大模型 | F2.3.1 |
| G2-21 | Given P2 事件+超4小时未处理, When 超时, Then 继续正常流程 | F2.3.2 |
| G2-22 | Given P3 事件, When 未处理, Then 排入每日批处理 | F2.3.2 |
| G2-23 | Given 处理队列, When SLA 扫描(每5分钟), Then 超时事件自动提升 | F2.3.3 |
| G2-24 | Given 一天结束, When SLA 报告生成, Then 显示效率统计 | F2.3.4 |
| G2-25 | Given 直接归档输出, When ReviewAgent, Then 直接通过 | F2.4.2 |
| G2-26 | Given 低置信度提取输出, When ReviewAgent, Then 仅规则层验证 | F2.4.2 |
| G2-27 | Given 高置信度提取输出, When ReviewAgent, Then 10%抽样审查 | F2.4.2 |
| G2-28 | Given 任务状态变更输出, When ReviewAgent, Then 规则验证+共享数据需确认 | F2.4.2 |
| G2-29 | Given 报告/摘要输出, When ReviewAgent, Then 规则+小模型一致性检查 | F2.4.2 |
| G2-30 | Given 涉及他人信息, When ReviewAgent, Then 规则+小模型+用户确认 | F2.4.2 |
| G2-31 | Given ReviewAgent needs_fix, When 返回, Then 回处理层重新处理 | F2.4.3 |
| G2-32 | Given ReviewAgent needs_review, When 返回, Then 推送通知 | F2.4.3 |
| G2-33 | Given ReviewAgent approved, When 返回, Then 进入存储层 | F2.4.3 |
| G2-34 | Given 同类问题频繁, When 阈值达到, Then 自动调整提示或阈值 | F2.4.4 |

### G3: 数据存储 (28) — 场景 72-99

| ID | 场景描述 | 功能点 |
|----|---------|--------|
| G3-01 | Given WorkRecord 持久化, When 写入 Obsidian, Then 放入正确目录 | F3.1.1 |
| G3-02 | Given 引用项目/人/实体, When 写入, Then 自动创建双向链接 | F3.1.6 |
| G3-03 | Given 被分类, When 写入, Then 自动应用标签 | F3.1.7 |
| G3-04 | Given 多上下文, When 查看任一位置, Then 不同维度可访问 | F3.1.1 |
| G3-05 | Given 自定义模板, When 配置, Then 新文件遵循模板 | F3.1.5 |
| G3-06 | Given 新文档写入, When 成功, Then 异步生成嵌入 | F3.2.1 |
| G3-07 | Given 文档修改, When 检测, Then 5分钟后重新嵌入(防抖) | F3.2.5 |
| G3-08 | Given 文档删除, When 检测, Then 嵌入移除 | F3.2.5 |
| G3-09 | Given 语义搜索, When 执行, Then 按相似度排序返回 | F3.2.2 |
| G3-10 | Given 大模型需上下文, When RAG 召回, Then 检索相关文档 | F3.2.4 |
| G3-11 | Given 结构化数据存在, When 查询, Then 按索引字段快速查询 | F3.3.1 |
| G3-12 | Given 任务状态变更, When 更新, Then 跟踪完整转换历史 | F3.3.2 |
| G3-13 | Given WorkRecord 持久化, When 完成, Then 顺序: Obsidian→向量DB→结构化DB | F3.4.1 |
| G3-14 | Given 用户在 Obsidian 编辑, When 保存, Then 向量DB和结构化DB更新 | F3.4.2 |
| G3-15 | Given 一致性检查(每周), When 发现差异, Then 标记并触发重建 | F3.4.6 |
| G3-16 | Given 向量DB数≠文档数, When 检查, Then 标记不匹配 | F3.4.6 |
| G3-17 | Given 飞书任务标记完成, When 新鲜度比对, Then Obsidian 更新为完成 | F3.4.1 |
| G3-18 | Given 飞书文档已更新, When 每日检查, Then 检测过时并重新生成摘要 | F3.4.2 |
| G3-19 | Given 双向链接指向已删除文件, When 每日扫描, Then 标记断链 | F3.4.3 |
| G3-20 | Given 标签命名不一致, When 每周规范化, Then 合并变体 | F3.4.5 |
| G3-21 | Given 信息多次记录, When 每周检测, Then 标记合并候选 | F3.4.4 |
| G3-22 | Given 知识已过时, When 每月审查, Then 标记需用户审查 | F3.4.7 |
| G3-23 | Given 检查完成+有需注意项, When 完成, Then 推送通知 | F3.4.8 |
| G3-24 | Given 检查完成+有可修复项, When 完成, Then 静默修复 | F3.4.8 |
| G3-25 | Given 检查完成, When 执行完毕, Then 生成新鲜度报告 | F3.4.9 |
| G3-26 | Given 用户触发重建向量DB, When 执行, Then 所有文档重新嵌入 | F3.6.1 |
| G3-27 | Given 用户触发重处理历史, When 执行, Then 所有事件重新处理 | F3.6.2 |
| G3-28 | Given 用户触发全量一致性检查, When 执行, Then 三层互相验证 | F3.6.3 |

### G4: 任务管理 (20) — 场景 100-119

| ID | 场景描述 | 功能点 |
|----|---------|--------|
| G4-01 | Given 用户手动创建任务, When 保存, Then Task(status=todo, source=obsidian) | F4.1.1 |
| G4-02 | Given 系统从会议发现任务, When AI 提取, Then needs_review=true | F4.2.1 |
| G4-03 | Given 飞书项目任务变更, When 同步, Then Obsidian 更新 source=feishu | F4.3.1 |
| G4-04 | Given 任务存在, When todo→in_progress, Then 合法并持久化 | F4.1.3 |
| G4-05 | Given in_progress, When →blocked, Then 合法 | F4.1.3 |
| G4-06 | Given blocked, When →in_progress, Then 合法 | F4.1.3 |
| G4-07 | Given 活跃状态, When →cancelled, Then 合法 | F4.1.3 |
| G4-08 | Given todo, When 直接→done, Then 拒绝并解释 | F4.1.3 |
| G4-09 | Given done, When →in_progress, Then 拒绝 | F4.1.3 |
| G4-10 | Given blocked, When 直接→done, Then 拒绝 | F4.1.3 |
| G4-11 | Given 标记 done, When 完成, Then 设置 completed_at | F4.1.3 |
| G4-12 | Given 有子任务, When 父任务创建, Then 通过 parent_task 关联 | F4.1.4 |
| G4-13 | Given 会议结束有待办, When 分析, Then 识别并创建 needs_review 任务 | F4.2.1 |
| G4-14 | Given 聊天消息含承诺, When 分析, Then 识别并创建 needs_review 任务 | F4.2.2 |
| G4-15 | Given 邮件含请求, When 分析, Then 识别并创建 needs_review 任务 | F4.2.3 |
| G4-16 | Given 文档评论含待办, When 分析, Then 识别并创建 needs_review 任务 | F4.2.4 |
| G4-17 | Given 自动发现的任务, When 呈现, Then 必须确认或拒绝才激活 | F4.2.5 |
| G4-18 | Given 飞书任务状态变更, When 捕获, Then Obsidian 自动更新(无需确认) | F4.3.1 |
| G4-19 | Given Obsidian 任务修改, When 同步飞书, Then 需用户确认 | F4.3.2 |
| G4-20 | Given 两端同时修改, When 检测冲突, Then 呈现解决机制 | F4.3.3 |

### G5: 报告生成 (12) — 场景 120-131

| ID | 场景描述 | 功能点 |
|----|---------|--------|
| G5-01 | Given 工作日18:00, When 日报触发, Then 生成含完成/计划/阻塞的日报 | F5.1.1 |
| G5-02 | Given 周五17:00, When 周报触发, Then 生成含进展/成果/计划/风险的周报 | F5.1.2 |
| G5-03 | Given 月末, When 月报触发, Then 生成含目标/时间/效率的月报 | F5.1.3 |
| G5-04 | Given 季末, When 季报触发, Then 生成含 OKR/里程碑/能力的季报 | F5.1.4 |
| G5-05 | Given 12/31 或 6/30, When 半年报/年报触发, Then 生成对应报告 | F5.1.5 |
| G5-06 | Given 月末同时季末, When 两者同时到期, Then 各自按 SLA 生成 | F5.1.6 |
| G5-07 | Given 报告生成完成, When 完毕, Then 通知用户审查确认 | F5.2.1 |
| G5-08 | Given 用户审查编辑, When 完成, Then 编辑版本为最终版本 | F5.2.2 |
| G5-09 | Given 用户确认, When 选择导出, Then 可导出 Markdown 或 PDF | F5.2.3 |
| G5-10 | Given 用户确认, When 选择同步飞书, Then 推送到飞书文档 | F5.2.4 |
| G5-11 | Given 用户自定义格式, When 修改模板, Then 后续遵循模板 | F5.2.5 |
| G5-12 | Given 用户改生成时间, When 配置, Then 后续在新时间生成 | F5.2.6 |

### G6: 系统能力 (38) — 场景 132-169

| ID | 场景描述 | 功能点 |
|----|---------|--------|
| G6-01 | Given 用户在任何应用, When 按 Cmd+Shift+Space, Then 快捷记录窗口出现 | F6.1.1 |
| G6-02 | Given 窗口可见, When 再按快捷键, Then 窗口隐藏 | F6.1.1 |
| G6-03 | Given 用户在任何应用, When 按截图键, Then 截图并打开窗口 | F6.1.1 |
| G6-04 | Given 有待确认项, When 通知触发, Then 显示通知含描述 | F6.1.2 |
| G6-05 | Given 有常规信息, When 轻提醒触发, Then 非侵入通知 | F6.1.2 |
| G6-06 | Given 用户查看状态, When 点击菜单栏, Then 显示待确认/任务/摘要等 | F6.1.4 |
| G6-07 | Given 用户要快速操作, When 交互菜单栏, Then 两次点击内完成 | F6.1.4 |
| G6-08 | Given 打开菜单栏, When 查看通知中心, Then 一屏可见所有可操作项 | F6.1.4 |
| G6-09 | Given 需要复杂操作, When 从菜单栏选择, Then 重定向主窗口 | F6.1.4 |
| G6-10 | Given 打开主窗口, When 查看时间线, Then 时间轴+缩放+过滤 | F6.1.4 |
| G6-11 | Given 点击时间线项, When 展开详情, Then 有 Obsidian 原文链接 | F6.1.4 |
| G6-12 | Given 打开主窗口, When 查看任务板, Then 按状态列分组 | F6.1.4 |
| G6-13 | Given 拖拽任务卡片, When 拖到不同列, Then 状态更新并同步 | F6.1.4 |
| G6-14 | Given 用户搜索, When 执行, Then RAG+结构化双路搜索 | F6.1.4 |
| G6-15 | Given 查看数据探索, When 统计显示, Then 时间/任务/会议/模式图表 | F6.1.4 |
| G6-16 | Given 搜索结果, When 点击, Then 打开 Obsidian 原文 | F6.1.4 |
| G6-17 | Given 打开设置, When 配置模型, Then API 端点/参数/预算 | F6.3.1 |
| G6-18 | Given 打开设置, When 配置收集器, Then 飞书凭据/开关 | F6.3.2 |
| G6-19 | Given 打开设置, When 配置存储, Then Obsidian 路径/向量DB/备份 | F6.3.3 |
| G6-20 | Given 打开设置, When 配置快捷键, Then 自定义组合键 | F6.3.4 |
| G6-21 | Given 打开设置, When 配置新鲜度规则, Then 频率和策略 | F6.3.5 |
| G6-22 | Given 打开设置, When 查看审计, Then 按维度查询并导出 | F6.3.6 |
| G6-23 | Given 调度器运行中, When cron 触发, Then 在偏移窗口内执行 | F6.2.1 |
| G6-24 | Given A 依赖 B, When B 未完成, Then A 不启动 | F6.2.2 |
| G6-25 | Given 任务超 SLA, When 超时检测, Then 自动终止 | F6.2.3 |
| G6-26 | Given 任务失败, When 失败, Then 重试最多3次递增间隔 | F6.2.4 |
| G6-27 | Given 3次重试全失败, When 最终失败, Then 标记 failed | F6.2.4 |
| G6-28 | Given 日预算不足, When 低优先级需执行, Then 推迟 | F6.2.5 |
| G6-29 | Given 用户激活暂停, When 触发, Then 所有定时任务停止 | F6.2.6 |
| G6-30 | Given 用户激活紧急停止, When 触发, Then 执行中任务立即终止 | F6.2.6 |
| G6-31 | Given 暂停中, When 恢复, Then 积压任务按优先级执行 | F6.2.6 |
| G6-32 | Given 查看调度 UI, When 检查任务, Then 显示 ID/名称/计划/状态/开关 | F6.2.7 |
| G6-33 | Given 查看执行日志, When 检查, Then 显示状态/时长/摘要/错误/重试 | F6.2.7 |
| G6-34 | Given 同类任务执行中, When 另一个触发, Then 不并行 | F6.2.8 |
| G6-35 | Given 采集层定时任务, When 触发, Then 整点后0-5分钟执行 | F6.2.9 |
| G6-36 | Given 处理层定时任务, When 触发, Then 整点后5-30分钟执行 | F6.2.9 |
| G6-37 | Given 存储层定时任务, When 触发, Then 低峰期(02:00-05:00)执行 | F6.2.9 |
| G6-38 | Given 报告层定时任务, When 触发, Then 用户配置时间执行 | F6.2.9 |

### G7: 横切关注 (13) — 场景 170-182

| ID | 场景描述 | 功能点 |
|----|---------|--------|
| G7-01 | Given AI 处理个人数据, When 处理, Then 自主执行无需确认 | F2.4.5 |
| G7-02 | Given AI 要修改共享数据, When 即将执行, Then 必须用户确认 | F2.4.5 |
| G7-03 | Given 用户确认共享操作, When 确认, Then 执行并同步飞书 | F2.4.5 |
| G7-04 | Given 事件被采集, When 进入系统, Then EventLog 不可变记录 | F1.4.5 |
| G7-05 | Given 事件被消费, When 处理完成, Then 标记为 processed | F1.4.5 |
| G7-06 | Given WorkRecord 产出, When 写入, Then 顺序: Obsidian→向量DB→结构化DB | F3.4.1 |
| G7-07 | Given 表示层读取, When 查询, Then 三层联合查询接口 | F3.4.2 |
| G7-08 | Given 用户在 Obsidian 编辑, When 保存, Then 两 DB 更新保持一致 | F3.4.2 |
| G7-09 | Given 事件进入处理, When 每步执行, Then 生成 ProcessingAudit | F2.5.1 |
| G7-10 | Given 同一事件有审计记录, When 查看, Then trace_id 链接完整链路 | F2.5.2 |
| G7-11 | Given 审计数据存在, When 查询, Then 可按多维度查询 | F2.5.3 |
| G7-12 | Given 审计数据积累, When 月度聚合, Then 聚合为统计摘要 | F2.5.4 |
| G7-13 | Given 检测到模式(同类错误频繁), When 生成建议, Then 产生改进建议 | F2.5.4 |
