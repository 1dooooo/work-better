//! Template —— 报告模板管理

use std::collections::HashMap;

/// 报告模板
#[derive(Debug, Clone)]
pub struct ReportTemplate {
    /// 模板名称
    pub name: String,
    /// 模板内容（Markdown，使用 `{{key}}` 占位符）
    pub content: String,
}

impl ReportTemplate {
    pub fn new(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            content: content.into(),
        }
    }

    /// 渲染模板，替换 `{{key}}` 占位符
    pub fn render(&self, context: &HashMap<&str, &str>) -> String {
        let mut result = self.content.clone();
        for (key, value) in context {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }
}

/// 模板仓库
///
/// 管理内置模板和自定义模板。
#[derive(Debug, Clone)]
pub struct TemplateRepository {
    templates: HashMap<String, ReportTemplate>,
}

impl TemplateRepository {
    pub fn new() -> Self {
        let mut repo = Self {
            templates: HashMap::new(),
        };
        repo.register_defaults();
        repo
    }

    /// 注册内置模板
    fn register_defaults(&mut self) {
        self.templates.insert(
            "daily".to_string(),
            ReportTemplate::new("daily", DAILY_TEMPLATE),
        );
        self.templates.insert(
            "weekly".to_string(),
            ReportTemplate::new("weekly", WEEKLY_TEMPLATE),
        );
        self.templates.insert(
            "monthly".to_string(),
            ReportTemplate::new("monthly", MONTHLY_TEMPLATE),
        );
    }

    /// 注册自定义模板
    pub fn register(&mut self, template: ReportTemplate) {
        self.templates.insert(template.name.clone(), template);
    }

    /// 获取模板
    pub fn get(&self, name: &str) -> Option<&ReportTemplate> {
        self.templates.get(name)
    }

    /// 获取模板名称列表
    pub fn names(&self) -> Vec<&str> {
        self.templates.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for TemplateRepository {
    fn default() -> Self {
        Self::new()
    }
}

// ---- 内置模板 ----

const DAILY_TEMPLATE: &str = r#"# 日报：{{date}}

## 完成事项

{{completed}}

## 进行中

{{in_progress}}

## 明日计划

{{planned}}

## 阻塞项

{{blockers}}
"#;

const WEEKLY_TEMPLATE: &str = r#"# 周报：{{week_range}}

## 本周进度

{{progress}}

## 关键成果

{{achievements}}

## 下周计划

{{next_week_plan}}

## 风险与阻塞

{{risks}}
"#;

const MONTHLY_TEMPLATE: &str = r#"# 月报：{{year}}年{{month}}月

## 目标进度

{{goals_progress}}

## 时间分配

{{time_distribution}}

## 效率趋势

{{efficiency_trend}}

## 总结

{{summary}}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_render() {
        let tpl = ReportTemplate::new("test", "Hello {{name}}, today is {{date}}");
        let mut ctx = HashMap::new();
        ctx.insert("name", "Alice");
        ctx.insert("date", "2026-06-06");
        assert_eq!(tpl.render(&ctx), "Hello Alice, today is 2026-06-06");
    }

    #[test]
    fn test_template_render_preserves_unknown() {
        let tpl = ReportTemplate::new("test", "Value: {{known}}, Missing: {{unknown}}");
        let mut ctx = HashMap::new();
        ctx.insert("known", "42");
        assert_eq!(tpl.render(&ctx), "Value: 42, Missing: {{unknown}}");
    }

    #[test]
    fn test_repository_has_builtin_templates() {
        let repo = TemplateRepository::new();
        assert!(repo.get("daily").is_some());
        assert!(repo.get("weekly").is_some());
        assert!(repo.get("monthly").is_some());
        assert!(repo.get("nonexistent").is_none());
    }

    #[test]
    fn test_repository_register_custom() {
        let mut repo = TemplateRepository::new();
        repo.register(ReportTemplate::new("custom", "Custom: {{val}}"));
        let tpl = repo.get("custom").unwrap();
        let mut ctx = HashMap::new();
        ctx.insert("val", "hello");
        assert_eq!(tpl.render(&ctx), "Custom: hello");
    }

    #[test]
    fn test_repository_names() {
        let repo = TemplateRepository::new();
        let names = repo.names();
        assert!(names.contains(&"daily"));
        assert!(names.contains(&"weekly"));
        assert!(names.contains(&"monthly"));
    }

    #[test]
    fn test_default_repository() {
        let repo = TemplateRepository::default();
        assert!(repo.get("daily").is_some());
    }

    #[test]
    fn test_builtin_daily_template_has_sections() {
        let tpl = ReportTemplate::new("daily", DAILY_TEMPLATE);
        assert!(tpl.content.contains("完成事项"));
        assert!(tpl.content.contains("明日计划"));
        assert!(tpl.content.contains("阻塞项"));
    }

    #[test]
    fn test_builtin_weekly_template_has_sections() {
        let tpl = ReportTemplate::new("weekly", WEEKLY_TEMPLATE);
        assert!(tpl.content.contains("本周进度"));
        assert!(tpl.content.contains("关键成果"));
        assert!(tpl.content.contains("风险"));
    }

    #[test]
    fn test_builtin_monthly_template_has_sections() {
        let tpl = ReportTemplate::new("monthly", MONTHLY_TEMPLATE);
        assert!(tpl.content.contains("目标进度"));
        assert!(tpl.content.contains("时间分配"));
        assert!(tpl.content.contains("效率趋势"));
    }
}
