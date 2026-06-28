---
title: 贡献指南
type: guide
domain: development
created: 2026-06-06
updated: 2026-06-28
status: active
---

# 贡献指南

## 快速开始

```bash
# 1. 克隆仓库
git clone <repo-url> work-better
cd work-better

# 2. 安装依赖
pnpm install  # 或 npm install / yarn

# 3. 运行开发环境 Setup
bash .claude/scripts/setup-dev.sh

# 4. 启动 Claude Code
claude
```

## 可用脚本

<!-- AUTO-GENERATED -->
> 以下表格从 `package.json` 和 `Cargo.toml` 自动生成，勿手动编辑。

### 前端 (pnpm)

| 命令 | 说明 |
|------|------|
| `pnpm dev` | 启动 Vite 开发服务器 |
| `pnpm build` | TypeScript 编译 + Vite 生产构建 |
| `pnpm preview` | 预览生产构建 |
| `pnpm tauri` | Tauri CLI 入口 |
| `pnpm test` | 运行单元测试（vitest run） |
| `pnpm test:unit` | 运行单元测试（同 test） |
| `pnpm test:unit:watch` | 单元测试 watch 模式 |
| `pnpm test:int` | 运行集成测试（vitest integration config） |
| `pnpm test:e2e` | 运行 E2E 测试（Playwright） |
| `pnpm test:all` | 运行全部测试（unit → int → e2e） |
| `pnpm test:coverage` | 运行单元测试并生成覆盖率报告 |
| `pnpm test:rust` | 运行 Rust 测试（cargo nextest） |
| `pnpm test:rust:ci` | 运行 Rust 测试（CI profile） |

### Rust Workspace (cargo)

| 命令 | 说明 |
|------|------|
| `cargo build` | 编译整个 workspace |
| `cargo nextest run --workspace` | 运行所有 crate 的测试 |
| `cargo clippy --workspace` | Rust 代码检查 |

### Rust Workspace 成员

| Crate | 职责 |
|-------|------|
| `src-tauri` | Tauri 桌面应用入口 |
| `crates/wb-core` | 核心数据结构（Event、Task、WorkRecord） |
| `crates/wb-collector` | 采集层（飞书、系统、手动） |
| `crates/wb-processor` | 处理层（分类、提取、审核、报告） |
| `crates/wb-storage` | 存储层（Obsidian、SQLite、向量DB） |
| `crates/wb-ai` | AI 模型路由、预算、适配器 |
| `crates/wb-scheduler` | 定时任务调度 |
| `tests/acceptance` | 验收测试 |
| `tests/integration` | 集成测试 |
| `tests/e2e` | 端到端测试 |
| `crates/wb-real-backend-tests` | 真实后端测试 |
<!-- /AUTO-GENERATED -->

## 开发环境要求

| 工具 | 最低版本 | 必需 |
|------|---------|------|
| Claude Code | 最新 | ✅ |
| Node.js | ≥ 18 | ✅ |
| pnpm / npm / yarn | - | ✅ |
| Git | ≥ 2.30 | ✅ |
| ECC 插件 | 最新 | ✅ |

### 安装 ECC 插件

