---
title: 安全测试层级定义
type: structural
domain: testing
created: 2026-06-07
updated: 2026-06-07
status: active
---

# 安全测试层级定义 (H 层)

> **维护说明**：当新增安全测试类型、修改扫描工具配置、或调整安全门禁策略时更新本文档。

## 设计目标

H 层安全测试作为独立的测试层级，与 L1 单元测试并行执行（Gate 1 快速门的一部分）。
安全测试覆盖依赖漏洞、敏感数据泄露、输入边界、权限越界和文件系统安全。

## 层级定位

```
                    ┌─────────────────────┐
                    │  L5: 黑盒验收测试    │  182 tests · 分钟级
                  ┌─┴─────────────────────┴─┐
                  │  L4: 跨层 E2E 测试      │  ~20 tests · 十秒级
                ┌─┴─────────────────────────┴─┐
                │  L3: 契约测试               │  ~9 tests · 秒级
              ┌─┴─────────────────────────────┴─┐
              │  L2: 集成测试                    │  ~63 tests · 秒级
            ┌─┴─────────────────────────────────┴─┐
            │  L1: 纯单元测试                      │  ~158 tests · 毫秒级
            ├─────────────────────────────────────┤
            │  H:  安全测试                        │  ~20 tests · 秒级（与 L1 并行）
            └─────────────────────────────────────┘
```

## 子层分类

| 子层 | 负责方 | 执行时机 | 场景数 | 工具 |
|------|--------|---------|--------|------|
| H1 依赖漏洞扫描 | test-agent | Gate 1（每次变更） | ~5 | `cargo audit` / `npm audit` |
| H2 敏感数据泄露检测 | test-agent | Gate 1（每次变更） | ~3 | `grep` / `gitleaks` |
| H3 输入边界/注入测试 | review-agent | Gate 2（L2 通过后） | ~5 | rstest 参数化测试 |
| H4 权限越界测试 | review-agent | Gate 2（L2 通过后） | ~4 | rstest + Tauri command 测试 |
| H5 文件系统安全 | review-agent | Gate 2（L2 通过后） | ~3 | rstest + tempfile |

## H1：依赖漏洞扫描

自动扫描项目依赖中的已知漏洞。

```bash
# Rust 侧
cargo audit --json

# TypeScript 侧
pnpm audit --json
```

**判定规则**：
- critical 漏洞 → 阻塞，必须修复
- high 漏洞 → 阻塞，必须修复
- moderate 漏洞 → 警告，建议修复
- low 漏洞 → 信息，记录即可

## H2：敏感数据泄露检测

扫描代码中的硬编码密钥、密码、token 等敏感信息。

```bash
# 基础扫描
grep -rn "sk-\|api_key\s*=\s*\"\|password\s*=\s*\"" \
  crates/ src/ --include="*.rs" --include="*.ts" \
  --exclude-dir=target --exclude-dir=node_modules

# 深度扫描（如安装了 gitleaks）
gitleaks detect --source . --report-path .workflow/artifacts/gitleaks.json
```

**检测模式**：
- API key 模式：`sk-*`, `api_key=*`, `token=*`
- 密码模式：`password=*`, `secret=*`, `credential=*`
- 私钥模式：`BEGIN.*PRIVATE KEY`
- 数据库连接串：`mysql://`, `postgres://`, `mongodb://`

## H3：输入边界/注入测试

针对 Tauri command 层的输入验证，测试注入攻击场景。

```rust
use rstest::*;

#[rstest]
#[case("'; DROP TABLE events; --")]
#[case("<script>alert('xss')</script>")]
#[case("../../etc/passwd")]
#[case("\0\0\0\0")]
#[case("a".repeat(10000).as_str())]
fn test_input_boundary_rejection(#[case] malicious_input: &str) {
    // 验证 Tauri command 层拒绝恶意输入
    let result = validate_input(malicious_input);
    assert!(result.is_err(), "Should reject: {}", malicious_input);
}
```

## H4：权限越界测试

验证 Tauri command 的权限控制。

```rust
#[tokio::test]
async fn test_command_requires_permission() {
    // 验证未授权的 command 调用被拒绝
    let result = call_command_without_permission("delete_event").await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::PermissionDenied);
}
```

## H5：文件系统安全

验证文件操作的安全边界。

```rust
#[tokio::test]
async fn test_path_traversal_blocked() {
    // 验证路径遍历攻击被拦截
    let result = write_to_path("../../../etc/passwd", "content").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_symlink_escape_blocked() {
    // 验证符号链接逃逸被拦截
    let temp = tempfile::tempdir().unwrap();
    let symlink = temp.path().join("escape");
    std::os::unix::fs::symlink("/etc", &symlink).unwrap();
    let result = write_to_path(symlink.join("hosts").to_str().unwrap(), "content").await;
    assert!(result.is_err());
}
```

## 执行策略

| 门禁 | 包含的 H 子层 | 并行策略 |
|------|-------------|---------|
| Gate 1（每次变更） | H1 + H2 | 与 L1 并行 |
| Gate 2（PR） | H1 + H2 + H3 + H4 + H5 | L2 通过后触发 |
| Gate 3（发布） | 全量 H | 与全量 A-G 并行 |

## 与其他层级的关系

- H 层独立于 A-G 层，不阻塞功能测试的执行
- H1/H2 失败 → 阻塞合并（安全漏洞不可接受）
- H3-H5 由 review-agent 生成，失败后回到 dev-agent 修复
- H 层结果写入 `test-report.json` 的 `security_scan` 字段

## 测试数量

| 子层 | 场景数 | 依据 |
|------|--------|------|
| H1 | 5 | Rust + TS 依赖审计 + 版本检查 |
| H2 | 3 | 硬编码密钥 + 连接串 + 私钥 |
| H3 | 5 | SQL 注入 + XSS + 路径遍历 + null 字节 + 超长输入 |
| H4 | 4 | Tauri command 权限矩阵 |
| H5 | 3 | 路径遍历 + 符号链接 + 目录创建限制 |
| **合计** | **20** | |
