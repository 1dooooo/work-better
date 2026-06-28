---
title: 开发环境搭建
type: guide
domain: development
created: 2026-06-06
updated: 2026-06-28
status: active
---

# 开发环境搭建指南

本文档帮助新加入的协作者快速搭建 Work Better 项目的开发环境。

## 前置条件

| 工具 | 最低版本 | 说明 |
|------|---------|------|
| **Claude Code** | 最新 | 本项目的核心开发工具，CLI 或 IDE 插件均可 |
| **Node.js** | ≥ 18 | 用于运行 hooks 脚本和项目工具链 |
| **pnpm** | ≥ 8 | 推荐的包管理器（亦可使用 npm/yarn） |
| **Git** | ≥ 2.30 | 版本控制 |
| **Obsidian** | 最新 | 产品核心载体，用于验证笔记输出效果 |

## 快速开始

### 1. 克隆仓库

```bash
git clone <repo-url> work-better
cd work-better
```

### 2. 运行 Setup 脚本

```bash
bash .claude/scripts/setup-dev.sh
```

脚本会自动完成：
- 检查前置工具是否已安装
- 安装项目依赖
- 按需安装语言特定的编码规则
- 验证 Claude Code hooks 是否正常

### 3. 打开 Claude Code

```bash
# CLI 方式
claude

# 或在 VS Code / JetBrains 中使用 Claude Code 插件
```

Claude Code 启动后会自动加载：
- `CLAUDE.md` → 项目入口指令
- `.claude/settings.json` → hooks、环境变量、插件配置（setup 脚本自动生成）
- `.claude/rules/` → 编码规范（按语言分层，已纳入 Git）
- `.claude/agents/` → 项目自定义 agent（多 Agent 协作体系）
- `.claude/skills/` → 可调用的技能

## 项目结构概览

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
├── CLAUDE.md              # Claude Code 入口（指向 agent.md）
├── agent.md               # Agent 指南：核心思想、文档索引、自维护规范
├── CONTRIBUTING.md         # 贡献指南（入口文档）
├── Cargo.toml             # Rust workspace 配置
├── package.json           # Node.js 项目配置
├── docs/                  # 产品与架构文档
│   ├── product/           # 产品文档
│   ├── architecture/      # 架构文档
│   ├── features/          # 功能索引
│   └── development/       # 开发指引（本文档）
└── .claude/               # Claude Code 配置
    ├── rules/             # ✅ 编码规范（纳入 Git）
    │   ├── common/        #   通用规范（语言无关）
    │   ├── zh/            #   中文翻译版
    │   ├── typescript/    #   TypeScript 特定
    │   └── ...            #   更多语言
    ├── contexts/          # ✅ 上下文模板（纳入 Git）
    ├── settings.template.json  # ✅ 配置模板（纳入 Git）
    ├── settings.json      # ❌ 个人配置（gitignore，从模板生成）
    ├── agents/            # ❌ ECC agent（gitignore，自行安装 ECC）
    ├── skills/            # ❌ ECC skills（gitignore，自行安装 ECC）
    ├── hooks/             # ❌ ECC hooks（gitignore，自行安装 ECC）
    ├── commands/          # ❌ ECC commands（gitignore，自行安装 ECC）
    ├── scripts/           # ❌ ECC scripts（gitignore，自行安装 ECC）
    └── mcp-configs/       # ❌ MCP 配置（gitignore）
