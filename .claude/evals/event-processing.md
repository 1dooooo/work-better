---
name: event-processing
description: 事件处理流水线的 eval 定义
created: 2026-06-14
---

## EVAL DEFINITION: event-processing (F2)

### Capability Evals

1. 新事件可以被正确分类（会议/开发/沟通/休息）
2. 事件可以被提取为结构化数据
3. 事件可以被审核（自动或手动）
4. 处理结果可以生成报告

### Regression Evals

1. 分类准确率不低于基线
2. 处理延迟不超过基线
3. 已有事件的处理结果不被覆盖

### Success Metrics

- pass@3 >= 90% for capability evals
- pass^3 = 100% for regression evals

### Code Grader

```bash
# 检查处理器模块存在
test -d crates/wb-processor/src && echo "PASS" || echo "FAIL"

# 检查分类器存在
grep -q "classify" crates/wb-processor/src/*.rs && echo "PASS" || echo "FAIL"

# 检查测试文件存在
find crates/wb-processor -name "*test*" -o -name "*spec*" | grep -q . && echo "PASS" || echo "FAIL"
```

### Model Grader

评估处理流水线的质量：
1. 分类结果是否合理？
2. 提取的结构化数据是否完整？
3. 报告内容是否有价值？
4. 处理过程是否有详细日志？

Score: 1-5
