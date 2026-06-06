---
title: Phase 3 — 深度洞察
date: 2026-06-06
status: done
goal: 使 Work Better 成为真正可用的 AI 工作观察者
phase: 3
depends_on:
  - 2026-06-06-phase1-mvp.md
  - 2026-06-06-phase2-core.md
---

# Phase 3：深度洞察

> 从"能跑"到"能用"——让信息真正流动起来。

## 前置条件

Phase 1 + Phase 2 全部完成（138 tests, 35 features ✅）

## 任务总览

| # | 任务 | 层 | 依赖 | 估时 |
|---|------|----|------|------|
| 1 | Obsidian 文档层 | storage | — | 3h |
| 2 | 模型适配器接入 | ai | — | 3h |
| 3 | 处理流水线串联 | processor | 1,2 | 3h |
| 4 | SLA 管理 | processor | 3 | 2h |
| 5 | 全链路审计 | processor | 3 | 2h |
| 6 | 信息保鲜引擎 | storage | 1 | 3h |
| 7 | 报告生成系统 | processor+storage | 1,3 | 4h |
| 8 | 补充采集器（7个） | collector | — | 3h |
| 9 | 系统行为采集器 | collector | — | 2h |
| 10 | 快速捕获窗口 | ui | — | 2h |
| 11 | 设置界面 | ui | — | 3h |
| 12 | 调度器增强 | scheduler | — | 2h |

**总计：~32h，12 个任务**

---

## Task 1：Obsidian 文档层

**目标**：结构化写入 Obsidian vault，不再只是扁平文件输出

**涉及特性**：F3.1.1 ~ F3.1.7

**新增 crate**：无（扩展 wb-storage）

**关键实现**：

```
wb-storage/src/obsidian/
├── mod.rs
├── vault.rs          # VaultManager — vault 路径、目录结构管理
├── daily.rs          # DailyJournal — 按日期生成日记文件
├── project.rs        # ProjectDir — 项目目录结构
├── people.rs         # PeopleProfile — 人物档案
├── template.rs       # TemplateEngine — 会议/任务/报告模板
├── links.rs          # LinkBuilder — 双向链接自动构建
└── tags.rs           # TagManager — 标签规范化
```

**Vault 目录结构**：
```
vault/
├── Daily/            # 日记
├── Projects/         # 项目
├── People/           # 人物
├── Tasks/            # 任务
├── Reports/          # 报告
├── Knowledge/        # 知识
└── System/           # 系统数据（模板、配置）
```

**核心 API**：
- `VaultManager::new(vault_path)` — 初始化目录结构
- `DailyJournal::append(date, content)` — 追加日记条目
- `ProjectDir::ensure(project_name)` — 确保项目目录存在
- `LinkBuilder::wikilink(target, alias?)` — 生成 `[[target|alias]]`
- `TagManager::normalize(tag)` — 标签规范化
- `TemplateEngine::render(template_name, context)` — 模板渲染

**验收**：
- [ ] vault 目录结构自动创建
- [ ] 日记文件按日期正确生成
- [ ] 双向链接格式正确
- [ ] 模板渲染正确
- [ ] 20+ 单元测试

---

## Task 2：模型适配器接入

**目标**：让 ModelRouter 真正调用 LLM，而不只是路由决策

**涉及特性**：F2.2.1, F2.2.2

**关键实现**：

```
wb-ai/src/
├── adapter/
│   ├── mod.rs          # ModelAdapter trait（已有）
│   ├── anthropic.rs    # AnthropicAdapter（已有，增强）
│   ├── openai.rs       # OpenAIAdapter — 兼容 OpenAI API 格式
│   └── factory.rs      # AdapterFactory — 按配置创建适配器
├── task_runner.rs      # TaskRunner — 接收任务，路由到模型，返回结果
└── prompt/
    ├── mod.rs
    ├── classify.rs     # 分类提示词
    ├── extract.rs      # 实体提取提示词
    ├── summarize.rs    # 摘要提示词
    └── analyze.rs      # 深度分析提示词
```

**核心 API**：
- `TaskRunner::new(router, budget, adapter_factory)` — 初始化
- `TaskRunner::run(task_type, input)` → `TaskOutput` — 执行任务
- `TaskOutput { content, confidence, model_used, tokens_used, duration_ms }`

**Prompt 策略**：
- 小模型：分类、打标签、实体提取、简单摘要
- 大模型：深度分析、跨事件关联、模式识别

