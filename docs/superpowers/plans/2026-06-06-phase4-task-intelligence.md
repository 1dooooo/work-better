---
title: Phase 4 — 任务智能
date: 2026-06-06
status: planning
goal: 从"信息流转"到"任务驱动"——让系统能发现、追踪、管理任务
phase: 4
depends_on:
  - 2026-06-06-phase1-mvp.md
  - 2026-06-06-phase2-core.md
  - 2026-06-06-phase3-deep-insight.md
---

# Phase 4：任务智能

> 从"信息流转"到"任务驱动"——让系统能发现、追踪、管理任务，并通过向量搜索实现语义理解。

## 前置条件

Phase 1 + Phase 2 + Phase 3 全部完成（391 tests, 68 features ✅）

## 任务总览

| # | 任务 | 层 | 依赖 | 估时 | 功能点 |
|---|------|----|------|------|--------|
| 1 | 向量数据库层 | storage | — | 4h | F3.2.1-F3.2.5 |
| 2 | 任务管理核心 | processor+storage | — | 4h | F4.1.1, F4.1.4, F4.1.5 |
| 3 | 任务自动发现 | processor | 2 | 4h | F4.2.1-F4.2.5 |
| 4 | 审核代理进阶 | processor | — | 3h | F2.4.2-F2.4.5 |
| 5 | 飞书任务同步 | collector+processor | 2,3 | 4h | F4.3.1-F4.3.3 |
| 6 | 报告进阶 | processor | — | 3h | F5.1.4-F5.1.6, F5.2.4-F5.2.5 |
| 7 | 系统增强 | ui+scheduler | — | 3h | F6.1.3, F6.2.7-F6.2.8, F6.3.4-F6.3.6 |
| 8 | 系统采集器集成 | collector | — | 2h | F1.2.1-F1.2.2, F1.3.2-F1.3.3, F1.4.4 |

**总计：~27h，8 个任务，覆盖 40 个功能点**

---

## 详细设计

### Task 1: 向量数据库层

**目标**：文档 Embedding + 语义搜索 + RAG 召回

**技术选型**：SQLite + `sqlite-vec` 扩展（嵌入式向量搜索，无需外部服务）

**新增文件**：
```
crates/wb-storage/src/vector/
├── mod.rs          — VectorStore trait + 配置
├── embedding.rs    — EmbeddingEngine: 文本向量化（调用 OpenAI/local model）
├── store.rs        — SqliteVectorStore: CRUD + 相似度搜索
├── search.rs       — SemanticSearch: 语义搜索 + RAG 召回
└── sync.rs         — VectorSync: 增量同步（文档变更触发 re-embedding）
```

**核心接口**：
```rust
pub trait VectorStore {
    async fn embed(&self, doc_id: &str, content: &str) -> Result<Vec<f32>>;
    async fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>>;
    async fn similar(&self, doc_id: &str, top_k: usize) -> Result<Vec<SearchResult>>;
    async fn rag_context(&self, query: &str, max_tokens: usize) -> Result<String>;
    async fn sync_changed(&self, changed_docs: &[String]) -> Result<SyncReport>;
}
```

**测试**：embedding 存取、相似度搜索排序、增量同步、RAG 上下文拼接

---

### Task 2: 任务管理核心

**目标**：任务 CRUD + 父子层级 + 归档

**新增文件**：
```
crates/wb-processor/src/task/
├── mod.rs          — TaskManager 入口
├── model.rs        — Task, TaskStatus, TaskPriority, TaskSource
├── create.rs       — 手动创建 + 自动发现创建
├── lifecycle.rs    — 状态流转 (open → in_progress → done → archived)
└── hierarchy.rs    — 父子任务、子任务拆分
```

**核心数据模型**：
```rust
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub source: TaskSource,        // Manual, Meeting, Message, Email, Document
    pub parent_id: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub feishu_task_id: Option<String>,
    pub obsidian_path: Option<String>,
}
```

**测试**：创建/更新/归档、父子关系、状态流转约束

---

### Task 3: 任务自动发现

