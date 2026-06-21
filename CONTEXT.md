# Work Better

以 Obsidian 为中心的 AI 工作观察者。被动采集、主动整理、数据归用户所有。

## Language

**工作台 (Dashboard)**:
默认首页视图，聚合今日任务、最近事件、待处理项。
_Avoid_: 首页、主页、概览

**命令面板 (Command Palette)**:
⌘K 触发的全局操作界面，支持导航、操作、搜索。
_Avoid_: 搜索框、快速命令

**渐进式引导 (Progressive Onboarding)**:
根据用户进度动态调整的空状态引导。
_Avoid_: 新手引导、教程

**主动推荐 (Proactive Suggestion)**:
AI 根据上下文自动推荐操作，用户确认后执行。
_Avoid_: AI 建议、智能提示

**事件 (Event)**:
从外部系统采集的原始数据（飞书消息、PR、Issue 等）。
_Avoid_: 消息、通知、记录

**任务 (Task)**:
用户需要完成的工作项，支持状态流转（待处理→进行中→已完成）。
_Avoid_: 待办、工作项、TODO

**采集器 (Collector)**:
从外部系统获取数据的组件（飞书、GitHub 等）。
_Avoid_: 数据源、连接器、集成

**工作记录 (WorkRecord)**:
用户的工作活动记录，用于生成报告。
_Avoid_: 工作日志、活动记录

**定时任务 (ScheduledTask)**:
按计划自动执行的任务（如每日采集、报告生成）。
_Avoid_: 调度任务、计划任务、Cron 任务

## Rules

- 命令面板是主要导航方式，所有功能都应可通过命令面板访问
- 工作台是默认视图，提供一站式工作概览
- 状态持久化使用 localStorage，不依赖 URL 参数
- 动画使用弹簧物理，保持克制风格
- AI 推荐需要用户确认，不自动执行