**验收**：
- [ ] 小模型任务正常执行
- [ ] 大模型任务正常执行
- [ ] TokenBudget 扣费正确
- [ ] ModelRouter 升级决策正确
- [ ] 超时和错误处理
- [ ] 15+ 测试

---

## Task 3：处理流水线串联

**目标**：把分类 → 路由 → 模型 → 审核 → 持久化 串成完整流水线

**涉及特性**：F2.2, F2.4

**关键实现**：

```
wb-processor/src/
├── pipeline.rs         # ProcessingPipeline — 串联所有步骤
├── extraction.rs       # EntityExtractor — 从模型输出提取结构化数据
└── persist.rs          # PersistStep — 审核通过后写入 Obsidian + SQLite
```

**Pipeline 流程**：
```
Event → Classifier(路由) → TaskRunner(模型处理) → Extraction(结构化)
      → ReviewAgent(审核) → PersistStep(持久化) → WorkRecord
```

**核心 API**：
- `ProcessingPipeline::new(classifier, task_runner, reviewer, persistor)`
- `ProcessingPipeline::process(event)` → `ProcessedResult`
- `ProcessedResult { work_record, audit_trail, review_result }`

**验收**：
- [ ] 完整 Event → WorkRecord 流程
- [ ] 每步产生审计记录
- [ ] 审核不通过时正确阻断
- [ ] 10+ 集成测试

---

## Task 4：SLA 管理

**目标**：处理任务超时自动升级

**涉及特性**：F2.3.1 ~ F2.3.4

**关键实现**：

```
wb-processor/src/
├── sla.rs              # SlaManager
└── priority.rs         # Priority 四级 + 升级逻辑
```

**核心 API**：
- `SlaManager::new(config)` — 初始化（P0=5min, P1=30min, P2=4h, P3=24h）
- `SlaManager::check_timeouts()` — 扫描超时任务
- `SlaManager::escalate(task_id)` — 升级优先级和模型
- `SlaManager::daily_report()` → `TimelinessReport` — 每日效率统计

**集成**：调度器每 5 分钟调用 `check_timeouts()`

**验收**：
- [ ] 四级优先级定义
- [ ] 超时自动升级
- [ ] 每日效率报告
- [ ] 10+ 测试

---

## Task 5：全链路审计

**目标**：记录每个处理步骤的完整审计链

**涉及特性**：F2.5.1 ~ F2.5.4

**关键实现**：

```
wb-processor/src/
└── audit_pipeline.rs   # AuditPipeline — 嵌入处理流水线的审计记录
```

**审计记录结构**（扩展 ProcessingAudit）：
- `trace_id` — 贯穿同一 Event 所有处理步骤
- `step_name` — 步骤名称（classify/extract/review/persist）
- `model_used` — 使用的模型
- `input/output` — 输入输出摘要
- `duration_ms` — 耗时

**核心 API**：
- `AuditPipeline::record(step, trace_id, details)` — 记录一步
- `AuditPipeline::query(filter)` → `Vec<ProcessingAudit>` — 条件查询
- `AuditPipeline::trace(trace_id)` → 完整链路

**验收**：
- [ ] 每步审计记录完整
- [ ] trace_id 链路可追溯
- [ ] 查询接口可用
- [ ] 10+ 测试

---

## Task 6：信息保鲜引擎

**目标**：自动维护 Obsidian 知识库的新鲜度

**涉及特性**：F3.4.1 ~ F3.4.9

**关键实现**：

```
wb-storage/src/freshness/
├── mod.rs           # FreshnessEngine
├── sync.rs          # 任务状态同步、文档变更检测
├── integrity.rs     # 链接完整性、重复检测、标签规范化
├── quality.rs       # 知识过期审查（LLM）
└── report.rs        # 保鲜报告
```

**保鲜任务分类**：

| 类别 | 任务 | 频率 |
|------|------|------|
| Sync | S-01 任务状态同步 | 每 15min |
| Sync | S-02 文档变更检测 | 每 30min |
| Integrity | S-03 链接完整性检查 | 每天 |
| Integrity | S-04 重复检测 | 每天 |
| Integrity | S-05 标签规范化 | 每天 |
| Quality | S-06 三层一致性检查 | 每周 |
| Quality | S-07 知识过期审查 | 每周 |
| Cleanup | S-08 审计数据聚合 | 每天 |
| Cleanup | S-09 保鲜报告 | 每次保鲜后 |

