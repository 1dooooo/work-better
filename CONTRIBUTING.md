---
title: 贡献指南
type: guide
domain: development
created: 2026-06-06
updated: 2026-06-06
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
├── CLAUDE.md              # Claude Code 入口
├── agent.md               # Agent 指南（核心思想、文档索引）
├── CONTRIBUTING.md         # 本文档
├── docs/                  # 产品与架构文档
└── .claude/               # Claude Code 配置
    ├── rules/             # 编码规范（✅ 纳入 Git）
    ├── contexts/          # 上下文模板（✅ 纳入 Git）
    ├── settings.template.json  # 配置模板（✅ 纳入 Git）
    ├── settings.json      # 个人配置（❌ gitignore）
    ├── agents/            # ECC agent（❌ gitignore，自行安装 ECC）
    ├── skills/            # ECC skills（❌ gitignore，自行安装 ECC）
    ├── hooks/             # ECC hooks（❌ gitignore，自行安装 ECC）
    └── scripts/           # ECC scripts（❌ gitignore，自行安装 ECC）
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
4. **代码审查** — code-reviewer agent 自动审查
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