**目标**：从会议、消息、邮件、文档中自动提取任务

**新增文件**：
```
crates/wb-processor/src/task/
├── discovery.rs         — TaskDiscovery: 统一发现入口
├── discovery_meeting.rs — 从会议纪要/妙记提取待办
├── discovery_message.rs — 从聊天消息识别任务
├── discovery_email.rs   — 从邮件识别请求和承诺
└── discovery_confirm.rs — 自动发现确认流（pending → user confirm → create）
```

**核心流程**：
1. 事件进入处理流水线 → TaskDiscovery 检查是否含任务信号
2. 使用 AI 模型提取任务候选（标题、优先级、截止日期）
3. 候选任务进入 pending 状态 → 推送用户确认
4. 用户确认后创建正式任务

**测试**：各来源的任务提取、确认流、去重

---

### Task 4: 审核代理进阶

**目标**：分层审核 + 用户确认推送

**修改文件**：`crates/wb-processor/src/review.rs`

**新增逻辑**：
- `SmallModelReview`: 小模型做一致性检查、关键信息覆盖度
- `LargeModelReview`: 大模型做复杂摘要的语义审核
- `TieredReview`: 按输出类型决定审核力度（简单摘要→小模型，复杂分析→大模型）
- `UserConfirmPush`: 涉及共享数据时推送用户确认

**测试**：分层策略选择、确认推送触发条件

---

### Task 5: 飞书任务同步

**目标**：飞书 ↔ Obsidian 双向任务同步 + 冲突处理

**新增文件**：
```
crates/wb-collector/src/feishu/
└── task_sync.rs    — FeishuTaskSync: 双向同步

crates/wb-processor/src/task/
└── sync.rs         — TaskSync: 同步编排 + 冲突处理
```

**同步策略**：
- 飞书 → Obsidian：定时拉取变更，自动更新
- Obsidian → 飞书：检测本地变更，推送确认后同步
- 冲突处理：时间戳优先 + 用户裁决

**测试**：单向同步、冲突检测、确认流

---

### Task 6: 报告进阶

**目标**：季报/半年报/年报 + 报告导出 + 飞书同步

**新增文件**：
```
crates/wb-processor/src/report/
├── quarterly.rs    — 季报: OKR 进度、项目里程碑、能力成长
├── semi_annual.rs  — 半年报: 阶段性总结、目标调整建议
├── annual.rs       — 年报: 年度全景、成长轨迹、下年规划
└── export.rs       — 报告导出: Markdown / PDF
```

**测试**：各周期报告生成、导出格式验证

---

### Task 7: 系统增强

**目标**：系统通知 + 执行日志 + 设置界面完善

**新增/修改**：
```
crates/wb-scheduler/src/log.rs       — ExecutionLog: 执行状态和结果记录
src-tauri/src/commands/notify.rs     — 系统通知 (tauri::notification)
src/components/settings/
├── ShortcutSettings.tsx              — 快捷键配置
├── FreshnessSettings.tsx             — 保鲜规则配置
└── ReportSettings.tsx                — 报告配置
```

**测试**：通知触发、日志记录、设置保存

---

### Task 8: 系统采集器集成

**目标**：完善系统采集器 + 图片/截图采集

**修改**：
- `app_switch.rs` / `browser.rs`：标记为 ✅（代码已存在）
- 新增 `capture.rs`：截图捕获（`screencapture` 命令）
- 新增 `image_paste.rs`：图片粘贴处理
- 飞书接入方式选择 UI

**测试**：采集器注册、截图命令调用

---

## 执行顺序

```
批次1（并行）: Task 1, Task 2, Task 4, Task 6, Task 7, Task 8
批次2（依赖批次1）: Task 3 (依赖 Task 2)
批次3（依赖批次2）: Task 5 (依赖 Task 2, Task 3)
```

## 验收标准

- [ ] 40 个 ⬜ 功能全部标记为 ✅
- [ ] 所有新增测试通过（目标 550+ tests）
- [ ] Clippy 无警告
- [ ] 前端构建正常
- [ ] cargo test 全绿