**核心 API**：
- `FreshnessEngine::run_category(category)` — 执行一类保鲜任务
- `FreshnessEngine::run_all()` — 执行全部保鲜
- `FreshnessEngine::last_report()` → `FreshnessReport`

**验收**：
- [ ] 9 个保鲜任务可独立执行
- [ ] 保鲜报告正确生成
- [ ] 可通过调度器定时触发
- [ ] 15+ 测试

---

## Task 7：报告生成系统

**目标**：自动生成日报、周报、月报

**涉及特性**：F5.1.1 ~ F5.1.3, F5.2.1 ~ F5.2.3

**关键实现**：

```
wb-processor/src/report/
├── mod.rs           # ReportGenerator
├── daily.rs         # 日报生成
├── weekly.rs        # 周报生成
├── monthly.rs       # 月报生成
├── template.rs      # 报告模板管理
└── confirm.rs       # 确认流程
```

**报告层级**：

| 层级 | 内容 | 触发 |
|------|------|------|
| 日报 | 完成事项、明日计划、阻塞项 | 每天 18:00 |
| 周报 | 周进度、关键成果、下周计划、风险 | 每周五 17:00 |
| 月报 | 目标进度、时间分配、效率趋势 | 每月最后一天 |

**核心 API**：
- `ReportGenerator::generate_daily(date)` → `Report`
- `ReportGenerator::generate_week(week_range)` → `Report`
- `ReportGenerator::generate_month(month)` → `Report`
- `ReportGenerator::list_reports(filter)` → `Vec<ReportSummary>`

**报告确认流程**：
1. 生成报告 → 推送通知
2. 用户查看 → 可编辑
3. 确认 → 写入 Obsidian + 可选同步飞书

**验收**：
- [ ] 日报从 WorkRecord 正确聚合
- [ ] 周报/月报正确聚合
- [ ] 报告模板可配置
- [ ] 确认流程完整
- [ ] 15+ 测试

---

## Task 8：补充采集器（7个）

**目标**：扩展飞书数据源覆盖

**涉及特性**：F1.1.5 ~ F1.1.12（除 F1.1.7）

**新增文件**（每个一个文件，遵循已有采集器模式）：

```
wb-collector/src/feishu/
├── meetings.rs       # FeishuMeetingCollector — lark-cli meeting
├── emails.rs         # FeishuEmailCollector — lark-cli email
├── okr.rs            # FeishuOkrCollector — lark-cli okr
├── bitable.rs        # FeishuBitableCollector — lark-cli bitable
├── spreadsheets.rs   # FeishuSpreadsheetCollector — lark-cli sheets
├── wiki.rs           # FeishuWikiCollector — lark-cli wiki
└── minutes.rs        # FeishuMinutesCollector — lark-cli minutes
```

每个采集器：
- 实现 `Collector` trait
- 调用对应的 `lark-cli` 子命令
- 映射到对应的 `Source` 和 `EventType`
- 3 个测试

**验收**：
- [ ] 7 个采集器全部注册到 CollectorManager
- [ ] 每个有独立测试
- [ ] 21+ 测试

---

## Task 9：系统行为采集器

**目标**：采集 macOS 系统行为数据

**涉及特性**：F1.2.1, F1.2.2

**关键实现**：

```
wb-collector/src/system/
├── mod.rs
├── app_switch.rs     # AppSwitchCollector — 前台应用切换监听
└── browser.rs        # BrowserHistoryCollector — 浏览器历史
```

**AppSwitchCollector**：
- 使用 macOS Accessibility API 或 `osascript` 获取前台应用
- 记录应用名、切换时间、停留时长
- Source::System, EventType::AppSwitch

**BrowserHistoryCollector**：
- 读取 Chrome/Safari 历史记录 SQLite 数据库
- 记录 URL、页面标题、访问时间
- Source::System, EventType::BrowserHistory

**验收**：
- [ ] 应用切换正确采集
- [ ] 浏览器历史正确采集
- [ ] 隐私敏感数据脱敏
- [ ] 6+ 测试

---

## Task 10：快速捕获窗口

**目标**：全局热键唤起轻量输入窗口

**涉及特性**：F6.1.1, F6.1.2

**关键实现**：

