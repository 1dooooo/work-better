---
name: settings-propagation
description: 设置传播功能的 eval 定义
created: 2026-06-14
---

## EVAL DEFINITION: settings-propagation (F4)

### Capability Evals

1. 设置界面可以修改所有配置项
2. 修改后设置立即生效
3. 设置持久化到磁盘
4. 重启后设置不丢失

### Regression Evals

1. 设置修改不影响正在运行的任务
2. 无效设置值被拒绝并提示
3. 默认设置在首次启动时正确加载

### Success Metrics

- pass@3 >= 90% for capability evals
- pass^3 = 100% for regression evals

### Code Grader

```bash
# 检查设置组件存在
test -f src/components/settings/StorageSettings.tsx && echo "PASS" || echo "FAIL"

# 检查设置测试存在
test -f src/components/settings/StorageSettings.test.tsx && echo "PASS" || echo "FAIL"

# 检查 Tauri 命令存在
grep -q "get_settings\|save_settings" src-tauri/src/commands/*.rs && echo "PASS" || echo "FAIL"
```

### Model Grader

评估设置体验：
1. 设置项是否有清晰的说明？
2. 修改后是否有即时反馈？
3. 错误输入是否有友好提示？
4. 设置分类是否合理？

Score: 1-5
