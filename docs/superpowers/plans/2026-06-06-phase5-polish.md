---
title: Phase 5 — 收尾打磨
date: 2026-06-06
status: done
goal: 完成剩余 8 个功能，实现 108/108 全覆盖
phase: 5
depends_on:
  - 2026-06-06-phase4-task-intelligence.md
---

# Phase 5：收尾打磨

> 最后一程——补齐细节，100% 功能覆盖。

## 前置条件

Phase 1-4 全部完成（605 tests, 100 features ✅）

## 任务总览

| # | 任务 | 层 | 估时 | 功能点 |
|---|------|----|------|--------|
| 1 | 审计优化建议 | processor | 1h | F2.5.4 |
| 2 | 同步日志 + 配置管理 | storage | 2h | F3.3.4, F3.3.5 |
| 3 | 图片/截图采集 | collector+ui | 2h | F1.3.2, F1.3.3 |
| 4 | 飞书接入方式选择 | ui | 1h | F1.4.4 |
| 5 | 报告同步飞书 | processor | 1h | F5.2.5 |
| 6 | 任务管理界面 | ui | 2h | F6.2.8 |

**总计：~9h，6 个任务，覆盖 8 个功能点**

---

## 详细设计

### Task 1: 审计优化建议 (F2.5.4)

**修改**：`crates/wb-processor/src/audit_pipeline.rs`

基于审计数据分析优化方向：
- 分析处理耗时分布，发现瓶颈
- 分类错误率，识别薄弱环节
- 生成优化建议列表

### Task 2: 同步日志 + 配置管理 (F3.3.4, F3.3.5)

**新增**：
- `crates/wb-storage/src/sync_log.rs` — 同步日志记录（三层：采集/处理/存储）
- `crates/wb-storage/src/config.rs` — 配置管理（采集器、模型、定时任务配置持久化）

### Task 3: 图片/截图采集 (F1.3.2, F1.3.3)

**修改**：CaptureWindow 支持图片粘贴和截图
- 图片粘贴：监听 paste 事件，读取剪贴板图片
- 截图捕获：调用 `screencapture` 命令

### Task 4: 飞书接入方式选择 (F1.4.4)

**修改**：`src/components/settings/CollectorSettings.tsx`
- 添加 API/CLI 切换开关

### Task 5: 报告同步飞书 (F5.2.5)

**修改**：`crates/wb-processor/src/report/`
- 报告确认后同步到飞书文档

### Task 6: 任务管理界面 (F6.2.8)

**新增**：`src/components/views/TaskView.tsx`
- 任务列表（按状态分组）
- 任务创建/编辑/归档
- 定时任务查看

---

## 验收标准

- [ ] 108/108 功能全部 ✅
- [ ] 650+ tests
- [ ] Clippy 无警告
- [ ] 前端构建正常