本项目的开发工具链基于 [ECC (Everything Claude Code)](https://github.com/anthropics/ecc) 插件。ECC 提供了：

- **Agents** — 代码审查、安全分析、TDD 等专业 agent
- **Skills** — 可调用的技能（`/code-review`、`/build-fix` 等）
- **Hooks** — 代码质量守护（自动格式化、类型检查）
- **Scripts** — 工具脚本

**安装方式：**

```bash
# 方式 1: 通过 Claude Code 插件市场安装（推荐）
# 在 Claude Code 中执行
/install-plugin ecc

# 方式 2: 手动安装
# 参考 ECC 仓库的安装文档
```

安装后，ECC 会自动配置 hooks、agents、skills 等。项目级的编码规则（`.claude/rules/`）会自动生效。

### 配置 settings.json

复制模板并填入你的本地路径：

```bash
cp .claude/settings.template.json .claude/settings.json
# 编辑 settings.json，将 <你的项目路径> 替换为实际路径
```

或运行 setup 脚本自动处理：

```bash
bash .claude/scripts/setup-dev.sh
```

## 项目结构

```
work-better/
├── src-tauri/             # Tauri 桌面应用（Rust 后端）
│   └── src/               #   Tauri 入口、commands、窗口管理
├── crates/                # Rust 业务逻辑 crate
│   ├── wb-core/           #   核心数据结构（Event、WorkRecord、Task）
│   ├── wb-collector/      #   采集层（飞书、系统、手动）
│   ├── wb-processor/      #   处理层（分类、审核、报告）
│   ├── wb-storage/        #   存储层（Obsidian、向量DB、SQLite）
│   ├── wb-ai/             #   AI 模型路由与预算
│   ├── wb-scheduler/      #   定时任务调度
│   └── wb-real-backend-tests/ # 真实后端测试
├── src/                   # React 19 前端（TypeScript）
├── tests/                 # 测试（acceptance/integration/e2e）
├── CLAUDE.md              # Claude Code 入口
├── agent.md               # Agent 指南（核心思想、文档索引）
├── CONTRIBUTING.md         # 本文档
├── Cargo.toml             # Rust workspace 配置
├── package.json           # Node.js 项目配置
├── docs/                  # 产品与架构文档
└── .claude/               # Claude Code 配置
    ├── rules/             # 编码规范（✅ 纳入 Git）
    ├── contexts/          # 上下文模板（✅ 纳入 Git）
    ├── settings.template.json  # 配置模板（✅ 纳入 Git）
    ├── settings.json      # 个人配置（❌ gitignore）
    ├── agents/            # 项目自定义 agent（多 Agent 协作体系）
    ├── skills/            # 可调用技能
    ├── hooks/             # 质量守护 hooks
    └── scripts/           # 工具脚本
```

## 编码规范

本项目使用分层编码规范，**语言特定规范优先于通用规范**：

- `common/` — 语言无关的通用原则（始终生效）
- `zh/` — 通用原则的中文翻译
- `typescript/`、`python/`、`web/` 等 — 语言特定规范

所有规范以 Markdown 形式定义在 `.claude/rules/` 中，Claude Code 启动时自动加载。

## 开发工作流

```
研究复用 → 先规划 → TDD → 代码审查 → 提交
```

1. **研究复用** — 搜索现有实现，优先复用
2. **先规划** — 使用 `/plan` 或 planner agent
3. **TDD** — 先写测试，再实现，再重构
4. **代码审查** — review-agent 自动审查
5. **提交** — 遵循 conventional commits 格式

## 提交规范

```
<type>: <description>

<optional body>
```

类型：`feat`、`fix`、`refactor`、`docs`、`test`、`chore`、`perf`、`ci`

## 文档自维护

代码变更时，检查是否需要同步更新文档：

| 变更类型 | 需要更新的文档 |
|---------|--------------|
| 新增/移除功能 | 功能索引 + 对应模块文档 |
| 架构变更 | 架构总览 + 对应模块文档 |
| 新增/移除模块 | 架构总览 + agent.md 文档索引 |
| 数据模型变更 | 事件模型文档 + 对应模块文档 |

详见 `agent.md` 中的「文档自维护规范」。

## 文档规范

所有文档遵循 [文档规范](docs/conventions.md)：
- 必须有 frontmatter
- 新增文档需更新对应 `_index.md`
- 单文件不超过 300 行
- 文档与代码变更同 PR

## 常见问题

### Q: Claude Code 没有加载项目配置？

确认在项目根目录下启动 `claude`，`CLAUDE.md` 会自动被发现。

### Q: ECC hooks 报错？

确保已安装 ECC 插件。临时禁用：

```bash
export ECC_HOOK_PROFILE=minimal
```

### Q: 想添加新的语言规则？

参考 `.claude/rules/README.md` 中的「Adding a New Language」章节。
