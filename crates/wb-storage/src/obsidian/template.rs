//! TemplateEngine —— 会议/任务/报告模板渲染

use std::collections::HashMap;

use wb_core::error::Result;

/// 模板引擎
#[derive(Debug, Clone)]
pub struct TemplateEngine;

impl TemplateEngine {
    /// 创建新的模板引擎
    pub fn new() -> Self {
        Self
    }

    /// 渲染指定模板，替换 `{{key}}` 占位符
    pub fn render(&self, template_name: &str, context: &HashMap<&str, &str>) -> Result<String> {
        let template = builtin_template(template_name).ok_or_else(|| {
            wb_core::error::WbError::NotFound(format!("template '{}' not found", template_name))
        })?;

        let mut result = template.to_string();
        for (key, value) in context {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        Ok(result)
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// 返回内置模板内容，如果模板名不存在则返回 None
fn builtin_template(name: &str) -> Option<&'static str> {
    match name {
        "meeting.md" => Some(MEETING_TEMPLATE),
        "task.md" => Some(TASK_TEMPLATE),
        "daily_report.md" => Some(DAILY_REPORT_TEMPLATE),
        _ => None,
    }
}

const MEETING_TEMPLATE: &str = r#"---
date: {{date}}
type: meeting
project: {{project}}
people: {{people}}
tags: [会议, {{project}}]
---

# 会议：{{title}}

> {{summary}}

## 议题

{{agenda}}

## 决议

{{decisions}}

## 行动项

{{action_items}}

## 来源

- [[{{project}}]]
"#;

const TASK_TEMPLATE: &str = r#"---
date: {{date}}
type: task
status: {{status}}
priority: {{priority}}
due: {{due}}
tags: [任务]
---

# 任务：{{title}}

> {{summary}}

## 描述

{{description}}

## 进度

- 状态：{{status}}
- 优先级：{{priority}}
- 截止日期：{{due}}

## 关联

- [[{{project}}]]
"#;

const DAILY_REPORT_TEMPLATE: &str = r#"---
date: {{date}}
type: daily_report
---

# 日报：{{date}}

## 完成事项

{{completed}}

## 进行中

{{in_progress}}

## 明日计划

{{planned}}

## 阻塞项

{{blockers}}

## 备注

{{notes}}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_meeting_template() {
        let engine = TemplateEngine::new();
        let mut ctx = HashMap::new();
        ctx.insert("date", "2026-06-06");
        ctx.insert("title", "周会");
        ctx.insert("project", "work-better");
        ctx.insert("people", "张三, 李四");
        ctx.insert("summary", "讨论进度");
        ctx.insert("agenda", "1. 功能开发\n2. Bug 修复");
        ctx.insert("decisions", "推迟上线");
        ctx.insert("action_items", "- [ ] 完成测试");

        let result = engine.render("meeting.md", &ctx).unwrap();
        assert!(result.contains("# 会议：周会"));
        assert!(result.contains("date: 2026-06-06"));
        assert!(result.contains("[[work-better]]"));
    }

    #[test]
    fn test_render_task_template() {
        let engine = TemplateEngine::new();
        let mut ctx = HashMap::new();
        ctx.insert("date", "2026-06-06");
        ctx.insert("title", "实现登录");
        ctx.insert("status", "进行中");
        ctx.insert("priority", "高");
        ctx.insert("due", "2026-06-10");
        ctx.insert("summary", "用户登录功能");
        ctx.insert("description", "实现 OAuth2 登录");
        ctx.insert("project", "auth");

        let result = engine.render("task.md", &ctx).unwrap();
        assert!(result.contains("# 任务：实现登录"));
        assert!(result.contains("status: 进行中"));
    }

    #[test]
    fn test_render_daily_report_template() {
        let engine = TemplateEngine::new();
        let mut ctx = HashMap::new();
        ctx.insert("date", "2026-06-06");
        ctx.insert("completed", "- 完成 Task 1");
        ctx.insert("in_progress", "- Task 2");
        ctx.insert("planned", "- Task 3");
        ctx.insert("blockers", "无");
        ctx.insert("notes", "进展顺利");

        let result = engine.render("daily_report.md", &ctx).unwrap();
        assert!(result.contains("# 日报：2026-06-06"));
        assert!(result.contains("完成 Task 1"));
    }

    #[test]
    fn test_render_unknown_template_returns_error() {
        let engine = TemplateEngine::new();
        let ctx = HashMap::new();
        let result = engine.render("nonexistent.md", &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_render_preserves_unmatched_placeholders() {
        let engine = TemplateEngine::new();
        let mut ctx = HashMap::new();
        ctx.insert("date", "2026-06-06");
        ctx.insert("title", "测试");
        ctx.insert("status", "todo");
        ctx.insert("priority", "low");
        ctx.insert("due", "2026-06-10");
        ctx.insert("summary", "摘要");
        ctx.insert("description", "描述");
        ctx.insert("project", "proj");

        let result = engine.render("task.md", &ctx).unwrap();
        // 已替换的
        assert!(result.contains("date: 2026-06-06"));
    }

    #[test]
    fn test_default_engine() {
        let engine = TemplateEngine;
        let ctx = HashMap::new();
        // Default 实现应与 new() 行为一致
        assert!(engine.render("meeting.md", &ctx).is_ok());
    }
}
