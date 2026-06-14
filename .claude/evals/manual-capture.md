---
name: manual-capture
description: 手动采集功能的 eval 定义
created: 2026-06-14
---

## EVAL DEFINITION: manual-capture (F1.3)

### Capability Evals

1. 用户可以通过快捷键打开采集窗口
2. 用户可以输入文本内容
3. 用户可以选择采集类型（事件/任务/笔记）
4. 提交后数据写入 SQLite
5. 采集窗口关闭后数据不丢失

### Regression Evals

1. 快捷键设置变更后采集窗口仍可打开
2. 采集窗口不影响主窗口操作
3. 大文本输入不导致崩溃

### Success Metrics

- pass@3 >= 90% for capability evals
- pass^3 = 100% for regression evals

### Code Grader

```bash
# 检查采集窗口组件存在
grep -q "CaptureWindow" src/capture/CaptureWindow.tsx && echo "PASS" || echo "FAIL"

# 检查 Tauri 命令存在
grep -q "create_manual_event" src-tauri/src/commands/*.rs && echo "PASS" || echo "FAIL"

# 检查测试文件存在
test -f src/capture/CaptureWindow.test.tsx && echo "PASS" || echo "FAIL"
```

### Model Grader

评估采集窗口的用户体验：
1. 窗口打开速度是否 < 500ms？
2. 输入框是否有合理的 placeholder？
3. 提交后是否有成功反馈？
4. 错误情况下是否有友好提示？

Score: 1-5
