---
title: Scripts
description: Work Better 构建与打包脚本
---

# Scripts

项目构建、打包、开发脚本集合。

## 脚本列表

| 脚本 | 用途 | 命令 |
|------|------|------|
| `dev.sh` | 开发模式，热重载 | `./scripts/dev.sh` |
| `build-test.sh` | 编译测试，不打包 | `./scripts/build-test.sh` |
| `build-app.sh` | 打包 macOS `.app` | `./scripts/build-app.sh` |
| `build-dmg.sh` | 打包 macOS `.dmg` 安装镜像 | `./scripts/build-dmg.sh` |
| `check-docs.sh` | 文档规范检查 | `./scripts/check-docs.sh` |

## 使用方式

所有脚本均可在项目根目录直接运行：

```bash
# 开发模式（热重载，修改代码自动刷新）
./scripts/dev.sh

# 构建测试（验证前端和 Rust 能否编译通过）
./scripts/build-test.sh

# 打包 .app（生成 macOS 应用程序包）
./scripts/build-app.sh

# 打包 .dmg（生成可分发的安装镜像）
./scripts/build-dmg.sh
```

## 前置依赖

- Node.js + pnpm
- Rust toolchain（rustup）
- Xcode Command Line Tools

## 打包产物位置

打包后的文件在 `src-tauri/target/release/bundle/` 下：

```
target/release/bundle/
├── macos/       ← .app 应用程序包
└── dmg/         ← .dmg 安装镜像
```