```
src-tauri/src/
├── hotkey.rs         # 全局热键注册（Tauri global-shortcut 插件）
└── capture_window.rs # 第二窗口管理

src/capture/
├── CaptureWindow.tsx   # 快速捕获窗口组件
└── capture-window.css
```

**Tauri 配置**：
- 注册全局热键（默认 `Cmd+Shift+K`）
- 创建独立的轻量窗口（400x300，无边框，置顶）
- 支持文本输入 + 图片粘贴
- 提交后写入 EventLog

**验收**：
- [ ] 全局热键唤起窗口
- [ ] 文本输入并提交
- [ ] 图片粘贴支持
- [ ] 提交后正确写入

---

## Task 11：设置界面

**目标**：统一的配置管理界面

**涉及特性**：F6.3.1 ~ F6.3.6

**关键实现**：

```
src/components/settings/
├── SettingsView.tsx      # 设置页面
├── ModelSettings.tsx     # 模型配置
├── CollectorSettings.tsx # 采集器配置
├── StorageSettings.tsx   # 存储配置
├── HotkeySettings.tsx    # 热键配置
├── FreshnessSettings.tsx # 保鲜规则配置
└── ReportSettings.tsx    # 报告配置
```

**配置持久化**：SQLite `config` 表

**设置项**：
- 模型：API endpoint、API key、模型名、token 预算
- 采集器：各采集器开关、飞书认证
- 存储：Obsidian vault 路径、备份策略
- 热键：全局热键自定义
- 保鲜：各保鲜任务频率
- 报告：模板、定时、确认策略

**验收**：
- [ ] 设置页面可导航
- [ ] 配置读取和保存
- [ ] 配置变更实时生效
- [ ] 8+ 测试

---

## Task 12：调度器增强

**目标**：支持 cron 表达式、依赖管理、资源感知

**涉及特性**：F6.2.1, F6.2.2, F6.2.5

**关键实现**：

```
wb-scheduler/src/
├── cron.rs           # CronScheduler — cron 表达式解析和调度
├── dependency.rs     # DependencyGraph — 任务依赖管理
└── resource.rs       # ResourceAwareness — token 预算感知
```

**增强项**：
- cron 表达式支持（`cron` crate）
- 任务依赖图（DAG）
- 低预算时延迟低优先级任务
- 执行日志记录

**验收**：
- [ ] cron 表达式正确解析和触发
- [ ] 依赖任务按序执行
- [ ] 资源感知正确延迟
- [ ] 10+ 测试

---

## 依赖关系图

```
Task 1 (Obsidian文档层) ──┬──→ Task 6 (信息保鲜)
                          └──→ Task 7 (报告生成)
Task 2 (模型适配器) ──→ Task 3 (流水线串联) ──┬──→ Task 4 (SLA管理)
                                              └──→ Task 5 (全链路审计)
Task 8 (补充采集器) ──独立
Task 9 (系统采集器) ──独立
Task 10 (快速捕获) ──独立
Task 11 (设置界面) ──独立
Task 12 (调度器增强) ──独立
```

## 执行顺序

**第一批**（无依赖，并行）：
- Task 1: Obsidian 文档层
- Task 2: 模型适配器接入
- Task 8: 补充采集器
- Task 9: 系统行为采集器
- Task 12: 调度器增强

**第二批**（依赖第一批）：
- Task 3: 处理流水线串联
- Task 6: 信息保鲜引擎

**第三批**（依赖第二批）：
- Task 4: SLA 管理
- Task 5: 全链路审计
- Task 7: 报告生成系统

**第四批**（无强依赖，可并行）：
- Task 10: 快速捕获窗口
- Task 11: 设置界面

## 验收标准

- [ ] 所有 12 个任务完成
- [ ] 测试覆盖 ≥ 200（当前 138 + 新增 ~80）
- [ ] cargo build + cargo test 全绿
- [ ] pnpm build 前端正常
- [ ] 完整的 Event → 处理 → 审核 → 持久化 → 报告 流程
- [ ] docs/features/index.md 更新

## 风险

| 风险 | 等级 | 缓解 |
|------|------|------|
| macOS 系统 API 权限 | 中 | 使用 Accessibility API 需要用户授权 |
| lark-cli 新子命令不稳定 | 低 | 已有 5 个采集器验证过模式 |
| 报告生成质量依赖 LLM | 中 | 先用规则聚合，LLM 做润色 |
| 全局热键与系统冲突 | 低 | 可配置热键，有默认值 |
