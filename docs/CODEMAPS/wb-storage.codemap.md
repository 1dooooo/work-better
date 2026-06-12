---
title: wb-storage CODEMAP
type: codemap
domain: architecture
crate: wb-storage
created: 2026-06-12
updated: 2026-06-12
status: active
---

# wb-storage CODEMAP

> **职责**：数据存储层。三层存储架构（Obsidian/向量DB/SQLite），信息保鲜，配置管理。
> **对应文档**：[存储层架构](../architecture/modules/storage.md)

## 文件导航

### 配置

| 文件 | 职责 | 关键类型 |
|------|------|---------|
| `lib.rs` | 模块导出 + pub use | — |
| `config.rs` | 应用配置管理 | `AppConfig`, `StorageConfig`, `CollectorConfig`, `ModelConfig`, `SchedulerConfig`, `ScheduledTaskConfig` |

### Obsidian 层 (`obsidian/`)

| 文件 | 职责 | 关键类型 |
|------|------|---------|
| `mod.rs` | Obsidian 子模块导出 | — |
| `writer.rs` | Obsidian 文档写入器 | `ObsidianWriter` — 核心写入接口 |
| `vault.rs` | Vault 操作 | 目录创建、文件读写、路径管理 |
| `daily.rs` | 日记生成 | 按日期生成日记文件 |
| `project.rs` | 项目目录 | 按项目组织的目录结构 |
| `template.rs` | 模板系统 | 会议、任务、报告等模板渲染 |
| `links.rs` | 双向链接 | 自动建立 `[[wiki-link]]` 关联 |
| `tags.rs` | 标签管理 | 多维度分类标签 |

### SQLite 层 (`sqlite/`)

| 文件 | 职责 | 关键类型 |
|------|------|---------|
| `mod.rs` | SQLite 子模块导出 | — |
| `schema.rs` | 数据库 schema | 表定义、索引、迁移 |
| `event_log.rs` | EventLog 实现 | `SqliteEventLog` — 实现 `EventLog` trait |
| `audit_log.rs` | 审计日志存储 | `AuditLogStore`, `AuditQueryFilter`, `ExecutionLogInsert`, `ExecutionLogFilter`, `ProcessingAuditInsert` |

### 向量数据库层 (`vector/`)

| 文件 | 职责 | 关键类型 |
|------|------|---------|
| `mod.rs` | 向量子模块导出 | — |
| `store.rs` | 向量存储 | 向量 DB 的读写接口 |
| `embedding.rs` | Embedding 生成 | 文本向量化 |
| `search.rs` | 语义搜索 | 相似度查询 |
| `sync.rs` | 增量同步 | 文档变更后重新 embedding |

### 信息保鲜 (`freshness/`)

| 文件 | 职责 | 关键类型 |
|------|------|---------|
| `mod.rs` | 保鲜子模块导出 | — |
| `sync.rs` | 同步类保鲜任务 | 任务状态同步、日历同步、文档变更检测 |
| `integrity.rs` | 完整性类保鲜任务 | 双向链接检查、三层一致性 |
| `quality.rs` | 质量类保鲜任务 | 重复检测、标签规范化、知识过时检测 |
| `report.rs` | 保鲜报告 | 每次执行后的报告生成 |

### 其他

| 文件 | 职责 | 关键类型 |
|------|------|---------|
| `sync_log.rs` | 三层同步日志 | 同步状态追踪 |

### 测试文件 (`tests/`)

| 文件 | 职责 |
|------|------|
| `g1_information_collection.rs` | G1 验收测试：信息采集 |
| `g2_intelligent_processing.rs` | G2 验收测试：智能处理 |
| `g3_obsidian_output.rs` | G3 验收测试：Obsidian 输出 |
| `g4_task_management.rs` | G4 验收测试：任务管理 |
| `g5_scheduling_system.rs` | G5 验收测试：调度系统 |
| `g6_menu_bar.rs` | G6 验收测试：菜单栏 |
| `g7_configuration.rs` | G7 验收测试：配置管理 |
| `sqlite_event_log.rs` | SQLite EventLog 单元测试 |
| `obsidian_writer.rs` | ObsidianWriter 测试 |
| `freshness.rs` | 保鲜引擎测试 |
| `vector_db.rs` | 向量 DB 测试 |
| `tauri_*.rs` | Tauri 命令集成测试 |
| `acceptance_helpers.rs` | 验收测试辅助函数 |
| `manual_capture.rs` | 手动捕获测试 |
| `real_backend_*.rs` | 真实后端测试 |

## 关键设计

- **三层存储**：Obsidian（人读）→ 向量DB（语义检索）→ SQLite（机器查询）
- **Obsidian 是唯一真相源**：向量 DB 和 SQLite 是派生索引
- **写入顺序**：先 Obsidian → 再向量 DB → 最后 SQLite
- **信息保鲜**：定期检查数据新鲜度，自动修复或推送用户确认

## 修改指引

| 你想改什么 | 先读 | 再改 |
|-----------|------|------|
| 修改 Obsidian 文档格式 | `obsidian/template.rs` + `obsidian/writer.rs` | 修改模板或写入逻辑 |
| 新增 SQLite 表 | `sqlite/schema.rs` | 添加表定义 + 迁移 |
| 修改保鲜策略 | `freshness/` 对应文件 | 调整检测逻辑 |
| 修改应用配置 | `config.rs` | 添加配置字段 |
| 修改向量搜索 | `vector/search.rs` | 调整搜索逻辑 |
