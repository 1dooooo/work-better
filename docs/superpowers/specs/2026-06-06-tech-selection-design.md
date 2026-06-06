---
title: 技术选型设计
type: spec
domain: architecture
created: 2026-06-06
updated: 2026-06-06
status: draft
---

# Work Better - 技术选型设计

> **维护说明**：当技术栈、工具选型或项目结构调整时更新本文档。
> 技术选型变更需评估对所有模块的影响，必要时同步更新架构文档。

## 背景

Work Better 是一个 macOS 桌面菜单栏应用，以 Obsidian 为中心，被动采集飞书等信息源的工作数据，自动整理成工作记录和任务。

本文档记录技术选型决策及理由，作为开发实施的技术基础。

## 核心架构决策

### 决策 1：Tauri 2.x + Rust 后端

**选择**：Tauri 2.x，业务逻辑全部用 Rust 实现。

**理由**：
- Tauri 设计为 Rust + WebView 两层架构，引入 Node sidecar 会增加不必要的第三层
- Rust 后端内存占用低（50-100MB），适合菜单栏常驻应用
- 最终产物为单个 Mac .dmg，体积小（~15MB vs Electron ~150MB）
- rusqlite、tokio、reqwest 等 Rust 生态成熟，覆盖所有业务需求

**备选方案**：
- Electron + Node：体积大、内存高，但 Node 生态丰富
- Tauri + Node sidecar：架构审查发现通信开销、进程管理复杂度、调试困难等问题
- Swift 原生：需要维护两套技术栈

**职责分工**：

| 职责 | 运行时 | 说明 |
|------|--------|------|
| 系统托盘、窗口管理 | Tauri (Rust) | 原生支持 |
| 全局快捷键 | Tauri (Rust) | 原生支持 |
| 系统通知 | Tauri (Rust) | 原生支持 |
| SQLite 访问 | Rust (rusqlite) | 原生绑定，零编译问题 |
| 向量搜索 | Rust (sqlite-vec) | SQLite 扩展 |
| AI 模型调用 | Rust (reqwest) | HTTP 客户端 |
| 飞书采集 | Rust (std::process) | 调用 lark-cli |
| Obsidian 写入 | Rust (std::process) | 调用 obsidian-cli，降级为文件 I/O |
| 定时任务调度 | Rust (tokio) | 异步运行时 |
| 前端 UI | React + TypeScript | Tauri WebView |

### 决策 2：React 19 + TypeScript 前端

**选择**：React 19 + TypeScript，Vite 6 构建。

**理由**：
- 生态最大，社区资源丰富
- .claude/rules 中已有详细的 React 规则配置
- Vite 是 Tauri 官方推荐的前端构建工具
- TypeScript 提供类型安全，与 Rust 侧通过 ts-rs 保持类型同步

### 决策 3：rusqlite + sqlite-vec 数据存储

**选择**：
- SQLite：rusqlite（Rust 原生绑定）
- 向量搜索：sqlite-vec（SQLite 向量扩展）

**理由**：
- rusqlite 是 Rust 生态中最成熟的 SQLite 库，零原生模块编译问题
- sqlite-vec 与 SQLite 完全集成，无需额外服务
- 桌面应用场景下数据量（10K-100K 文档）性能足够

**风险与缓解**：
- sqlite-vec 是较新项目（2024 年开源），社区规模小
- 缓解：向量搜索抽象为 trait，确保可替换

**备选方案**：
- sql.js（WASM）：无原生依赖但性能略低
- Chroma/Qdrant：成熟但需要额外服务，不适合桌面应用

### 决策 4：lark-cli 飞书采集

**选择**：通过 Rust std::process 调用 lark-cli 获取飞书数据。

**理由**：
- 飞书官方 CLI，覆盖 18 个业务域、200+ 命令
- OAuth 登录，比 SDK 的应用权限更容易获取
- 不需要维护飞书 API 适配层

**风险与缓解**：
- CLI 输出格式可能随版本变化
- 缓解：用 serde 对 CLI 输出做 schema 校验，CI 中锁定 lark-cli 版本
- 封装为 CommandRunner，统一处理超时、重试、输出解析、错误标准化

### 决策 5：obsidian-cli + 文件 I/O 降级

**选择**：Obsidian 运行时优先用 obsidian-cli，未运行时降级为直接文件读写。

**理由**：
- 用户本地已安装 obsidian-cli，操作更丰富（任务管理、模板等）
- Obsidian vault 本质是 markdown 文件，文件 I/O 作为可靠降级方案
- 启动时检测 Obsidian 是否运行，决定使用哪种方式

