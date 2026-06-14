---
name: obsidian-sync
description: Obsidian 同步功能的 eval 定义
created: 2026-06-14
---

## EVAL DEFINITION: obsidian-sync (F4)

### Capability Evals

1. 事件数据可以同步到 Obsidian vault
2. 同步的 markdown 文件格式正确
3. 文件名和路径符合约定
4. 增量同步不会覆盖用户手动修改

### Regression Evals

1. 同步速度不低于基线
2. 已同步文件不被意外删除
3. vault 结构不被破坏

### Success Metrics

- pass@3 >= 90% for capability evals
- pass^3 = 100% for regression evals

### Code Grader

```bash
# 检查存储模块存在
test -d crates/wb-storage/src && echo "PASS" || echo "FAIL"

# 检查 Obsidian 相关代码
grep -q "obsidian\|vault\|markdown" crates/wb-storage/src/*.rs && echo "PASS" || echo "FAIL"

# 检查配置中有 vault 路径
grep -q "vault_path\|obsidian" src/components/settings/StorageSettings.tsx && echo "PASS" || echo "FAIL"
```

### Model Grader

评估同步质量：
1. 生成的 markdown 是否可读？
2. frontmatter 是否包含必要字段？
3. 文件组织是否符合 Obsidian 约定？
4. 同步冲突是否有合理处理？

Score: 1-5
