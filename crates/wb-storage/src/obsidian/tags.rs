//! TagManager —— 标签规范化管理

/// 标签管理器
#[derive(Debug, Clone)]
pub struct TagManager;

impl TagManager {
    /// 创建新的标签管理器
    pub fn new() -> Self {
        Self
    }

    /// 标签规范化：转小写，空格转连字符，去除首尾空白
    pub fn normalize(&self, tag: &str) -> String {
        tag.trim().to_lowercase().replace(' ', "-")
    }

    /// 将标签列表格式化为 YAML frontmatter 格式
    ///
    /// 输出示例:
    /// ```yaml
    /// tags:
    ///   - meeting
    ///   - project-alpha
    /// ```
    pub fn format_tags(&self, tags: &[String]) -> String {
        if tags.is_empty() {
            return String::new();
        }

        let mut out = String::from("tags:\n");
        for tag in tags {
            let normalized = self.normalize(tag);
            out.push_str(&format!("  - {}\n", normalized));
        }
        out
    }
}

impl Default for TagManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_lowercase() {
        let tm = TagManager::new();
        assert_eq!(tm.normalize("Meeting"), "meeting");
        assert_eq!(tm.normalize("TASK"), "task");
    }

    #[test]
    fn test_normalize_spaces_to_hyphens() {
        let tm = TagManager::new();
        assert_eq!(tm.normalize("project alpha"), "project-alpha");
        assert_eq!(tm.normalize("Bug Fix"), "bug-fix");
    }

    #[test]
    fn test_normalize_trim_whitespace() {
        let tm = TagManager::new();
        assert_eq!(tm.normalize("  meeting  "), "meeting");
        assert_eq!(tm.normalize("\tproject\t"), "project");
    }

    #[test]
    fn test_normalize_preserves_hyphens() {
        let tm = TagManager::new();
        assert_eq!(tm.normalize("already-normalized"), "already-normalized");
    }

    #[test]
    fn test_normalize_chinese() {
        let tm = TagManager::new();
        // 中文字符不含空格，应原样保留
        assert_eq!(tm.normalize("会议"), "会议");
        assert_eq!(tm.normalize(" 产品 规划 "), "产品-规划");
    }

    #[test]
    fn test_format_tags_normal() {
        let tm = TagManager::new();
        let tags = vec!["Meeting".to_string(), "Project Alpha".to_string()];
        let result = tm.format_tags(&tags);
        assert_eq!(result, "tags:\n  - meeting\n  - project-alpha\n");
    }

    #[test]
    fn test_format_tags_empty() {
        let tm = TagManager::new();
        let tags: Vec<String> = vec![];
        assert_eq!(tm.format_tags(&tags), "");
    }

    #[test]
    fn test_format_tags_single() {
        let tm = TagManager::new();
        let tags = vec!["urgent".to_string()];
        assert_eq!(tm.format_tags(&tags), "tags:\n  - urgent\n");
    }

    #[test]
    fn test_default_manager() {
        let tm = TagManager;
        assert_eq!(tm.normalize("Test"), "test");
    }
}
