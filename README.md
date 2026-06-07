---
title: Work Better
type: structural
domain: product
created: 2026-06-06
updated: 2026-06-06
status: active
---

# Work Better

以 Obsidian 为中心的 AI 工作观察者。被动采集工作信息，智能处理与分析，自动维护任务与报告。

## 开发环境

```bash
git clone <repo-url> work-better
cd work-better
bash .claude/scripts/setup-dev.sh
```

前置条件：[Claude Code](https://docs.anthropic.com/en/docs/claude-code) + Node.js ≥ 18 + Rust toolchain + [ECC 插件](https://github.com/anthropics/ecc)

详细指引见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 文档

| 文档 | 说明 |
|------|------|
| [agent.md](agent.md) | 核心思想、文档索引、自维护规范 |
| [docs/product/overview.md](docs/product/overview.md) | 产品理念与功能方向 |
| [docs/architecture/overview.md](docs/architecture/overview.md) | 架构设计 |
| [docs/development/setup.md](docs/development/setup.md) | 开发环境搭建指南 |
| [docs/conventions.md](docs/conventions.md) | 导航规则、维护约束、生命周期 |
| [CONTRIBUTING.md](CONTRIBUTING.md) | 贡献指南 |