**实现策略**：
```
1. 检测 Obsidian 是否运行（进程检查或 obsidian-cli vault 命令）
2. 运行中 → 使用 obsidian-cli
3. 未运行 → 直接 fs 读写 vault 目录
4. 用户首次使用 → 引导安装 Obsidian 和 obsidian-cli
```

### 决策 6：自建 AI 适配层

**选择**：Rust 侧用 reqwest 调用 AI API，支持 Anthropic 和 OpenAI 两种格式。

**理由**：
- 大部分厂商兼容这两种 API 格式之一
- 自建适配层控制模型切换、重试、token 预算
- 统一在 Rust 侧处理，避免跨运行时调用

**接口设计**：
```rust
trait ModelAdapter {
    async fn classify(&self, input: &str, schema: &Schema) -> Result<Classification>;
    async fn extract(&self, input: &str, schema: &Schema) -> Result<Extraction>;
    async fn summarize(&self, input: &str, max_len: usize) -> Result<String>;
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
}

struct AnthropicAdapter { /* ... */ }
struct OpenAIAdapter { /* ... */ }
```

## 项目结构

### 目录布局

```
work-better/
├── src-tauri/                    # Tauri 应用主 crate
│   ├── src/
│   │   ├── main.rs               # 入口
│   │   ├── commands/             # Tauri commands（前端调用入口）
│   │   ├── tray/                 # 系统托盘
│   │   └── window/               # 窗口管理
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── crates/                       # Rust workspace crates
│   ├── wb-core/                  # 核心领域模型
│   │   └── src/
│   │       ├── event.rs          # Event, EventLog trait
│   │       ├── record.rs         # WorkRecord
│   │       ├── task.rs           # Task, TaskStatus
│   │       ├── project.rs        # Project
│   │       └── audit.rs          # ProcessingAudit
│   │
│   ├── wb-collector/             # 采集层
│   │   └── src/
│   │       ├── manager.rs        # CollectorManager
│   │       ├── feishu/           # 飞书采集器（调 lark-cli）
│   │       ├── system/           # 系统行为采集器
│   │       └── capture/          # 用户手动采集器
│   │
│   ├── wb-processor/             # 处理层
│   │   └── src/
│   │       ├── classifier.rs     # 分类器
│   │       ├── router.rs         # ModelRouter
│   │       ├── review.rs         # ReviewAgent
│   │       ├── paths/            # 四条处理路径
│   │       └── audit.rs          # 审计记录
│   │
│   ├── wb-storage/               # 存储层
│   │   └── src/
│   │       ├── obsidian/         # Obsidian 集成
│   │       │   ├── cli.rs        # obsidian-cli 调用
│   │       │   ├── fs.rs         # 文件 I/O 降级
│   │       │   └── detect.rs     # Obsidian 运行状态检测
│   │       ├── sqlite/           # SQLite（rusqlite）
│   │       │   ├── event_log.rs  # EventLog 实现
│   │       │   ├── tasks.rs      # 任务存储
│   │       │   └── audits.rs     # 审计存储
│   │       ├── vector/           # 向量数据库（sqlite-vec）
│   │       └── freshness/        # 信息保鲜引擎
│   │
│   ├── wb-ai/                    # AI 模型适配层
│   │   └── src/
│   │       ├── adapter.rs        # ModelAdapter trait
│   │       ├── anthropic.rs      # Anthropic 格式
│   │       ├── openai.rs         # OpenAI 格式
│   │       └── budget.rs         # Token 预算管理
│   │
│   └── wb-scheduler/             # 定时任务调度器
│       └── src/
│           ├── scheduler.rs      # tokio-cron-scheduler 封装
│           └── tasks/            # 各层定时任务实现
│
├── src/                          # React 前端
│   ├── app/                      # 应用入口与路由
│   ├── components/               # UI 组件
│   │   ├── menu-bar/             # 菜单栏
│   │   ├── main-window/          # 主窗口
│   │   ├── quick-capture/        # 速记窗口
│   │   └── ui/                   # 通用组件
│   ├── hooks/                    # React hooks
│   ├── lib/                      # 工具函数
│   └── styles/                   # 全局样式
│
├── Cargo.toml                    # Rust workspace root
├── package.json                  # pnpm（仅前端依赖）
├── pnpm-lock.yaml
├── vite.config.ts
├── tsconfig.json
└── vitest.config.ts
```

### Crate 依赖图

