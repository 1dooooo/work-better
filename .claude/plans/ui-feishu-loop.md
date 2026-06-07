---
name: ui-feishu-loop
description: UI重构 + 飞书连接修复 自主循环计划
pattern: sequential
mode: safe
branch: dev/design
stop_condition: 所有任务完成且 build-test 通过
---

# UI 重构 + 飞书连接修复 循环计划

## 已完成

### UI 线
- [x] Phase 0.1: 设计 Token 系统（spacing/font/shadow/motion/border）
- [x] Phase 0.2: 消除硬编码颜色值
- [x] Phase 0.3: 清理 MenuBar 死代码（168 行）
- [x] Phase 2.1: 三个设置面板 BEM 类名修复
- [x] Phase 2.2: TasksView 缺失样式补充
- [x] Phase 2.3: 响应式 Sidebar（768px 折叠）
- [x] Phase 3: 深色模式（手动 + 系统偏好）
- [x] focus-visible 无障碍样式
- [x] Sidebar 主题切换按钮

### 飞书线（后端）
- [x] 1.1: 真实健康检查（lark-cli --version）
- [x] 1.2: CollectorManager 接入
- [x] 1.3: feishu_mode 配置命令
- [x] 1.4: Tauri 事件通知（feishu:collect-complete）
- [x] 后端 API 函数（tauri.ts）

## 待完成（本次循环）

### Task 1: 飞书前端集成
- [ ] CollectorSettings.tsx: feishu_mode 从后端读写
- [ ] EventsView.tsx: 监听 feishu:collect-complete 自动刷新
- [ ] EventsView.tsx: 采集失败错误反馈 UI
- [ ] MainWindow.tsx: 监听事件更新 unprocessedCount

### Task 2: chat_id 配置化
- [ ] EventsView.tsx: 移除硬编码 "default"，从配置读取
- [ ] CollectorSettings.tsx: 新增 chat_id 配置项

### Task 3: 构建验证
- [ ] build-test.sh 通过
- [ ] build-app.sh 通过

### Task 4: 提交
- [ ] 分组提交（UI 改动 / 飞书后端 / 飞书前端）

## 循环规则
1. 每个 Task 完成后运行 build-test.sh 验证
2. 构建失败立即修复再继续
3. 所有 Task 完成后停止循环