```

### 提交策略

`.claude/` 目录**部分提交**：

| 内容 | 是否提交 | 说明 |
|------|---------|------|
| `rules/` | ✅ | 项目定义的编码规范，所有人共享 |
| `contexts/` | ✅ | 上下文模板 |
| `settings.template.json` | ✅ | 配置模板 |
| `settings.json` | ❌ | 包含本地路径，每人不同 |
| `agents/`、`skills/`、`hooks/`、`commands/`、`scripts/` | ❌ | ECC 插件产物，开发者需自行安装 |
| `sessions/` | ❌ | 运行时产物 |

### 安装 ECC 插件

本项目的开发工具链基于 ECC 插件。安装后会自动获得 agents、skills、hooks 等。

```bash
# 在 Claude Code 中执行
/install-plugin ecc
```

或参考 [ECC 仓库](https://github.com/anthropics/ecc) 的安装文档。

## `.claude/` 目录说明

本项目的核心开发基础设施。**部分纳入 Git 管理**，ECC 插件产物需开发者自行安装。

### settings.json

由 setup 脚本从 `settings.template.json` 自动生成，包含：
- **env**：环境变量（自动填入本地路径）
- **hooks**：ECC 插件提供的质量守护

> ⚠️ `settings.json` 已 gitignore，每人不同。修改配置请更新 `settings.template.json`。

### rules/ — 编码规范

分层设计，按需加载：

| 层级 | 说明 |
|------|------|
| `common/` | 语言无关的通用原则（始终生效） |
| `frontend/` | 前端特定规范（React/TypeScript） |
| `rust/` | Rust 特定规范 |
| `workflow.md` | 多 Agent 工作流规则 |

**语言特定规范优先于通用规范**（同 CSS 特异性）。

### agents/ — 项目自定义 Agent

项目定义了一套多 Agent 协作体系，通过 `Agent` 工具自动调度或手动使用：

| Agent | 用途 |
|-------|------|
| `workflow-advisor` | 流程顾问——分析任务、制定执行计划、监督流程 |
| `dev-agent` | 开发 agent——代码实现 |
| `test-agent` | 测试 agent——测试编写与验证 |
| `review-agent` | 代码审查 agent |
| `product-reviewer` | 产品审查 agent——从用户视角验证 |
| `validator` | 验证 agent——功能正确性验证 |
| `system-inspector` | 系统巡检 agent——全局一致性检查 |
| `optimizer` | 优化 agent——性能与代码质量优化建议 |

### skills/ — 可调用技能（ECC 提供）

需先安装 ECC 插件。通过 `/skill-name` 形式调用，例如：
- `/code-review` — 代码审查
- `/build-fix` — 修复构建错误
- `/feature-dev` — 功能开发全流程

### hooks/ — 质量守护（ECC 提供）

需先安装 ECC 插件。自动在工具执行前后运行的检查：

| 类型 | 说明 |
|------|------|
| PreToolUse | 工具执行前（可阻止危险操作） |
| PostToolUse | 工具执行后（自动格式化、类型检查） |
| Stop | 会话结束时（最终验证） |

详细说明见 `.claude/hooks/README.md`。

## 开发工作流

### 标准流程

```
研究复用 → 先规划 → TDD → 代码审查 → 提交
```

1. **研究复用**：搜索现有实现，优先复用
2. **先规划**：使用 `/plan` 或 planner agent
3. **TDD**：先写测试，再实现，再重构
4. **代码审查**：review-agent 自动审查
5. **提交**：遵循 conventional commits 格式

### Hooks 行为

开发过程中 hooks 会自动：
- 拦截危险的 bash 命令（如非 tmux 下运行 dev server）
- 编辑文件后自动格式化（Prettier）
- 编辑 TypeScript 后自动类型检查
- 检测 `console.log` 调试语句
- 在适当时机建议手动 `/compact`

### Hook 配置控制

通过环境变量控制 hook 行为（无需修改 `hooks.json`）：

```bash
# minimal | standard | strict（默认: standard）
export ECC_HOOK_PROFILE=standard

# 禁用特定 hook
export ECC_DISABLED_HOOKS="pre:bash:tmux-reminder"

# 禁用 GateGuard（setup 或恢复期间）
export ECC_GATEGUARD=off
```

## 文档自维护

本项目的文档采用 AI 自维护模式，详见 `agent.md` 中的「文档自维护规范」。

核心原则：
- **代码变更时检查是否触发文档更新**
- **先更新文档，再提交代码**
- **架构图、数据模型必须与代码一致**

## 常见问题

### Q: Claude Code 没有加载项目配置？

确认你在项目根目录下启动 Claude Code，`CLAUDE.md` 会自动被发现。

### Q: hooks 运行报错？

```bash
# 检查 Node.js 版本
node --version  # 需要 ≥ 18

# 临时禁用所有 hooks
export ECC_HOOK_PROFILE=minimal
```

### Q: 想添加新的语言规则？

在 `.claude/rules/` 下新建子目录，放入 Markdown 规则文件即可。参考 `common/` 目录的结构。

### Q: settings.json 和 settings.local.json 的区别？

- `settings.template.json` — 项目级配置模板，**纳入 Git，所有人共享**
- `settings.json` — 从模板生成的个人配置，**已 gitignore，每人不同**（由 `setup-dev.sh` 自动生成）
- `settings.local.json` — 个人覆盖，**已 gitignore，不影响他人**