```
wb-core           ← 所有 crate 依赖（纯类型，零运行时依赖）
wb-ai             ← wb-processor, wb-storage
wb-collector      ← wb-core（std::process 调 lark-cli）
wb-processor      ← wb-core, wb-ai
wb-storage        ← wb-core, wb-ai（rusqlite + sqlite-vec + 文件 I/O）
wb-scheduler      ← wb-collector, wb-processor, wb-storage
src-tauri         ← 所有 crate（Tauri commands 入口）
src/ (React)      ← 通过 Tauri IPC 调用 src-tauri commands
```

### 依赖关系原则

- **单向依赖**：crate 之间只允许自下而上的依赖，禁止循环依赖
- **core 零依赖**：wb-core 只定义类型和 trait，不依赖任何运行时
- **trait 解耦**：crate 间通过 trait 交互，不直接依赖具体实现
- **前端隔离**：React 前端只通过 Tauri commands 与 Rust 通信，不直接调用 crate

## Rust 与 TypeScript 类型同步

**方案**：使用 ts-rs crate 从 Rust struct 自动生成 TypeScript 类型定义。

```rust
// wb-core/src/event.rs
use ts_rs::TS;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Event {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub source: Source,
    pub event_type: EventType,
    pub content: serde_json::Value,
    // ...
}
```

构建时自动生成 `src/types/generated/Event.ts`，前端直接 import 使用。

## 构建与开发

### 开发环境要求

- Rust 1.75+ (stable)
- Node.js 18+ (仅前端构建)
- pnpm 8+
- lark-cli（飞书数据采集）
- obsidian-cli（Obsidian 操作，可选）

### 构建命令

```bash
# 开发模式（Tauri dev，前端 HMR + Rust 热编译）
pnpm tauri dev

# 生产构建（生成 .dmg）
pnpm tauri build

# 仅前端
pnpm vite build

# 仅 Rust
cargo build --release

# 测试
cargo test              # Rust 测试
pnpm vitest             # 前端测试
```

### CI/CD

- GitHub Actions
- macOS runner（arm64 + x64）
- Rust 缓存（cargo cache）
- pnpm 缓存
- 生成 .dmg artifact

## 测试策略

### Rust 侧

| 测试类型 | 工具 | 说明 |
|---------|------|------|
| 单元测试 | cargo test | 每个 crate 内部测试 |
| Mock | mockall | trait mock，隔离外部依赖 |
| 集成测试 | cargo test --test | crate 间交互测试 |
| CLI mock | 模拟 lark-cli 输出 | 不依赖真实飞书 API |

### 前端侧

| 测试类型 | 工具 | 说明 |
|---------|------|------|
| 单元测试 | Vitest | 工具函数、hooks |
| 组件测试 | Vitest + Testing Library | React 组件 |
| E2E | Playwright | 关键用户流程 |

### Tauri commands 测试

通过 Tauri 的测试工具验证 Rust command 的输入输出，确保前后端契约正确。

## 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| sqlite-vec 成熟度不足 | 向量搜索不稳定 | 抽象为 trait，可替换 |
| lark-cli 输出格式变化 | 采集解析失败 | serde schema 校验 + 版本锁定 |
| obsidian-cli 不可用 | Obsidian 写入失败 | 降级为文件 I/O |
| AI API 不可用 | 处理层 SLA 无法保证 | 队列化 + 指数退避重试 |
| SQLite 写竞争 | SQLITE_BUSY 错误 | 写操作队列化，考虑分库 |
| Rust 开发速度慢 | 交付周期长 | 核心逻辑优先，渐进式完善 |

## 实施阶段

### Phase 1：最小可用

- Tauri 应用框架 + 系统托盘
- rusqlite EventLog 基础实现
- lark-cli 飞书消息采集（1 个子采集器）
- Obsidian 文件写入（直接 I/O）
- AI 适配层（单模型）
- 前端菜单栏基础 UI

### Phase 2：核心功能

- 完整采集层（12 个飞书子采集器）
- 处理层（分类、提取、审核）
- obsidian-cli 集成 + 降级
- 向量搜索（sqlite-vec）
- 定时任务基础调度
- 主窗口 UI

### Phase 3：完善体验

- 信息保鲜引擎
- 完整定时任务系统（30+ 任务）
- 报告生成
- 速记窗口
- 设置界面

## 相关文档

- [架构总览](../architecture/overview.md)
- [采集层](../architecture/modules/collection.md)
- [处理层](../architecture/modules/processing.md)
- [存储层](../architecture/modules/storage.md)
- [呈现层](../architecture/modules/presentation.md)
- [定时任务](../architecture/modules/scheduler.md)
- [事件模型](../architecture/modules/event-model.md)
